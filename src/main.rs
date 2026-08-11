mod gfx3d;
mod gfx3d_box;

#[cfg(not(target_arch = "wasm32"))]
mod picker;
#[cfg(not(target_arch = "wasm32"))]
mod terminal;
#[cfg(not(target_arch = "wasm32"))]
mod terminal_3d;

#[cfg(not(target_arch = "wasm32"))]
use std::process::ExitCode;

use tetris::battle::BattleState;
use tetris::cli::{self, RendererChoice};



#[cfg(not(target_arch = "wasm32"))]
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mode_arg = match cli::parse_mode_arg(&args) {
        Ok(m) => m,
        Err(msg) => {
            eprintln!("{msg}");
            return ExitCode::FAILURE;
        }
    };

    let renderer_arg = match cli::parse_renderer_arg(&args) {
        Ok(choice) => choice,
        Err(msg) => {
            eprintln!("{msg}");
            return ExitCode::FAILURE;
        }
    };

    let (mode, choice) = match (mode_arg, renderer_arg) {
        (Some(m), Some(r)) => (m, r),
        (Some(m), None) => match picker::pick_renderer() {
            Ok(Some(r)) => (m, r),
            Ok(None) => return ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("{err}");
                return ExitCode::FAILURE;
            }
        },
        _ => match picker::pick_options() {
            Ok(Some((m, r))) => (m, r),
            Ok(None) => return ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("{err}");
                return ExitCode::FAILURE;
            }
        },
    };

    let battle = BattleState::new(mode);

    match choice {
        RendererChoice::Terminal => run_terminal(battle),
        RendererChoice::Gfx3d => run_gfx3d_with_fallback(battle),
        RendererChoice::Terminal3d => run_terminal_3d(battle),
        RendererChoice::Gfx3dBox => run_gfx3d_box_with_fallback(battle),
    }
}

// The WASM build has no CLI/picker, so it needs to offer a renderer choice (2D Neon Grid vs 3D
// Spatial Box) itself, alongside the mode choice — both on one combined options screen.
// macroquad only allows one `Window::from_config` call per module lifetime, so this owns that
// single window and drives both renderers' `run_match` from one shared loop rather than calling
// either renderer's standalone `run_app` (which would each try to open their own window).
#[cfg(target_arch = "wasm32")]
fn main() {
    macroquad::Window::from_config(gfx3d::window_conf(), wasm_app_main());
}

#[cfg(target_arch = "wasm32")]
async fn wasm_app_main() {
    use tetris::menu::{OptionsScreen, RendererKind};

    loop {
        let (renderer, mode) = match OptionsScreen::new().run_until_choice().await {
            Some(rm) => rm,
            None => return, // window closed while on the options screen
        };

        let quit_to_menu = match renderer {
            RendererKind::NeonGrid2D => gfx3d::run_match(mode).await,
            RendererKind::SpatialBox3D => gfx3d_box::run_match(mode).await,
        };
        if !quit_to_menu {
            return; // window closed mid-match
        }
    }
}


#[cfg(not(target_arch = "wasm32"))]
fn run_terminal(battle: BattleState) -> ExitCode {
    match terminal::run_battle(battle) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn run_gfx3d_with_fallback(battle: BattleState) -> ExitCode {
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let mode = battle.mode;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        gfx3d::run_app(Some(mode));
    }));

    std::panic::set_hook(prev_hook);

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => {
            eprintln!("3D rendering unavailable on this system — starting terminal mode instead.");
            run_terminal(battle)
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn run_terminal_3d(battle: BattleState) -> ExitCode {
    match terminal_3d::run_battle(battle) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn run_gfx3d_box_with_fallback(battle: BattleState) -> ExitCode {
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let mode = battle.mode;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        gfx3d_box::run_app(Some(mode));
    }));
    std::panic::set_hook(prev_hook);

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => {
            eprintln!("3D Box rendering unavailable on this system — starting terminal 3D mode instead.");
            run_terminal_3d(battle)
        }
    }
}


