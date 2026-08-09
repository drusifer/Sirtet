use macroquad::prelude::*;

use tetris::board::{HEIGHT, WIDTH};
use tetris::camera::{OrbitCamera, ViewCubeGizmo};
use tetris::fx::{format_clear_banner, ClearFx, LandingFx, ScoreBanner, FX_DURATION};
use tetris::game::{Game, GameState};

const CELL_SIZE: f32 = 0.5;
const BOARD_ORIGIN_X: f32 = -(WIDTH as f32 * CELL_SIZE) / 2.0;
const BOARD_ORIGIN_Y: f32 = (HEIGHT as f32 * CELL_SIZE) / 2.0;

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

fn cell_world_pos(x: i32, y: f32) -> Vec3 {
    vec3(
        BOARD_ORIGIN_X + x as f32 * CELL_SIZE + CELL_SIZE / 2.0,
        BOARD_ORIGIN_Y - y * CELL_SIZE - CELL_SIZE / 2.0,
        0.0,
    )
}

fn window_conf() -> Conf {
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

pub fn run(game: Game) {
    macroquad::Window::from_config(window_conf(), amain(game));
}

async fn amain(mut game: Game) {
    let mut orbit_cam = OrbitCamera::default_2d_fancy();
    let mut viewcube = ViewCubeGizmo::new(0.12);
    let mut last_tick = get_time();

    let mut landing_fx = LandingFx::new();
    let mut clear_fx = ClearFx::new();
    let mut banner = ScoreBanner::new();

    let mut prev_active_y = game.active().y;

    loop {
        let now = get_time();
        let interval = (game.gravity_interval_ms() as f64 / 1000.0).max(0.001);

        if is_quit_requested() {
            break;
        }

        if handle_input(&mut game) {
            break;
        }

        if now - last_tick >= interval {
            game.tick();
            last_tick = now;

            // Detect genuine piece lock landing
            if game.active().y < prev_active_y {
                landing_fx.trigger(now);
                orbit_cam.add_shake(0.35);
            }
        }
        prev_active_y = game.active().y;

        // Detect line clear event immediately on lock
        let cleared = game.last_lines_cleared();
        if cleared > 0 && clear_fx.start_time.is_none() {
            clear_fx.trigger(now, cleared);
            orbit_cam.add_shake(0.85);

            let msg = format_clear_banner(cleared, false);
            banner.trigger(msg.to_string(), now);
        }

        clear_background(Color::new(0.02, 0.02, 0.07, 1.0));

        set_camera(&orbit_cam.camera_3d());
        draw_board(&game);

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
        draw_hud(&mut game);

        // Draw Line Clear Full-Screen Flash Burst
        clear_fx.draw_flash_burst(now);

        // Draw Floating Score Banner
        banner.draw(now);

        let reset_requested = viewcube.update_and_draw(&mut orbit_cam.yaw, &mut orbit_cam.pitch);
        if reset_requested {
            orbit_cam = OrbitCamera::default_2d_fancy();
        }

        orbit_cam.update(viewcube.is_dragging);

        next_frame().await;
    }
}

fn handle_input(game: &mut Game) -> bool {
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
    if is_key_pressed(KeyCode::P) {
        game.toggle_pause();
    }
    if is_key_pressed(KeyCode::R) {
        game.restart();
    }

    is_key_pressed(KeyCode::Q) || is_key_pressed(KeyCode::Escape)
}

fn draw_board(game: &Game) {
    draw_faint_grid_and_border();

    for y in 0..HEIGHT as i32 {
        for x in 0..WIDTH as i32 {
            if let Some(id) = game.board().cell(x, y) {
                draw_neon_cell(x, y as f32, id, false);
            }
        }
    }

    if game.state() != GameState::GameOver {
        let id = game.active().piece_type.id();
        let base = game.active();
        for (x, y) in base.cells() {
            draw_neon_cell(x, y as f32, id, true);
        }
    }
}

fn draw_faint_grid_and_border() {
    let grid_color = Color::new(0.15, 0.25, 0.45, 0.35);
    let border_color = Color::new(0.0, 0.85, 1.0, 0.8);

    let left = BOARD_ORIGIN_X;
    let right = BOARD_ORIGIN_X + WIDTH as f32 * CELL_SIZE;
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
        let gx = BOARD_ORIGIN_X + x as f32 * CELL_SIZE;
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

fn draw_neon_cell(x: i32, y: f32, id: u8, is_active: bool) {
    let pos = cell_world_pos(x, y);
    let color = piece_color(id);
    let size = vec3(CELL_SIZE, CELL_SIZE, CELL_SIZE * 0.5);
    if is_active {
        let glow = Color::new(color.r, color.g, color.b, 0.12);
        draw_cube(pos, vec3(CELL_SIZE * 1.05, CELL_SIZE * 1.05, CELL_SIZE * 0.5), None, glow);
    }
    draw_cube(pos, size, None, color);
    draw_cube_wires(pos, size, WHITE);
    draw_cube_wires(pos, vec3(CELL_SIZE * 0.82, CELL_SIZE * 0.82, CELL_SIZE * 0.5), Color::new(0.0, 0.0, 0.0, 0.5));
}

const HUD_COLOR: Color = Color::new(0.6, 0.95, 1.0, 1.0);

fn draw_hud(game: &mut Game) {
    draw_next_preview(game);
    draw_stats(game);
    draw_controls_legend();
    draw_status_overlay(game);
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
        "P     pause",
        "R     restart",
        "Q/Esc quit",
    ];
    let origin_x = screen_width() - 250.0;
    for (i, line) in lines.iter().enumerate() {
        draw_text(line, origin_x, 180.0 + i as f32 * 22.0, 18.0, HUD_COLOR);
    }
}

fn draw_status_overlay(game: &Game) {
    let cx = screen_width() / 2.0;
    let cy = screen_height() / 2.0;
    match game.state() {
        GameState::Paused => {
            let bw = 240.0;
            let bh = 80.0;
            draw_rectangle(cx - bw / 2.0, cy - bh / 2.0, bw, bh, Color::new(0.0, 0.0, 0.0, 0.85));
            draw_rectangle_lines(cx - bw / 2.0, cy - bh / 2.0, bw, bh, 2.0, HUD_COLOR);
            draw_text("PAUSED", cx - 60.0, cy + 10.0, 40.0, WHITE);
        }
        GameState::GameOver => {
            let bw = 360.0;
            let bh = 160.0;
            draw_rectangle(cx - bw / 2.0, cy - bh / 2.0, bw, bh, Color::new(0.0, 0.0, 0.0, 0.85));
            draw_rectangle_lines(cx - bw / 2.0, cy - bh / 2.0, bw, bh, 2.0, Color::new(1.0, 0.2, 0.3, 1.0));
            draw_text("GAME OVER", cx - 110.0, cy - 20.0, 40.0, Color::new(1.0, 0.2, 0.3, 1.0));
            draw_text(
                format!("Final score: {}", game.score()),
                cx - 100.0,
                cy + 20.0,
                24.0,
                WHITE,
            );
            draw_text("Press R to restart", cx - 100.0, cy + 50.0, 24.0, HUD_COLOR);
        }
        GameState::Playing => {}
    }
}
