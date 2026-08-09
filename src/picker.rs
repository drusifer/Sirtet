use std::io::{self, Write};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::style::Print;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use crossterm::{cursor, execute, queue};

use tetris::battle::GameMode;
use tetris::cli::RendererChoice;




const RENDERER_OPTIONS: [(RendererChoice, &str); 4] = [
    (RendererChoice::Terminal, "Terminal 2D (Classic ANSI rendering)"),
    (RendererChoice::Gfx3d, "Fancy GPU 2D (Futuristic GPU rendering)"),
    (RendererChoice::Terminal3d, "Terminal 3D Box (Isometric ANSI wireframe 3D Tetris)"),
    (RendererChoice::Gfx3dBox, "Fancy GPU 3D Box (Macroquad GPU 3D spatial Tetris)"),
];

const MODE_OPTIONS: [(GameMode, &str); 3] = [
    (GameMode::Single, "Single Player (Classic Solo Mode)"),
    (GameMode::TwoPlayerLocal, "Local 2-Player Battle (1v1 Split-screen)"),
    (GameMode::VsCpu, "VS CPU Opponent (1 Player vs Computer AI)"),
];

/// Shows an interactive startup picker for choosing game mode and renderer.
pub fn pick_options() -> io::Result<Option<(GameMode, RendererChoice)>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    let mode = run_picker(&mut stdout, "Choose Game Mode:", &MODE_OPTIONS)?;
    let result = if let Some(m) = mode {
        let renderer = run_picker(&mut stdout, "Choose Renderer:", &RENDERER_OPTIONS)?;
        renderer.map(|r| (m, r))
    } else {
        None
    };

    execute!(stdout, cursor::Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(result)
}

pub fn pick_renderer() -> io::Result<Option<RendererChoice>> {
    if let Some((_, renderer)) = pick_options()? {
        Ok(Some(renderer))
    } else {
        Ok(None)
    }
}

fn run_picker<T: Copy>(
    stdout: &mut io::Stdout,
    title: &str,
    options: &[(T, &str)],
) -> io::Result<Option<T>> {
    let mut selected = 0usize;
    loop {
        render(stdout, title, options, selected)?;
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Up => {
                    selected = selected.checked_sub(1).unwrap_or(options.len() - 1);
                }
                KeyCode::Down => {
                    selected = (selected + 1) % options.len();
                }
                KeyCode::Enter => {
                    return Ok(Some(options[selected].0));
                }
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                    return Ok(None);
                }
                _ => {}
            }
        }
    }
}

fn render<T>(
    stdout: &mut io::Stdout,
    title: &str,
    options: &[(T, &str)],
    selected: usize,
) -> io::Result<()> {
    queue!(stdout, Clear(ClearType::All))?;
    queue!(stdout, cursor::MoveTo(0, 0), Print(title))?;
    for (i, (_, label)) in options.iter().enumerate() {
        let marker = if i == selected { "> " } else { "  " };
        queue!(stdout, cursor::MoveTo(0, 2 + i as u16), Print(format!("{marker}{label}")))?;
    }
    queue!(
        stdout,
        cursor::MoveTo(0, 2 + options.len() as u16 + 1),
        Print("Up/Down select, Enter confirm, Esc/Q quit")
    )?;
    stdout.flush()
}
