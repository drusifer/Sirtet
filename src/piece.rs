#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceType {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

pub const ALL_PIECE_TYPES: [PieceType; 7] = [
    PieceType::I,
    PieceType::O,
    PieceType::T,
    PieceType::S,
    PieceType::Z,
    PieceType::J,
    PieceType::L,
];

impl PieceType {
    /// 1-indexed id used for board cell storage / renderer color lookup.
    pub fn id(self) -> u8 {
        match self {
            PieceType::I => 1,
            PieceType::O => 2,
            PieceType::T => 3,
            PieceType::S => 4,
            PieceType::Z => 5,
            PieceType::J => 6,
            PieceType::L => 7,
        }
    }

    /// Local cell offsets within a 4x4 bounding box for a given rotation state (0-3).
    /// Basic (non-SRS) rotation: I/O/S/Z classically have only 2 visually distinct
    /// states, so states 2 and 0, and 3 and 1, are identical by design (matches
    /// ARCHITECTURE.md decision #4: fixed tables, no wall-kick).
    pub fn cells(self, rotation: u8) -> [(i32, i32); 4] {
        let r = (rotation % 4) as usize;
        match self {
            PieceType::I => [
                [(0, 1), (1, 1), (2, 1), (3, 1)],
                [(2, 0), (2, 1), (2, 2), (2, 3)],
                [(0, 1), (1, 1), (2, 1), (3, 1)],
                [(2, 0), (2, 1), (2, 2), (2, 3)],
            ][r],
            PieceType::O => [(1, 0), (2, 0), (1, 1), (2, 1)],
            PieceType::T => [
                [(1, 0), (0, 1), (1, 1), (2, 1)],
                [(1, 0), (1, 1), (2, 1), (1, 2)],
                [(0, 1), (1, 1), (2, 1), (1, 2)],
                [(1, 0), (0, 1), (1, 1), (1, 2)],
            ][r],
            PieceType::S => [
                [(1, 0), (2, 0), (0, 1), (1, 1)],
                [(1, 0), (1, 1), (2, 1), (2, 2)],
                [(1, 0), (2, 0), (0, 1), (1, 1)],
                [(1, 0), (1, 1), (2, 1), (2, 2)],
            ][r],
            PieceType::Z => [
                [(0, 0), (1, 0), (1, 1), (2, 1)],
                [(2, 0), (1, 1), (2, 1), (1, 2)],
                [(0, 0), (1, 0), (1, 1), (2, 1)],
                [(2, 0), (1, 1), (2, 1), (1, 2)],
            ][r],
            PieceType::J => [
                [(0, 0), (0, 1), (1, 1), (2, 1)],
                [(1, 0), (2, 0), (1, 1), (1, 2)],
                [(0, 1), (1, 1), (2, 1), (2, 2)],
                [(1, 0), (1, 1), (0, 2), (1, 2)],
            ][r],
            PieceType::L => [
                [(2, 0), (0, 1), (1, 1), (2, 1)],
                [(1, 0), (1, 1), (1, 2), (2, 2)],
                [(0, 1), (1, 1), (2, 1), (0, 2)],
                [(0, 0), (1, 0), (1, 1), (1, 2)],
            ][r],
        }
    }
}

/// A tetromino instance positioned on the board (origin = top-left of its 4x4
/// bounding box, in board coordinates).
#[derive(Debug, Clone, Copy)]
pub struct Piece {
    pub piece_type: PieceType,
    pub rotation: u8,
    pub x: i32,
    pub y: i32,
}

impl Piece {
    /// Spawns horizontally centered (4-wide bounding box on a 10-wide board) at the top.
    pub fn spawn(piece_type: PieceType) -> Self {
        Piece {
            piece_type,
            rotation: 0,
            x: 3,
            y: 0,
        }
    }

    /// Absolute board cells this piece currently occupies.
    pub fn cells(&self) -> [(i32, i32); 4] {
        let local = self.piece_type.cells(self.rotation);
        let mut out = [(0, 0); 4];
        for (i, (lx, ly)) in local.iter().enumerate() {
            out[i] = (self.x + lx, self.y + ly);
        }
        out
    }

    /// Returns a copy rotated 90 degrees clockwise (caller checks collision before committing).
    pub fn rotated_cw(&self) -> Self {
        Piece {
            rotation: (self.rotation + 1) % 4,
            ..*self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_piece_every_rotation_has_four_cells_and_is_stable() {
        for &pt in ALL_PIECE_TYPES.iter() {
            for r in 0..4u8 {
                let cells = pt.cells(r);
                assert_eq!(cells.len(), 4);
                // no duplicate cells within a single piece
                for i in 0..4 {
                    for j in (i + 1)..4 {
                        assert_ne!(cells[i], cells[j], "{:?} r{} has overlapping cells", pt, r);
                    }
                }
            }
        }
    }

    #[test]
    fn o_piece_identical_in_all_rotations() {
        let base = PieceType::O.cells(0);
        for r in 1..4u8 {
            assert_eq!(PieceType::O.cells(r), base);
        }
    }

    #[test]
    fn i_piece_alternates_between_two_states() {
        assert_eq!(PieceType::I.cells(0), PieceType::I.cells(2));
        assert_eq!(PieceType::I.cells(1), PieceType::I.cells(3));
        assert_ne!(PieceType::I.cells(0), PieceType::I.cells(1));
    }

    #[test]
    fn spawn_is_centered_at_top() {
        let piece = Piece::spawn(PieceType::T);
        assert_eq!(piece.x, 3);
        assert_eq!(piece.y, 0);
        assert_eq!(piece.rotation, 0);
    }

    #[test]
    fn cells_are_offset_by_position() {
        let piece = Piece::spawn(PieceType::O);
        let cells = piece.cells();
        let expected = [(1 + 3, 0), (2 + 3, 0), (1 + 3, 1), (2 + 3, 1)];
        assert_eq!(cells, expected);
    }

    #[test]
    fn rotated_cw_advances_rotation_and_wraps() {
        let piece = Piece::spawn(PieceType::T);
        let r1 = piece.rotated_cw();
        assert_eq!(r1.rotation, 1);
        let r4 = r1.rotated_cw().rotated_cw().rotated_cw();
        assert_eq!(r4.rotation, 0);
    }

    #[test]
    fn every_piece_has_a_unique_id() {
        let mut ids: Vec<u8> = ALL_PIECE_TYPES.iter().map(|p| p.id()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 7);
    }
}
