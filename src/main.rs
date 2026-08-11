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
#[cfg(target_arch = "wasm32")]
use tetris::battle::GameMode;
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

#[cfg(target_arch = "wasm32")]
fn main() {
    let battle = BattleState::new(GameMode::VsCpu);
    gfx3d::run_battle(battle);
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
    let battle_clone = battle.clone();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        gfx3d::run_battle(battle_clone);
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
    let battle_clone = battle.clone();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        gfx3d_box::run_battle(battle_clone);
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


