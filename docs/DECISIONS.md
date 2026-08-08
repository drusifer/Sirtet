# Decisions — Tetris (Rust)

Consolidated from CHAT.md and persona state files by Oracle at sprint close (2026-08-07).

## Product Scope (Cypher, Gate 1 approved by Smith)
- Single-player terminal Tetris, classic rules: 7 tetrominoes, 10x20 board, standard scoring
  (100/300/500/800 x level), level-up every 10 lines, basic (non-SRS) rotation, pause/restart.
- Explicitly out of scope: hold piece, ghost piece, SRS wall-kick, sound, persistent high
  scores, configurable keybindings. Logged as fast-follow backlog in docs/USER_STORIES.md.
- Smith added US-8 (on-screen control legend) at Gate 1 — the game has no menu, so the play
  screen is the only place a first-time player can discover the keys. Also pinned exact key
  bindings (Left/Right/Down/Up/Space/P/R/Q) which were originally left vague in the draft
  stories and would have made Trin's acceptance tests ambiguous.

## Architecture (Morpheus, Gate 2 approved by Smith)
- Lib crate (`tetris`) + thin `main.rs` binary, so the engine (board/piece/game) has zero
  terminal dependency and is fully unit-testable via `cargo test`.
- Terminal backend: `crossterm` directly, not `ratatui` — a fixed grid + a few side panels
  doesn't need a widget framework; direct cell rendering is simpler and easier to test.
- Piece randomizer: 7-bag (`rand` shuffle, refilled when empty) — fairness improvement over
  pure random, not a scope change (Smith confirmed no UX downside).
- Rotation: fixed 4-orientation cell tables per piece, in-place only, reject-on-collision, no
  wall-kick — matches US-2's AC exactly and PRD's non-goals.
- Gravity curve: `interval_ms = max(100, 1000 * 0.85^(level-1))` — monotonic, no cliff, floor
  at 100ms. Chosen so level 2 is ~15% faster than level 1, satisfying US-5's "visibly faster
  by level 2" AC without needing a hand-tuned lookup table.

## Implementation decisions made during the Phase Bloop
- `Board::is_area_free` takes raw `(i32,i32)` coordinates rather than a `Piece` type, so
  board.rs has zero dependency on piece.rs — game.rs is the only place that bridges them.
  This was a deliberate refinement of the architecture doc during Phase 1 implementation.
- `Board` grew `lock_cells`, `cell`, and `clear_full_lines` incrementally as later phases
  needed them (Phase 3 and Phase 4) rather than being fully speced upfront in Phase 1 — normal
  incremental growth, not scope creep, since Board's public surface stayed narrow and testable.
- `#[cfg(test)]`-gated `Board::test_fill_row` helper lets game.rs tests set up deterministic
  line-clear/game-over scenarios without depending on random piece placement, and compiles out
  of release builds entirely.
- `Piece::rotated_cw()` returns a copy rather than mutating in place; the caller (`Game`) is
  responsible for collision-checking before committing — keeps rotation rejection (US-2 AC)
  a one-line check at the call site instead of needing rollback logic.

## Process decisions
- Project had no Makefile before this sprint (bob-protocol scaffolding doesn't create one);
  Neo added one with build/test/run/release targets so `bobp make` tooling works going forward.
