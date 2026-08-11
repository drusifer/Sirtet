use macroquad::prelude::*;

const CAMERA_PRESET_KEYS: [KeyCode; 5] = [
    KeyCode::Key1,
    KeyCode::Key2,
    KeyCode::Key3,
    KeyCode::Key4,
    KeyCode::Key5,
];

/// (yaw, pitch) pairs for the 1-5 camera preset hotkeys, in radians. Distance/target are left to
/// whichever renderer owns the `OrbitCamera` instance, so the same five orientations reasonably
/// suit both the flat 2D board and the deeper 3D spatial box.
const CAMERA_PRESETS: [(f32, f32); 5] = [
    (0.0, 0.12),                         // 1: Front (matches the default startup angle)
    (0.0, 0.7),                          // 2: High angle
    (std::f32::consts::FRAC_PI_2, 0.2),  // 3: Side
    (std::f32::consts::FRAC_PI_4, 0.45), // 4: Corner (isometric-ish)
    (0.0, 1.5),                          // 5: Fully top-down (looking straight down the well)
];

/// `camera_3d()`'s `up` vector is a fixed (0, 1, 0) — at pitch exactly ±FRAC_PI_2 the view
/// direction becomes parallel to it (gimbal lock), which can make the render flip or jitter.
/// Presets stay strictly inside this bound so "top-down" is a clean overhead view, not the
/// literal singularity.
const MAX_PRESET_PITCH: f32 = 1.55;

pub struct OrbitCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub target: Vec3,
    pub shake_intensity: f32,
}

impl OrbitCamera {
    pub fn default_2d_fancy() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.12,
            distance: 15.5,
            target: vec3(0.0, 0.0, 0.0),
            shake_intensity: 0.0,
        }
    }

    pub fn default_3d_box() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.35,
            distance: 12.0,
            target: vec3(0.0, 0.0, 0.0),
            shake_intensity: 0.0,
        }
    }

    pub fn add_shake(&mut self, intensity: f32) {
        self.shake_intensity = (self.shake_intensity + intensity).min(1.0);
    }

    pub fn update(&mut self, gizmo_handled: bool) {
        self.apply_preset_hotkeys();

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
    }

    /// Jumps to a named (yaw, pitch) orientation preset when its number key (1-5) was just
    /// pressed. Called from `update()`, so every renderer using `OrbitCamera` gets presets for
    /// free — no per-renderer wiring or duplicated preset tables. Distance/target are untouched
    /// (presets only change orientation, like the ViewCube gizmo's drag does), so they compose
    /// with each renderer's own default distance/target and with manual zoom.
    fn apply_preset_hotkeys(&mut self) {
        for (key, &(yaw, pitch)) in CAMERA_PRESET_KEYS.iter().zip(CAMERA_PRESETS.iter()) {
            if is_key_pressed(*key) {
                self.yaw = yaw;
                self.pitch = pitch.clamp(-MAX_PRESET_PITCH, MAX_PRESET_PITCH);
            }
        }
    }

    pub fn camera_3d(&self) -> Camera3D {
        let shake_offset = if self.shake_intensity > 0.0 {
            vec3(
                rand::gen_range(-0.18, 0.18) * self.shake_intensity,
                rand::gen_range(-0.18, 0.18) * self.shake_intensity,
                rand::gen_range(-0.18, 0.18) * self.shake_intensity,
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

pub struct ViewCubeGizmo {
    pub center: Vec2,
    pub radius: f32,
    pub is_dragging: bool,
    pub drag_start_mouse: Vec2,
    pub drag_start_yaw: f32,
    pub drag_start_pitch: f32,
}

impl ViewCubeGizmo {
    pub fn new(initial_pitch: f32) -> Self {
        Self {
            center: vec2(0.0, 0.0),
            radius: 45.0,
            is_dragging: false,
            drag_start_mouse: vec2(0.0, 0.0),
            drag_start_yaw: 0.0,
            drag_start_pitch: initial_pitch,
        }
    }

    pub fn update_and_draw(&mut self, yaw: &mut f32, pitch: &mut f32) -> bool {
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

pub fn rotate_gizmo_vertex(vx: f32, vy: f32, vz: f32, yaw: f32, pitch: f32) -> (f32, f32, f32) {
    let x1 = vx * yaw.cos() - vz * yaw.sin();
    let z1 = vx * yaw.sin() + vz * yaw.cos();
    let y1 = vy;

    let y2 = y1 * pitch.cos() - z1 * pitch.sin();
    let z2 = y1 * pitch.sin() + z1 * pitch.cos();
    let x2 = x1;

    (x2, y2, z2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orbit_camera_defaults() {
        let cam2d = OrbitCamera::default_2d_fancy();
        assert_eq!(cam2d.yaw, 0.0);
        assert_eq!(cam2d.pitch, 0.12);

        let cam3d = OrbitCamera::default_3d_box();
        assert_eq!(cam3d.yaw, 0.0);
        assert_eq!(cam3d.pitch, 0.35);
    }

    #[test]
    fn test_camera_shake_decay() {
        let mut cam = OrbitCamera::default_2d_fancy();
        cam.add_shake(0.5);
        assert_eq!(cam.shake_intensity, 0.5);
        cam.shake_intensity = (cam.shake_intensity - 0.04).max(0.0);
        assert!((cam.shake_intensity - 0.46).abs() < 1e-4);
    }


    #[test]
    fn test_gizmo_vertex_rotation() {
        let (rx, ry, rz) = rotate_gizmo_vertex(1.0, 0.0, 0.0, 0.0, 0.0);
        assert!((rx - 1.0).abs() < 1e-4);
        assert!((ry - 0.0).abs() < 1e-4);
        assert!((rz - 0.0).abs() < 1e-4);
    }

    #[test]
    fn camera_presets_have_one_key_per_preset() {
        assert_eq!(CAMERA_PRESET_KEYS.len(), CAMERA_PRESETS.len());
    }

    #[test]
    fn preset_one_matches_the_default_front_facing_angle() {
        let cam = OrbitCamera::default_2d_fancy();
        assert_eq!(CAMERA_PRESETS[0], (cam.yaw, cam.pitch));
    }

    #[test]
    fn no_preset_pitch_reaches_the_up_vector_singularity() {
        for &(_, pitch) in CAMERA_PRESETS.iter() {
            assert!(
                pitch.abs() < MAX_PRESET_PITCH,
                "preset pitch {pitch} is too close to vertical — camera_3d()'s fixed up vector \
                 becomes degenerate near +/-FRAC_PI_2"
            );
        }
    }
}
