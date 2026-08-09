use crate::board::{WIDTH, HEIGHT};
use crate::game::Game;

#[derive(Debug, Clone, Copy)]
pub struct CpuMove {
    pub target_rotations: usize,
    pub target_x: i32,
}

pub struct CpuAgent;

impl CpuAgent {
    pub fn new() -> Self {
        CpuAgent
    }

    pub fn compute_best_move(&self, game: &Game) -> Option<CpuMove> {
        if game.state() != crate::game::GameState::Playing {
            return None;
        }

        let mut best_score = f64::NEG_INFINITY;
        let mut best_move = None;

        for rotations in 0..4 {
            for target_x in 0..WIDTH as i32 {
                if let Some(score) = self.evaluate_placement(game, rotations, target_x)
                    && score > best_score
                {
                    best_score = score;
                    best_move = Some(CpuMove {
                        target_rotations: rotations,
                        target_x,
                    });
                }
            }
        }


        best_move
    }

    fn evaluate_placement(&self, game: &Game, rotations: usize, target_x: i32) -> Option<f64> {
        let active = game.active();
        let mut test_piece = *active;
        for _ in 0..rotations {
            test_piece = test_piece.rotated_cw();
        }
        test_piece.x = target_x;

        if !game.board().is_area_free(&test_piece.cells()) {
            return None;
        }

        // Hard drop simulation
        while game.board().is_area_free(&test_piece.cells()) {
            test_piece.y += 1;
        }
        test_piece.y -= 1;

        // Clone board to simulate lock and line clear
        let mut sim_board = game.board().clone();
        sim_board.lock_cells(&test_piece.cells(), test_piece.piece_type.id());
        let lines_cleared = sim_board.clear_full_lines();

        let (agg_height, holes, bumpiness) = self.calculate_board_metrics(&sim_board);

        let score = (-0.51 * agg_height as f64)
            + (0.76 * lines_cleared as f64)
            + (-0.36 * holes as f64)
            + (-0.18 * bumpiness as f64);

        Some(score)
    }

    fn calculate_board_metrics(&self, board: &crate::board::Board) -> (usize, usize, usize) {
        let mut col_heights = [0; WIDTH];
        let mut holes = 0;

        for (x, height_item) in col_heights.iter_mut().enumerate().take(WIDTH) {
            let mut found_top = false;
            for y in 0..HEIGHT {
                if board.cell(x as i32, y as i32).is_some() {
                    if !found_top {
                        *height_item = HEIGHT - y;
                        found_top = true;
                    }
                } else if found_top {
                    holes += 1;
                }
            }
        }

        let agg_height: usize = col_heights.iter().sum();
        let mut bumpiness = 0;
        for i in 0..WIDTH - 1 {
            bumpiness += col_heights[i].abs_diff(col_heights[i + 1]);
        }

        (agg_height, holes, bumpiness)
    }


    pub fn make_move(&self, game: &mut Game) {
        if let Some(m) = self.compute_best_move(game) {
            for _ in 0..m.target_rotations {
                game.rotate();
            }
            let active_x = game.active().x;
            if active_x < m.target_x {
                for _ in 0..(m.target_x - active_x) {
                    game.move_right();
                }
            } else if active_x > m.target_x {
                for _ in 0..(active_x - m.target_x) {
                    game.move_left();
                }
            }
            game.hard_drop();
        }
    }
}

impl Default for CpuAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_agent_compute_move() {
        let game = Game::new();
        let agent = CpuAgent::new();
        let best = agent.compute_best_move(&game);
        assert!(best.is_some());
    }

    #[test]
    fn test_cpu_agent_make_move() {
        let mut game = Game::new();
        let agent = CpuAgent::new();
        agent.make_move(&mut game);
        let has_occupied = (0..WIDTH as i32)
            .any(|x| (0..HEIGHT as i32).any(|y| game.board().cell(x, y).is_some()));
        assert!(has_occupied);
    }
}

