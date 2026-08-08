use std::io::{self, Write};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::style::Print;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use crossterm::{cursor, execute, queue};

use crate::cli::RendererChoice;

const OPTIONS: [(RendererChoice, &str); 2] = [
    (RendererChoice::Terminal, "Terminal (classic ANSI rendering)"),
    (RendererChoice::Gfx3d, "3D Accelerated (futuristic GPU rendering)"),
];

/// Shows an interactive startup picker for choosing a renderer.
///
/// Returns `Ok(Some(choice))` if the player selected a renderer, or `Ok(None)` if they quit
/// (Esc/Q) without choosing — the caller should exit cleanly without starting a game.
pub fn pick_renderer() -> io::Result<Option<RendererChoice>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    let result = run(&mut stdout);

    execute!(stdout, cursor::Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    result
}

fn run(stdout: &mut io::Stdout) -> io::Result<Option<RendererChoice>> {
    let mut selected = 0usize;
    loop {
        render(stdout, selected)?;
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Up => {
                    selected = selected.checked_sub(1).unwrap_or(OPTIONS.len() - 1);
                }
                KeyCode::Down => {
                    selected = (selected + 1) % OPTIONS.len();
                }
                KeyCode::Enter => {
                    return Ok(Some(OPTIONS[selected].0));
                }
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                    return Ok(None);
                }
                _ => {}
            }
        }
    }
}

fn render(stdout: &mut io::Stdout, selected: usize) -> io::Result<()> {
    queue!(stdout, Clear(ClearType::All))?;
    queue!(stdout, cursor::MoveTo(0, 0), Print("Choose a renderer:"))?;
    for (i, (_, label)) in OPTIONS.iter().enumerate() {
        let marker = if i == selected { "> " } else { "  " };
        queue!(stdout, cursor::MoveTo(0, 2 + i as u16), Print(format!("{marker}{label}")))?;
    }
    queue!(
        stdout,
        cursor::MoveTo(0, 2 + OPTIONS.len() as u16 + 1),
        Print("Up/Down select, Enter confirm, Esc/Q quit")
    )?;
    stdout.flush()
}
