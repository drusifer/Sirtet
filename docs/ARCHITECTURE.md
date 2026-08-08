# Architecture — Tetris (Rust)

**Owner:** Morpheus (Tech Lead)
**Status:** Draft for Smith Gate 2 review
**Date:** 2026-08-07

## Decision Summary

| # | Decision | Rationale |
|---|----------|-----------|
| 1 | Split into lib crate (`tetris`) + thin `main.rs` binary | PRD Success Criteria requires `cargo test` to cover engine logic independent of the terminal renderer. Engine has zero terminal deps. |
| 2 | Terminal backend: `crossterm` (raw mode, cell-positioned draws, non-blocking input) | Cross-platform, lightweight, no widget-framework overhead. This game is a fixed grid + a few side panels — direct cell rendering is simpler and easier to test than adopting a full TUI widget framework (ratatui) for something this small. |
| 3 | Randomizer: 7-bag (each of the 7 pieces appears once per shuffled bag of 7, refilled when empty) via `rand` | Prevents long droughts/floods of one piece type. This is an implementation detail of "next piece spawns" (US-1/US-3) — not new user-facing scope, so no PRD/story change needed. Flagging for Smith's awareness. |
| 4 | Rotation: fixed 4-orientation cell tables per piece, in-place only, reject-on-collision, no wall-kick | Matches US-2 AC exactly ("rejected... stays"); SRS wall-kick is explicitly out of scope (PRD Non-Goals). |
| 5 | Gravity curve: `interval_ms = max(100, 1000 * 0.85^(level-1))` | Level 1 = 1000ms, Level 2 = 850ms (15% faster — satisfies US-5 AC "visibly faster by level 2"), floor at 100ms so it never becomes unplayable. Monotonic, simple, no lookup table to maintain. |
| 6 | Main loop: single-threaded, `crossterm::event::poll` with a short timeout (~16ms) driving both input handling and a level-dependent gravity timer via `Instant` | Keeps input responsive regardless of gravity interval; no threading complexity needed for a single-player terminal game. |

## Module Layout

```
Cargo.toml
src/
  lib.rs      — pub mod board; pub mod piece; pub mod game;  (no terminal deps)
  board.rs    — Board (10x20 grid), collision checks, line-clear detection + row shift
  piece.rs    — Tetromino shapes (I,O,T,S,Z,J,L), 4 rotation states each, spawn positions
  game.rs     — Game (board + active piece + 7-bag queue + score + level + lines +
                state: Playing/Paused/GameOver), tick(), input handlers, scoring table
  main.rs     — binary: crossterm terminal setup, render loop, input polling, calls into
                `tetris::game::Game` only — no game rules live here
```

## Key Bindings (locked at Gate 1 by Smith)
Left/Right = move, Down = soft drop, Up = rotate CW, Space = hard drop, P = pause,
R = restart, Q/Esc = quit. Legend rendered on-screen per US-8.

## Scoring Table (from PRD/US-4)
1 line = 100 x level, 2 = 300 x level, 3 = 500 x level, 4 (Tetris) = 800 x level.

## Testability
`board.rs`, `piece.rs`, `game.rs` are pure logic (no I/O) — fully unit-testable via
`cargo test` without a terminal, satisfying the PRD's engine/renderer separation requirement.
`main.rs` is a thin adapter and is not unit-tested (verified manually/UAT per Trin + Smith).

## Dependencies
- `crossterm` — terminal control, input, rendering primitives.
- `rand` — 7-bag shuffling.

No other runtime dependencies. No async runtime needed (single-threaded poll loop).

## Open Items for Smith (Gate 2)
- Confirm 7-bag randomizer (decision #3) doesn't conflict with any user expectation — it's
  strictly a fairness improvement over pure-random, not a new visible feature.
- Confirm gravity curve (decision #5) reads as "fair" — no hard cliff, floor prevents the
  game from becoming physically unplayable at high levels.
