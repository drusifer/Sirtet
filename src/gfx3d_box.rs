use macroquad::prelude::*;

use tetris::spatial_game::{
    Axis, GameState, SpatialGame, BOX_DEPTH, BOX_HEIGHT, BOX_WIDTH,
};

const CUBE_SIZE: f32 = 0.85;
const ORIGIN_X: f32 = -(BOX_WIDTH as f32 * CUBE_SIZE) / 2.0;
const ORIGIN_Y: f32 = (BOX_HEIGHT as f32 * CUBE_SIZE) / 2.0;
const ORIGIN_Z: f32 = -(BOX_DEPTH as f32 * CUBE_SIZE) / 2.0;

const FX_DURATION: f64 = 0.5;

struct OrbitCamera {
    yaw: f32,
    pitch: f32,
    distance: f32,
    target: Vec3,
    shake_intensity: f32,
}

impl OrbitCamera {
    fn default_3d_box() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.35,
            distance: 12.0,
            target: vec3(0.0, 0.0, 0.0),
            shake_intensity: 0.0,
        }
    }

    fn add_shake(&mut self, intensity: f32) {
        self.shake_intensity = (self.shake_intensity + intensity).min(1.0);
    }

    fn update(&mut self, gizmo_handled: bool) {
        if self.shake_intensity > 0.0 {
            self.shake_intensity = (self.shake_intensity - 0.04).max(0.0);
        }

        if !gizmo_handled && (is_mouse_button_down(MouseButton::Right) || is_mouse_button_down(MouseButton::Left)) {
            let delta = mouse_delta_position();
            self.yaw += delta.x * 3.5;
            self.pitch = (self.pitch + delta.y * 3.5).clamp(-1.4, 1.4);
        }

        if is_key_down(KeyCode::J) { self.yaw -= 0.04; }
        if is_key_down(KeyCode::L) { self.yaw += 0.04; }
        if is_key_down(KeyCode::I) { self.pitch = (self.pitch + 0.04).clamp(-1.4, 1.4); }
        if is_key_down(KeyCode::K) { self.pitch = (self.pitch - 0.04).clamp(-1.4, 1.4); }

        let wheel = mouse_wheel().1;
        if wheel != 0.0 {
            self.distance = (self.distance - wheel * 0.8).clamp(4.0, 30.0);
        }
        if is_key_down(KeyCode::Equal) { self.distance = (self.distance - 0.2).max(4.0); }
        if is_key_down(KeyCode::Minus) { self.distance = (self.distance + 0.2).min(30.0); }

        if is_key_pressed(KeyCode::C) {
            *self = Self::default_3d_box();
        }
    }

    fn camera_3d(&self) -> Camera3D {
        let shake_offset = if self.shake_intensity > 0.0 {
            vec3(
                rand::gen_range(-0.15, 0.15) * self.shake_intensity,
                rand::gen_range(-0.15, 0.15) * self.shake_intensity,
                rand::gen_range(-0.15, 0.15) * self.shake_intensity,
            )
        } else {
            vec3(0.0, 0.0, 0.0)
        };

        let x = self.target.x + self.distance * self.pitch.cos() * self.yaw.sin() + shake_offset.x;
        let y = self.target.y + self.distance * self.pitch.sin() + shake_offset.y;
        let z = self.target.z + self.distance * self.pitch.cos() * self.yaw.cos() + shake_offset.z;
        Camera3D {
            position: vec3(x, y, z),
            target: self.target + shake_offset,
            up: vec3(0.0, 1.0, 0.0),
            ..Default::default()
        }
    }
}

struct ViewCubeGizmo {
    center: Vec2,
    radius: f32,
    is_dragging: bool,
    drag_start_mouse: Vec2,
    drag_start_yaw: f32,
    drag_start_pitch: f32,
}

impl ViewCubeGizmo {
    fn new() -> Self {
        Self {
            center: vec2(0.0, 0.0),
            radius: 45.0,
            is_dragging: false,
            drag_start_mouse: vec2(0.0, 0.0),
            drag_start_yaw: 0.0,
            drag_start_pitch: 0.35,
        }
    }

    fn update_and_draw(&mut self, yaw: &mut f32, pitch: &mut f32) -> bool {
        let cx = screen_width() - 80.0;
        let cy = 80.0;
        self.center = vec2(cx, cy);

        let mouse_pos = vec2(mouse_position().0, mouse_position().1);
        let dist_to_center = (mouse_pos - self.center).length();
        let is_hovered = dist_to_center <= self.radius + 10.0;

        if is_mouse_button_pressed(MouseButton::Left) && is_hovered {
            self.is_dragging = true;
            self.drag_start_mouse = mouse_pos;
            self.drag_start_yaw = *yaw;
            self.drag_start_pitch = *pitch;
        }

        if is_mouse_button_down(MouseButton::Left) && self.is_dragging {
            let delta = mouse_pos - self.drag_start_mouse;
            *yaw = self.drag_start_yaw + delta.x * 0.03;
            *pitch = (self.drag_start_pitch + delta.y * 0.03).clamp(-1.4, 1.4);
        } else if !is_mouse_button_down(MouseButton::Left) {
            self.is_dragging = false;
        }

        self.draw_viewcube(*yaw, *pitch, is_hovered);

        let home_center = vec2(cx, cy + 65.0);
        let dist_home = (mouse_pos - home_center).length();
        let is_home_hovered = dist_home <= 18.0;

        let home_bg = if is_home_hovered {
            Color::new(0.3, 0.7, 1.0, 0.9)
        } else {
            Color::new(0.15, 0.15, 0.25, 0.8)
        };
        draw_circle(home_center.x, home_center.y, 18.0, home_bg);
        draw_circle_lines(home_center.x, home_center.y, 18.0, 2.0, WHITE);
        draw_text("H", home_center.x - 5.0, home_center.y + 5.0, 16.0, WHITE);

        let reset_requested = is_mouse_button_pressed(MouseButton::Left) && is_home_hovered;
        reset_requested || self.is_dragging || is_hovered
    }

    fn draw_viewcube(&self, yaw: f32, pitch: f32, is_hovered: bool) {
        let cx = self.center.x;
        let cy = self.center.y;
        let scale = 30.0;

        let halo_color = if is_hovered || self.is_dragging {
            Color::new(0.0, 0.85, 1.0, 0.35)
        } else {
            Color::new(0.2, 0.2, 0.3, 0.2)
        };
        draw_circle(cx, cy, self.radius + 6.0, halo_color);

        let cube_verts: [(f32, f32, f32); 8] = [
            (-1.0, -1.0, -1.0),
            ( 1.0, -1.0, -1.0),
            ( 1.0,  1.0, -1.0),
            (-1.0,  1.0, -1.0),
            (-1.0, -1.0,  1.0),
            ( 1.0, -1.0,  1.0),
            ( 1.0,  1.0,  1.0),
            (-1.0,  1.0,  1.0),
        ];

        let mut proj_2d = [(0.0f32, 0.0f32, 0.0f32); 8];
        for (i, &(vx, vy, vz)) in cube_verts.iter().enumerate() {
            let (rx, ry, rz) = rotate_gizmo_vertex(vx, vy, vz, yaw, pitch);
            proj_2d[i] = (cx + rx * scale, cy - ry * scale, rz);
        }

        let faces: [([usize; 4], &str); 6] = [
            ([4, 5, 6, 7], "FRONT"),
            ([1, 0, 3, 2], "BACK"),
            ([0, 4, 7, 3], "LEFT"),
            ([5, 1, 2, 6], "RIGHT"),
            ([3, 2, 6, 7], "TOP"),
            ([0, 1, 5, 4], "BOT"),
        ];

        let mut face_order: Vec<(usize, f32)> = Vec::new();
        for (idx, (v_indices, _)) in faces.iter().enumerate() {
            let avg_z: f32 = v_indices.iter().map(|&v| proj_2d[v].2).sum::<f32>() / 4.0;
            face_order.push((idx, avg_z));
        }
        face_order.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        for &(idx, avg_z) in &face_order {
            if avg_z <= 0.0 { continue; }

            let (v_indices, label) = &faces[idx];
            let p0 = vec2(proj_2d[v_indices[0]].0, proj_2d[v_indices[0]].1);
            let p1 = vec2(proj_2d[v_indices[1]].0, proj_2d[v_indices[1]].1);
            let p2 = vec2(proj_2d[v_indices[2]].0, proj_2d[v_indices[2]].1);
            let p3 = vec2(proj_2d[v_indices[3]].0, proj_2d[v_indices[3]].1);

            let face_bg = match *label {
                "TOP" => Color::new(0.85, 0.85, 0.9, 0.92),
                "FRONT" => Color::new(0.75, 0.78, 0.85, 0.92),
                "RIGHT" | "LEFT" => Color::new(0.65, 0.68, 0.75, 0.92),
                _ => Color::new(0.55, 0.58, 0.65, 0.92),
            };

            draw_triangle(p0, p1, p2, face_bg);
            draw_triangle(p0, p2, p3, face_bg);

            let line_color = Color::new(0.1, 0.15, 0.25, 0.95);
            draw_line(p0.x, p0.y, p1.x, p1.y, 2.0, line_color);
            draw_line(p1.x, p1.y, p2.x, p2.y, 2.0, line_color);
            draw_line(p2.x, p2.y, p3.x, p3.y, 2.0, line_color);
            draw_line(p3.x, p3.y, p0.x, p0.y, 2.0, line_color);

            let fc = (p0 + p1 + p2 + p3) / 4.0;
            draw_text(label, fc.x - 12.0, fc.y + 4.0, 11.0, Color::new(0.05, 0.08, 0.15, 1.0));
        }
    }
}

fn rotate_gizmo_vertex(vx: f32, vy: f32, vz: f32, yaw: f32, pitch: f32) -> (f32, f32, f32) {
    let x1 = vx * yaw.cos() - vz * yaw.sin();
    let z1 = vx * yaw.sin() + vz * yaw.cos();
    let y1 = vy;

    let y2 = y1 * pitch.cos() - z1 * pitch.sin();
    let z2 = y1 * pitch.sin() + z1 * pitch.cos();
    let x2 = x1;

    (x2, y2, z2)
}

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
    let mut viewcube = ViewCubeGizmo::new();
    let mut last_tick = get_time();

    let mut land_fx_start: Option<f64> = None;
    let mut clear_fx_start: Option<f64> = None;
    let mut clear_fx_layers;
    let mut prev_active_z = game.active_piece.z;

    let mut banner_msg: Option<(String, f64)> = None;



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

            // Detect genuine 3D piece lock landing (piece locked and new piece spawned at top z=0)
            if game.active_piece.z < prev_active_z {
                land_fx_start = Some(now);
                orbit_cam.add_shake(0.35);
            }
        }
        prev_active_z = game.active_piece.z;

        // Detect 3D layer clear event immediately on lock (hard drop or gravity tick)
        if game.last_layers_cleared > 0 && clear_fx_start.is_none() {
            clear_fx_start = Some(now);
            clear_fx_layers = game.last_layers_cleared;
            orbit_cam.add_shake(0.85);

            let msg = match clear_fx_layers {
                1 => "SINGLE LAYER CLEAR! +100",
                2 => "DOUBLE LAYER CLEAR! +300",
                3 => "TRIPLE LAYER CLEAR! +600",
                _ => "💥 QUAD 3D LAYER EXPLOSION! +1000",
            };
            banner_msg = Some((msg.to_string(), now));
        }


        clear_background(Color::new(0.02, 0.02, 0.07, 1.0));

        set_camera(&orbit_cam.camera_3d());
        draw_bounding_box_well();
        draw_spatial_grid(&game);

        // Draw 3D Landing shockwave pulses
        if let Some(start) = land_fx_start {
            let elapsed = now - start;
            if elapsed < FX_DURATION {
                let t = (elapsed / FX_DURATION) as f32;
                let floor_y = ORIGIN_Y - BOX_HEIGHT as f32 * CUBE_SIZE;

                for ring_i in 1..=3 {
                    let r_scale = 1.0 + t * (0.25 * ring_i as f32);
                    let alpha = (1.0 - t) * (0.8 / ring_i as f32);
                    let ring_color = Color::new(0.1, 1.0, 0.85, alpha);
                    let sz = (BOX_WIDTH as f32 * CUBE_SIZE) * r_scale;
                    draw_cube_wires(vec3(0.0, floor_y, 0.0), vec3(sz, 0.3 * ring_i as f32, sz), ring_color);
                }
            } else {
                land_fx_start = None;
            }
        }

        // Draw 3D Layer Clear shockwave burst rings
        if let Some(start) = clear_fx_start {
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
                clear_fx_start = None;
            }
        }

        set_default_camera();
        draw_hud(&game);

        // Draw Layer Clear Full-Screen Flash Burst
        if let Some(start) = clear_fx_start {
            let elapsed = now - start;
            if elapsed < FX_DURATION {
                let t = (elapsed / FX_DURATION) as f32;
                let flash_alpha = (1.0 - t) * 0.75;
                draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(1.0, 0.95, 0.3, flash_alpha));
            }
        }

        // Draw Layer Clear Score Banner
        if let Some((ref msg, start)) = banner_msg {
            let elapsed = now - start;
            if elapsed < 1.2 {
                let t = (elapsed / 1.2) as f32;
                let alpha = (1.0 - t).min(1.0);
                let font_size = 32.0 + (1.0 - t) * 8.0;
                let msg_len = msg.len() as f32;
                let cx = screen_width() / 2.0 - (msg_len * font_size * 0.28);
                let cy = 130.0 - t * 20.0;
                draw_text(msg, cx + 2.0, cy + 2.0, font_size, Color::new(0.0, 0.0, 0.0, alpha * 0.8));
                draw_text(msg, cx, cy, font_size, Color::new(1.0, 0.9, 0.1, alpha));
            } else {
                banner_msg = None;
            }
        }


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

    // Translucent Back Wall (max_z)
    draw_cube(vec3(center_x, center_y, max_z), vec3(box_w, box_h, 0.04), None, Color::new(0.02, 0.15, 0.35, 0.30));
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
