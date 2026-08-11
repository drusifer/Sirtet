use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::execute;

use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};

use ratatui::widgets::canvas::{Canvas, Line};

use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;

use tetris::spatial_game::{
    Axis, GameState, SpatialGame, BOX_DEPTH, BOX_HEIGHT, BOX_WIDTH,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CameraMode {
    BlockoutTopDown,
    Isometric3D,
}

pub fn run(game: SpatialGame) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, game);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

pub fn run_battle(battle: tetris::battle::BattleState) -> io::Result<()> {
    if battle.mode == tetris::battle::GameMode::Single {
        return run(SpatialGame::new());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_battle_loop(&mut terminal, battle);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run_battle_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    battle: tetris::battle::BattleState,
) -> io::Result<()> {
    let mut p1_game = SpatialGame::new();
    let mut p2_game = SpatialGame::new();
    let mut last_tick = Instant::now();
    let camera_mode = CameraMode::Isometric3D;

    loop {
        let interval = Duration::from_millis(tetris::spatial_game::spatial_gravity_interval_ms(p1_game.level));
        let elapsed = last_tick.elapsed();
        let timeout = interval.saturating_sub(elapsed);

        if event::poll(timeout.min(Duration::from_millis(16)).max(Duration::ZERO))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => { p1_game.move_x(-1); }
                KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => { p1_game.move_x(1); }
                KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => { p1_game.move_y(-1); }
                KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => { p1_game.move_y(1); }
                KeyCode::Char('x') | KeyCode::Char('X') => { p1_game.rotate(Axis::X); }
                KeyCode::Char('y') | KeyCode::Char('Y') => { p1_game.rotate(Axis::Y); }
                KeyCode::Char('z') | KeyCode::Char('Z') => { p1_game.rotate(Axis::Z); }
                KeyCode::Char(' ') => { p1_game.hard_drop(); }
                KeyCode::Enter => { if battle.mode == tetris::battle::GameMode::TwoPlayerLocal { p2_game.hard_drop(); } }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    p1_game = SpatialGame::new();
                    p2_game = SpatialGame::new();
                    last_tick = Instant::now();
                }
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => { return Ok(()); }
                _ => {}
            }
        }

        if last_tick.elapsed() >= interval {
            if battle.mode == tetris::battle::GameMode::VsCpu && macroquad::rand::gen_range(0, 10) < 3 {
                p2_game.move_x(if macroquad::rand::gen_range(0, 2) == 0 { 1 } else { -1 });
            }

            p1_game.tick();
            p2_game.tick();
            last_tick = Instant::now();
        }

        let ref_p1 = &p1_game;
        let ref_p2 = &p2_game;
        terminal.draw(|f| {
            f.render_widget(Block::default().style(Style::default().bg(Color::Black)), f.area());
            let split = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(f.area());

            let p2_label = if battle.mode == tetris::battle::GameMode::VsCpu { "CPU OPPONENT" } else { "PLAYER 2" };

            let p1_canvas = Canvas::default()
                .block(Block::default().borders(Borders::ALL).title(format!(" P1 (3D Spatial) — Score: {} ", ref_p1.score)))
                .x_bounds([-60.0, 60.0])
                .y_bounds([-60.0, 60.0])
                .paint(move |ctx| { draw_tui_3d_well(ctx, ref_p1, camera_mode, false); });

            let p2_canvas = Canvas::default()
                .block(Block::default().borders(Borders::ALL).title(format!(" {p2_label} (3D Spatial) — Score: {} ", ref_p2.score)))
                .x_bounds([-60.0, 60.0])
                .y_bounds([-60.0, 60.0])
                .paint(move |ctx| { draw_tui_3d_well(ctx, ref_p2, camera_mode, false); });

            f.render_widget(p1_canvas, split[0]);
            f.render_widget(p2_canvas, split[1]);
        })?;

    }
}



fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut game: SpatialGame,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let mut camera_mode = CameraMode::BlockoutTopDown;
    let mut fx_start: Option<Instant> = None;
    let mut prev_z = game.active_piece.z;

    loop {
        let interval = Duration::from_millis(tetris::spatial_game::spatial_gravity_interval_ms(game.level));
        let elapsed = last_tick.elapsed();
        let timeout = interval.saturating_sub(elapsed);

        if event::poll(timeout.min(Duration::from_millis(16)).max(Duration::ZERO))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => {
                    game.move_x(-1);
                }
                KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => {
                    game.move_x(1);
                }
                KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                    game.move_y(-1);
                }
                KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                    game.move_y(1);
                }
                KeyCode::Char('x') | KeyCode::Char('X') => {
                    game.rotate(Axis::X);
                }
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    game.rotate(Axis::Y);
                }
                KeyCode::Char('z') | KeyCode::Char('Z') => {
                    game.rotate(Axis::Z);
                }
                KeyCode::Char(' ') => {
                    game.hard_drop();
                }
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    camera_mode = match camera_mode {
                        CameraMode::BlockoutTopDown => CameraMode::Isometric3D,
                        CameraMode::Isometric3D => CameraMode::BlockoutTopDown,
                    };
                }
                KeyCode::Char('p') | KeyCode::Char('P') => {
                    game.toggle_pause();
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    game.restart();
                    last_tick = Instant::now();
                }
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    return Ok(());
                }
                _ => {}
            }
        }

        if last_tick.elapsed() >= interval {
            game.tick();
            last_tick = Instant::now();

            if (game.active_piece.z == 1 && prev_z > 1) || game.last_layers_cleared > 0 {
                fx_start = Some(Instant::now());
            }
        }
        prev_z = game.active_piece.z;

        let fx_active = if let Some(start) = fx_start {
            if start.elapsed() < Duration::from_millis(300) {
                true
            } else {
                fx_start = None;
                false
            }
        } else {
            false
        };

        terminal.draw(|f| ui(f, &game, camera_mode, fx_active))?;
    }
}

fn ui(f: &mut ratatui::Frame, game: &SpatialGame, camera_mode: CameraMode, fx_active: bool) {
    f.render_widget(Block::default().style(Style::default().bg(Color::Black)), f.area());

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(72), Constraint::Percentage(28)].as_ref())
        .split(f.area());

    let title_mode = match camera_mode {
        CameraMode::BlockoutTopDown => "Top-Down Pit (Classic Mac)",
        CameraMode::Isometric3D => "3/4 Isometric 3D",
    };

    let canvas_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" 3D Spatial Box Tetris — {title_mode} [C to toggle] "))
        .style(Style::default().bg(Color::Black));

    let canvas = Canvas::default()
        .block(canvas_block)
        .x_bounds([-110.0, 110.0])
        .y_bounds([-100.0, 100.0])
        .paint(move |ctx| {
            draw_tui_3d_well(ctx, game, camera_mode, fx_active);
        });

    f.render_widget(canvas, chunks[0]);

    // Side panel HUD
    let hud_text = format!(
        "SCORE : {}\nLEVEL : {}\nLAYERS: {}\nNEXT  : {:?}\n\n{}\n\n3D CONTROLS:\nLeft/Right/A/D: Move X\nUp/Down/W/S   : Move Y\nX / Y / Z     : Rotate 3D\nSpace         : Drop Z\nC             : Camera View\nP             : Pause\nR             : Restart\nQ/Esc         : Quit",
        game.score,
        game.level,
        game.layers_cleared,
        game.next_piece,
        match game.state {
            GameState::Paused => "*** PAUSED ***",
            GameState::GameOver => "*** GAME OVER ***\nPress R to restart",
            GameState::Playing => "",
        }
    );

    let hud = Paragraph::new(hud_text)
        .block(Block::default().borders(Borders::ALL).title(" Info & Controls "))
        .style(Style::default().fg(Color::Cyan).bg(Color::Black));

    f.render_widget(hud, chunks[1]);
}



fn project(x: f64, y: f64, z: f64, camera_mode: CameraMode) -> (f64, f64) {
    match camera_mode {
        CameraMode::BlockoutTopDown => {
            // Perspective expanding to fill full canvas window
            let scale = 1.0 - (z / 14.0);
            let px = (x - 2.5) * 35.0 * scale;
            let py = (2.5 - y) * 32.0 * scale;
            (px, py)
        }
        CameraMode::Isometric3D => {
            // Isometric 3D projection
            let iso_x = (x - y) * 18.0;
            let iso_y = (x + y) * 9.0 - (z - 5.0) * 14.0;
            (iso_x, iso_y)
        }
    }
}

fn draw_tui_3d_well(
    ctx: &mut ratatui::widgets::canvas::Context,
    game: &SpatialGame,
    camera_mode: CameraMode,
    fx_active: bool,
) {
    // Draw 10 Z-level rectangular frames for pit depth walls
    for z in 0..=BOX_HEIGHT {
        let (x0, y0) = project(0.0, 0.0, z as f64, camera_mode);
        let (x1, y1) = project(BOX_WIDTH as f64, 0.0, z as f64, camera_mode);
        let (x2, y2) = project(BOX_WIDTH as f64, BOX_DEPTH as f64, z as f64, camera_mode);
        let (x3, y3) = project(0.0, BOX_DEPTH as f64, z as f64, camera_mode);

        let color = if fx_active {
            Color::White
        } else if z == 0 || z == BOX_HEIGHT {
            Color::Cyan
        } else if z % 2 == 0 {
            Color::LightBlue
        } else {
            Color::DarkGray
        };

        // Layer rectangle boundary on pit walls
        ctx.draw(&Line { x1: x0, y1: y0, x2: x1, y2: y1, color });
        ctx.draw(&Line { x1, y1, x2, y2, color });
        ctx.draw(&Line { x1: x2, y1: y2, x2: x3, y2: y3, color });
        ctx.draw(&Line { x1: x3, y1: y3, x2: x0, y2: y0, color });
    }

    // Corner connecting lines
    for &(x, y) in &[(0.0, 0.0), (BOX_WIDTH as f64, 0.0), (BOX_WIDTH as f64, BOX_DEPTH as f64), (0.0, BOX_DEPTH as f64)] {
        let (p0_x, p0_y) = project(x, y, 0.0, camera_mode);
        let (p1_x, p1_y) = project(x, y, BOX_HEIGHT as f64, camera_mode);
        let pillar_color = if fx_active { Color::White } else { Color::Blue };
        ctx.draw(&Line { x1: p0_x, y1: p0_y, x2: p1_x, y2: p1_y, color: pillar_color });
    }

    // Draw locked blocks
    let block_color = if fx_active { Color::LightYellow } else { Color::Yellow };
    for z in 0..BOX_HEIGHT as i8 {
        for x in 0..BOX_WIDTH as i8 {
            for y in 0..BOX_DEPTH as i8 {
                if game.board.cells[z as usize][x as usize][y as usize].is_some() {
                    draw_block_cube(ctx, x as f64, y as f64, z as f64, block_color, camera_mode);
                }
            }
        }
    }

    // Draw active piece blocks
    if game.state == GameState::Playing {
        let active_color = if fx_active { Color::LightCyan } else { Color::Green };
        for (wx, wy, wz) in game.active_piece.world_blocks() {
            if wz >= 0 && wz < BOX_HEIGHT as i8 {
                draw_block_cube(ctx, wx as f64, wy as f64, wz as f64, active_color, camera_mode);
            }
        }
    }
}


fn draw_block_cube(
    ctx: &mut ratatui::widgets::canvas::Context,
    x: f64,
    y: f64,
    z: f64,
    color: Color,
    camera_mode: CameraMode,
) {
    // Projected 8 vertices of the 1x1x1 unit cube in grid space [x..x+1, y..y+1, z..z+1]
    let (v0_x, v0_y) = project(x, y, z, camera_mode);
    let (v1_x, v1_y) = project(x + 1.0, y, z, camera_mode);
    let (v2_x, v2_y) = project(x + 1.0, y + 1.0, z, camera_mode);
    let (v3_x, v3_y) = project(x, y + 1.0, z, camera_mode);

    let (v4_x, v4_y) = project(x, y, z + 1.0, camera_mode);
    let (v5_x, v5_y) = project(x + 1.0, y, z + 1.0, camera_mode);
    let (v6_x, v6_y) = project(x + 1.0, y + 1.0, z + 1.0, camera_mode);
    let (v7_x, v7_y) = project(x, y + 1.0, z + 1.0, camera_mode);

    // Front face (z)
    ctx.draw(&Line { x1: v0_x, y1: v0_y, x2: v1_x, y2: v1_y, color });
    ctx.draw(&Line { x1: v1_x, y1: v1_y, x2: v2_x, y2: v2_y, color });
    ctx.draw(&Line { x1: v2_x, y1: v2_y, x2: v3_x, y2: v3_y, color });
    ctx.draw(&Line { x1: v3_x, y1: v3_y, x2: v0_x, y2: v0_y, color });

    // Back face (z+1)
    ctx.draw(&Line { x1: v4_x, y1: v4_y, x2: v5_x, y2: v5_y, color });
    ctx.draw(&Line { x1: v5_x, y1: v5_y, x2: v6_x, y2: v6_y, color });
    ctx.draw(&Line { x1: v6_x, y1: v6_y, x2: v7_x, y2: v7_y, color });
    ctx.draw(&Line { x1: v7_x, y1: v7_y, x2: v4_x, y2: v4_y, color });

    // Connecting 4 depth edges
    ctx.draw(&Line { x1: v0_x, y1: v0_y, x2: v4_x, y2: v4_y, color });
    ctx.draw(&Line { x1: v1_x, y1: v1_y, x2: v5_x, y2: v5_y, color });
    ctx.draw(&Line { x1: v2_x, y1: v2_y, x2: v6_x, y2: v6_y, color });
    ctx.draw(&Line { x1: v3_x, y1: v3_y, x2: v7_x, y2: v7_y, color });
}



