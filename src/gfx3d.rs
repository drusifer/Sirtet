use macroquad::prelude::*;

use tetris::battle::{BattleState, GameMode, MatchWinner};
use tetris::board::{HEIGHT, WIDTH};
use tetris::camera::{OrbitCamera, ViewCubeGizmo};
use tetris::cpu_ai::CpuAgent;
use tetris::fx::{format_clear_banner, ClearFx, LandingFx, ScoreBanner, FX_DURATION};
use tetris::game::{Game, GameState};
use tetris::menu::{piece_color, resolve_menu_action, Menu, MenuAction, SingleScreen};

const CELL_SIZE: f32 = 0.5;
const BOARD_ORIGIN_X: f32 = -(WIDTH as f32 * CELL_SIZE) / 2.0;
const BOARD_ORIGIN_Y: f32 = (HEIGHT as f32 * CELL_SIZE) / 2.0;

fn cell_world_pos_at(x: i32, y: f32, offset_x: f32) -> Vec3 {
    vec3(
        BOARD_ORIGIN_X + offset_x + x as f32 * CELL_SIZE + CELL_SIZE / 2.0,
        BOARD_ORIGIN_Y - y * CELL_SIZE - CELL_SIZE / 2.0,
        0.0,
    )
}

pub fn window_conf() -> Conf {
    Conf {
        window_title: "Sirtet — Neon Grid".to_owned(),
        window_width: 1024,
        window_height: 768,
        platform: miniquad::conf::Platform {
            linux_backend: miniquad::conf::LinuxBackend::WaylandOnly,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Runs the full standalone app: main menu (mode select) -> match -> pause/game-over -> back to
/// menu, looping until the window closes. `initial_mode` bypasses the main menu on the very
/// first entry when the caller already decided a mode (native CLI/picker); `None` always shows
/// the menu first. For an entry point that needs to offer more than this one renderer (the WASM
/// build, which lets the player switch between the 2D and 3D views), call `run_match` directly
/// from a shared orchestrator instead of this function.
pub fn run_app(initial_mode: Option<GameMode>) {
    macroquad::Window::from_config(window_conf(), run_app_async(initial_mode));
}

async fn run_app_async(initial_mode: Option<GameMode>) {
    let mut pending_mode = initial_mode;
    loop {
        let mode = match pending_mode.take() {
            Some(m) => m,
            None => match Menu::main_menu().run_until_choice().await {
                Some(MenuAction::StartMode(m)) => m,
                _ => return, // window closed while on the main menu
            },
        };

        if !run_match(mode).await {
            return; // window closed
        }
        // else: player chose "Quit to Main Menu" from the pause/game-over menu — loop back.
    }
}

/// Runs one match to completion in `mode`. Returns `true` if the player chose "Quit to Main
/// Menu" (the caller should show a menu and may call this again), `false` if the window was
/// closed. Shared by `run_app`'s own standalone loop above and by external multi-renderer
/// orchestrators (used by the WASM build to offer both this 2D view and the 3D spatial box).
pub async fn run_match(mode: GameMode) -> bool {
    if mode == GameMode::Single {
        amain(Game::new()).await;
        // amain currently always returns via its own Q/Esc quit; treat that as "back to menu"
        // rather than "window closed" — a genuine window-close is caught by the caller's next
        // menu-loop iteration (its own is_quit_requested() check), so nothing is lost.
        true
    } else {
        abattle_main(BattleState::new(mode)).await
    }
}

enum BattleScreen {
    Playing,
    Paused(Menu),
    GameOver(Menu),
}

/// Runs one battle match. Returns `true` if the player chose "Quit to Main Menu" (caller should
/// loop back to the main menu), `false` if the window was closed entirely.
/// Battle-mode per-frame state: P1/P2 input, CPU move, tick, menu-screen transitions. Split out
/// of `abattle_main` (Phase 4, Sprint 8 tech debt) — no behavior change, same mutations in the
/// same order.
fn abattle_update(
    screen: &mut BattleScreen,
    battle: &mut BattleState,
    last_tick: &mut f64,
    cpu_agent: &mut CpuAgent,
    quit_to_menu: &mut bool,
) {
    match screen {
        BattleScreen::Playing => {
            if is_key_pressed(KeyCode::Escape) {
                *screen = BattleScreen::Paused(Menu::pause_menu());
            } else if is_key_pressed(KeyCode::R) {
                *screen = BattleScreen::Paused(Menu::pause_menu_restart_selected());
            } else {
                // P1 Controls
                if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
                    battle.player1.move_left();
                }
                if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
                    battle.player1.move_right();
                }
                if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
                    battle.player1.soft_drop();
                }
                if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
                    battle.player1.rotate();
                }
                if is_key_pressed(KeyCode::Space) {
                    battle.p1_hard_drop();
                }

                // P2 Controls (Local)
                if battle.mode == GameMode::TwoPlayerLocal && is_key_pressed(KeyCode::Enter) {
                    battle.p2_hard_drop();
                }

                let now = get_time();
                let interval = (battle.player1.gravity_interval_ms() as f64 / 1000.0).max(0.001);
                if now - *last_tick >= interval {
                    if battle.mode == GameMode::VsCpu
                        && let Some(ref mut p2) = battle.player2
                        && p2.state() == GameState::Playing
                    {
                        cpu_agent.make_move(p2);
                    }
                    battle.tick();
                    *last_tick = now;
                }

                if battle.winner != MatchWinner::None {
                    *screen = BattleScreen::GameOver(Menu::game_over_menu());
                }
            }
        }
        BattleScreen::Paused(menu) => {
            if is_key_pressed(KeyCode::Escape) {
                *screen = BattleScreen::Playing;
            } else if let Some(action) = menu.update() {
                match action {
                    MenuAction::Resume => *screen = BattleScreen::Playing,
                    MenuAction::Restart => {
                        *battle = BattleState::new(battle.mode);
                        *last_tick = get_time();
                        *screen = BattleScreen::Playing;
                    }
                    MenuAction::QuitToMenu => *quit_to_menu = true,
                    MenuAction::StartMode(_) => {}
                }
            }
        }
        BattleScreen::GameOver(menu) => {
            if let Some(action) = menu.update() {
                match action {
                    MenuAction::Restart => {
                        *battle = BattleState::new(battle.mode);
                        *last_tick = get_time();
                        *screen = BattleScreen::Playing;
                    }
                    MenuAction::QuitToMenu => *quit_to_menu = true,
                    MenuAction::Resume | MenuAction::StartMode(_) => {}
                }
            }
        }
    }
}

/// Battle-mode per-frame drawing: both boards, HUD, winner title, menu overlay. Split out of
/// `abattle_main` (Phase 4, Sprint 8 tech debt) — no behavior change, same draw calls in order.
fn abattle_draw(orbit_cam: &OrbitCamera, battle: &BattleState, screen: &BattleScreen) {
    clear_background(Color::new(0.02, 0.02, 0.07, 1.0));

    set_camera(&orbit_cam.camera_3d());
    draw_board_at(&battle.player1, -3.5);
    if let Some(ref p2) = battle.player2 {
        draw_board_at(p2, 3.5);
    }

    set_default_camera();
    draw_battle_hud(battle);
    if let BattleScreen::GameOver(_) = screen {
        draw_match_winner_title(battle.winner);
    }
    if let BattleScreen::Paused(menu) | BattleScreen::GameOver(menu) = screen {
        menu.draw(screen_width(), screen_height());
    }
}

async fn abattle_main(mut battle: BattleState) -> bool {
    let orbit_cam = OrbitCamera::default_2d_fancy();
    let mut last_tick = get_time();
    let mut cpu_agent = CpuAgent::new();
    let mut screen = BattleScreen::Playing;

    loop {
        if is_quit_requested() {
            return false;
        }

        let mut quit_to_menu = false;

        abattle_update(&mut screen, &mut battle, &mut last_tick, &mut cpu_agent, &mut quit_to_menu);
        abattle_draw(&orbit_cam, &battle, &screen);

        // Cross a frame boundary before returning, even on quit — otherwise the same Enter
        // press that confirmed "Quit to Main Menu" is still "just pressed" on the next screen's
        // very first input poll (see Menu::run_until_choice for the same class of bug), so the
        // options screen shown right after this would instantly auto-confirm its defaults
        // instead of ever being shown to the player.
        next_frame().await;
        if quit_to_menu {
            return true;
        }
    }
}



/// Single Player per-frame state: gameplay input/tick, menu-screen transitions, FX triggers.
/// Split out of `amain` (Phase 4, Sprint 8 tech debt) purely to make the ~150-line loop body
/// readable in two pieces — no behavior change, same mutations in the same order.
#[allow(clippy::too_many_arguments)]
fn amain_update(
    screen: &mut SingleScreen,
    game: &mut Game,
    now: f64,
    last_tick: &mut f64,
    prev_active_y: &mut i32,
    landing_fx: &mut LandingFx,
    clear_fx: &mut ClearFx,
    banner: &mut ScoreBanner,
    orbit_cam: &mut OrbitCamera,
    quit_to_menu: &mut bool,
) {
    match screen {
        SingleScreen::Playing => {
            if is_key_pressed(KeyCode::Escape) {
                *screen = SingleScreen::Paused(Menu::pause_menu());
            } else if is_key_pressed(KeyCode::R) {
                *screen = SingleScreen::Paused(Menu::pause_menu_restart_selected());
            } else {
                handle_playing_input(game);

                let interval = (game.gravity_interval_ms() as f64 / 1000.0).max(0.001);
                if now - *last_tick >= interval {
                    game.tick();
                    *last_tick = now;

                    // Detect genuine piece lock landing
                    if game.active().y < *prev_active_y {
                        landing_fx.trigger(now);
                        orbit_cam.add_shake(0.35);
                    }
                }
                *prev_active_y = game.active().y;

                // Detect line clear event immediately on lock
                let cleared = game.last_lines_cleared();
                if cleared > 0 && clear_fx.start_time.is_none() {
                    clear_fx.trigger(now, cleared);
                    orbit_cam.add_shake(0.85);

                    let msg = format_clear_banner(cleared, false);
                    banner.trigger(msg.to_string(), now);
                }

                if game.state() == GameState::GameOver {
                    *screen = SingleScreen::GameOver(Menu::game_over_menu());
                }
            }
        }
        SingleScreen::Paused(menu) => {
            if is_key_pressed(KeyCode::Escape) {
                *screen = SingleScreen::Playing;
            } else if let Some(action) = menu.update() {
                let restart = matches!(action, MenuAction::Restart);
                if resolve_menu_action(action, quit_to_menu) {
                    if restart {
                        *game = Game::new();
                        *landing_fx = LandingFx::new();
                        *clear_fx = ClearFx::new();
                        *banner = ScoreBanner::new();
                        *prev_active_y = game.active().y;
                        *last_tick = get_time();
                    }
                    *screen = SingleScreen::Playing;
                }
            }
        }
        SingleScreen::GameOver(menu) => {
            if let Some(action) = menu.update() {
                let restart = matches!(action, MenuAction::Restart);
                if resolve_menu_action(action, quit_to_menu) {
                    if restart {
                        *game = Game::new();
                        *landing_fx = LandingFx::new();
                        *clear_fx = ClearFx::new();
                        *banner = ScoreBanner::new();
                        *prev_active_y = game.active().y;
                        *last_tick = get_time();
                    }
                    *screen = SingleScreen::Playing;
                }
            }
        }
    }
}

/// Single Player per-frame drawing: board/HUD/FX/menu overlay + camera/viewcube update. Split out
/// of `amain` (Phase 4, Sprint 8 tech debt) — no behavior change, same draw calls in the same
/// order (drawing and the viewcube's camera-drag handling were already interleaved in the
/// original code, so this function does both rather than pretending they're separable).
#[allow(clippy::too_many_arguments)]
fn amain_draw(
    screen: &SingleScreen,
    game: &mut Game,
    now: f64,
    landing_fx: &mut LandingFx,
    clear_fx: &mut ClearFx,
    banner: &mut ScoreBanner,
    orbit_cam: &mut OrbitCamera,
    viewcube: &mut ViewCubeGizmo,
) {
    clear_background(Color::new(0.02, 0.02, 0.07, 1.0));

    set_camera(&orbit_cam.camera_3d());
    draw_board(game);

    // Draw 3D Landing shockwave pulses on 2D board
    landing_fx.draw_2d_shockwave(now, BOARD_ORIGIN_Y, HEIGHT, CELL_SIZE, WIDTH);

    // Draw 3D Line Clear shockwave expansion rings
    if let Some(start) = clear_fx.start_time {
        let elapsed = now - start;
        if elapsed < FX_DURATION {
            let t = (elapsed / FX_DURATION) as f32;
            let alpha = (1.0 - t) * 0.95;
            let gold = Color::new(1.0, 0.9, 0.2, alpha);
            let cyan = Color::new(0.0, 0.95, 1.0, alpha);

            let center_y = BOARD_ORIGIN_Y - (HEIGHT as f32 * CELL_SIZE) / 2.0;
            let w = (WIDTH as f32 * CELL_SIZE) * (1.0 + t * 0.5);
            let h = (HEIGHT as f32 * CELL_SIZE) * (1.0 + t * 0.5);

            draw_cube(vec3(0.0, center_y, 0.0), vec3(w, h, CELL_SIZE), None, Color::new(1.0, 0.85, 0.1, alpha * 0.25));
            draw_cube_wires(vec3(0.0, center_y, 0.0), vec3(w, h, CELL_SIZE), gold);
            draw_cube_wires(vec3(0.0, center_y, 0.0), vec3(w * 1.12, h * 1.12, CELL_SIZE * 1.5), cyan);
        } else {
            clear_fx.start_time = None;
        }
    }

    set_default_camera();
    draw_hud(game);

    // Draw Line Clear Full-Screen Flash Burst
    clear_fx.draw_flash_burst(now);

    // Draw Floating Score Banner
    banner.draw(now);

    if let SingleScreen::Paused(menu) | SingleScreen::GameOver(menu) = screen {
        menu.draw(screen_width(), screen_height());
    }

    let reset_requested = viewcube.update_and_draw(&mut orbit_cam.yaw, &mut orbit_cam.pitch);
    if reset_requested {
        *orbit_cam = OrbitCamera::default_2d_fancy();
    }

    orbit_cam.update(viewcube.is_dragging);
}

async fn amain(mut game: Game) {
    let mut orbit_cam = OrbitCamera::default_2d_fancy();
    let mut viewcube = ViewCubeGizmo::new(0.12);
    let mut last_tick = get_time();

    let mut landing_fx = LandingFx::new();
    let mut clear_fx = ClearFx::new();
    let mut banner = ScoreBanner::new();

    let mut prev_active_y = game.active().y;
    let mut screen = SingleScreen::Playing;

    loop {
        let now = get_time();

        if is_quit_requested() {
            return;
        }

        let mut quit_to_menu = false;

        amain_update(
            &mut screen,
            &mut game,
            now,
            &mut last_tick,
            &mut prev_active_y,
            &mut landing_fx,
            &mut clear_fx,
            &mut banner,
            &mut orbit_cam,
            &mut quit_to_menu,
        );

        amain_draw(
            &screen,
            &mut game,
            now,
            &mut landing_fx,
            &mut clear_fx,
            &mut banner,
            &mut orbit_cam,
            &mut viewcube,
        );

        // See abattle_main's quit_to_menu handling — always cross a frame boundary before
        // returning so the next screen doesn't see a stale "just pressed" Enter.
        next_frame().await;
        if quit_to_menu {
            return;
        }
    }
}

fn handle_playing_input(game: &mut Game) {
    if is_key_pressed(KeyCode::Left) {
        game.move_left();
    }
    if is_key_pressed(KeyCode::Right) {
        game.move_right();
    }
    if is_key_pressed(KeyCode::Down) {
        game.soft_drop();
    }
    if is_key_pressed(KeyCode::Up) {
        game.rotate();
    }
    if is_key_pressed(KeyCode::Space) {
        game.hard_drop();
    }
}

fn draw_board(game: &Game) {
    draw_board_at(game, 0.0);
}

fn draw_board_at(game: &Game, offset_x: f32) {
    draw_faint_grid_and_border_at(offset_x);

    for y in 0..HEIGHT as i32 {
        for x in 0..WIDTH as i32 {
            if let Some(id) = game.board().cell(x, y) {
                draw_neon_cell_at(x, y as f32, id, offset_x);
            }
        }
    }

    if game.state() != GameState::GameOver {
        let id = game.active().piece_type.id();
        let base = game.active();
        let cells = base.cells();
        draw_active_piece_glow(&cells, id, offset_x);
        for (x, y) in cells {
            draw_neon_cell_at(x, y as f32, id, offset_x);
        }
    }
}

/// Draws a single soft halo sized to the active piece's bounding box, rather than
/// per-cell glow quads — overlapping per-cell glows double up alpha along the seams
/// between adjacent cells of the same piece, producing a bright halo artifact there.
fn draw_active_piece_glow(cells: &[(i32, i32); 4], id: u8, offset_x: f32) {
    let color = piece_color(id);
    let glow = Color::new(color.r, color.g, color.b, 0.12);
    let min_x = cells.iter().map(|(x, _)| *x).min().unwrap();
    let max_x = cells.iter().map(|(x, _)| *x).max().unwrap();
    let min_y = cells.iter().map(|(_, y)| *y).min().unwrap();
    let max_y = cells.iter().map(|(_, y)| *y).max().unwrap();
    let center_x = (min_x + max_x) as f32 / 2.0;
    let center_y = (min_y + max_y) as f32 / 2.0;
    let pos = vec3(
        BOARD_ORIGIN_X + offset_x + center_x * CELL_SIZE + CELL_SIZE / 2.0,
        BOARD_ORIGIN_Y - center_y * CELL_SIZE - CELL_SIZE / 2.0,
        0.0,
    );
    let w = (max_x - min_x + 1) as f32 * CELL_SIZE * 1.05;
    let h = (max_y - min_y + 1) as f32 * CELL_SIZE * 1.05;
    draw_cube(pos, vec3(w, h, CELL_SIZE * 0.5), None, glow);
}

fn draw_faint_grid_and_border_at(offset_x: f32) {
    let grid_color = Color::new(0.15, 0.25, 0.45, 0.35);
    let border_color = Color::new(0.0, 0.85, 1.0, 0.8);

    let left = BOARD_ORIGIN_X + offset_x;
    let right = left + WIDTH as f32 * CELL_SIZE;
    let top = BOARD_ORIGIN_Y;
    let bottom = BOARD_ORIGIN_Y - HEIGHT as f32 * CELL_SIZE;
    let center_x = (left + right) / 2.0;
    let center_y = (top + bottom) / 2.0;
    let board_w = WIDTH as f32 * CELL_SIZE;
    let board_h = HEIGHT as f32 * CELL_SIZE;

    // Translucent Back Wall
    draw_cube(vec3(center_x, center_y, -CELL_SIZE * 0.3), vec3(board_w, board_h, 0.04), None, Color::new(0.02, 0.12, 0.28, 0.35));
    // Translucent Left Side Wall
    draw_cube(vec3(left, center_y, 0.0), vec3(0.04, board_h, CELL_SIZE * 0.8), None, Color::new(0.0, 0.4, 0.7, 0.28));
    // Translucent Right Side Wall
    draw_cube(vec3(right, center_y, 0.0), vec3(0.04, board_h, CELL_SIZE * 0.8), None, Color::new(0.0, 0.4, 0.7, 0.28));
    // Translucent Bottom Floor
    draw_cube(vec3(center_x, bottom, 0.0), vec3(board_w, 0.04, CELL_SIZE * 0.8), None, Color::new(0.0, 0.5, 0.85, 0.38));

    for x in 0..=WIDTH {
        let gx = left + x as f32 * CELL_SIZE;
        draw_line_3d(vec3(gx, top, 0.0), vec3(gx, bottom, 0.0), grid_color);
    }
    for y in 0..=HEIGHT {
        let gy = BOARD_ORIGIN_Y - y as f32 * CELL_SIZE;
        draw_line_3d(vec3(left, gy, 0.0), vec3(right, gy, 0.0), grid_color);
    }

    draw_line_3d(vec3(left, top, 0.0), vec3(right, top, 0.0), border_color);
    draw_line_3d(vec3(right, top, 0.0), vec3(right, bottom, 0.0), border_color);
    draw_line_3d(vec3(right, bottom, 0.0), vec3(left, bottom, 0.0), border_color);
    draw_line_3d(vec3(left, top, 0.0), vec3(left, bottom, 0.0), border_color);
}

fn draw_neon_cell_at(x: i32, y: f32, id: u8, offset_x: f32) {
    let pos = cell_world_pos_at(x, y, offset_x);
    let color = piece_color(id);
    let size = vec3(CELL_SIZE, CELL_SIZE, CELL_SIZE * 0.5);
    draw_cube(pos, size, None, color);
    draw_cube_wires(pos, size, WHITE);
}

const HUD_COLOR: Color = Color::new(0.6, 0.95, 1.0, 1.0);

fn draw_hud(game: &mut Game) {
    draw_next_preview(game);
    draw_stats(game);
    draw_controls_legend();
}

fn draw_next_preview(game: &mut Game) {
    draw_text("NEXT", 20.0, 30.0, 24.0, HUD_COLOR);
    let next_type = game.peek_next();
    let color = piece_color(next_type.id());
    let cell_px = 18.0;
    let origin_x = 20.0;
    let origin_y = 40.0;
    for (x, y) in next_type.cells(0) {
        let px = origin_x + x as f32 * cell_px;
        let py = origin_y + y as f32 * cell_px;
        draw_rectangle(px, py, cell_px - 2.0, cell_px - 2.0, color);
    }
}

fn draw_stats(game: &Game) {
    let base_y = screen_height() - 90.0;
    draw_text(format!("SCORE: {}", game.score()), 20.0, base_y, 24.0, HUD_COLOR);
    draw_text(format!("LEVEL: {}", game.level()), 20.0, base_y + 28.0, 24.0, HUD_COLOR);
    draw_text(format!("LINES: {}", game.lines_cleared()), 20.0, base_y + 56.0, 24.0, HUD_COLOR);
}

fn draw_controls_legend() {
    let lines = [
        "CONTROLS:",
        "<-/-> move",
        "Down  soft drop",
        "Up    rotate",
        "Space hard drop",
        "ViewCube Drag   Orbit 3D Gizmo",
        "Mouse Drag/IJKL Orbit View",
        "Scroll / +/-    Zoom View",
        "H / C           Home Reset View",
        "1-5             Camera Presets",
        "Esc   Pause Menu",
        "R     Pause (Restart)",
    ];
    let origin_x = screen_width() - 250.0;
    for (i, line) in lines.iter().enumerate() {
        draw_text(line, origin_x, 180.0 + i as f32 * 22.0, 18.0, HUD_COLOR);
    }
}


/// Draws just the winner headline above the game-over menu box (the menu itself provides the
/// Restart/Main Menu actions, so this only needs to say who won).
fn draw_match_winner_title(winner: MatchWinner) {
    let title = match winner {
        MatchWinner::Player1 => "PLAYER 1 WINS!",
        MatchWinner::Player2 => "PLAYER 2 WINS!",
        MatchWinner::Cpu => "CPU WINS!",
        MatchWinner::None => return,
    };
    let cx = screen_width() / 2.0;
    let cy = screen_height() / 2.0;
    draw_text(title, cx - 110.0, cy - 130.0, 36.0, Color::new(1.0, 0.9, 0.1, 1.0));
}

fn draw_battle_hud(battle: &BattleState) {
    let p1 = &battle.player1;
    // P1 Panel (Left)
    draw_text("PLAYER 1", 40.0, 50.0, 24.0, Color::new(0.0, 0.95, 1.0, 1.0));
    draw_text(format!("Score: {}", p1.score()), 40.0, 80.0, 18.0, WHITE);
    draw_text(format!("Level: {}", p1.level()), 40.0, 105.0, 18.0, WHITE);
    draw_text(format!("Lines: {}", p1.lines_cleared()), 40.0, 130.0, 18.0, WHITE);
    if p1.pending_garbage() > 0 {
        draw_text(format!("ATTACK QUEUE: +{}", p1.pending_garbage()), 40.0, 160.0, 18.0, Color::new(1.0, 0.2, 0.3, 1.0));
    }

    // P2 / CPU Panel (Right)
    if let Some(ref p2) = battle.player2 {
        let p2_title = match battle.mode {
            GameMode::VsCpu => "CPU OPPONENT",
            _ => "PLAYER 2",
        };
        let rx = screen_width() - 240.0;
        draw_text(p2_title, rx, 50.0, 24.0, Color::new(1.0, 0.6, 0.05, 1.0));
        draw_text(format!("Score: {}", p2.score()), rx, 80.0, 18.0, WHITE);
        draw_text(format!("Level: {}", p2.level()), rx, 105.0, 18.0, WHITE);
        draw_text(format!("Lines: {}", p2.lines_cleared()), rx, 130.0, 18.0, WHITE);
        if p2.pending_garbage() > 0 {
            draw_text(format!("ATTACK QUEUE: +{}", p2.pending_garbage()), rx, 160.0, 18.0, Color::new(1.0, 0.2, 0.3, 1.0));
        }
    }


    // Controls at bottom
    let p2_controls = match battle.mode {
        GameMode::VsCpu => "P2: CPU AI (Auto)",
        _ => "P2: Enter (Hard Drop)",
    };
    draw_text("P1: WASD / Space (Hard Drop)", 40.0, screen_height() - 40.0, 16.0, HUD_COLOR);
    draw_text(p2_controls, screen_width() - 240.0, screen_height() - 40.0, 16.0, HUD_COLOR);
    draw_text("Esc: Pause Menu | R: Pause (Restart)", screen_width() / 2.0 - 140.0, screen_height() - 20.0, 16.0, WHITE);
}


