use std::io::{self, Write};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use crossterm::{cursor, execute, queue};

use tetris::board::{HEIGHT, WIDTH};
use tetris::game::{Game, GameState};

const BOARD_X: u16 = 1;
const BOARD_Y: u16 = 1;
const PANEL_X: u16 = BOARD_X + (WIDTH as u16) * 2 + 3;

fn piece_color(id: u8) -> Color {
    match id {
        1 => Color::Cyan,
        2 => Color::Yellow,
        3 => Color::Magenta,
        4 => Color::Green,
        5 => Color::Red,
        6 => Color::Blue,
        7 => Color::DarkYellow,
        _ => Color::White,
    }
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    let result = run(&mut stdout);

    execute!(stdout, cursor::Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    result
}

fn run(stdout: &mut io::Stdout) -> io::Result<()> {
    let mut game = Game::new();
    let mut last_tick = Instant::now();

    loop {
        let interval = Duration::from_millis(game.gravity_interval_ms());
        let elapsed = last_tick.elapsed();
        let timeout = interval.saturating_sub(elapsed);

        if event::poll(timeout.min(Duration::from_millis(16)).max(Duration::ZERO))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Left => {
                    game.move_left();
                }
                KeyCode::Right => {
                    game.move_right();
                }
                KeyCode::Down => {
                    game.soft_drop();
                }
                KeyCode::Up => {
                    game.rotate();
                }
                KeyCode::Char(' ') => {
                    game.hard_drop();
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
        }

        render(stdout, &mut game)?;
    }
}

fn render(stdout: &mut io::Stdout, game: &mut Game) -> io::Result<()> {
    queue!(stdout, Clear(ClearType::All))?;

    draw_border(stdout)?;
    draw_locked_cells(stdout, game)?;
    draw_active_piece(stdout, game)?;
    draw_side_panel(stdout, game)?;
    draw_controls_legend(stdout)?;
    draw_status_overlay(stdout, game)?;

    stdout.flush()
}

fn draw_border(stdout: &mut io::Stdout) -> io::Result<()> {
    let w = (WIDTH as u16) * 2;
    queue!(stdout, cursor::MoveTo(BOARD_X - 1, BOARD_Y - 1))?;
    queue!(stdout, Print("+"), Print("-".repeat(w as usize)), Print("+"))?;
    for y in 0..HEIGHT as u16 {
        queue!(stdout, cursor::MoveTo(BOARD_X - 1, BOARD_Y + y))?;
        queue!(stdout, Print("|"))?;
        queue!(stdout, cursor::MoveTo(BOARD_X + w, BOARD_Y + y))?;
        queue!(stdout, Print("|"))?;
    }
    queue!(stdout, cursor::MoveTo(BOARD_X - 1, BOARD_Y + HEIGHT as u16))?;
    queue!(stdout, Print("+"), Print("-".repeat(w as usize)), Print("+"))?;
    Ok(())
}

fn draw_cell(stdout: &mut io::Stdout, x: i32, y: i32, id: u8) -> io::Result<()> {
    if x < 0 || y < 0 || x as usize >= WIDTH || y as usize >= HEIGHT {
        return Ok(());
    }
    let screen_x = BOARD_X + (x as u16) * 2;
    let screen_y = BOARD_Y + y as u16;
    queue!(stdout, cursor::MoveTo(screen_x, screen_y))?;
    queue!(stdout, SetForegroundColor(piece_color(id)), Print("[]"), ResetColor)?;
    Ok(())
}

fn draw_locked_cells(stdout: &mut io::Stdout, game: &Game) -> io::Result<()> {
    for y in 0..HEIGHT as i32 {
        for x in 0..WIDTH as i32 {
            if let Some(id) = game.board().cell(x, y) {
                draw_cell(stdout, x, y, id)?;
            }
        }
    }
    Ok(())
}

fn draw_active_piece(stdout: &mut io::Stdout, game: &Game) -> io::Result<()> {
    if game.state() == GameState::GameOver {
        return Ok(());
    }
    let id = game.active().piece_type.id();
    for (x, y) in game.active().cells() {
        draw_cell(stdout, x, y, id)?;
    }
    Ok(())
}

fn draw_side_panel(stdout: &mut io::Stdout, game: &mut Game) -> io::Result<()> {
    let mut row = BOARD_Y;
    queue!(stdout, cursor::MoveTo(PANEL_X, row), Print("NEXT:"))?;
    row += 1;
    let next_type = game.peek_next();
    for (x, y) in next_type.cells(0) {
        let screen_x = PANEL_X + (x as u16) * 2;
        let screen_y = row + y as u16;
        queue!(stdout, cursor::MoveTo(screen_x, screen_y))?;
        queue!(
            stdout,
            SetForegroundColor(piece_color(next_type.id())),
            Print("[]"),
            ResetColor
        )?;
    }
    row += 5;

    queue!(stdout, cursor::MoveTo(PANEL_X, row), Print(format!("SCORE: {}", game.score())))?;
    row += 1;
    queue!(stdout, cursor::MoveTo(PANEL_X, row), Print(format!("LEVEL: {}", game.level())))?;
    row += 1;
    queue!(stdout, cursor::MoveTo(PANEL_X, row), Print(format!("LINES: {}", game.lines_cleared())))?;
    Ok(())
}

/// Row where the score panel ends, so the controls legend can start below it without
/// overlapping (score panel: NEXT label + 4-row preview + SCORE/LEVEL/LINES = 10 rows).
const SIDE_PANEL_HEIGHT: u16 = 10;

fn draw_controls_legend(stdout: &mut io::Stdout) -> io::Result<()> {
    let lines = [
        "CONTROLS:",
        "<-/-> move",
        "Down  soft drop",
        "Up    rotate",
        "Space hard drop",
        "P     pause",
        "R     restart",
        "Q/Esc quit",
    ];
    let start_row = BOARD_Y + SIDE_PANEL_HEIGHT + 1;
    for (i, line) in lines.iter().enumerate() {
        queue!(stdout, cursor::MoveTo(PANEL_X, start_row + i as u16), Print(*line))?;
    }
    Ok(())
}

fn draw_status_overlay(stdout: &mut io::Stdout, game: &Game) -> io::Result<()> {
    let center_x = BOARD_X + (WIDTH as u16);
    let center_y = BOARD_Y + (HEIGHT as u16) / 2;
    match game.state() {
        GameState::Paused => {
            queue!(stdout, cursor::MoveTo(center_x.saturating_sub(3), center_y), Print("PAUSED"))?;
        }
        GameState::GameOver => {
            queue!(
                stdout,
                cursor::MoveTo(center_x.saturating_sub(5), center_y),
                Print("GAME OVER")
            )?;
            queue!(
                stdout,
                cursor::MoveTo(center_x.saturating_sub(9), center_y + 1),
                Print(format!("Final score: {}", game.score()))
            )?;
            queue!(
                stdout,
                cursor::MoveTo(center_x.saturating_sub(8), center_y + 2),
                Print("Press R to restart")
            )?;
        }
        GameState::Playing => {}
    }
    Ok(())
}
