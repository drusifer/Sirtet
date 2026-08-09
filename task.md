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

---

# Sprint 2 — Dual Renderer: Terminal + Accelerated 3D ("Neon Grid")

**Status:** Sprint 2 launched (2026-08-08). All 6 phases implemented and approved; Smith's
e2e test passed with 2 disclosed gaps needing real-hardware verification (see Known
Limitations below).
**Date:** 2026-08-08

Gates cleared: Smith Gate 1 (stories US-9..US-14, approved w/ 2 additions) + Gate 2
(architecture, approved). See docs/PRD.md, docs/USER_STORIES.md, docs/ARCHITECTURE.md
Sprint 2 addenda.

Cycle per phase: Neo implements (TDD where applicable) -> Trin UAT -> Morpheus review -> next phase.

## Phase 1 — Scaffold: dependency + module extraction + CLI parsing
- [x] 1.1 Add `macroquad` to Cargo.toml (per ARCHITECTURE.md decision #1)
- [x] 1.2 Extract Sprint 1's `main.rs` body into `src/terminal.rs` as `pub fn run(game: Game) -> ExitCode`; `main.rs` shrinks to a stub that calls it (no behavior change — pure move)
- [x] 1.3 Hand-rolled `--renderer=terminal|3d` parsing in `main.rs`; invalid value prints valid options and exits before rendering (US-9 AC); unit test the parsing function
**Stories:** US-9 (flag form), US-10 (regression guard via pure-move extraction)

## Phase 2 — Startup picker
- [x] 2.1 Crossterm-based picker (Up/Down move selection, Enter confirms, Esc/Q exits cleanly) shown when no `--renderer` flag is given
- [x] 2.2 Manual PTY test: picker navigation, selection launches correct mode, Esc/Q quits without starting a game
**Stories:** US-9 (picker + Gate 1 keyboard-nav addition)

## Phase 3 — Engine accessor (additive only)
- [x] 3.1 `Game::last_lines_cleared() -> u32` accessor (ARCHITECTURE.md decision #8), non-breaking
- [x] 3.2 Unit tests: accessor reflects 0/1/2/3/4-line clears correctly, resets appropriately on next lock with no clear
**Stories:** US-11 (feeds 3D line-clear effect)

## Phase 4 — 3D renderer scaffold: static scene
- [x] 4.1 `src/gfx3d.rs`: macroquad window/camera setup, board + active piece rendered as cubes, dark background, neon per-piece palette, cheap glow via translucent backing quad (decision #6)
- [x] 4.2 Next-piece preview + score/level/lines HUD in 3D mode (parity with terminal's info, US-12)
- [x] 4.3 Manual smoke test: `cargo run -- --renderer=3d` shows a playable-looking static board (verified via X11 screenshot + xdotool window-title check, this sandbox has a real display)
**Stories:** US-11, US-12 (info parity)

## Phase 5 — 3D motion, input, effects
- [x] 5.1 Smooth interpolated piece motion between gravity ticks (decision #7) + input wired to the same Game methods as terminal mode (US-12)
- [x] 5.2 Line-clear flash effect (~300ms) using `last_lines_cleared()` (US-11) — whole-scene flash, not per-row, since the engine only exposes a count (see Known Limitations below)
- [x] 5.3 Control legend/HUD in 3D mode, styled to theme (US-12 parity w/ US-8); window-close (OS "X") relies on macroquad's normal frame-loop exit (US-14)
**Stories:** US-11, US-12, US-14

## Phase 6 — Fallback + full integration
- [x] 6.1 `catch_unwind` around the 3D renderer call (decision #9, scope note below): on
      failure, print the one-line fallback message and call `terminal::run(game)` (US-13).
      Verified end-to-end by temporarily forcing a panic in `gfx3d::run`, confirming the
      message prints, terminal mode starts and is playable, and the panic hook suppression
      means no raw backtrace is shown — then reverted the forced panic.
- [x] 6.2 Full regression pass: `cargo test` (34 engine + 4 cli), PTY smoke test of terminal
      mode confirms US-1..US-8 unchanged (US-10)
- [x] 6.3 `cargo build --release` clean + `cargo clippy --all-targets` clean; manual smoke
      test of all 4 entry paths: `--renderer=terminal`, `--renderer=3d`, no-flag picker,
      forced-fallback (US-13) — all verified working
**Stories:** cross-cutting — final integration before Stage 3 close

**Scope note on 6.1 (decision #9 deviation, flagged for Morpheus/Smith):** the architecture
doc's original plan was to scope `catch_unwind` to *only* the window/GPU init step, not the
full 3D session. In practice `macroquad::Window::from_config` bundles init and the entire
game loop into one call with no way to separate them, so the fallback wraps the whole 3D
session instead. This still satisfies US-13's AC (no crash, clean fallback message, terminal
mode starts) and is arguably a stronger guarantee (any 3D-mode panic recovers gracefully,
not just init failures) — the tradeoff is that a genuine mid-session 3D bug would also
silently fall back to terminal rather than being visibly distinguishable from "no GPU".

---

## Sprint 2 out of scope (backlog, not this sprint)
Persistent renderer preference, additional 3D themes/skins, graphics settings UI,
mid-session renderer switching. (Sprint 1 backlog — hold/ghost/SRS/sound/persistence/
keybind-config — still separately open, not part of Sprint 2.)

## Sprint 2 known limitations (updated post-launch, real-hardware verification)
- **RESOLVED — real Wayland session bug found and fixed post-launch:** on a real Wayland
  laptop (not this sandbox), `--renderer=3d` opened a window that was created but never
  mapped (`xwininfo` showed `Map State: IsUnMapped` deterministically, 3/3 repro attempts)
  — the window existed and rendered correctly (confirmed via direct pixel capture) but was
  never actually shown on screen, because `miniquad` defaults to X11/XWayland on Linux and
  XWayland's window-mapping handshake silently failed on this compositor. Fixed by forcing
  `miniquad::conf::Platform { linux_backend: LinuxBackend::WaylandOnly, .. }` in
  `gfx3d.rs`'s `window_conf()`, sidestepping XWayland entirely. **Confirmed working on real
  hardware afterward: the game is visible, and all keyboard controls (movement, rotation,
  hard drop, pause, restart, quit) work correctly.** This also resolves the sandbox's
  synthetic-input-testing gap noted below — real input works fine now that rendering goes
  through native Wayland instead of XWayland.
- Live keyboard input in 3D mode could not be verified *in this development sandbox*
  (confirmed via `xev` that synthetic/XTest key events aren't delivered to any X window
  here) — now moot for real users since native Wayland is the default path and has been
  confirmed working directly by a user on real hardware.
- Line-clear visual effect (US-11) is a whole-scene flash, not per-row, since
  `Game::last_lines_cleared()` only exposes a count, not row indices (ARCHITECTURE.md
  decision #8's chosen scope). Still accurate, not affected by the Wayland fix.
- The `catch_unwind` fallback (US-13) covers the whole 3D session, not just init — see the
  Phase 6.1 scope note above. Still accurate.

---

# Sprint 3 — Spatial 3D Box Tetris (TUI & Fancy GPU Modes)

**Status:** Phase 1 ready — ARCHITECTURE.md + USER_STORIES.md approved (both gates cleared).
**Date:** 2026-08-08

Gates cleared: Smith Gate 1 (stories US-15..US-20 approved) + Gate 2 (architecture approved).
Cycle per phase: Neo implements (TDD) -> Trin UAT -> Morpheus review -> next phase.

## Phase 1 — Core 3D Spatial Engine & Polycubes (`src/spatial_game.rs`)
- [x] 1.1 `src/spatial_game.rs`: 5x5x10 3D spatial grid, `SpatialBoard`, `SpatialPiece` (3D polycube shapes), 3D pitch/yaw/roll rotations, 3D boundary collision
- [x] 1.2 `SpatialGame`: Z-gravity tick, 3D translation (X/Y), 3D rotations (X/Y/Z), soft/hard drop down Z, lock-on-landing
- [x] 1.3 Unit tests for 3D collision, 3D rotations, piece spawning, and movement (covering `spatial_game.rs` pure engine logic)
**Stories:** US-16, US-17


## Phase 2 — 3D Layer Clears & Scoring
- [x] 2.1 3D Layer clear detection (filling 5x5=25 cells at Z level) + shift down Z + exponential scoring scale (100/300/600/1000 x level)
- [x] 2.2 Unit tests: single, double, multi-layer clears, score increment, level progression
**Stories:** US-18


## Phase 3 — CLI Parser & 4-Way Startup Picker
- [x] 3.1 Expand `RendererChoice` enum and CLI parsing in `src/cli.rs` (`terminal`, `3d`, `terminal_3d`, `3d_box`)
- [x] 3.2 Update `src/picker.rs` to display 4 menu options with Up/Down + Enter nav & Esc/Q quit
- [x] 3.3 Unit tests for CLI parsing + manual PTY smoke test for 4-way picker
**Stories:** US-15


## Phase 4 — Terminal (TUI) 3D Box Renderer (`src/terminal_3d.rs`)
- [x] 4.1 `src/terminal_3d.rs`: Crossterm isometric ANSI wireframe well renderer with depth shading & block representation
- [x] 4.2 HUD overlay (Score, Level, Layers Cleared, Next Piece, 3D Control Legend)
- [x] 4.3 Manual PTY smoke test of TUI Spatial 3D Box mode (`cargo run -- --renderer=terminal_3d`)
**Stories:** US-19


## Phase 5 — Fancy GPU 3D Box Renderer (`src/gfx3d_box.rs`)
- [x] 5.1 `src/gfx3d_box.rs`: Macroquad 3D viewport setup (`Camera3D`), wireframe well lines (`draw_line_3d`), 3D glowing polycube blocks (`draw_cube`)
- [x] 5.2 3D motion interpolation, layer clear flash effect, and HUD overlay
- [x] 5.3 Init fallback wrapping (`catch_unwind` falling back to `terminal_3d`)
- [x] 5.4 Manual smoke test of Fancy GPU 3D Box mode (`cargo run -- --renderer=3d_box`)
**Stories:** US-20


## Phase 6 — Integration & Final Verification
- [x] 6.1 Full test suite pass (`cargo test`) covering engine + CLI parser + spatial engine tests (46/46 tests)
- [x] 6.2 Clean build (`cargo build --release`) and clippy check (`cargo clippy --all-targets` via `bobp make lint`)
**Stories:** cross-cutting final verification


