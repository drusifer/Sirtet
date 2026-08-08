mod cli;
mod gfx3d;
mod picker;
mod terminal;

use std::process::ExitCode;

use cli::RendererChoice;
use tetris::game::Game;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let choice = match cli::parse_renderer_arg(&args) {
        Ok(choice) => choice,
        Err(msg) => {
            eprintln!("{msg}");
            return ExitCode::FAILURE;
        }
    };

    let choice = match choice {
        Some(choice) => choice,
        None => match picker::pick_renderer() {
            Ok(Some(choice)) => choice,
            Ok(None) => return ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("{err}");
                return ExitCode::FAILURE;
            }
        },
    };

    match choice {
        RendererChoice::Terminal => run_terminal(),
        RendererChoice::Gfx3d => run_gfx3d_with_fallback(),
    }
}

fn run_terminal() -> ExitCode {
    match terminal::run(Game::new()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

/// Attempts the 3D renderer; on any failure (GPU/window init, per US-13 — macroquad reports
/// this via panic rather than a `Result`, so `catch_unwind` is the only interception point
/// available, per ARCHITECTURE.md decision #9) falls back to terminal mode instead of
/// crashing. The default panic hook is suppressed for the duration so the player sees the
/// one-line fallback message instead of a raw panic/backtrace dump.
fn run_gfx3d_with_fallback() -> ExitCode {
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        gfx3d::run(Game::new());
    }));
    std::panic::set_hook(prev_hook);

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => {
            eprintln!("3D rendering unavailable on this system — starting terminal mode instead.");
            run_terminal()
        }
    }
}
