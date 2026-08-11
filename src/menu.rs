use macroquad::prelude::*;

use crate::battle::GameMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererKind {
    NeonGrid2D,
    SpatialBox3D,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    StartMode(GameMode),
    Resume,
    Restart,
    QuitToMenu,
}

/// Shared single-player screen state — identical shape in both `gfx3d.rs` and `gfx3d_box.rs`'s
/// `amain`, unlike `BattleScreen` (kept per-file since its `GameOver` data differs per renderer).
pub enum SingleScreen {
    Playing,
    Paused(Menu),
    GameOver(Menu),
}

pub struct Menu {
    pub title: &'static str,
    pub options: Vec<(MenuAction, &'static str)>,
    pub selected: usize,
}

impl Menu {
    pub fn main_menu() -> Self {
        Menu {
            title: "SIRTET",
            options: vec![
                (MenuAction::StartMode(GameMode::Single), "Single Player"),
                (MenuAction::StartMode(GameMode::TwoPlayerLocal), "Local 2-Player Battle"),
                (MenuAction::StartMode(GameMode::VsCpu), "VS CPU Opponent"),
            ],
            selected: 0,
        }
    }

    pub fn pause_menu() -> Self {
        Menu {
            title: "PAUSED",
            options: vec![
                (MenuAction::Resume, "Resume"),
                (MenuAction::Restart, "Restart Match"),
                (MenuAction::QuitToMenu, "Quit to Main Menu"),
            ],
            selected: 0,
        }
    }

    /// Same as `pause_menu()` but pre-selects Restart — used when the pause menu is opened via
    /// the `R` shortcut during play, so the player isn't forced to navigate down to it.
    pub fn pause_menu_restart_selected() -> Self {
        Menu {
            selected: 1,
            ..Self::pause_menu()
        }
    }

    pub fn game_over_menu() -> Self {
        Menu {
            title: "GAME OVER",
            options: vec![
                (MenuAction::Restart, "Restart Match"),
                (MenuAction::QuitToMenu, "Main Menu"),
            ],
            selected: 0,
        }
    }

    /// Moves the selection cursor by `delta` (+1 down, -1 up), wrapping around.
    pub fn move_selection(&mut self, delta: i32) {
        let len = self.options.len() as i32;
        self.selected = (self.selected as i32 + delta).rem_euclid(len) as usize;
    }

    /// The action for the currently selected option.
    pub fn confirm(&self) -> MenuAction {
        self.options[self.selected].0
    }

    /// Polls keyboard input and returns the confirmed action, if any, this frame. Thin wrapper
    /// over `move_selection`/`confirm` — those are unit-tested directly since this needs a live
    /// macroquad context to run.
    pub fn update(&mut self) -> Option<MenuAction> {
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            self.move_selection(-1);
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            self.move_selection(1);
        }
        if is_key_pressed(KeyCode::Enter) {
            return Some(self.confirm());
        }
        None
    }

    /// Runs this menu's own input/draw loop until the player confirms an action or closes the
    /// window. Shared by every renderer/orchestrator that shows a menu screen, so the
    /// poll-input/draw/next_frame boilerplate isn't duplicated in each one.
    pub async fn run_until_choice(mut self) -> Option<MenuAction> {
        loop {
            if is_quit_requested() {
                return None;
            }
            let action = self.update();
            clear_background(Color::new(0.02, 0.02, 0.07, 1.0));
            self.draw(screen_width(), screen_height());
            // Always cross a frame boundary before returning, even when an action was just
            // confirmed — otherwise the same Enter press that confirmed it is still "just
            // pressed" on the very next poll, so a menu shown immediately after this one (e.g.
            // the mode menu right after the renderer menu) would instantly auto-confirm its own
            // first option instead of ever being shown to the player.
            next_frame().await;
            if let Some(action) = action {
                return Some(action);
            }
        }
    }

    pub fn draw(&self, screen_w: f32, screen_h: f32) {
        let cx = screen_w / 2.0;
        let cy = screen_h / 2.0;
        let bw = 420.0;
        let row_h = 40.0;
        let bh = 90.0 + self.options.len() as f32 * row_h;
        let left = cx - bw / 2.0;
        let top = cy - bh / 2.0;
        let cyan = Color::new(0.0, 0.95, 1.0, 1.0);

        draw_rectangle(left, top, bw, bh, Color::new(0.0, 0.0, 0.0, 0.88));
        draw_rectangle_lines(left, top, bw, bh, 2.0, cyan);
        draw_text(self.title, left + 30.0, top + 45.0, 36.0, cyan);

        for (i, (_, label)) in self.options.iter().enumerate() {
            let y = top + 80.0 + i as f32 * row_h;
            let (marker, color) = if i == self.selected {
                ("> ", WHITE)
            } else {
                ("  ", Color::new(0.7, 0.8, 0.9, 1.0))
            };
            draw_text(format!("{marker}{label}"), left + 30.0, y, 26.0, color);
        }
    }
}

const RENDERER_OPTIONS: [(RendererKind, &str); 2] = [
    (RendererKind::NeonGrid2D, "2D"),
    (RendererKind::SpatialBox3D, "3D"),
];

const MODE_OPTIONS: [(GameMode, &str); 3] = [
    (GameMode::Single, "1P"),
    (GameMode::TwoPlayerLocal, "2P"),
    (GameMode::VsCpu, "vs CPU"),
];

/// One combined options screen (renderer + mode on the same screen as radio-button rows, rather
/// than two separate menus in sequence) — used by entry points that can offer more than one
/// rendering engine (the WASM build). Up/Down moves between the two rows; Left/Right moves the
/// selected radio button within the highlighted row; Enter confirms both at once.
pub struct OptionsScreen {
    renderer_idx: usize,
    mode_idx: usize,
    field: usize,
}

impl OptionsScreen {
    pub fn new() -> Self {
        OptionsScreen {
            renderer_idx: 0,
            mode_idx: 0,
            field: 0,
        }
    }

    pub fn renderer(&self) -> RendererKind {
        RENDERER_OPTIONS[self.renderer_idx].0
    }

    pub fn mode(&self) -> GameMode {
        MODE_OPTIONS[self.mode_idx].0
    }

    /// Moves the highlighted field by `delta` (+1 down, -1 up), wrapping around.
    fn move_field(&mut self, delta: i32) {
        self.field = (self.field as i32 + delta).rem_euclid(2) as usize;
    }

    /// Cycles the currently highlighted field's value by `delta`, wrapping around.
    fn cycle_value(&mut self, delta: i32) {
        if self.field == 0 {
            self.renderer_idx =
                (self.renderer_idx as i32 + delta).rem_euclid(RENDERER_OPTIONS.len() as i32) as usize;
        } else {
            self.mode_idx =
                (self.mode_idx as i32 + delta).rem_euclid(MODE_OPTIONS.len() as i32) as usize;
        }
    }

    /// Runs this screen's own input/draw loop until the player confirms (Enter) or closes the
    /// window, returning the chosen `(renderer, mode)` pair.
    pub async fn run_until_choice(mut self) -> Option<(RendererKind, GameMode)> {
        loop {
            if is_quit_requested() {
                return None;
            }
            if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                self.move_field(-1);
            }
            if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                self.move_field(1);
            }
            if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                self.cycle_value(-1);
            }
            if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                self.cycle_value(1);
            }
            let confirmed = is_key_pressed(KeyCode::Enter);

            clear_background(Color::new(0.02, 0.02, 0.07, 1.0));
            self.draw(screen_width(), screen_height());
            // See Menu::run_until_choice — always cross a frame boundary before returning so a
            // screen shown right after this one doesn't see a stale "just pressed" Enter.
            next_frame().await;
            if confirmed {
                return Some((self.renderer(), self.mode()));
            }
        }
    }

    fn draw(&self, screen_w: f32, screen_h: f32) {
        let cx = screen_w / 2.0;
        let cy = screen_h / 2.0;
        let bw = 480.0;
        let bh = 200.0;
        let left = cx - bw / 2.0;
        let top = cy - bh / 2.0;
        let cyan = Color::new(0.0, 0.95, 1.0, 1.0);

        draw_rectangle(left, top, bw, bh, Color::new(0.0, 0.0, 0.0, 0.88));
        draw_rectangle_lines(left, top, bw, bh, 2.0, cyan);
        draw_text("SIRTET", left + 30.0, top + 45.0, 36.0, cyan);

        let renderer_labels: Vec<&str> = RENDERER_OPTIONS.iter().map(|(_, l)| *l).collect();
        let mode_labels: Vec<&str> = MODE_OPTIONS.iter().map(|(_, l)| *l).collect();

        draw_radio_row("Game:", &renderer_labels, self.renderer_idx, self.field == 0, left + 30.0, top + 90.0);
        draw_radio_row("Players:", &mode_labels, self.mode_idx, self.field == 1, left + 30.0, top + 130.0);

        draw_text(
            "Up/Down: Row   Left/Right: Select   Enter: Start",
            left + 30.0,
            top + bh - 25.0,
            18.0,
            Color::new(0.6, 0.95, 1.0, 1.0),
        );
    }
}

/// Draws one label followed by a horizontal row of radio buttons, e.g. `Game: (*) 2D  ( ) 3D`.
fn draw_radio_row(label: &str, options: &[&str], selected: usize, row_active: bool, x: f32, y: f32) {
    let marker = if row_active { "> " } else { "  " };
    let label_color = if row_active { WHITE } else { Color::new(0.7, 0.8, 0.9, 1.0) };
    let label_text = format!("{marker}{label:<9}");
    draw_text(&label_text, x, y, 24.0, label_color);

    let mut ox = x + measure_text(&label_text, None, 24, 1.0).width + 10.0;
    for (i, opt) in options.iter().enumerate() {
        let radio = if i == selected { "(*)" } else { "( )" };
        let color = if i == selected { WHITE } else { Color::new(0.5, 0.6, 0.7, 1.0) };
        let text = format!("{radio} {opt}");
        draw_text(&text, ox, y, 22.0, color);
        ox += measure_text(&text, None, 22, 1.0).width + 28.0;
    }
}

impl Default for OptionsScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_menu_has_three_mode_options() {
        let menu = Menu::main_menu();
        assert_eq!(menu.options.len(), 3);
        assert_eq!(menu.selected, 0);
    }

    #[test]
    fn move_selection_wraps_around_downward() {
        let mut menu = Menu::pause_menu();
        menu.selected = 2;
        menu.move_selection(1);
        assert_eq!(menu.selected, 0);
    }

    #[test]
    fn move_selection_wraps_around_upward() {
        let mut menu = Menu::pause_menu();
        menu.selected = 0;
        menu.move_selection(-1);
        assert_eq!(menu.selected, 2);
    }

    #[test]
    fn confirm_resolves_selected_action() {
        let mut menu = Menu::main_menu();
        menu.selected = 2;
        assert_eq!(menu.confirm(), MenuAction::StartMode(GameMode::VsCpu));
    }

    #[test]
    fn pause_menu_restart_selected_preselects_restart() {
        let menu = Menu::pause_menu_restart_selected();
        assert_eq!(menu.confirm(), MenuAction::Restart);
    }

    #[test]
    fn game_over_menu_has_two_options() {
        let menu = Menu::game_over_menu();
        assert_eq!(menu.options.len(), 2);
    }

    #[test]
    fn options_screen_defaults_to_first_renderer_and_mode() {
        let opts = OptionsScreen::new();
        assert_eq!(opts.renderer(), RendererKind::NeonGrid2D);
        assert_eq!(opts.mode(), GameMode::Single);
    }

    #[test]
    fn options_screen_cycle_value_wraps_around_in_current_field() {
        let mut opts = OptionsScreen::new();
        opts.cycle_value(-1);
        assert_eq!(opts.renderer(), RendererKind::SpatialBox3D);
        opts.cycle_value(1);
        assert_eq!(opts.renderer(), RendererKind::NeonGrid2D);
    }

    #[test]
    fn options_screen_move_field_switches_which_value_cycle_value_affects() {
        let mut opts = OptionsScreen::new();
        opts.move_field(1);
        opts.cycle_value(1);
        assert_eq!(opts.mode(), GameMode::TwoPlayerLocal);
        assert_eq!(opts.renderer(), RendererKind::NeonGrid2D);
    }
}
