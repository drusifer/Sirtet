use macroquad::prelude::*;

use tetris::camera::{OrbitCamera, ViewCubeGizmo};
use tetris::fx::{format_clear_banner, ClearFx, LandingFx, ScoreBanner, FX_DURATION};
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

fn block_world_pos(x: i8, y: i8, z: i8) -> Vec3 {
    vec3(
        ORIGIN_X + x as f32 * CUBE_SIZE + CUBE_SIZE / 2.0,
        ORIGIN_Y - z as f32 * CUBE_SIZE - CUBE_SIZE / 2.0,
        ORIGIN_Z + y as f32 * CUBE_SIZE + CUBE_SIZE / 2.0,
    )
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Sirtet — Spatial 3D Box Tetris (Blockout Mode)".to_owned(),
        window_width: 1024,
        window_height: 768,
        platform: miniquad::conf::Platform {
            linux_backend: miniquad::conf::LinuxBackend::WaylandOnly,
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn run(game: SpatialGame) {
    macroquad::Window::from_config(window_conf(), amain(game));
}

async fn amain(mut game: SpatialGame) {
    let mut orbit_cam = OrbitCamera::default_3d_box();
    let mut viewcube = ViewCubeGizmo::new(0.35);
    let mut last_tick = get_time();

    let mut landing_fx = LandingFx::new();
    let mut clear_fx = ClearFx::new();
    let mut banner = ScoreBanner::new();

    let mut prev_active_z = game.active_piece.z;

    loop {
        let now = get_time();
        let interval = (tetris::spatial_game::spatial_gravity_interval_ms(game.level) as f64) / 1000.0;

        if is_quit_requested() {
            break;
        }

        if handle_input(&mut game) {
            break;
        }

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

        clear_background(Color::new(0.02, 0.02, 0.07, 1.0));

        set_camera(&orbit_cam.camera_3d());
        draw_bounding_box_well();
        draw_spatial_grid(&game);

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

        let reset_requested = viewcube.update_and_draw(&mut orbit_cam.yaw, &mut orbit_cam.pitch);
        if reset_requested {
            orbit_cam = OrbitCamera::default_3d_box();
        }

        orbit_cam.update(viewcube.is_dragging);

        next_frame().await;
    }
}

fn handle_input(game: &mut SpatialGame) -> bool {
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
    if is_key_pressed(KeyCode::P) {
        game.toggle_pause();
    }
    if is_key_pressed(KeyCode::R) {
        game.restart();
    }

    is_key_pressed(KeyCode::Q) || is_key_pressed(KeyCode::Escape)
}

fn draw_bounding_box_well() {
    let min_x = ORIGIN_X;
    let max_x = ORIGIN_X + BOX_WIDTH as f32 * CUBE_SIZE;
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

fn draw_spatial_grid(game: &SpatialGame) {
    for z in 0..BOX_HEIGHT as i8 {
        for x in 0..BOX_WIDTH as i8 {
            for y in 0..BOX_DEPTH as i8 {
                if let Some(id) = game.board.cells[z as usize][x as usize][y as usize] {
                    draw_neon_cube(x, y, z, id, false);
                }
            }
        }
    }

    if game.state == GameState::Playing {
        let color_id = game.active_piece.piece_type.color_id();
        for (wx, wy, wz) in game.active_piece.world_blocks() {
            if wz >= 0 && wz < BOX_HEIGHT as i8 {
                draw_neon_cube(wx, wy, wz, color_id, true);
            }
        }
    }
}

fn draw_neon_cube(x: i8, y: i8, z: i8, id: u8, is_active: bool) {
    let pos = block_world_pos(x, y, z);
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
    draw_text("CAMERA: ViewCube Drag / Mouse Drag / IJKL | Scroll / +/- Zoom | H/C: Home Reset", 20.0, 56.0, 18.0, Color::new(1.0, 0.85, 0.2, 1.0));

    let base_y = screen_height() - 90.0;
    draw_text(format!("SCORE: {}", game.score), 20.0, base_y, 24.0, HUD_COLOR);
    draw_text(format!("LEVEL: {}", game.level), 20.0, base_y + 28.0, 24.0, HUD_COLOR);
    draw_text(format!("LAYERS: {}", game.layers_cleared), 20.0, base_y + 56.0, 24.0, HUD_COLOR);

    let lines = [
        "3D CONTROLS:",
        "Left/Right/A/D  Translate X",
        "Up/Down/W/S     Translate Y",
        "X / Y / Z       Rotate 3D Axes",
        "Space           Hard Drop Z",
        "ViewCube Drag   Orbit 3D Gizmo",
        "Mouse Drag/IJKL Orbit View",
        "Scroll / +/-    Zoom View",
        "H / C           Home Reset View",
        "P               Pause",
        "R               Restart",
        "Q/Esc           Quit",
    ];
    let origin_x = screen_width() - 260.0;
    for (i, line) in lines.iter().enumerate() {
        draw_text(line, origin_x, 180.0 + i as f32 * 22.0, 18.0, HUD_COLOR);
    }

    let cx = screen_width() / 2.0;
    let cy = screen_height() / 2.0;
    match game.state {
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
            draw_text(format!("Final score: {}", game.score), cx - 100.0, cy + 20.0, 24.0, WHITE);
            draw_text("Press R to restart", cx - 100.0, cy + 50.0, 24.0, HUD_COLOR);
        }
        GameState::Playing => {}
    }
}
