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

---

# Sprint 2 Addendum: Accelerated 3D Renderer ("Neon Grid" mode)

**Owner:** Cypher (PM)
**Status:** Approved (Smith Gate 1, 2026-08-08)
**Date:** 2026-08-08

## Vision
Give players a second way to experience the same Tetris engine: a GPU-accelerated,
futuristic-themed 3D visual mode, selectable alongside the existing terminal renderer.
Core gameplay (rules, timing, scoring) stays identical between both — only the renderer
changes. Sprint 1's terminal mode is not replaced, it becomes one of two options.

## Goals (Sprint 2 Scope — IN)
1. A GPU-accelerated rendering mode with a futuristic aesthetic (neon/cyberpunk grid,
   glowing blocks, smooth piece motion, a distinct visual moment on line-clear).
2. Player chooses "Terminal" or "3D Accelerated" rendering at startup — via CLI flag, or an
   interactive picker when no flag is given.
3. Same game engine (board/piece/game) drives both renderers, unchanged — renderer is
   swappable; engine stays pure logic (per Sprint 1 Architecture Decision #1).
4. Full input/gameplay parity between renderer modes — same keybindings, same rules, same
   scoring/leveling/pause/restart/game-over behavior.
5. Graceful degradation: if 3D mode can't initialize (no GPU/driver/display), report it
   clearly and fall back to terminal mode rather than crashing.

## Explicit Non-Goals (OUT of scope this sprint)
- Multiple selectable 3D themes/skins — one futuristic theme only.
- Mid-game renderer switching (choice made once at launch).
- Mobile/touch, VR, or web/WASM targets.
- Multiplayer, networking, sound (still out per Sprint 1 PRD, unchanged).
- Custom shader/graphics settings UI (resolution, quality sliders) — fixed, tuned defaults.
- Persistent user preference/config file for a default renderer (fast-follow candidate).

## Target Platform
Same Rust binary, Linux primary target. 3D mode requires a GPU accessible via the chosen
graphics backend (Morpheus's call at architecture stage) and must run on typical desktop
Linux GPU/driver setups.

## Success Criteria
- `cargo run -- --renderer=terminal` and `cargo run -- --renderer=3d` both launch a
  playable game.
- `cargo run` with no flag shows a startup picker to choose the renderer.
- Gameplay (movement/rotation/gravity/scoring/leveling/pause/restart/game-over) is
  behavior-identical between the two modes — same engine, zero engine test regressions.
- 3D mode visibly delivers a "futuristic" look: dark background, neon/glowing block colors,
  smoothly animated piece movement (not an instant per-tick redraw), and a distinct visual
  moment on line-clear.
- If 3D initialization fails, the game does not crash — it reports the failure and starts
  in terminal mode.

## Open Questions
- Which Rust graphics/windowing crate (e.g. wgpu, macroquad, three-d, bevy) best fits "one
  small futuristic theme, GPU-accelerated, minimal scope creep" — Morpheus's call.
- Whether the engine/renderer boundary needs a new `Renderer` trait so `main.rs` dispatches
  to either backend while `board`/`piece`/`game` stay unchanged — likely yes, Morpheus
  confirms in the Architecture doc.

---

# Sprint 3 Addendum: Spatial 3D Box Tetris in TUI and Fancy GPU Renderers

**Owner:** Cypher (PM)
**Status:** Draft for review (Smith Gate 1, 2026-08-08)
**Date:** 2026-08-08

## Vision
Expand Sirtet with a true 3D spatial Tetris game mode (classic Mac 3D Tetris / Blockout style) featuring an X x Y x Z rectangular box grid and 3D polycube pieces falling down Z. To maintain our core principle of total renderer parity across platforms, 3D Spatial Tetris will be available in both a Terminal UI (TUI ASCII/ANSI isometric wireframe) renderer and a Fancy GPU 3D renderer.

The complete game suite now offers 4 selectable modes:
1. 2D Tetris — Terminal (TUI)
2. 2D Tetris — Fancy GPU 3D
3. 3D Spatial Box Tetris — Terminal (TUI Isometric Wireframe)
4. 3D Spatial Box Tetris — Fancy GPU 3D

## Goals (Sprint 3 Scope — IN)
1. Core 3D Spatial Engine: `SpatialGame` engine handling 3D polycube pieces, 3D grid collision (default 5x5x10 box), 3D movement (X/Y translation), 3D rotation (X, Y, Z axes), Z gravity/drops, and XxY horizontal layer clearing.
2. Terminal (TUI) Spatial Renderer (`terminal_3d`): Isometric ASCII/ANSI wireframe well representation rendered in terminal with depth cues, piece position indicators, layer indicators, and full HUD parity.
3. Fancy GPU Spatial Renderer (`3d_box`): GPU-accelerated 3D Macroquad scene rendering the wireframe box well, 3D glowing blocks, smooth motion, and line/layer clear effects.
4. Universal Launcher & CLI flags: Startup picker updated to 4 options (`1. Terminal 2D`, `2. Fancy GPU 2D`, `3. Terminal 3D Box`, `4. Fancy GPU 3D Box`). CLI `--renderer` supports `terminal`, `3d`, `terminal_3d`, and `3d_box` (with aliases `tui_3d`, `blockout`).
5. Control & Gameplay Parity: Universal keybindings for 3D translation (Arrows / HJKL / WASD), 3D rotation (X/Y/Z keys), drop (Space), pause (P), restart (R), quit (Q/Esc) across both 3D renderers.

## Explicit Non-Goals (OUT of scope this sprint)
- Custom user-configurable box dimensions — fixed 5x5x10 box size for engine consistency.
- Pentacubes or custom piece editor — standard tricubes/tetracubes piece set.
- Sound effects or music (still out per PRD).

## Success Criteria
- `cargo run -- --renderer=terminal_3d` launches TUI Spatial 3D Box Tetris.
- `cargo run -- --renderer=3d_box` launches Fancy GPU Spatial 3D Box Tetris.
- Interactive startup picker displays all 4 game modes.
- `SpatialGame` engine is fully unit-tested for 3D collision, 3D rotation, and 3D layer clearing.
- Terminal 3D mode renders clear isometric box projection and depth cues in ANSI terminal.
- Fancy GPU 3D mode renders 3D box scene with Macroquad camera and neon 3D blocks.

