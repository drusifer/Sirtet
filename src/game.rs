use crate::board::Board;
use crate::piece::{Piece, PieceType, ALL_PIECE_TYPES};
use rand::seq::SliceRandom;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Playing,
    Paused,
    GameOver,
}

/// Level-dependent gravity interval per ARCHITECTURE.md decision #5.
pub fn gravity_interval_ms(level: u32) -> u64 {
    let ms = 1000.0 * 0.85f64.powi(level as i32 - 1);
    ms.round().max(100.0) as u64
}

/// Points for clearing `lines` rows in a single lock, before the level multiplier
/// (US-4 AC: 100/300/500/800 x level for 1/2/3/4 lines).
pub fn line_clear_base_score(lines: usize) -> u32 {
    match lines {
        1 => 100,
        2 => 300,
        3 => 500,
        4 => 800,
        _ => 0,
    }
}

/// Lines cleared per level per US-5 AC.
const LINES_PER_LEVEL: u32 = 10;

pub struct Game {
    board: Board,
    active: Piece,
    bag: Vec<PieceType>,
    state: GameState,
    score: u32,
    level: u32,
    lines_cleared: u32,
    last_lines_cleared: u32,
}

impl Game {
    pub fn new() -> Self {
        let mut bag = Vec::new();
        Self::refill_bag(&mut bag);
        let first = bag.pop().expect("freshly refilled bag is never empty");
        Game {
            board: Board::new(),
            active: Piece::spawn(first),
            bag,
            state: GameState::Playing,
            score: 0,
            level: 1,
            lines_cleared: 0,
            last_lines_cleared: 0,
        }
    }

    fn refill_bag(bag: &mut Vec<PieceType>) {
        let mut fresh = ALL_PIECE_TYPES.to_vec();
        fresh.shuffle(&mut rand::rng());
        bag.extend(fresh);
    }

    fn next_from_bag(&mut self) -> PieceType {
        if self.bag.is_empty() {
            Self::refill_bag(&mut self.bag);
        }
        self.bag.pop().expect("bag refilled if empty")
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn active(&self) -> &Piece {
        &self.active
    }

    /// The piece that will spawn after the current one locks (preview only, does not
    /// consume the bag).
    pub fn peek_next(&mut self) -> PieceType {
        if self.bag.is_empty() {
            Self::refill_bag(&mut self.bag);
        }
        *self.bag.last().expect("bag refilled if empty")
    }

    pub fn state(&self) -> GameState {
        self.state
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn level(&self) -> u32 {
        self.level
    }

    pub fn lines_cleared(&self) -> u32 {
        self.lines_cleared
    }

    /// How many lines were cleared by the most recent lock (0 if the last lock cleared
    /// none, or if no piece has locked yet). Renderers use this to trigger a one-shot
    /// visual effect; it is not cumulative like `lines_cleared()`.
    pub fn last_lines_cleared(&self) -> u32 {
        self.last_lines_cleared
    }

    pub fn gravity_interval_ms(&self) -> u64 {
        gravity_interval_ms(self.level)
    }

    fn is_playing(&self) -> bool {
        self.state == GameState::Playing
    }

    fn try_move(&mut self, dx: i32, dy: i32) -> bool {
        if !self.is_playing() {
            return false;
        }
        let candidate = Piece {
            x: self.active.x + dx,
            y: self.active.y + dy,
            ..self.active
        };
        if self.board.is_area_free(&candidate.cells()) {
            self.active = candidate;
            true
        } else {
            false
        }
    }

    pub fn move_left(&mut self) -> bool {
        self.try_move(-1, 0)
    }

    pub fn move_right(&mut self) -> bool {
        self.try_move(1, 0)
    }

    /// Soft drop: move down one row. Does not lock even if blocked.
    pub fn soft_drop(&mut self) -> bool {
        self.try_move(0, 1)
    }

    pub fn rotate(&mut self) -> bool {
        if !self.is_playing() {
            return false;
        }
        let candidate = self.active.rotated_cw();
        if self.board.is_area_free(&candidate.cells()) {
            self.active = candidate;
            true
        } else {
            false
        }
    }

    /// Instantly drops to the lowest legal position and locks.
    pub fn hard_drop(&mut self) {
        if !self.is_playing() {
            return;
        }
        while self.try_move(0, 1) {}
        self.lock_active();
    }

    /// One gravity step: try to fall one row; if blocked, lock and spawn the next piece.
    pub fn tick(&mut self) {
        if !self.is_playing() {
            return;
        }
        if !self.try_move(0, 1) {
            self.lock_active();
        } else {
            self.last_lines_cleared = 0;
        }
    }


    fn lock_active(&mut self) {
        let id = self.active.piece_type.id();
        self.board.lock_cells(&self.active.cells(), id);
        let cleared = self.board.clear_full_lines();
        self.last_lines_cleared = cleared as u32;
        if cleared > 0 {
            self.score += line_clear_base_score(cleared) * self.level;
            self.lines_cleared += cleared as u32;
            self.level = 1 + self.lines_cleared / LINES_PER_LEVEL;
        }
        self.spawn_next();
    }

    fn spawn_next(&mut self) {
        let next_type = self.next_from_bag();
        let spawned = Piece::spawn(next_type);
        if self.board.is_area_free(&spawned.cells()) {
            self.active = spawned;
        } else {
            self.active = spawned;
            self.state = GameState::GameOver;
        }
    }

    pub fn toggle_pause(&mut self) {
        self.state = match self.state {
            GameState::Playing => GameState::Paused,
            GameState::Paused => GameState::Playing,
            GameState::GameOver => GameState::GameOver,
        };
    }

    pub fn restart(&mut self) {
        *self = Game::new();
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_left_blocked_at_left_wall() {
        let mut game = Game::new();
        for _ in 0..20 {
            game.move_left();
        }
        let cells = game.active().cells();
        let min_x = cells.iter().map(|&(x, _)| x).min().unwrap();
        assert!(min_x >= 0, "piece pushed off the left edge: min_x={min_x}");
    }

    #[test]
    fn move_right_blocked_at_right_wall() {
        let mut game = Game::new();
        for _ in 0..20 {
            game.move_right();
        }
        let cells = game.active().cells();
        let max_x = cells.iter().map(|&(x, _)| x).max().unwrap();
        assert!(max_x < crate::board::WIDTH as i32);
    }

    #[test]
    fn rotation_rejected_when_it_would_collide() {
        let mut game = Game::new();
        for _ in 0..20 {
            game.move_left();
        }
        for _ in 0..8 {
            let before = (game.active().rotation, game.active().x, game.active().y);
            let accepted = game.rotate();
            let after = (game.active().rotation, game.active().x, game.active().y);
            if accepted {
                assert!(game.board().is_area_free(&game.active().cells()));
                assert_ne!(before, after, "rotate() reported success but nothing changed");
            } else {
                assert_eq!(before, after, "rotate() rejected but piece state changed anyway");
            }
        }
    }

    #[test]
    fn piece_locks_when_it_cannot_fall_further() {
        let mut game = Game::new();
        game.hard_drop();
        // after locking, a new active piece has spawned at the top again
        assert_eq!(game.active().y, 0);
        // and the board now has at least one occupied cell from the locked piece
        let occupied = (0..crate::board::HEIGHT as i32).any(|y| {
            (0..crate::board::WIDTH as i32).any(|x| !game.board().is_cell_free(x, y))
        });
        assert!(occupied);
    }

    #[test]
    fn new_piece_spawns_from_bag_after_lock() {
        let mut game = Game::new();
        let first_type = game.active().piece_type;
        // hard-drop 7 times (one full bag) - every piece type must appear exactly once
        let mut seen = vec![first_type];
        for _ in 0..7 {
            game.hard_drop();
            seen.push(game.active().piece_type);
        }
        // first 7 spawns (index 0..7) should be a permutation of all 7 types
        let mut first_seven: Vec<_> = seen[0..7].to_vec();
        first_seven.sort_by_key(|p| p.id());
        let mut all: Vec<_> = crate::piece::ALL_PIECE_TYPES.to_vec();
        all.sort_by_key(|p| p.id());
        assert_eq!(first_seven, all);
    }

    #[test]
    fn line_clear_base_score_matches_us4_table() {
        assert_eq!(line_clear_base_score(1), 100);
        assert_eq!(line_clear_base_score(2), 300);
        assert_eq!(line_clear_base_score(3), 500);
        assert_eq!(line_clear_base_score(4), 800);
        assert_eq!(line_clear_base_score(0), 0);
    }

    #[test]
    fn locking_with_a_complete_row_awards_score_and_counts_the_line() {
        let mut game = Game::new();
        game.board.test_fill_row(crate::board::HEIGHT - 1, None, 9);
        game.lock_active();
        assert_eq!(game.score, 100); // 1 line x level 1 x 100
        assert_eq!(game.lines_cleared, 1);
        assert_eq!(game.level, 1); // not yet at the 10-line boundary
    }

    #[test]
    fn locking_with_four_complete_rows_scores_as_a_tetris() {
        let mut game = Game::new();
        for y in (crate::board::HEIGHT - 4)..crate::board::HEIGHT {
            game.board.test_fill_row(y, None, 9);
        }
        game.lock_active();
        assert_eq!(game.score, 800);
        assert_eq!(game.lines_cleared, 4);
    }

    #[test]
    fn level_increments_every_ten_lines_cumulative() {
        let mut game = Game::new();
        for _ in 0..9 {
            game.board.test_fill_row(crate::board::HEIGHT - 1, None, 9);
            game.lock_active();
        }
        assert_eq!(game.lines_cleared, 9);
        assert_eq!(game.level, 1, "level must not tick up before the 10th line");

        game.board.test_fill_row(crate::board::HEIGHT - 1, None, 9);
        game.lock_active();
        assert_eq!(game.lines_cleared, 10);
        assert_eq!(game.level, 2, "level must tick up exactly at the 10th line");
    }

    #[test]
    fn score_multiplier_uses_level_at_time_of_clear() {
        let mut game = Game::new();
        for _ in 0..10 {
            game.board.test_fill_row(crate::board::HEIGHT - 1, None, 9);
            game.lock_active();
        }
        // now at level 2; score so far = 10 x (100 x 1) = 1000
        assert_eq!(game.score, 1000);
        assert_eq!(game.level, 2);

        game.board.test_fill_row(crate::board::HEIGHT - 1, None, 9);
        game.lock_active();
        // 11th line clears at level 2: +100 x 2 = 200
        assert_eq!(game.score, 1200);
    }

    #[test]
    fn locking_without_a_complete_row_awards_no_score() {
        let mut game = Game::new();
        game.lock_active();
        assert_eq!(game.score, 0);
        assert_eq!(game.lines_cleared, 0);
    }

    #[test]
    fn last_lines_cleared_reflects_most_recent_lock_only() {
        let mut game = Game::new();
        assert_eq!(game.last_lines_cleared(), 0, "nothing locked yet");

        game.board.test_fill_row(crate::board::HEIGHT - 1, None, 9);
        game.lock_active();
        assert_eq!(game.last_lines_cleared(), 1);

        for y in (crate::board::HEIGHT - 4)..crate::board::HEIGHT {
            game.board.test_fill_row(y, None, 9);
        }
        game.lock_active();
        assert_eq!(game.last_lines_cleared(), 4, "a tetris updates the accessor to 4");

        game.lock_active();
        assert_eq!(
            game.last_lines_cleared(),
            0,
            "a lock with no completed row must reset the accessor to 0, not leave it stale"
        );
    }

        #[test]
    fn gravity_interval_decreases_and_has_a_floor() {
        assert_eq!(gravity_interval_ms(1), 1000);
        assert!(gravity_interval_ms(2) < gravity_interval_ms(1));
        assert!(gravity_interval_ms(50) >= 100);
    }

    #[test]
    fn tick_locks_piece_when_blocked_below() {
        let mut game = Game::new();
        let mut saw_descent = false;
        let mut relocked = false;
        // bounded by board height: no piece needs more than HEIGHT ticks to lock once
        for _ in 0..(crate::board::HEIGHT + 2) {
            let y_before = game.active().y;
            game.tick();
            if game.active().y > 0 {
                saw_descent = true;
            }
            if saw_descent && game.active().y == 0 && y_before > 0 {
                relocked = true;
                break;
            }
        }
        assert!(saw_descent, "piece never moved down under gravity");
        assert!(relocked, "piece never locked and respawned at the top");
    }

    #[test]
    fn paused_state_blocks_gravity_and_movement() {
        let mut game = Game::new();
        game.toggle_pause();
        assert_eq!(game.state(), GameState::Paused);

        let before = (game.active().x, game.active().y, game.active().rotation);
        assert!(!game.move_left());
        assert!(!game.move_right());
        assert!(!game.soft_drop());
        assert!(!game.rotate());
        game.tick();
        game.hard_drop();
        let after = (game.active().x, game.active().y, game.active().rotation);
        assert_eq!(before, after, "paused game must not move, rotate, or drop the piece");
    }

    #[test]
    fn unpause_restores_normal_play() {
        let mut game = Game::new();
        game.toggle_pause();
        game.toggle_pause();
        assert_eq!(game.state(), GameState::Playing);
        assert!(game.soft_drop(), "movement must work again after unpause");
    }

    #[test]
    fn toggle_pause_is_a_no_op_once_game_over() {
        let mut game = Game::new();
        for _ in 0..250 {
            if game.state() == GameState::GameOver {
                break;
            }
            game.hard_drop();
        }
        assert_eq!(game.state(), GameState::GameOver);
        game.toggle_pause();
        assert_eq!(game.state(), GameState::GameOver, "pause must not resurrect a game-over game");
    }

    #[test]
    fn restart_resets_score_level_lines_and_board() {
        let mut game = Game::new();
        game.board.test_fill_row(crate::board::HEIGHT - 1, None, 9);
        game.lock_active();
        assert!(game.score() > 0);

        game.restart();
        assert_eq!(game.score(), 0);
        assert_eq!(game.level(), 1);
        assert_eq!(game.lines_cleared(), 0);
        assert_eq!(game.state(), GameState::Playing);
        for y in 0..crate::board::HEIGHT as i32 {
            for x in 0..crate::board::WIDTH as i32 {
                assert!(game.board().is_cell_free(x, y), "restart must clear the board");
            }
        }
    }

    #[test]
    fn restart_works_after_game_over() {
        let mut game = Game::new();
        for _ in 0..250 {
            if game.state() == GameState::GameOver {
                break;
            }
            game.hard_drop();
        }
        assert_eq!(game.state(), GameState::GameOver);
        game.restart();
        assert_eq!(game.state(), GameState::Playing);
        assert!(game.soft_drop());
    }

        #[test]
    fn spawn_collision_triggers_game_over() {
        let mut game = Game::new();
        // stack hard-drops until the board fills near the top and a spawn collides
        for _ in 0..250 {
            if game.state() == GameState::GameOver {
                break;
            }
            game.hard_drop();
        }
        assert_eq!(game.state(), GameState::GameOver);
    }
}
