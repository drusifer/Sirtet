# Task Board — Tetris (Rust) Sprint

**Maintained by:** Mouse (SM)
**Status:** Phase 1 ready — ARCHITECTURE.md + USER_STORIES.md approved (both gates cleared)
**Date:** 2026-08-07

Cycle per phase: Neo implements (TDD) -> Trin UAT -> Morpheus review -> next phase.

---

## Phase 1 — Project scaffold + Board core
- [x] 1.1 `cargo init` for lib+bin crate `tetris`; add `crossterm`, `rand` to Cargo.toml
- [x] 1.2 `src/board.rs`: 10x20 grid `Board`, `is_cell_free`, `is_area_free` (decoupled
      from Piece type - board.rs has zero dependency on piece.rs, by design)
- [x] 1.3 Unit tests: empty board free everywhere; out-of-bounds & occupied-cell collisions
      (4/4 passing via `cargo test` / `bobp make test`)
**Stories:** US-1 (board exists)

## Phase 2 — Piece definitions + rotation
- [x] 2.1 `src/piece.rs`: I/O/T/S/Z/J/L shapes, 4 fixed rotation-state cell tables, spawn position
- [x] 2.2 Unit tests: each piece's 4 rotation states correct; spawn position centered/top
      (7/7 new tests passing, 11/11 total via `cargo test`)
**Stories:** US-1 (piece spawns), US-2 (rotation shapes)

## Phase 3 — Game engine: movement, gravity, lock, next-queue
- [x] 3.1 `src/game.rs` `Game`: move_left/right, soft_drop, hard_drop, rotate (reject-on-collision)
- [x] 3.2 Gravity tick (interval by level, per ARCHITECTURE.md formula) + lock-on-landing +
      spawn-next-from-7-bag-queue. Added `Board::lock_cells` (grew board.rs API for this phase).
- [x] 3.3 Unit tests: movement blocked at walls/stack; rotation rejected on collision; lock
      triggers when piece can't fall; new piece spawns from queue after lock. 8 new tests,
      19/19 total, swept 30x for RNG-flakiness (2 test bugs found+fixed, no impl bugs).
**Stories:** US-1 (next preview data), US-2, US-3

## Phase 4 — Line clear, scoring, leveling
- [x] 4.1 Line-clear detection + row shift-down (`Board::clear_full_lines`)
- [x] 4.2 Scoring table (100/300/500/800 x level) + level-up every 10 lines. Gravity
      interval already recomputes automatically (derived from level, not cached).
- [x] 4.3 Unit tests: single/double/triple/tetris clears score correctly; level increments at
      10-line boundaries; interval decreases correctly. 9 new tests, 28/28 total, swept 20x.
**Stories:** US-4, US-5

## Phase 5 — Game over, restart, pause
- [x] 5.1 Game-over detection (spawn collision) + `GameOver` state; `Game::restart()`
      (both already existed from Phase 3's forward-looking design; verified with new tests)
- [x] 5.2 `Paused` state: gravity/input no-ops except unpause/quit while paused
      (already gated via `is_playing()`; verified with new tests)
- [x] 5.3 Unit tests: spawn-collision -> game over; restart resets all fields; paused state
      blocks gravity/movement. 5 new tests, 33/33 total, swept 20x.
**Stories:** US-3 (game over trigger), US-6, US-7

## Phase 6 — Terminal UI (main.rs)
- [x] 6.1 crossterm raw-mode setup/teardown; render board + next-piece panel + score/level/lines
- [x] 6.2 Input polling loop wired to Game methods + gravity timer via `Instant`
- [x] 6.3 On-screen control legend (US-8) + GAME OVER / PAUSED overlays. Manually
      smoke-tested via a real PTY (crossterm needs a TTY, not unit-testable): startup render,
      movement, pause overlay, restart, game-over overlay w/ score, clean quit (exit 0) all
      verified. Found+fixed one real bug (SCORE/LEVEL/LINES row overlapped CONTROLS legend).
**Stories:** US-1, US-6, US-7, US-8 (rendering side of all stories)

## Phase 7 — Integration & smoke test
- [x] 7.1 Wire main.rs end-to-end via `cargo run`; fix any engine/renderer seams found manually
      (done during Phase 6's PTY testing - the layout overlap was the one seam found)
- [x] 7.2 `cargo test` full pass (33/33) + `cargo build --release` clean + `cargo clippy
      --all-targets` clean (fixed 2 collapsible-if style warnings)
**Stories:** cross-cutting — final integration before Stage 3 close (Oracle groom -> Smith
end-to-end *user test)

---

## Out of scope (backlog, not this sprint)
Hold piece, ghost piece, SRS wall-kick, sound, persistent high scores, configurable keybinds.
