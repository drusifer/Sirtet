use crate::game::{Game, GameState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Single,
    TwoPlayerLocal,
    VsCpu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchWinner {
    None,
    Player1,
    Player2,
    Cpu,
}

#[derive(Clone)]
pub struct BattleState {

    pub mode: GameMode,
    pub player1: Game,
    pub player2: Option<Game>,
    pub winner: MatchWinner,
}

impl BattleState {
    pub fn new(mode: GameMode) -> Self {
        let player1 = Game::new();
        let player2 = match mode {
            GameMode::Single => None,
            GameMode::TwoPlayerLocal | GameMode::VsCpu => Some(Game::new()),
        };
        BattleState {
            mode,
            player1,
            player2,
            winner: MatchWinner::None,
        }
    }

    pub fn is_game_over(&self) -> bool {
        if self.mode == GameMode::Single {
            self.player1.state() == GameState::GameOver
        } else {
            self.winner != MatchWinner::None
        }
    }

    pub fn tick(&mut self) {
        if self.is_game_over() {
            return;
        }

        let g1 = self.player1.tick();
        let g2 = if let Some(ref mut p2) = self.player2 {
            p2.tick()
        } else {
            0
        };

        self.apply_garbage_attack(g1, g2);
        self.update_match_status();
    }

    pub fn p1_hard_drop(&mut self) {
        if self.is_game_over() {
            return;
        }
        let g1 = self.player1.hard_drop();
        self.apply_garbage_attack(g1, 0);
        self.update_match_status();
    }

    pub fn p2_hard_drop(&mut self) {
        if self.is_game_over() {
            return;
        }
        let g2 = if let Some(ref mut p2) = self.player2 {
            p2.hard_drop()
        } else {
            0
        };
        self.apply_garbage_attack(0, g2);
        self.update_match_status();
    }

    fn apply_garbage_attack(&mut self, g1: u32, g2: u32) {
        if g1 > 0 && let Some(ref mut p2) = self.player2 {
            p2.add_garbage(g1);
        }
        if g2 > 0 {
            self.player1.add_garbage(g2);
        }
    }


    pub fn update_match_status(&mut self) {
        if self.mode == GameMode::Single {
            return;
        }

        let p1_over = self.player1.state() == GameState::GameOver;
        let p2_over = self
            .player2
            .as_ref()
            .map(|p| p.state() == GameState::GameOver)
            .unwrap_or(false);

        if p1_over && !p2_over {
            self.winner = if self.mode == GameMode::VsCpu {
                MatchWinner::Cpu
            } else {
                MatchWinner::Player2
            };
        } else if p2_over && !p1_over {
            self.winner = MatchWinner::Player1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battle_state_init() {
        let single = BattleState::new(GameMode::Single);
        assert!(single.player2.is_none());
        assert_eq!(single.winner, MatchWinner::None);

        let vs_local = BattleState::new(GameMode::TwoPlayerLocal);
        assert!(vs_local.player2.is_some());
        assert_eq!(vs_local.winner, MatchWinner::None);

        let vs_cpu = BattleState::new(GameMode::VsCpu);
        assert!(vs_cpu.player2.is_some());
        assert_eq!(vs_cpu.winner, MatchWinner::None);
    }

    #[test]
    fn test_garbage_attack_routing() {
        let mut battle = BattleState::new(GameMode::TwoPlayerLocal);
        battle.apply_garbage_attack(2, 0);
        assert_eq!(battle.player2.as_ref().unwrap().pending_garbage(), 2);
    }
}
