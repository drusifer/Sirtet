mod cli;
mod gfx3d;
mod gfx3d_box;
mod picker;
mod terminal;
mod terminal_3d;

use std::process::ExitCode;

use cli::RendererChoice;
use tetris::game::Game;
use tetris::spatial_game::SpatialGame;

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
        RendererChoice::Terminal3d => run_terminal_3d(),
        RendererChoice::Gfx3dBox => run_gfx3d_box_with_fallback(),
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

fn run_terminal_3d() -> ExitCode {
    match terminal_3d::run(SpatialGame::new()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

fn run_gfx3d_box_with_fallback() -> ExitCode {
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        gfx3d_box::run(SpatialGame::new());
    }));
    std::panic::set_hook(prev_hook);

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => {
            eprintln!("3D Box rendering unavailable on this system — starting terminal 3D mode instead.");
            run_terminal_3d()
        }
    }
}
