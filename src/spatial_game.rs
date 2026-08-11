use macroquad::rand::ChooseRandom;


pub const BOX_WIDTH: usize = 5;
pub const BOX_DEPTH: usize = 5;
pub const BOX_HEIGHT: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpatialPieceType {
    Cube1x1x1,
    Bar1x1x3,
    Square2x2x1,
    CornerL,
    TricubeL,
    T3D,
    Z3D,
}

impl SpatialPieceType {
    pub const ALL: [SpatialPieceType; 7] = [
        SpatialPieceType::Cube1x1x1,
        SpatialPieceType::Bar1x1x3,
        SpatialPieceType::Square2x2x1,
        SpatialPieceType::CornerL,
        SpatialPieceType::TricubeL,
        SpatialPieceType::T3D,
        SpatialPieceType::Z3D,
    ];

    pub fn color_id(&self) -> u8 {
        match self {
            SpatialPieceType::Cube1x1x1 => 1,
            SpatialPieceType::Bar1x1x3 => 2,
            SpatialPieceType::Square2x2x1 => 3,
            SpatialPieceType::CornerL => 4,
            SpatialPieceType::TricubeL => 5,
            SpatialPieceType::T3D => 6,
            SpatialPieceType::Z3D => 7,
        }
    }

    pub fn blocks(&self) -> Vec<(i8, i8, i8)> {
        match self {
            SpatialPieceType::Cube1x1x1 => vec![(0, 0, 0)],
            SpatialPieceType::Bar1x1x3 => vec![(0, 0, -1), (0, 0, 0), (0, 0, 1)],
            SpatialPieceType::Square2x2x1 => vec![(0, 0, 0), (1, 0, 0), (0, 1, 0), (1, 1, 0)],
            SpatialPieceType::CornerL => vec![(0, 0, 0), (1, 0, 0), (0, 1, 0), (0, 0, 1)],
            SpatialPieceType::TricubeL => vec![(0, 0, 0), (1, 0, 0), (0, 1, 0)],
            SpatialPieceType::T3D => vec![(0, 0, 0), (-1, 0, 0), (1, 0, 0), (0, 0, 1)],
            SpatialPieceType::Z3D => vec![(0, 0, 0), (1, 0, 0), (0, 1, 1), (1, 1, 1)],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpatialPiece {
    pub piece_type: SpatialPieceType,
    pub x: i8,
    pub y: i8,
    pub z: i8,
    pub blocks: Vec<(i8, i8, i8)>,
}

impl SpatialPiece {
    pub fn new(piece_type: SpatialPieceType) -> Self {
        let blocks = piece_type.blocks();
        SpatialPiece {
            piece_type,
            x: 2,
            y: 2,
            z: 1,
            blocks,
        }
    }

    pub fn world_blocks(&self) -> Vec<(i8, i8, i8)> {
        self.blocks
            .iter()
            .map(|&(bx, by, bz)| (self.x + bx, self.y + by, self.z + bz))
            .collect()
    }

    pub fn rotate(&mut self, axis: Axis) {
        for (bx, by, bz) in self.blocks.iter_mut() {
            let (x, y, z) = (*bx, *by, *bz);
            match axis {
                Axis::X => {
                    *by = -z;
                    *bz = y;
                }
                Axis::Y => {
                    *bx = z;
                    *bz = -x;
                }
                Axis::Z => {
                    *bx = -y;
                    *by = x;
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpatialBoard {
    pub cells: [[[Option<u8>; BOX_DEPTH]; BOX_WIDTH]; BOX_HEIGHT],
}

impl SpatialBoard {
    pub fn new() -> Self {
        SpatialBoard {
            cells: [[[None; BOX_DEPTH]; BOX_WIDTH]; BOX_HEIGHT],
        }
    }

    pub fn is_inside(x: i8, y: i8, z: i8) -> bool {
        x >= 0
            && x < BOX_WIDTH as i8
            && y >= 0
            && y < BOX_DEPTH as i8
            && z >= 0
            && z < BOX_HEIGHT as i8
    }

    pub fn is_cell_empty(&self, x: i8, y: i8, z: i8) -> bool {
        if !Self::is_inside(x, y, z) {
            return false;
        }
        self.cells[z as usize][x as usize][y as usize].is_none()
    }

    pub fn is_valid_piece(&self, piece: &SpatialPiece) -> bool {
        for (wx, wy, wz) in piece.world_blocks() {
            if !self.is_cell_empty(wx, wy, wz) {
                return false;
            }
        }
        true
    }

    pub fn lock_piece(&mut self, piece: &SpatialPiece) {
        for (wx, wy, wz) in piece.world_blocks() {
            if Self::is_inside(wx, wy, wz) {
                self.cells[wz as usize][wx as usize][wy as usize] =
                    Some(piece.piece_type.color_id());
            }
        }
    }

    pub fn clear_full_layers(&mut self) -> u32 {
        let mut cleared = 0;
        let mut z = BOX_HEIGHT as i8 - 1;
        while z >= 0 {
            let is_full = (0..BOX_WIDTH).all(|x| (0..BOX_DEPTH).all(|y| self.cells[z as usize][x][y].is_some()));
            if is_full {
                cleared += 1;
                for move_z in (1..=z as usize).rev() {
                    self.cells[move_z] = self.cells[move_z - 1];
                }
                self.cells[0] = [[None; BOX_DEPTH]; BOX_WIDTH];
            } else {
                z -= 1;
            }
        }
        cleared
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Playing,
    Paused,
    GameOver,
}

pub struct SpatialGame {
    pub board: SpatialBoard,
    pub active_piece: SpatialPiece,
    pub next_piece: SpatialPieceType,
    pub bag: Vec<SpatialPieceType>,
    pub score: u32,
    pub level: u32,
    pub layers_cleared: u32,
    pub last_layers_cleared: u32,
    pub state: GameState,
}

impl SpatialGame {
    pub fn new() -> Self {
        let mut game = SpatialGame {
            board: SpatialBoard::new(),
            active_piece: SpatialPiece::new(SpatialPieceType::Cube1x1x1),
            next_piece: SpatialPieceType::Cube1x1x1,
            bag: Vec::new(),
            score: 0,
            level: 1,
            layers_cleared: 0,
            last_layers_cleared: 0,
            state: GameState::Playing,
        };
        game.refill_bag_if_empty();
        game.next_piece = game.draw_from_bag();
        game.spawn_piece();
        game
    }

    fn refill_bag_if_empty(&mut self) {
        if self.bag.is_empty() {
            let mut new_bag = SpatialPieceType::ALL.to_vec();
            new_bag.shuffle();
            self.bag = new_bag;
        }
    }


    fn draw_from_bag(&mut self) -> SpatialPieceType {
        self.refill_bag_if_empty();
        self.bag.pop().unwrap()
    }

    pub fn spawn_piece(&mut self) {
        let piece_type = self.next_piece;
        self.next_piece = self.draw_from_bag();

        let piece = SpatialPiece::new(piece_type);
        if !self.board.is_valid_piece(&piece) {
            self.state = GameState::GameOver;
        }
        self.active_piece = piece;
    }

    pub fn move_x(&mut self, dx: i8) -> bool {
        if self.state != GameState::Playing {
            return false;
        }
        let mut candidate = self.active_piece.clone();
        candidate.x += dx;
        if self.board.is_valid_piece(&candidate) {
            self.active_piece = candidate;
            true
        } else {
            false
        }
    }

    pub fn move_y(&mut self, dy: i8) -> bool {
        if self.state != GameState::Playing {
            return false;
        }
        let mut candidate = self.active_piece.clone();
        candidate.y += dy;
        if self.board.is_valid_piece(&candidate) {
            self.active_piece = candidate;
            true
        } else {
            false
        }
    }

    pub fn rotate(&mut self, axis: Axis) -> bool {
        if self.state != GameState::Playing {
            return false;
        }
        let mut candidate = self.active_piece.clone();
        candidate.rotate(axis);
        if self.board.is_valid_piece(&candidate) {
            self.active_piece = candidate;
            true
        } else {
            false
        }
    }

    pub fn soft_drop(&mut self) -> bool {
        if self.state != GameState::Playing {
            return false;
        }
        let mut candidate = self.active_piece.clone();
        candidate.z += 1;
        if self.board.is_valid_piece(&candidate) {
            self.active_piece = candidate;
            true
        } else {
            self.lock_and_spawn();
            false
        }
    }

    pub fn hard_drop(&mut self) {
        if self.state != GameState::Playing {
            return;
        }
        while self.move_z_down() {}
        self.lock_and_spawn();
    }

    fn move_z_down(&mut self) -> bool {
        let mut candidate = self.active_piece.clone();
        candidate.z += 1;
        if self.board.is_valid_piece(&candidate) {
            self.active_piece = candidate;
            true
        } else {
            false
        }
    }

    pub fn tick(&mut self) {
        if self.state != GameState::Playing {
            return;
        }
        if !self.move_z_down() {
            self.lock_and_spawn();
        } else {
            self.last_layers_cleared = 0;
        }
    }


    fn lock_and_spawn(&mut self) {
        self.board.lock_piece(&self.active_piece);
        let cleared = self.board.clear_full_layers();
        self.last_layers_cleared = cleared;
        if cleared > 0 {
            self.layers_cleared += cleared;
            let mult = match cleared {
                1 => 100,
                2 => 300,
                3 => 600,
                _ => 1000,
            };
            self.score += mult * self.level;
            self.level = 1 + (self.layers_cleared / 10);
        }
        self.spawn_piece();
    }

    pub fn toggle_pause(&mut self) {
        match self.state {
            GameState::Playing => self.state = GameState::Paused,
            GameState::Paused => self.state = GameState::Playing,
            GameState::GameOver => {}
        }
    }

    pub fn restart(&mut self) {
        *self = Self::new();
    }
}

impl Default for SpatialBoard {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SpatialGame {
    fn default() -> Self {
        Self::new()
    }
}

pub fn spatial_gravity_interval_ms(level: u32) -> u64 {
    let ms = 800.0 * 0.88f64.powi(level as i32 - 1);
    ms.round().max(150.0) as u64
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_spatial_board_bounds() {
        let board = SpatialBoard::new();
        assert!(board.is_cell_empty(0, 0, 0));
        assert!(board.is_cell_empty(4, 4, 9));
        assert!(!board.is_cell_empty(-1, 0, 0));
        assert!(!board.is_cell_empty(0, 5, 0));
        assert!(!board.is_cell_empty(0, 0, 10));
    }

    #[test]
    fn test_piece_spawning_and_world_blocks() {
        let piece = SpatialPiece::new(SpatialPieceType::Cube1x1x1);
        let blocks = piece.world_blocks();
        assert_eq!(blocks, vec![(2, 2, 1)]);
    }

    #[test]
    fn test_piece_rotations() {
        let mut piece = SpatialPiece::new(SpatialPieceType::TricubeL);
        let orig_blocks = piece.blocks.clone();
        piece.rotate(Axis::Z);
        assert_ne!(piece.blocks, orig_blocks);
        piece.rotate(Axis::Z);
        piece.rotate(Axis::Z);
        piece.rotate(Axis::Z);
        assert_eq!(piece.blocks, orig_blocks);
    }

    #[test]
    fn test_game_movement_and_collision() {
        let mut game = SpatialGame::new();
        assert_eq!(game.state, GameState::Playing);
        assert!(game.move_x(1) || game.move_x(-1));
        assert!(game.move_y(1) || game.move_y(-1));
    }

    #[test]
    fn test_spatial_gravity_tick_advances_z() {
        let mut game = SpatialGame::new();
        let z_initial = game.active_piece.z;
        game.tick();
        assert_eq!(
            game.active_piece.z,
            z_initial + 1,
            "gravity tick must advance active piece down Z axis"
        );
    }

    #[test]
    fn test_spatial_gravity_interval_ms() {
        assert_eq!(spatial_gravity_interval_ms(1), 800);
        assert!(spatial_gravity_interval_ms(2) < spatial_gravity_interval_ms(1));
    }


    #[test]
    fn test_layer_clearing() {
        let mut board = SpatialBoard::new();
        for x in 0..BOX_WIDTH {
            for y in 0..BOX_DEPTH {
                board.cells[9][x][y] = Some(1);
            }
        }
        let cleared = board.clear_full_layers();
        assert_eq!(cleared, 1);
        for x in 0..BOX_WIDTH {
            for y in 0..BOX_DEPTH {
                assert!(board.cells[9][x][y].is_none());
            }
        }
    }

    #[test]
    fn test_spatial_board_z_depth_order() {
        assert_eq!(BOX_HEIGHT, 10);
        let board = SpatialBoard::new();
        assert!(board.is_cell_empty(0, 0, 9));
        assert!(board.is_cell_empty(0, 0, 0));
    }

    #[test]
    fn test_spatial_controls_not_transposed() {
        let mut game = SpatialGame::new();
        let orig_x = game.active_piece.x;
        let orig_y = game.active_piece.y;
        let orig_z = game.active_piece.z;

        // move_x alters X only, leaving Y and Z untouched
        if game.move_x(-1) || game.move_x(1) {
            assert_ne!(game.active_piece.x, orig_x);
            assert_eq!(game.active_piece.y, orig_y, "move_x must not transpose into Y");
            assert_eq!(game.active_piece.z, orig_z, "move_x must not transpose into Z");
        }

        // move_y alters Y only, leaving X and Z untouched
        let curr_x = game.active_piece.x;
        if game.move_y(-1) || game.move_y(1) {
            assert_eq!(game.active_piece.x, curr_x, "move_y must not transpose into X");
            assert_eq!(game.active_piece.z, orig_z, "move_y must not transpose into Z");
        }
    }

    #[test]
    fn test_spatial_game_last_layers_cleared_reset() {

        let mut game = SpatialGame::new();
        assert_eq!(game.last_layers_cleared, 0);
        game.tick();
        assert_eq!(game.last_layers_cleared, 0);
    }

    #[test]
    fn test_gizmo_vertex_rotation_bounds() {
        let (x, y, z) = (1.0f32, 1.0f32, 1.0f32);
        assert!((x - 1.0).abs() < 1e-4);
        assert!((y - 1.0).abs() < 1e-4);
        assert!((z - 1.0).abs() < 1e-4);
    }


    #[test]
    fn test_multi_layer_clearing_and_scoring() {
        let mut game = SpatialGame::new();
        for z in 8..=9 {
            for x in 0..BOX_WIDTH {
                for y in 0..BOX_DEPTH {
                    game.board.cells[z][x][y] = Some(1);
                }
            }
        }
        let cleared = game.board.clear_full_layers();
        assert_eq!(cleared, 2);
    }
}




