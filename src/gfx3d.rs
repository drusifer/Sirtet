use macroquad::prelude::*;

use tetris::board::{HEIGHT, WIDTH};
use tetris::game::{Game, GameState};

const CELL_SIZE: f32 = 0.5;
const BOARD_ORIGIN_X: f32 = -(WIDTH as f32 * CELL_SIZE) / 2.0;
const BOARD_ORIGIN_Y: f32 = (HEIGHT as f32 * CELL_SIZE) / 2.0;

/// How long the line-clear flash (decision #8's consumer) stays visible.
const LINE_CLEAR_FX_DURATION: f64 = 0.3;

/// Neon per-piece palette (decision #6) — brighter/more saturated than the terminal
/// renderer's ANSI colors, distinct look for the "futuristic" theme (US-11).
fn piece_color(id: u8) -> Color {
    match id {
        1 => Color::new(0.0, 0.95, 1.0, 1.0),  // I - electric cyan
        2 => Color::new(1.0, 0.92, 0.1, 1.0),  // O - neon yellow
        3 => Color::new(0.85, 0.1, 1.0, 1.0),  // T - neon magenta
        4 => Color::new(0.1, 1.0, 0.4, 1.0),   // S - neon green
        5 => Color::new(1.0, 0.15, 0.35, 1.0), // Z - neon red
        6 => Color::new(0.2, 0.45, 1.0, 1.0),  // J - electric blue
        7 => Color::new(1.0, 0.6, 0.05, 1.0),  // L - neon orange
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
        // miniquad defaults to X11/XWayland on Linux. On at least one real Wayland session
        // (confirmed on hardware post-sprint) that path creates the window but never maps
        // it (`xwininfo` showed `Map State: IsUnMapped` deterministically), so the game
        // silently renders into an invisible window instead of erroring. Forcing native
        // Wayland sidesteps XWayland's window-mapping path entirely and fixed it.
        platform: miniquad::conf::Platform {
            linux_backend: miniquad::conf::LinuxBackend::WaylandOnly,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Runs the GPU-accelerated 3D renderer for the given game until the player quits or
/// closes the window (US-14 — macroquad's own frame-loop exit handles the OS close button,
/// no special-casing needed per ARCHITECTURE.md decision #10).
pub fn run(game: Game) {
    macroquad::Window::from_config(window_conf(), amain(game));
}

async fn amain(mut game: Game) {
    let camera = Camera3D {
        position: vec3(0.0, 1.5, 13.0),
        target: vec3(0.0, -0.5, 0.0),
        up: vec3(0.0, 1.0, 0.0),
        ..Default::default()
    };

    let mut last_tick = get_time();
    // Tracks the active piece's row at the start of the current gravity interval, so its
    // rendered Y can be lerped toward its current row (decision #7: smooth motion, engine
    // timing untouched). Reset to "no interpolation" on any player-initiated move so input
    // still feels instant.
    let mut anim_from_y = game.active().y as f32;
    let mut anim_tick_time = last_tick;

    let mut fx_start: Option<f64> = None;
    let mut fx_lines: u32 = 0;

    loop {
        let now = get_time();
        let interval = (game.gravity_interval_ms() as f64 / 1000.0).max(0.001);

        if is_quit_requested() {
            break;
        }

        if handle_input(&mut game, &mut anim_from_y, &mut anim_tick_time, now) {
            break;
        }

        if now - last_tick >= interval {
            let y_before = game.active().y;
            game.tick();
            let y_after = game.active().y;
            // A lock (new piece spawns higher up) or a no-op must snap, not glide backward.
            anim_from_y = if y_after > y_before { y_before as f32 } else { y_after as f32 };
            anim_tick_time = now;
            last_tick = now;

            let cleared = game.last_lines_cleared();
            if cleared > 0 {
                fx_start = Some(now);
                fx_lines = cleared;
            }
        }

        let t = ((now - anim_tick_time) / interval).clamp(0.0, 1.0) as f32;
        let interp_y = anim_from_y + (game.active().y as f32 - anim_from_y) * t;

        clear_background(Color::new(0.02, 0.02, 0.07, 1.0));

        set_camera(&camera);
        draw_board(&game, interp_y);

        set_default_camera();
        draw_hud(&mut game);
        draw_line_clear_fx(now, &mut fx_start, fx_lines);

        next_frame().await;
    }
}

/// Applies queued input for one frame. Returns `true` if the player asked to quit.
fn handle_input(game: &mut Game, anim_from_y: &mut f32, anim_tick_time: &mut f64, now: f64) -> bool {
    let mut moved = false;

    if is_key_pressed(KeyCode::Left) {
        game.move_left();
        moved = true;
    }
    if is_key_pressed(KeyCode::Right) {
        game.move_right();
        moved = true;
    }
    if is_key_pressed(KeyCode::Down) {
        game.soft_drop();
        moved = true;
    }
    if is_key_pressed(KeyCode::Up) {
        game.rotate();
        moved = true;
    }
    if is_key_pressed(KeyCode::Space) {
        game.hard_drop();
        moved = true;
    }
    if is_key_pressed(KeyCode::P) {
        game.toggle_pause();
    }
    if is_key_pressed(KeyCode::R) {
        game.restart();
        moved = true;
    }

    if moved {
        *anim_from_y = game.active().y as f32;
        *anim_tick_time = now;
    }

    is_key_pressed(KeyCode::Q) || is_key_pressed(KeyCode::Escape)
}

fn draw_board(game: &Game, active_render_y: f32) {
    for y in 0..HEIGHT as i32 {
        for x in 0..WIDTH as i32 {
            if let Some(id) = game.board().cell(x, y) {
                draw_neon_cell(x, y as f32, id);
            }
        }
    }

    if game.state() != GameState::GameOver {
        let id = game.active().piece_type.id();
        let base = game.active();
        let dy = active_render_y - base.y as f32;
        for (x, y) in base.cells() {
            draw_neon_cell(x, y as f32 + dy, id);
        }
    }
}

/// Cheap glow (decision #6): a larger, translucent backing cube drawn behind the solid
/// cell cube, instead of a real bloom/post-process pass.
fn draw_neon_cell(x: i32, y: f32, id: u8) {
    let pos = cell_world_pos(x, y);
    let color = piece_color(id);
    let glow = Color::new(color.r, color.g, color.b, 0.25);
    draw_cube(pos, vec3(CELL_SIZE * 1.5, CELL_SIZE * 1.5, CELL_SIZE * 0.4), None, glow);
    draw_cube(pos, vec3(CELL_SIZE * 0.85, CELL_SIZE * 0.85, CELL_SIZE * 0.85), None, color);
    draw_cube_wires(pos, vec3(CELL_SIZE * 0.85, CELL_SIZE * 0.85, CELL_SIZE * 0.85), WHITE);
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
        "P     pause",
        "R     restart",
        "Q/Esc quit",
    ];
    let origin_x = screen_width() - 240.0;
    for (i, line) in lines.iter().enumerate() {
        draw_text(line, origin_x, 30.0 + i as f32 * 22.0, 20.0, HUD_COLOR);
    }
}

fn draw_status_overlay(game: &Game) {
    let cx = screen_width() / 2.0;
    let cy = screen_height() / 2.0;
    match game.state() {
        GameState::Paused => {
            draw_text("PAUSED", cx - 60.0, cy, 40.0, WHITE);
        }
        GameState::GameOver => {
            draw_text("GAME OVER", cx - 100.0, cy, 40.0, Color::new(1.0, 0.2, 0.3, 1.0));
            draw_text(
                format!("Final score: {}", game.score()),
                cx - 100.0,
                cy + 40.0,
                24.0,
                WHITE,
            );
            draw_text("Press R to restart", cx - 100.0, cy + 70.0, 24.0, WHITE);
        }
        GameState::Playing => {}
    }
}

/// One-shot full-scene flash on line clear. The engine only exposes a cleared-line *count*
/// (decision #8), not which rows, so this flashes the whole scene rather than targeting
/// specific rows; intensity scales with lines cleared (a tetris flashes brighter than a
/// single). Clears itself once `LINE_CLEAR_FX_DURATION` has elapsed.
fn draw_line_clear_fx(now: f64, fx_start: &mut Option<f64>, fx_lines: u32) {
    let Some(start) = *fx_start else { return };
    let elapsed = now - start;
    if elapsed >= LINE_CLEAR_FX_DURATION {
        *fx_start = None;
        return;
    }
    let t = (elapsed / LINE_CLEAR_FX_DURATION) as f32;
    let intensity = (fx_lines as f32 / 4.0).min(1.0);
    let alpha = (1.0 - t) * 0.5 * intensity;
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(1.0, 1.0, 1.0, alpha));
}
