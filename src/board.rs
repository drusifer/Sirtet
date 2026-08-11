pub const WIDTH: usize = 10;
pub const HEIGHT: usize = 20;

#[derive(Clone)]
pub struct Board {
    cells: Vec<Vec<Option<u8>>>,
}


impl Board {
    pub fn new() -> Self {
        Board {
            cells: vec![vec![None; WIDTH]; HEIGHT],
        }
    }

    pub fn is_cell_free(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x as usize >= WIDTH || y as usize >= HEIGHT {
            return false;
        }
        self.cells[y as usize][x as usize].is_none()
    }

    pub fn is_area_free(&self, cells: &[(i32, i32)]) -> bool {
        cells.iter().all(|&(x, y)| self.is_cell_free(x, y))
    }

    /// The piece-id occupying (x, y), or `None` if empty/out-of-bounds. Used by the
    /// renderer to look up a locked cell's color.
    pub fn cell(&self, x: i32, y: i32) -> Option<u8> {
        if x < 0 || y < 0 || x as usize >= WIDTH || y as usize >= HEIGHT {
            return None;
        }
        self.cells[y as usize][x as usize]
    }

    /// Writes `id` into every cell in `cells` (caller guarantees they are in-bounds/free
    /// via `is_area_free` first).
    pub fn lock_cells(&mut self, cells: &[(i32, i32)], id: u8) {
        for &(x, y) in cells {
            if x >= 0 && y >= 0 && (x as usize) < WIDTH && (y as usize) < HEIGHT {
                self.cells[y as usize][x as usize] = Some(id);
            }
        }
    }

    /// Removes every fully-occupied row, shifts everything above down by the number
    /// cleared, and returns how many rows were cleared.
    pub fn clear_full_lines(&mut self) -> usize {
        let before = self.cells.len();
        self.cells.retain(|row| row.iter().any(|c| c.is_none()));
        let cleared = before - self.cells.len();
        for _ in 0..cleared {
            self.cells.insert(0, vec![None; WIDTH]);
        }
        cleared
    }

    /// Pushes `count` garbage lines into the bottom of the board, shifting top rows up.
    /// Each garbage row is solid (block ID 8) except for a random hole column.
    pub fn push_garbage_lines(&mut self, count: usize) {
        for _ in 0..count {
            if !self.cells.is_empty() {
                self.cells.remove(0);
            }
            let hole_x = macroquad::rand::gen_range(0, WIDTH);
            let mut row = vec![Some(8); WIDTH];
            row[hole_x] = None;
            self.cells.push(row);
        }
    }

}



impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl Board {
    /// Test-only: force row `y` to be fully occupied (or all-but-one, if `except_x` is
    /// given), so game.rs tests can drive line-clear scenarios deterministically without
    /// depending on random piece placement.
    pub(crate) fn test_fill_row(&mut self, y: usize, except_x: Option<usize>, id: u8) {
        for x in 0..WIDTH {
            if Some(x) != except_x {
                self.cells[y][x] = Some(id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_board_is_empty_everywhere() {
        let board = Board::new();
        for y in 0..HEIGHT as i32 {
            for x in 0..WIDTH as i32 {
                assert!(board.is_cell_free(x, y));
            }
        }
    }

    #[test]
    fn out_of_bounds_cells_are_not_free() {
        let board = Board::new();
        assert!(!board.is_cell_free(-1, 0));
        assert!(!board.is_cell_free(0, -1));
        assert!(!board.is_cell_free(WIDTH as i32, 0));
        assert!(!board.is_cell_free(0, HEIGHT as i32));
    }

    #[test]
    fn occupied_cell_is_not_free() {
        let mut board = Board::new();
        board.cells[5][3] = Some(1);
        assert!(!board.is_cell_free(3, 5));
        assert!(board.is_cell_free(4, 5));
    }

    #[test]
    fn clear_full_lines_removes_full_rows_and_shifts_down() {
        let mut board = Board::new();
        // fill row HEIGHT-1 completely
        for x in 0..WIDTH {
            board.cells[HEIGHT - 1][x] = Some(1);
        }
        // put a marker in the row above, at column 0, to verify it shifts down by one
        board.cells[HEIGHT - 2][0] = Some(2);

        let cleared = board.clear_full_lines();
        assert_eq!(cleared, 1);
        assert_eq!(board.cells[HEIGHT - 1][0], Some(2));
        assert!(board.is_cell_free(1, (HEIGHT - 1) as i32));
        // top row is now empty (shifted in)
        assert!((0..WIDTH).all(|x| board.cells[0][x].is_none()));
    }

    #[test]
    fn clear_full_lines_handles_multiple_rows_at_once() {
        let mut board = Board::new();
        for row in [HEIGHT - 1, HEIGHT - 2, HEIGHT - 3, HEIGHT - 4] {
            for x in 0..WIDTH {
                board.cells[row][x] = Some(3);
            }
        }
        let cleared = board.clear_full_lines();
        assert_eq!(cleared, 4);
        for y in 0..HEIGHT as i32 {
            for x in 0..WIDTH as i32 {
                assert!(board.is_cell_free(x, y));
            }
        }
    }

    #[test]
    fn clear_full_lines_ignores_partial_rows() {
        let mut board = Board::new();
        for x in 0..WIDTH - 1 {
            board.cells[HEIGHT - 1][x] = Some(1);
        }
        let cleared = board.clear_full_lines();
        assert_eq!(cleared, 0);
        assert!(!board.is_cell_free(0, (HEIGHT - 1) as i32));
    }

        #[test]
    fn is_area_free_true_only_when_all_cells_free() {
        let mut board = Board::new();
        board.cells[0][0] = Some(2);
        assert!(!board.is_area_free(&[(0, 0), (1, 0)]));
        assert!(board.is_area_free(&[(1, 0), (2, 0)]));
        assert!(!board.is_area_free(&[(1, 0), (-1, 0)]));
    }
}
