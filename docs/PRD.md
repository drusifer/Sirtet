# PRD — Tetris (Rust)

**Owner:** Cypher (PM)
**Status:** Draft for review
**Date:** 2026-08-07

## Vision
A faithful, single-player terminal Tetris implementation in Rust. Clean, dependable, classic
falling-block gameplay playable in any terminal, with standard scoring, levels, and controls.
No networking, no accounts — a small, complete, well-tested game.

## Goals (Sprint Scope — IN)
1. Standard 10x20 Tetris playfield with the 7 tetrominoes (I, O, T, S, Z, J, L).
2. Piece spawn, movement (left/right/soft drop/hard drop), rotation, and collision.
3. Line clear detection + clearing, with score awarded per Tetris scoring conventions
   (single/double/triple/tetris).
4. Level progression: gravity speed increases as lines clear.
5. Next-piece preview.
6. Game over detection (stack reaches top) and restart.
7. Terminal UI (TUI) rendering: board, current piece, next piece, score, level, lines cleared.
8. Keyboard controls, pause.

## Explicit Non-Goals (OUT of scope this sprint)
- Hold piece, ghost piece, wall-kick "advanced" SRS rotation (basic rotation only — flag as
  fast-follow, not this sprint).
- Multiplayer / networking.
- Sound.
- Persistent high scores / save files.
- Mouse or GUI (non-terminal) rendering.
- Configurable key bindings.

## Target Platform
- Rust, compiled binary, runs in any ANSI-capable terminal (Linux primary target).
- Crate choice (ratatui/crossterm vs. alternative) is Morpheus's call — architecture stage.

## Success Criteria
- `cargo run` starts a playable game from a terminal with no manual setup.
- All 7 tetrominoes spawn, move, rotate, and lock correctly.
- Line clears score correctly and the board updates immediately.
- Level speed increases are observable and match a documented curve.
- Game over is detected and the player can restart without relaunching the binary.
- `cargo test` covers board logic, collision, rotation, and line-clear/scoring rules
  (i.e., the game engine is unit-testable independent of the terminal renderer).

## Open Questions
- Exact scoring table and level speed curve — Cypher pins standard values (see USER_STORIES.md)
  unless Morpheus flags a technical reason to deviate.
