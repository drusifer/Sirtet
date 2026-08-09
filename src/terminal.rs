use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::execute;

use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Terminal;

use tetris::board::{HEIGHT, WIDTH};
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
        _ => Color::White,
    }
}

pub fn run(game: Game) -> io::Result<()> {
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

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut game: Game,
) -> io::Result<()> {
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

        terminal.draw(|f| ui(f, &mut game))?;
    }
}

fn ui(f: &mut ratatui::Frame, game: &mut Game) {
    f.render_widget(Block::default().style(Style::default().bg(Color::Black)), f.area());

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)].as_ref())
        .split(f.area());

    draw_board_widget(f, chunks[0], game);
    draw_side_panel_widget(f, chunks[1], game);
    draw_status_overlay_widget(f, f.area(), game);
}

fn draw_board_widget(f: &mut ratatui::Frame, area: Rect, game: &Game) {
    let active_cells = if game.state() != GameState::GameOver {
        game.active().cells().to_vec()
    } else {
        vec![]
    };
    let active_id = game.active().piece_type.id();

    let avail_height = area.height.saturating_sub(2) as usize;
    let avail_width = area.width.saturating_sub(2) as usize;

    let k_height = avail_height / HEIGHT;
    let k_width = avail_width / (WIDTH * 2);
    let scale_k = k_height.min(k_width).max(1);

    let empty_dot_str = " ".repeat(2 * scale_k);

    let mut lines = Vec::with_capacity(HEIGHT * scale_k);

    for y in 0..HEIGHT as i32 {
        if scale_k == 1 {
            let mut spans = Vec::with_capacity(WIDTH);
            for x in 0..WIDTH as i32 {
                if let Some(id) = game.board().cell(x, y) {
                    spans.push(Span::styled("[#]", Style::default().fg(piece_color(id)).bg(Color::Black)));
                } else if active_cells.contains(&(x, y)) {
                    spans.push(Span::styled("[#]", Style::default().fg(piece_color(active_id)).bg(Color::Black)));
                } else {
                    spans.push(Span::styled("  ", Style::default().fg(Color::DarkGray).bg(Color::Black)));
                }
            }
            lines.push(Line::from(spans));
        } else {
            // Multi-line block with crisp border lines around each block
            let inner_w = 2 * scale_k - 2;
            let top_bot_block = format!("[{}]", "═".repeat(inner_w));
            let mid_block = format!("║{}║", "█".repeat(inner_w));

            // Sub-row 0 (top line)
            let mut spans_top = Vec::with_capacity(WIDTH);
            for x in 0..WIDTH as i32 {
                if let Some(id) = game.board().cell(x, y) {
                    spans_top.push(Span::styled(top_bot_block.clone(), Style::default().fg(piece_color(id)).bg(Color::Black)));
                } else if active_cells.contains(&(x, y)) {
                    spans_top.push(Span::styled(top_bot_block.clone(), Style::default().fg(piece_color(active_id)).bg(Color::Black)));
                } else {
                    spans_top.push(Span::styled(empty_dot_str.clone(), Style::default().fg(Color::DarkGray).bg(Color::Black)));
                }
            }
            lines.push(Line::from(spans_top));

            // Sub-rows 1..scale_k-1 (middle filled lines)
            for sub in 1..scale_k {
                let mut spans_mid = Vec::with_capacity(WIDTH);
                let current_block = if sub == scale_k - 1 {
                    top_bot_block.clone()
                } else {
                    mid_block.clone()
                };
                for x in 0..WIDTH as i32 {
                    if let Some(id) = game.board().cell(x, y) {
                        spans_mid.push(Span::styled(current_block.clone(), Style::default().fg(piece_color(id)).bg(Color::Black)));
                    } else if active_cells.contains(&(x, y)) {
                        spans_mid.push(Span::styled(current_block.clone(), Style::default().fg(piece_color(active_id)).bg(Color::Black)));
                    } else {
                        spans_mid.push(Span::styled(empty_dot_str.clone(), Style::default().fg(Color::DarkGray).bg(Color::Black)));
                    }
                }
                lines.push(Line::from(spans_mid));
            }
        }
    }

    let board_width = (WIDTH * 2 * scale_k + 2) as u16;
    let board_height = (HEIGHT * scale_k + 2) as u16;
    let board_area = Rect {
        x: area.x + area.width.saturating_sub(board_width) / 2,
        y: area.y + area.height.saturating_sub(board_height) / 2,
        width: board_width.min(area.width),
        height: board_height.min(area.height),
    };

    let board_paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" 2D Classic Tetris "))
        .style(Style::default().bg(Color::Black));

    f.render_widget(board_paragraph, board_area);
}




fn draw_side_panel_widget(f: &mut ratatui::Frame, area: Rect, game: &mut Game) {
    let next_type = game.peek_next();
    let next_cells = next_type.cells(0).to_vec();
    let next_id = next_type.id();

    let block_str = "[]";
    let empty_str = "  ";

    let mut next_lines = Vec::with_capacity(4);
    for py in 0..4i32 {
        let mut spans = Vec::with_capacity(4);
        for px in 0..4i32 {
            if next_cells.contains(&(px, py)) {
                spans.push(Span::styled(block_str, Style::default().fg(piece_color(next_id)).bg(Color::Black)));
            } else {
                spans.push(Span::styled(empty_str, Style::default().bg(Color::Black)));
            }
        }
        next_lines.push(Line::from(spans));
    }

    let side_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // Next preview block
            Constraint::Length(5), // Stats block
            Constraint::Min(8),   // Controls legend
        ])
        .split(area);

    let next_widget = Paragraph::new(next_lines)
        .block(Block::default().borders(Borders::ALL).title(" NEXT "))
        .style(Style::default().bg(Color::Black));
    f.render_widget(next_widget, side_chunks[0]);

    let stats_text = vec![
        Line::from(format!("SCORE : {}", game.score())),
        Line::from(format!("LEVEL : {}", game.level())),
        Line::from(format!("LINES : {}", game.lines_cleared())),
    ];
    let stats_widget = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL).title(" Stats "))
        .style(Style::default().fg(Color::Cyan).bg(Color::Black));
    f.render_widget(stats_widget, side_chunks[1]);

    let controls_text = vec![
        Line::from("CONTROLS:"),
        Line::from("Left / Right : Move"),
        Line::from("Down         : Soft Drop"),
        Line::from("Up           : Rotate"),
        Line::from("Space        : Hard Drop"),
        Line::from("P            : Pause"),
        Line::from("R            : Restart"),
        Line::from("Q / Esc      : Quit"),
    ];
    let controls_widget = Paragraph::new(controls_text)
        .block(Block::default().borders(Borders::ALL).title(" Controls "))
        .style(Style::default().fg(Color::Yellow).bg(Color::Black));
    f.render_widget(controls_widget, side_chunks[2]);
}

fn draw_status_overlay_widget(f: &mut ratatui::Frame, area: Rect, game: &Game) {
    match game.state() {
        GameState::Paused => {
            let popup = Paragraph::new("\n  *** PAUSED ***\n  Press P to resume")
                .block(Block::default().borders(Borders::ALL).title(" Status "))
                .style(Style::default().fg(Color::Yellow).bg(Color::Black).add_modifier(Modifier::BOLD));
            let popup_area = centered_rect(area, 40, 20);
            f.render_widget(Clear, popup_area);
            f.render_widget(popup, popup_area);
        }
        GameState::GameOver => {
            let msg = format!(
                "\n   *** GAME OVER ***\n   Final Score: {}\n   Press R to restart",
                game.score()
            );
            let popup = Paragraph::new(msg)
                .block(Block::default().borders(Borders::ALL).title(" Game Over "))
                .style(Style::default().fg(Color::Red).bg(Color::Black).add_modifier(Modifier::BOLD));
            let popup_area = centered_rect(area, 45, 25);
            f.render_widget(Clear, popup_area);
            f.render_widget(popup, popup_area);
        }
        GameState::Playing => {}
    }
}


fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
