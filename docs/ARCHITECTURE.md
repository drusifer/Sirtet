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

---

# Sprint 2 Addendum: Dual Renderer (Terminal + Accelerated 3D)

**Owner:** Morpheus (Tech Lead)
**Status:** Approved (Smith Gate 2, 2026-08-08); implemented across Phases 1-6, see
Phase 6's scope note for one disclosed deviation (decision #9's catch_unwind scoping)
**Date:** 2026-08-08

## Decision Summary

| # | Decision | Rationale |
|---|----------|-----------|
| 1 | Graphics crate: `macroquad` (0.4.x) | Immediate-mode API with built-in window/input/3D primitives (`draw_cube`, 3D camera) and no separate windowing crate to wire up. `wgpu` is too low-level for one small stylized scene (shader/pipeline boilerplate); `bevy` is a full ECS engine, far more than a fixed-camera falling-block scene needs (PRD explicitly flags "minimal scope creep"). macroquad is the smallest crate that still gets us real GPU-accelerated rendering. |
| 2 | No `Renderer` trait | Cypher's open question suggested one; declining it. There are exactly two backends, chosen once at startup and never swapped at runtime (PRD non-goal). Two free functions — `terminal::run(game) -> ExitCode` and `gfx3d::run(game) -> ExitCode` — dispatched via a `match` in `main.rs` are simpler and equally easy to test at the boundary; a trait would add indirection for a closed set of 2 variants that never grows this sprint. |
| 3 | CLI parsing: hand-rolled, one accepted syntax | `--renderer=terminal` / `--renderer=3d` only (no separate `--renderer terminal` form, no external arg-parsing crate). Two flags, no subcommands — not worth a `clap` dependency. Unknown/malformed values print the two valid options and exit(1) before any rendering starts (US-9 AC). |
| 4 | Startup picker reuses `crossterm` (no new dep for it) | When no `--renderer` flag is given, the picker is a small crossterm raw-mode menu (same dependency Sprint 1 already added), not part of macroquad — no GPU context exists yet at picker time. Up/Down move selection, Enter confirms, Esc/Q exits cleanly (US-9 AC, Smith Gate 1 addition). |
| 5 | Module split: extract Sprint 1's `main.rs` body into `src/terminal.rs`; new `src/gfx3d.rs` for the 3D backend; `main.rs` shrinks to arg parsing + picker + dispatch | Keeps each backend independently readable and keeps `main.rs` a thin dispatcher, matching Sprint 1's "engine has zero I/O deps" principle extended to "each renderer is a self-contained adapter over the same `tetris::game::Game`." Neither backend gains access to the other's internals. |
| 6 | 3D scene: fixed perspective camera over the board, each filled cell drawn as a `draw_cube`, dark near-black background, neon palette (bright saturated per-piece colors distinct from Sprint 1's terminal palette), cheap "glow" via a larger translucent color quad drawn behind each cube (no bloom shader/post-processing pass) | Delivers the "futuristic, glowing" look (US-11) without a shader-authoring task — macroquad has no built-in bloom, and writing a custom post-process pass would be disproportionate scope for one visual theme. |
| 7 | Falling-piece motion: engine still ticks at the fixed gravity interval (unchanged from decision #5 in the base Architecture doc); the 3D renderer linearly interpolates the piece's drawn Y position between the last tick's row and the current tick's row using wall-clock time since the last tick | Satisfies "smoothly animated, not an instant jump" (US-11 AC) with zero engine changes — animation is purely a rendering-layer concern, engine timing/tests are untouched (US-10 regression guard). |
| 8 | Line-clear effect: `Game` gains a small additive read-only accessor exposing how many lines were cleared on the most recent lock (e.g. `Game::last_lines_cleared() -> u32`), a non-breaking addition to the existing engine API | The 3D renderer uses this to trigger a ~300ms flash + particle-burst effect on the cleared rows (US-11 AC) without polling board diffs itself. Terminal renderer is free to ignore the new accessor (US-10: no terminal behavior change). Exact field/method naming is Neo's implementation call within this shape. |
| 9 | 3D init failure handling (US-13): wrap only the initial window/GPU-context creation step in `std::panic::catch_unwind` (with a temporary custom panic hook installed first, so no raw panic/backtrace is printed to the user), not the whole play session | miniquad (macroquad's backend) reports context-creation failure via panic, not a `Result`, so `catch_unwind` around init is the only interception point available. Scoping it to init only (not the full game loop) is deliberate: a panic *during play* after successful init is a real bug to fix, not a supported fallback path — conflating the two would silently swallow genuine crashes as if they were missing-GPU cases. On init failure: print one clear line ("3D rendering unavailable on this system — starting terminal mode instead.") and call `terminal::run(game)` with the same `Game` the picker/CLI already constructed. |
| 10 | Window-close (US-14, Smith Gate 1 addition): rely on macroquad's normal frame-loop exit (`next_frame().await` loop ends when the OS window is closed) | No special-case handling needed — process exits cleanly at 0 the same way Q/Esc does today (US-6 pattern), since neither path leaves the loop mid-frame or holds a lock/raw-mode terminal state (3D mode never enters crossterm raw mode). |

## Module Layout (Sprint 2 changes)

```
src/
  lib.rs        — unchanged: pub mod board; pub mod piece; pub mod game;
  board.rs      — unchanged
  piece.rs      — unchanged
  game.rs       — + last_lines_cleared() accessor (additive only, decision #8)
  terminal.rs   — NEW: Sprint 1's main.rs body moved here verbatim (crossterm render/input
                  loop). `pub fn run(game: Game) -> ExitCode`
  gfx3d.rs      — NEW: macroquad-based 3D loop, `pub fn run(game: Game) -> ExitCode`, plus
                  a private init step wrapped in catch_unwind per decision #9
  main.rs       — SHRINKS to: parse `--renderer` flag -> if absent, run crossterm picker ->
                  match on choice -> call terminal::run or attempt gfx3d::run (falling back
                  to terminal::run on init failure)
```

## Dependencies (Sprint 2 additions)
- `macroquad` (0.4.x) — GPU-accelerated windowing, input, and 3D drawing primitives for the
  3D renderer. Pulls in its own transitive deps (miniquad, glam, etc.); no additional direct
  dependency needed on top of it for this sprint's scope.
- No new dependency for CLI parsing (decision #3) or the picker (decision #4) — both reuse
  existing project conventions/deps.

## Testability
`board.rs`/`piece.rs`/`game.rs` remain pure logic, unit-tested exactly as in Sprint 1, plus
new tests for `last_lines_cleared()`. `terminal.rs` and `gfx3d.rs` are both thin I/O adapters
(consistent with Sprint 1's `main.rs` precedent) and are not unit-tested — verified manually
(PTY for terminal mode, direct display run for 3D mode) by Trin/Smith, including the US-13
fallback path (simulated by forcing the init failure branch, since CI/sandboxed environments
may not always have a GPU available to trigger it naturally).

## Open Items for Smith (Gate 2)
- Confirm the single-syntax CLI flag (`--renderer=terminal|3d`, no space-separated form) is
  acceptable — chosen to avoid parser ambiguity without adding a dependency.
- Confirm decision #9's fallback message wording direction ("starting terminal mode
  instead") reads as reassuring rather than alarming — exact copy is still open for Smith's
  input before Neo implements it.

## Post-launch decision #11: force native Wayland, not XWayland (2026-08-08)
`miniquad` defaults to X11/XWayland on Linux. On a real Wayland compositor (found via a
user report after launch, reproduced in the dev sandbox once we knew what to check),
XWayland's window-mapping handshake silently failed: the window was created and rendering
correctly but never actually shown (`xwininfo` reported `Map State: IsUnMapped`
deterministically), with no panic or error — so `catch_unwind`'s fallback (decision #9)
never triggered, since it only catches panics, not "the window exists but is invisible."
`gfx3d.rs`'s `window_conf()` now sets
`platform: miniquad::conf::Platform { linux_backend: LinuxBackend::WaylandOnly, .. }` to
route around XWayland entirely. Confirmed fixed on the reporting user's real hardware,
including full keyboard input working correctly through the native Wayland path.
- Confirm scoping catch_unwind to init-only (not full session) is the right boundary for
  US-13, given it means a genuine mid-game 3D crash is *not* auto-caught by this mechanism.

---

# Sprint 3 Addendum: Spatial 3D Box Tetris (TUI & Fancy GPU Renderers)

**Owner:** Morpheus (Tech Lead)
**Status:** Draft for Smith Gate 2 review
**Date:** 2026-08-08

## Decision Summary

| # | Decision | Rationale |
|---|----------|-----------|
| 1 | Dedicated pure engine module: `src/spatial_game.rs` | 3D Spatial Tetris requires a 3D box grid (5x5x10, X=width, Y=depth, Z=height) and 3D polycube pieces. Splitting logic into `spatial_game.rs` keeps 2D Tetris (`game.rs`/`board.rs`) 100% untouched while keeping the 3D spatial engine pure logic (zero I/O), fully unit-testable via `cargo test`. |
| 2 | 3D Rotation matrices (Pitch X, Yaw Y, Roll Z) | Polycubes rotate 90 degrees around X, Y, or Z axes. Rotation matrices transform block local coordinates `(x, y, z)`. Collisions against 5x5x10 bounds (`0<=x<5`, `0<=y<5`, `0<=z<10`) and existing locked blocks reject illegal rotations. |
| 3 | Horizontal XxY Layer Clearing | When all 5x5=25 cells at a given Z depth level are occupied by locked blocks, that Z layer clears. All layers above (smaller Z) shift down Z by 1. Score scales exponentially for multi-layer clears (1=100xL, 2=300xL, 3=600xL, 4=1000xL). |
| 4 | Terminal TUI 3D Box Renderer (`src/terminal_3d.rs`) | Crossterm-based isometric wireframe well rendering. Draws the 5x5x10 box outline using ANSI characters (`/`, `\`, `|`, `-`, `+`) with depth shading and layer indicators. Provides TUI playability without requiring a GPU display. |
| 5 | Fancy GPU 3D Box Renderer (`src/gfx3d_box.rs`) | Macroquad 3D scene using `Camera3D` positioned at a top-3/4 angle looking into the 5x5x10 well. Bounding well drawn with 3D wireframe lines (`draw_line_3d`); blocks drawn as colored 3D cubes (`draw_cube`). Smooth motion interpolation and layer-clear flash effects included. |
| 6 | Launcher & CLI expansion to 4 modes | `RendererChoice` enum updated: `Terminal` (`terminal`), `Gfx3d` (`3d`), `Terminal3d` (`terminal_3d` / `tui_3d`), `Gfx3dBox` (`3d_box` / `blockout`). Picker UI in `picker.rs` updated to display all 4 options. |
| 7 | Fallback path for GPU 3D Box | Reuses `catch_unwind` pattern (from Sprint 2 Decision #9): if `gfx3d_box` GPU init fails, prints error message and falls back gracefully to `terminal_3d` renderer. |

## Module Layout (Sprint 3 additions)

```
src/
  lib.rs            — + pub mod spatial_game;
  spatial_game.rs   — NEW: SpatialGame, SpatialPiece, SpatialBoard (5x5x10 3D grid, 3D polycubes, 3D rotations, layer clears, pure logic)
  terminal_3d.rs    — NEW: Crossterm isometric wireframe 3D box renderer. `pub fn run(game: SpatialGame) -> ExitCode`
  gfx3d_box.rs      — NEW: Macroquad 3D spatial viewport renderer. `pub fn run(game: SpatialGame) -> ExitCode`
  cli.rs            — Updated: supports terminal, 3d, terminal_3d, 3d_box CLI flags
  picker.rs         — Updated: 4-option startup selector
  main.rs           — Updated: routes 4 choices to respective backend run functions
```

## Testability
`spatial_game.rs` is 100% pure engine logic — covered by comprehensive unit tests (`cargo test`) for 3D piece spawning, 3D translation/rotation, 3D boundary & stack collision, and 3D layer clearing. Renderers `terminal_3d.rs` and `gfx3d_box.rs` are thin I/O adapters tested via UAT.

---

# Sprint 5 Addendum: Two-Player Battle Mode (Local 1v1 & VS CPU)

**Owner:** Morpheus (Tech Lead)
**Status:** Draft for Smith Gate 2 review
**Date:** 2026-08-09

## Decision Summary

| # | Decision | Rationale |
|---|----------|-----------|
| 1 | Battle Engine Wrapper (`src/battle.rs`) | Wraps two independent `Game` engine instances (`player1` & `player2`). Supports `GameMode::Single`, `GameMode::TwoPlayerLocal`, and `GameMode::VsCpu`. Manages match state (Playing, Winner P1/P2/CPU) without mutating underlying single-player core game logic. |
| 2 | Garbage Line Attack Mechanics (`src/board.rs` & `src/game.rs`) | Multi-line clears generate garbage lines (2 lines = 1 garbage, 3 lines = 2 garbage, 4 lines = 4 garbage). Garbage lines are queued and injected at the bottom of opponent's board on piece lock. Garbage lines consist of solid blocks with 1 randomly placed hole. |
| 3 | Autonomous CPU AI Agent (`src/cpu_ai.rs`) | In `VsCpu` mode, Player 2 is driven by a heuristic AI (`CpuAgent`). Evaluates candidate drop positions across all rotations, scoring based on aggregate stack height, holes created, surface bumpiness, and lines cleared. |
| 4 | Dual Board Viewport Rendering (`src/terminal.rs` & `src/gfx3d.rs`) | Layout renderers support dual side-by-side boards with independent HUD panels (Score, Lines, Level, Next Piece) for P1 and P2/CPU. Control mapping assigns P1 (WASD / Space) and P2 (Arrows / Enter) in local 2P mode. |
| 5 | Mode Selection CLI & Picker Integration (`src/cli.rs` & `src/picker.rs`) | Adds `--mode=single|2p_local|vs_cpu` CLI flags. Startup picker expanded to select both Game Mode and Renderer Choice. |

## Module Layout (Sprint 5 additions)

```
src/
  lib.rs            — + pub mod battle; + pub mod cpu_ai;
  battle.rs         — NEW: BattleState, GameMode, MatchWinner, garbage routing logic
  cpu_ai.rs         — NEW: CpuAgent, heuristic evaluation, placement calculator
  board.rs          — + push_garbage_lines(count, rng) logic
  game.rs           — + pending_garbage queue handling on piece lock
  terminal.rs       — Updated: supports side-by-side dual board TUI layout
  gfx3d.rs          — Updated: supports dual board 2D/3D GPU viewports
  cli.rs            — Updated: `--mode` flag parsing
  picker.rs         — Updated: mode selection UI
```

## Testability
`battle.rs`, `cpu_ai.rs`, and garbage injection in `board.rs` are pure logic with zero I/O dependencies. Covered 100% by unit tests (`cargo test`) for dual-engine tick, garbage calculation, garbage row generation, CPU AI candidate scoring, and match victory conditions.


