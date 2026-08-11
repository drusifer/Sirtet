use macroquad::prelude::*;

use tetris::battle::{GameMode, MatchWinner};
use tetris::camera::{OrbitCamera, ViewCubeGizmo};
use tetris::fx::{format_clear_banner, ClearFx, LandingFx, ScoreBanner, FX_DURATION};
use tetris::menu::{Menu, MenuAction, SingleScreen};
use tetris::spatial_game::{
    Axis, GameState, SpatialGame, BOX_DEPTH, BOX_HEIGHT, BOX_WIDTH,
};

const CUBE_SIZE: f32 = 0.85;
const ORIGIN_X: f32 = -(BOX_WIDTH as f32 * CUBE_SIZE) / 2.0;
const ORIGIN_Y: f32 = (BOX_HEIGHT as f32 * CUBE_SIZE) / 2.0;
const ORIGIN_Z: f32 = -(BOX_DEPTH as f32 * CUBE_SIZE) / 2.0;

fn piece_color(id: u8) -> Color {
    match id {
        1 => Color::new(0.0, 0.95, 1.0, 1.0),  // Cyan
        2 => Color::new(1.0, 0.92, 0.1, 1.0),  // Yellow
        3 => Color::new(0.85, 0.1, 1.0, 1.0),  // Magenta
        4 => Color::new(0.1, 1.0, 0.4, 1.0),   // Green
        5 => Color::new(1.0, 0.15, 0.35, 1.0), // Red
        6 => Color::new(0.2, 0.45, 1.0, 1.0),  // Blue
        7 => Color::new(1.0, 0.6, 0.05, 1.0),  // Orange
        _ => WHITE,
    }
}

#[allow(dead_code)]
fn block_world_pos(x: i8, y: i8, z: i8) -> Vec3 {
    block_world_pos_at(x, y, z, 0.0)
}

pub fn window_conf() -> Conf {
    Conf {
        window_title: "Sirtet — 3D Spatial Box".to_owned(),
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
/// menu, looping until the window closes. See `gfx3d::run_app` for the `initial_mode` contract
/// — identical here. For a multi-renderer orchestrator (the WASM build), call `run_match`
/// directly instead.
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
/// Menu", `false` if the window was closed. See `gfx3d::run_match` — same contract.
pub async fn run_match(mode: GameMode) -> bool {
    if mode == GameMode::Single {
        amain(SpatialGame::new()).await;
        true
    } else {
        abattle_main(mode).await
    }
}

enum BattleScreen {
    Playing,
    Paused(Menu),
    GameOver(Menu, MatchWinner),
}

async fn abattle_main(mode: GameMode) -> bool {
    let orbit_cam = OrbitCamera::default_3d_box();
    let mut p1_game = SpatialGame::new();
    let mut p2_game = SpatialGame::new();
    let mut last_tick = get_time();
    let mut screen = BattleScreen::Playing;

    loop {
        if is_quit_requested() {
            return false;
        }

        let mut quit_to_menu = false;

        match &mut screen {
            BattleScreen::Playing => {
                if is_key_pressed(KeyCode::Escape) {
                    screen = BattleScreen::Paused(Menu::pause_menu());
                } else if is_key_pressed(KeyCode::R) {
                    screen = BattleScreen::Paused(Menu::pause_menu_restart_selected());
                } else {
                    // P1 Controls
                    if is_key_pressed(KeyCode::A) | is_key_pressed(KeyCode::Left) { p1_game.move_x(-1); }
                    if is_key_pressed(KeyCode::D) | is_key_pressed(KeyCode::Right) { p1_game.move_x(1); }
                    if is_key_pressed(KeyCode::W) | is_key_pressed(KeyCode::Up) { p1_game.move_y(-1); }
                    if is_key_pressed(KeyCode::S) | is_key_pressed(KeyCode::Down) { p1_game.move_y(1); }
                    if is_key_pressed(KeyCode::X) { p1_game.rotate(Axis::X); }
                    if is_key_pressed(KeyCode::Y) { p1_game.rotate(Axis::Y); }
                    if is_key_pressed(KeyCode::Z) { p1_game.rotate(Axis::Z); }
                    if is_key_pressed(KeyCode::Space) { p1_game.hard_drop(); }

                    // P2 Controls (Local)
                    if mode == GameMode::TwoPlayerLocal && is_key_pressed(KeyCode::Enter) {
                        p2_game.hard_drop();
                    }

                    let now = get_time();
                    let interval = (tetris::spatial_game::spatial_gravity_interval_ms(p1_game.level) as f64) / 1000.0;
                    if now - last_tick >= interval {
                        if mode == GameMode::VsCpu && macroquad::rand::gen_range(0, 10) < 3 {
                            p2_game.move_x(if macroquad::rand::gen_range(0, 2) == 0 { 1 } else { -1 });
                        }
                        p1_game.tick();
                        p2_game.tick();
                        last_tick = now;
                    }

                    let p1_dead = p1_game.state == GameState::GameOver;
                    let p2_dead = p2_game.state == GameState::GameOver;
                    if p1_dead || p2_dead {
                        let winner = if p1_dead && !p2_dead {
                            if mode == GameMode::VsCpu { MatchWinner::Cpu } else { MatchWinner::Player2 }
                        } else if p2_dead && !p1_dead {
                            MatchWinner::Player1
                        } else {
                            MatchWinner::None
                        };
                        screen = BattleScreen::GameOver(Menu::game_over_menu(), winner);
                    }
                }
            }
            BattleScreen::Paused(menu) => {
                if is_key_pressed(KeyCode::Escape) {
                    screen = BattleScreen::Playing;
                } else if let Some(action) = menu.update() {
                    match action {
                        MenuAction::Resume => screen = BattleScreen::Playing,
                        MenuAction::Restart => {
                            p1_game = SpatialGame::new();
                            p2_game = SpatialGame::new();
                            last_tick = get_time();
                            screen = BattleScreen::Playing;
                        }
                        MenuAction::QuitToMenu => quit_to_menu = true,
                        MenuAction::StartMode(_) => {}
                    }
                }
            }
            BattleScreen::GameOver(menu, _winner) => {
                if let Some(action) = menu.update() {
                    match action {
                        MenuAction::Restart => {
                            p1_game = SpatialGame::new();
                            p2_game = SpatialGame::new();
                            last_tick = get_time();
                            screen = BattleScreen::Playing;
                        }
                        MenuAction::QuitToMenu => quit_to_menu = true,
                        MenuAction::Resume | MenuAction::StartMode(_) => {}
                    }
                }
            }
        }

        clear_background(Color::new(0.02, 0.02, 0.07, 1.0));

        set_camera(&orbit_cam.camera_3d());
        draw_bounding_box_well_at(-3.5);
        draw_spatial_grid_at(&p1_game, -3.5);

        draw_bounding_box_well_at(3.5);
        draw_spatial_grid_at(&p2_game, 3.5);

        set_default_camera();
        draw_battle_spatial_hud(&p1_game, &p2_game, mode);
        if let BattleScreen::GameOver(_, winner) = &screen {
            draw_spatial_match_winner_title(*winner);
        }
        if let BattleScreen::Paused(menu) | BattleScreen::GameOver(menu, _) = &screen {
            menu.draw(screen_width(), screen_height());
        }

        // See gfx3d.rs's abattle_main — always cross a frame boundary before returning so the
        // next screen doesn't see a stale "just pressed" Enter.
        next_frame().await;
        if quit_to_menu {
            return true;
        }
    }
}



async fn amain(mut game: SpatialGame) {
    let mut orbit_cam = OrbitCamera::default_3d_box();
    let mut viewcube = ViewCubeGizmo::new(0.35);
    let mut last_tick = get_time();

    let mut landing_fx = LandingFx::new();
    let mut clear_fx = ClearFx::new();
    let mut banner = ScoreBanner::new();

    let mut prev_active_z = game.active_piece.z;
    let mut screen = SingleScreen::Playing;

    loop {
        let now = get_time();

        if is_quit_requested() {
            return;
        }

        let mut quit_to_menu = false;

        match &mut screen {
            SingleScreen::Playing => {
                if is_key_pressed(KeyCode::Escape) {
                    screen = SingleScreen::Paused(Menu::pause_menu());
                } else if is_key_pressed(KeyCode::R) {
                    screen = SingleScreen::Paused(Menu::pause_menu_restart_selected());
                } else {
                    handle_playing_input(&mut game);

                    let interval = (tetris::spatial_game::spatial_gravity_interval_ms(game.level) as f64) / 1000.0;
                    if now - last_tick >= interval {
                        game.tick();
                        last_tick = now;

                        // Detect genuine 3D piece lock landing
                        if game.active_piece.z < prev_active_z {
                            landing_fx.trigger(now);
                            orbit_cam.add_shake(0.35);
                        }
                    }
                    prev_active_z = game.active_piece.z;

                    // Detect 3D layer clear event immediately on lock
                    if game.last_layers_cleared > 0 && clear_fx.start_time.is_none() {
                        let count = game.last_layers_cleared;
                        clear_fx.trigger(now, count);
                        orbit_cam.add_shake(0.85);

                        let msg = format_clear_banner(count, true);
                        banner.trigger(msg.to_string(), now);
                    }

                    if game.state == GameState::GameOver {
                        screen = SingleScreen::GameOver(Menu::game_over_menu());
                    }
                }
            }
            SingleScreen::Paused(menu) => {
                if is_key_pressed(KeyCode::Escape) {
                    screen = SingleScreen::Playing;
                } else if let Some(action) = menu.update() {
                    match action {
                        MenuAction::Resume => screen = SingleScreen::Playing,
                        MenuAction::Restart => {
                            game = SpatialGame::new();
                            landing_fx = LandingFx::new();
                            clear_fx = ClearFx::new();
                            banner = ScoreBanner::new();
                            prev_active_z = game.active_piece.z;
                            last_tick = get_time();
                            screen = SingleScreen::Playing;
                        }
                        MenuAction::QuitToMenu => quit_to_menu = true,
                        MenuAction::StartMode(_) => {}
                    }
                }
            }
            SingleScreen::GameOver(menu) => {
                if let Some(action) = menu.update() {
                    match action {
                        MenuAction::Restart => {
                            game = SpatialGame::new();
                            landing_fx = LandingFx::new();
                            clear_fx = ClearFx::new();
                            banner = ScoreBanner::new();
                            prev_active_z = game.active_piece.z;
                            last_tick = get_time();
                            screen = SingleScreen::Playing;
                        }
                        MenuAction::QuitToMenu => quit_to_menu = true,
                        MenuAction::Resume | MenuAction::StartMode(_) => {}
                    }
                }
            }
        }

        clear_background(Color::new(0.02, 0.02, 0.07, 1.0));

        set_camera(&orbit_cam.camera_3d());
        draw_bounding_box_well_at(0.0);
        draw_spatial_grid_at(&game, 0.0);


        // Draw 3D Landing shockwave pulses
        let floor_y = ORIGIN_Y - BOX_HEIGHT as f32 * CUBE_SIZE;
        landing_fx.draw_3d_shockwave(now, floor_y, BOX_WIDTH, CUBE_SIZE);

        // Draw 3D Layer Clear shockwave burst rings
        if let Some(start) = clear_fx.start_time {
            let elapsed = now - start;
            if elapsed < FX_DURATION {
                let t = (elapsed / FX_DURATION) as f32;
                let alpha = (1.0 - t) * 0.95;
                let gold = Color::new(1.0, 0.9, 0.2, alpha);
                let cyan = Color::new(0.0, 0.95, 1.0, alpha);

                let center_y = ORIGIN_Y - (BOX_HEIGHT as f32 * CUBE_SIZE) / 2.0;
                let size = (BOX_WIDTH as f32 * CUBE_SIZE) * (1.0 + t * 0.6);

                draw_cube(vec3(0.0, center_y, 0.0), vec3(size, CUBE_SIZE * 2.0, size), None, Color::new(1.0, 0.8, 0.1, alpha * 0.3));
                draw_cube_wires(vec3(0.0, center_y, 0.0), vec3(size, CUBE_SIZE * 2.0, size), gold);
                draw_cube_wires(vec3(0.0, center_y, 0.0), vec3(size * 1.15, CUBE_SIZE * 2.5, size * 1.15), cyan);
            } else {
                clear_fx.start_time = None;
            }
        }

        set_default_camera();
        draw_hud(&game);

        // Draw Layer Clear Full-Screen Flash Burst
        clear_fx.draw_flash_burst(now);

        // Draw Layer Clear Score Banner
        banner.draw(now);

        if let SingleScreen::Paused(menu) | SingleScreen::GameOver(menu) = &screen {
            menu.draw(screen_width(), screen_height());
        }

        let reset_requested = viewcube.update_and_draw(&mut orbit_cam.yaw, &mut orbit_cam.pitch);
        if reset_requested {
            orbit_cam = OrbitCamera::default_3d_box();
        }

        orbit_cam.update(viewcube.is_dragging);

        // See abattle_main's quit_to_menu handling — always cross a frame boundary before
        // returning so the next screen doesn't see a stale "just pressed" Enter.
        next_frame().await;
        if quit_to_menu {
            return;
        }
    }
}

fn handle_playing_input(game: &mut SpatialGame) {
    if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
        game.move_x(-1);
    }
    if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
        game.move_x(1);
    }
    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
        game.move_y(-1);
    }
    if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
        game.move_y(1);
    }
    if is_key_pressed(KeyCode::X) {
        game.rotate(Axis::X);
    }
    if is_key_pressed(KeyCode::Y) {
        game.rotate(Axis::Y);
    }
    if is_key_pressed(KeyCode::Z) {
        game.rotate(Axis::Z);
    }
    if is_key_pressed(KeyCode::Space) {
        game.hard_drop();
    }
}

fn block_world_pos_at(x: i8, y: i8, z: i8, offset_x: f32) -> Vec3 {
    vec3(
        ORIGIN_X + offset_x + x as f32 * CUBE_SIZE + CUBE_SIZE / 2.0,
        ORIGIN_Y - z as f32 * CUBE_SIZE - CUBE_SIZE / 2.0,
        ORIGIN_Z + y as f32 * CUBE_SIZE + CUBE_SIZE / 2.0,
    )
}

fn draw_bounding_box_well_at(offset_x: f32) {
    let min_x = ORIGIN_X + offset_x;
    let max_x = min_x + BOX_WIDTH as f32 * CUBE_SIZE;
    let min_z = ORIGIN_Z;
    let max_z = ORIGIN_Z + BOX_DEPTH as f32 * CUBE_SIZE;
    let floor_y = ORIGIN_Y - BOX_HEIGHT as f32 * CUBE_SIZE;

    let center_x = (min_x + max_x) / 2.0;
    let center_y = (floor_y + ORIGIN_Y) / 2.0;
    let center_z = (min_z + max_z) / 2.0;
    let box_w = BOX_WIDTH as f32 * CUBE_SIZE;
    let box_h = BOX_HEIGHT as f32 * CUBE_SIZE;
    let box_d = BOX_DEPTH as f32 * CUBE_SIZE;

    // Translucent Back Wall (min_z - deep background)
    draw_cube(vec3(center_x, center_y, min_z), vec3(box_w, box_h, 0.04), None, Color::new(0.02, 0.15, 0.35, 0.30));

    // Translucent Right Side Wall (max_x)
    draw_cube(vec3(max_x, center_y, center_z), vec3(0.04, box_h, box_d), None, Color::new(0.0, 0.35, 0.65, 0.28));
    // Translucent Bottom Floor Wall (floor_y)
    draw_cube(vec3(center_x, floor_y, center_z), vec3(box_w, 0.04, box_d), None, Color::new(0.0, 0.45, 0.85, 0.38));

    for z in 0..=BOX_HEIGHT {
        let y_pos = ORIGIN_Y - z as f32 * CUBE_SIZE;
        let color = if z == 0 || z == BOX_HEIGHT {
            Color::new(0.4, 0.85, 1.0, 0.95)
        } else {
            Color::new(0.25, 0.5, 0.8, 0.6)
        };
        draw_line_3d(vec3(min_x, y_pos, min_z), vec3(max_x, y_pos, min_z), color);
        draw_line_3d(vec3(max_x, y_pos, min_z), vec3(max_x, y_pos, max_z), color);
        draw_line_3d(vec3(max_x, y_pos, max_z), vec3(min_x, y_pos, max_z), color);
        draw_line_3d(vec3(min_x, y_pos, max_z), vec3(min_x, y_pos, min_z), color);
    }

    let color_pillars = Color::new(0.3, 0.65, 0.95, 0.8);
    draw_line_3d(vec3(min_x, ORIGIN_Y, min_z), vec3(min_x, floor_y, min_z), color_pillars);
    draw_line_3d(vec3(max_x, ORIGIN_Y, min_z), vec3(max_x, floor_y, min_z), color_pillars);
    draw_line_3d(vec3(max_x, ORIGIN_Y, max_z), vec3(max_x, floor_y, max_z), color_pillars);
    draw_line_3d(vec3(min_x, ORIGIN_Y, max_z), vec3(min_x, floor_y, max_z), color_pillars);
}

fn draw_spatial_grid_at(game: &SpatialGame, offset_x: f32) {
    for z in 0..BOX_HEIGHT as i8 {
        for x in 0..BOX_WIDTH as i8 {
            for y in 0..BOX_DEPTH as i8 {
                if let Some(id) = game.board.cells[z as usize][x as usize][y as usize] {
                    draw_neon_cube_at(x, y, z, id, false, offset_x);
                }
            }
        }
    }

    if game.state == GameState::Playing {
        let color_id = game.active_piece.piece_type.color_id();
        for (wx, wy, wz) in game.active_piece.world_blocks() {
            if wz >= 0 && wz < BOX_HEIGHT as i8 {
                draw_neon_cube_at(wx, wy, wz, color_id, true, offset_x);
            }
        }
    }
}

fn draw_neon_cube_at(x: i8, y: i8, z: i8, id: u8, is_active: bool, offset_x: f32) {
    let pos = block_world_pos_at(x, y, z, offset_x);
    let color = piece_color(id);
    let size = vec3(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE);
    if is_active {
        let active_fill = Color::new(color.r, color.g, color.b, 0.40);
        draw_cube(pos, size, None, active_fill);
        draw_cube_wires(pos, size, Color::new(color.r, color.g, color.b, 0.95));
    } else {
        draw_cube(pos, size, None, color);
        draw_cube_wires(pos, size, WHITE);
    }
}

const HUD_COLOR: Color = Color::new(0.6, 0.95, 1.0, 1.0);

fn draw_hud(game: &SpatialGame) {
    draw_text("3D SPATIAL BOX TETRIS", 20.0, 30.0, 24.0, HUD_COLOR);
    draw_text("CAMERA: ViewCube Drag / Mouse Drag / IJKL | Scroll / +/- Zoom | H/C: Home Reset | 1-5: Presets", 20.0, 56.0, 18.0, Color::new(1.0, 0.85, 0.2, 1.0));

    let base_y = screen_height() - 90.0;
    draw_text(format!("SCORE: {}", game.score), 20.0, base_y, 24.0, HUD_COLOR);
    draw_text(format!("LEVEL: {}", game.level), 20.0, base_y + 28.0, 24.0, HUD_COLOR);
    draw_text(format!("LAYERS: {}", game.layers_cleared), 20.0, base_y + 56.0, 24.0, HUD_COLOR);
}

fn draw_battle_spatial_hud(p1: &SpatialGame, p2: &SpatialGame, mode: GameMode) {

    // P1 Info (Left)
    draw_text("PLAYER 1 (3D SPATIAL)", 40.0, 50.0, 24.0, Color::new(0.0, 0.95, 1.0, 1.0));
    draw_text(format!("Score: {}", p1.score), 40.0, 80.0, 18.0, WHITE);
    draw_text(format!("Level: {}", p1.level), 40.0, 105.0, 18.0, WHITE);
    draw_text(format!("Layers: {}", p1.layers_cleared), 40.0, 130.0, 18.0, WHITE);

    // P2 / CPU Info (Right)
    let p2_title = match mode {
        GameMode::VsCpu => "CPU OPPONENT (3D SPATIAL)",
        _ => "PLAYER 2 (3D SPATIAL)",
    };
    let rx = screen_width() - 280.0;
    draw_text(p2_title, rx, 50.0, 24.0, Color::new(1.0, 0.6, 0.05, 1.0));
    draw_text(format!("Score: {}", p2.score), rx, 80.0, 18.0, WHITE);
    draw_text(format!("Level: {}", p2.level), rx, 105.0, 18.0, WHITE);
    draw_text(format!("Layers: {}", p2.layers_cleared), rx, 130.0, 18.0, WHITE);

    // Controls legend
    draw_text("P1: WASD/Arrows (X/Y) | XYZ (Rotate 3D) | Space (Drop Z)", 40.0, screen_height() - 40.0, 16.0, HUD_COLOR);
    draw_text("Esc: Pause Menu | R: Pause (Restart)", screen_width() / 2.0 - 140.0, screen_height() - 20.0, 16.0, WHITE);
}

/// Draws just the winner headline above the game-over menu box (the menu itself provides the
/// Restart/Main Menu actions, so this only needs to say who won).
fn draw_spatial_match_winner_title(winner: MatchWinner) {
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

