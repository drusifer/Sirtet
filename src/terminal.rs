use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};

use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Terminal;

use tetris::battle::{BattleState, GameMode, MatchWinner};
use tetris::board::{HEIGHT, WIDTH};
use tetris::cpu_ai::CpuAgent;
use tetris::game::{Game, GameState};

fn piece_color(id: u8) -> Color {
    match id {
        1 => Color::Cyan,
        2 => Color::Yellow,
        3 => Color::Magenta,
        4 => Color::Green,
        5 => Color::Red,
        6 => Color::Blue,
        7 => Color::Rgb(255, 140, 0),
        8 => Color::DarkGray,
        _ => Color::White,
    }
}

#[allow(dead_code)]
pub fn run(game: Game) -> io::Result<()> {

    let mut battle = BattleState::new(GameMode::Single);
    battle.player1 = game;
    run_battle(battle)
}

pub fn run_battle(battle: BattleState) -> io::Result<()> {
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
    mut battle: BattleState,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let cpu_agent = CpuAgent::new();

    loop {
        let interval = Duration::from_millis(battle.player1.gravity_interval_ms());
        let elapsed = last_tick.elapsed();
        let timeout = interval.saturating_sub(elapsed);

        if event::poll(timeout.min(Duration::from_millis(16)).max(Duration::ZERO))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => {
                    battle.player1.move_left();
                }
                KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => {
                    battle.player1.move_right();
                }
                KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                    battle.player1.soft_drop();
                }
                KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                    battle.player1.rotate();
                }
                KeyCode::Char(' ') => {
                    battle.p1_hard_drop();
                }
                KeyCode::Enter if battle.mode == GameMode::TwoPlayerLocal => {
                    battle.p2_hard_drop();
                }
                KeyCode::Char('p') | KeyCode::Char('P') => {
                    battle.player1.toggle_pause();
                    if let Some(ref mut p2) = battle.player2 {
                        p2.toggle_pause();
                    }
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    let mode = battle.mode;
                    battle = BattleState::new(mode);
                    last_tick = Instant::now();
                }
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    return Ok(());
                }
                _ => {}
            }
        }

        if last_tick.elapsed() >= interval {
            if battle.mode == GameMode::VsCpu
                && let Some(ref mut p2) = battle.player2
                && p2.state() == GameState::Playing
            {
                cpu_agent.make_move(p2);
            }
            battle.tick();
            last_tick = Instant::now();
        }


        terminal.draw(|f| battle_ui(f, &mut battle))?;
    }
}

fn battle_ui(f: &mut ratatui::Frame, battle: &mut BattleState) {
    f.render_widget(Block::default().style(Style::default().bg(Color::Black)), f.area());

    if battle.mode == GameMode::Single {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)].as_ref())
            .split(f.area());

        draw_board_widget(f, chunks[0], &battle.player1, " PLAYER 1 ");
        draw_side_panel_widget(f, chunks[1], &mut battle.player1);
        draw_status_overlay_widget(f, f.area(), &battle.player1);
    } else {
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(f.area());

        // P1
        let p1_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
            .split(main_chunks[0]);
        draw_board_widget(f, p1_chunks[0], &battle.player1, " PLAYER 1 ");
        draw_side_panel_widget(f, p1_chunks[1], &mut battle.player1);

        // P2
        if let Some(ref mut p2) = battle.player2 {
            let p2_title = if battle.mode == GameMode::VsCpu { " CPU OPPONENT " } else { " PLAYER 2 " };
            let p2_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                .split(main_chunks[1]);
            draw_board_widget(f, p2_chunks[0], p2, p2_title);
            draw_side_panel_widget(f, p2_chunks[1], p2);
        }

        draw_match_winner_overlay(f, f.area(), battle.winner);
    }
}

fn draw_board_widget(f: &mut ratatui::Frame, area: Rect, game: &Game, title: &str) {
    let active_cells = if game.state() != GameState::GameOver {
        game.active().cells().to_vec()
    } else {
        vec![]
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let cell_w = (inner.width / WIDTH as u16).max(2);
    let cell_h = (inner.height / HEIGHT as u16).max(1);

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let cx = inner.x + x as u16 * cell_w;
            let cy = inner.y + y as u16 * cell_h;

            if cx + cell_w > inner.x + inner.width || cy + cell_h > inner.y + inner.height {
                continue;
            }

            let is_active = active_cells.contains(&(x as i32, y as i32));
            let color = if is_active {
                piece_color(game.active().piece_type.id())
            } else if let Some(id) = game.board().cell(x as i32, y as i32) {
                piece_color(id)
            } else {
                Color::DarkGray
            };

            let symbol = if is_active || game.board().cell(x as i32, y as i32).is_some() {
                "██"
            } else {
                " ."
            };

            let rect = Rect::new(cx, cy, cell_w, cell_h);
            let p = Paragraph::new(symbol).style(Style::default().fg(color));
            f.render_widget(p, rect);
        }
    }
}

fn draw_side_panel_widget(f: &mut ratatui::Frame, area: Rect, game: &mut Game) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(8)].as_ref())
        .split(area);

    let next_type = game.peek_next();
    let next_block = Block::default()
        .borders(Borders::ALL)
        .title(" NEXT ")
        .style(Style::default().fg(Color::Yellow));

    let next_inner = next_block.inner(chunks[0]);
    f.render_widget(next_block, chunks[0]);

    let mut next_lines = Vec::new();
    let coords = next_type.cells(0);

    for r in 0..3i32 {
        let mut spans = Vec::new();
        for c in 0..4i32 {
            if coords.contains(&(c, r)) {

                spans.push(Span::styled("██", Style::default().fg(piece_color(next_type.id()))));
            } else {
                spans.push(Span::raw("  "));
            }
        }
        next_lines.push(Line::from(spans));
    }
    f.render_widget(Paragraph::new(next_lines), next_inner);

    let stats_text = vec![
        Line::from(vec![Span::raw("SCORE: "), Span::styled(format!("{}", game.score()), Style::default().fg(Color::Green))]),
        Line::from(vec![Span::raw("LEVEL: "), Span::styled(format!("{}", game.level()), Style::default().fg(Color::Yellow))]),
        Line::from(vec![Span::raw("LINES: "), Span::styled(format!("{}", game.lines_cleared()), Style::default().fg(Color::Cyan))]),
        Line::from(vec![Span::raw("ATTACK QUEUE: "), Span::styled(format!("{}", game.pending_garbage()), Style::default().fg(Color::Red))]),
    ];

    let stats_block = Block::default()
        .borders(Borders::ALL)
        .title(" STATS ")
        .style(Style::default().fg(Color::White));
    f.render_widget(Paragraph::new(stats_text).block(stats_block), chunks[1]);
}

fn draw_status_overlay_widget(f: &mut ratatui::Frame, area: Rect, game: &Game) {
    if game.state() == GameState::Playing {
        return;
    }

    let title = match game.state() {
        GameState::Paused => " PAUSED ",
        GameState::GameOver => " GAME OVER ",
        _ => return,
    };

    let text = vec![
        Line::from(Span::styled(title, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("Press R to restart"),
        Line::from("Press Q to quit"),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::White));

    let popup_area = Rect::new(area.width / 4, area.height / 3, area.width / 2, 7);
    f.render_widget(Clear, popup_area);
    f.render_widget(Paragraph::new(text).block(block), popup_area);
}

fn draw_match_winner_overlay(f: &mut ratatui::Frame, area: Rect, winner: MatchWinner) {
    if winner == MatchWinner::None {
        return;
    }

    let title = match winner {
        MatchWinner::Player1 => " PLAYER 1 WINS! ",
        MatchWinner::Player2 => " PLAYER 2 WINS! ",
        MatchWinner::Cpu => " CPU WINS! ",
        MatchWinner::None => return,
    };

    let text = vec![
        Line::from(Span::styled(title, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("Press R to play again"),
        Line::from("Press Q to quit"),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::Yellow));

    let popup_area = Rect::new(area.width / 4, area.height / 3, area.width / 2, 7);
    f.render_widget(Clear, popup_area);
    f.render_widget(Paragraph::new(text).block(block), popup_area);
}
