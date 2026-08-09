use macroquad::prelude::*;

pub const FX_DURATION: f64 = 0.5;

pub struct LandingFx {
    pub start_time: Option<f64>,
}

impl LandingFx {
    pub fn new() -> Self {
        Self { start_time: None }
    }

    pub fn trigger(&mut self, now: f64) {
        self.start_time = Some(now);
    }

    pub fn draw_2d_shockwave(&mut self, now: f64, origin_y: f32, height: usize, cell_size: f32, width: usize) {
        let Some(start) = self.start_time else { return };
        let elapsed = now - start;
        if elapsed < FX_DURATION {
            let t = (elapsed / FX_DURATION) as f32;
            let bottom_y = origin_y - height as f32 * cell_size;
            for ring_i in 1..=3 {
                let r_scale = 1.0 + t * (0.3 * ring_i as f32);
                let alpha = (1.0 - t) * (0.85 / ring_i as f32);
                let ring_color = Color::new(0.1, 1.0, 0.85, alpha);
                let w = (width as f32 * cell_size) * r_scale;
                draw_cube_wires(vec3(0.0, bottom_y, 0.0), vec3(w, 0.4 * ring_i as f32, 0.4), ring_color);
            }
        } else {
            self.start_time = None;
        }
    }

    pub fn draw_3d_shockwave(&mut self, now: f64, floor_y: f32, box_width: usize, cube_size: f32) {
        let Some(start) = self.start_time else { return };
        let elapsed = now - start;
        if elapsed < FX_DURATION {
            let t = (elapsed / FX_DURATION) as f32;
            for ring_i in 1..=3 {
                let r_scale = 1.0 + t * (0.25 * ring_i as f32);
                let alpha = (1.0 - t) * (0.8 / ring_i as f32);
                let ring_color = Color::new(0.1, 1.0, 0.85, alpha);
                let sz = (box_width as f32 * cube_size) * r_scale;
                draw_cube_wires(vec3(0.0, floor_y, 0.0), vec3(sz, 0.3 * ring_i as f32, sz), ring_color);
            }
        } else {
            self.start_time = None;
        }
    }
}

pub struct ClearFx {
    pub start_time: Option<f64>,
    pub lines_cleared: u32,
}

impl ClearFx {
    pub fn new() -> Self {
        Self {
            start_time: None,
            lines_cleared: 0,
        }
    }

    pub fn trigger(&mut self, now: f64, count: u32) {
        if self.start_time.is_none() {
            self.start_time = Some(now);
            self.lines_cleared = count;
        }
    }

    pub fn draw_flash_burst(&self, now: f64) {
        let Some(start) = self.start_time else { return };
        let elapsed = now - start;
        if elapsed < FX_DURATION {
            let t = (elapsed / FX_DURATION) as f32;
            let flash_alpha = (1.0 - t) * 0.75;
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(1.0, 0.9, 0.2, flash_alpha));
        }
    }
}

pub struct ScoreBanner {
    pub current: Option<(String, f64)>,
}

impl ScoreBanner {
    pub fn new() -> Self {
        Self { current: None }
    }

    pub fn trigger(&mut self, msg: String, now: f64) {
        self.current = Some((msg, now));
    }

    pub fn draw(&mut self, now: f64) {
        let Some((ref msg, start)) = self.current else { return };
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
            self.current = None;
        }
    }
}

impl Default for LandingFx {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ClearFx {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ScoreBanner {
    fn default() -> Self {
        Self::new()
    }
}

pub fn format_clear_banner(count: u32, is_3d: bool) -> &'static str {

    match (count, is_3d) {
        (1, false) => "SINGLE LINE CLEAR! +100",
        (2, false) => "DOUBLE LINE CLEAR! +300",
        (3, false) => "TRIPLE LINE CLEAR! +500",
        (_, false) => "💥 TETRIS LINE CLEAR! +800",
        (1, true) => "SINGLE LAYER CLEAR! +100",
        (2, true) => "DOUBLE LAYER CLEAR! +300",
        (3, true) => "TRIPLE LAYER CLEAR! +600",
        (_, true) => "💥 QUAD 3D LAYER EXPLOSION! +1000",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_clear_banner() {
        assert_eq!(format_clear_banner(1, false), "SINGLE LINE CLEAR! +100");
        assert_eq!(format_clear_banner(4, false), "💥 TETRIS LINE CLEAR! +800");
        assert_eq!(format_clear_banner(1, true), "SINGLE LAYER CLEAR! +100");
        assert_eq!(format_clear_banner(4, true), "💥 QUAD 3D LAYER EXPLOSION! +1000");
    }

    #[test]
    fn test_fx_timers() {
        let mut land = LandingFx::new();
        assert!(land.start_time.is_none());
        land.trigger(10.0);
        assert_eq!(land.start_time, Some(10.0));

        let mut clear = ClearFx::new();
        clear.trigger(10.0, 2);
        assert_eq!(clear.lines_cleared, 2);
    }
}
