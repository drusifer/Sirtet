# User Stories — Tetris (Rust)

**Owner:** Cypher (PM)
**Status:** Draft for Smith review (Gate 1)
**Date:** 2026-08-07

Each story includes testable acceptance criteria (AC) so Trin can verify without ambiguity.

---

## US-1: Spawn and see the board
**As a** player, **I want** to launch the game and immediately see an empty playfield with a
falling piece, **so that** I can start playing with no setup.

**AC:**
- `cargo run` launches directly into a playing state (no menu required for MVP).
- A 10-wide x 20-tall board is rendered with visible borders.
- One of the 7 tetrominoes (I, O, T, S, Z, J, L) spawns centered at the top on start and after
  every piece lock.
- The next piece is shown in a "next" preview panel.

## US-2: Move and rotate the falling piece
**As a** player, **I want** to move the falling piece left, right, and down, and rotate it,
**so that** I can position it deliberately.

**AC:**
- Left/right arrow keys move the piece one column per press, blocked by walls or stack.
- Down arrow performs a soft drop (moves down one row, does not lock immediately).
- The Up arrow rotates the piece 90 degrees clockwise; rotation that would collide with a
  wall/stack/floor is rejected (piece stays in prior position/orientation) — basic rotation,
  no wall-kick required.
- Spacebar performs a hard drop: instantly drops the piece to the lowest legal position and
  locks it.
- Default key bindings (Left/Right/Down/Up/Space/P/R/Q) are fixed for this sprint — no
  remapping (see PRD Non-Goals).

## US-3: Gravity and locking
**As a** player, **I want** the piece to fall automatically and lock into the stack when it
lands, **so that** the game has real-time pressure.

**AC:**
- The active piece falls one row automatically at a fixed interval determined by current level.
- When the piece cannot move down further, it locks into the board within one tick.
- Immediately after lock, a new piece spawns from the next-piece queue.
- If the newly spawned piece immediately collides with existing stack, the game transitions to
  game-over (US-6).

## US-4: Clear lines and score
**As a** player, **I want** completed rows to clear and my score to increase, **so that** I'm
rewarded for good play.

**AC:**
- Any row fully filled across all 10 columns is cleared after the piece that completed it locks.
- Rows above a cleared row shift down by the number of rows cleared in that lock event.
- Score increases per standard convention: 1 line = 100 x level, 2 = 300 x level,
  3 = 500 x level, 4 (Tetris) = 800 x level (level = current level, 1-indexed).
- Total lines-cleared counter is visible and updates immediately.

## US-5: Level up and speed increase
**As a** player, **I want** the game to speed up as I clear lines, **so that** difficulty scales.

**AC:**
- Level starts at 1 and increases by 1 every 10 lines cleared (cumulative).
- Current level is visible on screen and updates immediately on level-up.
- Gravity interval decreases as level increases, following a documented, monotonic curve
  (exact function is Morpheus's implementation call; must be visibly faster by level 2).

## US-6: Game over and restart
**As a** player, **I want** to know when I've lost and restart without relaunching, **so that**
I can play again quickly.

**AC:**
- Game over triggers when a spawning piece immediately collides with the existing stack.
- A clear "GAME OVER" state is shown along with the final score.
- A documented key (e.g., `R`) restarts into a fresh board/score/level without leaving the
  process.
- A documented key (e.g., `Q` / `Esc`) quits the process cleanly from game-over state.

## US-7: Pause
**As a** player, **I want** to pause mid-game, **so that** I can step away without losing.

**AC:**
- `P` toggles pause.
- While paused, gravity and input (other than unpause/quit) have no effect on the board.
- Paused state is visibly indicated on screen.

## US-8: Discoverable controls (added by Smith, Gate 1)
**As a** first-time player, **I want** to see the controls without reading external docs,
**so that** I can play without guessing keys (Heuristic #10 Help & Documentation, #6
Recognition Rather Than Recall — the game has no menu, so the game screen itself is the
only place a control legend can live).

**AC:**
- The game screen displays a persistent (or always-accessible) control legend covering:
  move (Left/Right), soft drop (Down), rotate (Up), hard drop (Space), pause (P), restart (R),
  quit (Q/Esc).
- The legend is visible without needing to read source code, `--help`, or external docs.

---

## Fast-Follow (explicitly out of this sprint — logged for backlog)
- Hold piece
- Ghost piece (drop preview)
- SRS wall-kick rotation
- Persistent high-score file
- Configurable keybindings

## Traceability
US-1<->Goals 1,5; US-2<->Goal 2; US-3<->Goal 2; US-4<->Goal 3; US-5<->Goal 4; US-6<->Goal 6;
US-7<->Goal 8; US-8<->Goal 7 (TUI rendering, discoverability — added at Smith's Gate 1 review).
Goal 7 is otherwise cross-cutting, verified visually in each story.

---

# Sprint 2: Renderer Selection — Terminal vs. Accelerated 3D

**Owner:** Cypher (PM)
**Status:** Approved (Smith Gate 1, 2026-08-08 — w/ US-9 picker-nav addition + new US-14)
**Date:** 2026-08-08

## US-9: Choose a rendering mode at launch
**As a** player, **I want** to pick between classic terminal rendering and a futuristic 3D
accelerated mode when I start the game, **so that** I can play in whichever style I prefer
or whatever my terminal/GPU supports.

**AC:**
- `cargo run -- --renderer=terminal` launches directly in terminal mode (Sprint 1 behavior,
  unchanged).
- `cargo run -- --renderer=3d` launches directly in the 3D accelerated mode.
- `cargo run` with no `--renderer` flag shows an interactive startup picker (terminal-based,
  since no window exists yet) listing both options; selecting one launches that mode.
- An invalid `--renderer` value prints a clear error listing valid options and exits without
  starting a game.
- The picker is navigable with Up/Down + Enter (consistent with in-game key conventions);
  Esc/Q quits the picker and the process cleanly without starting a game (added by Smith,
  Gate 1 — consistency with existing keybindings, Heuristic #4).

## US-10: Classic terminal mode keeps working exactly as before
**As a** returning player, **I want** terminal mode to look and play exactly like Sprint 1's
game, **so that** nothing regresses.

**AC:**
- All Sprint 1 acceptance criteria (US-1 through US-8) pass unmodified when launched in
  terminal mode.
- No behavior, timing, or control changes are introduced to terminal mode by this sprint.

## US-11: Play in a futuristic 3D accelerated mode
**As a** player, **I want** an eye-catching, GPU-accelerated 3D version of the board with a
futuristic visual style, **so that** I get an alternative, more immersive way to play.

**AC:**
- The board renders as a GPU-accelerated 3D/2.5D scene (not a redraw of ANSI text) with a
  dark, neon-accented "futuristic" theme (glowing block edges/colors, background distinct
  from terminal mode).
- Falling-piece movement is visually smooth (animated/interpolated), not an instant per-tick
  jump — without changing the underlying tick-based game logic or timing.
- Line clears trigger a distinct visual effect (e.g. glow/flash/particle burst) on the
  cleared row(s) before the rows shift down.
- Board, next-piece preview, score, level, and lines-cleared are all visible on screen in 3D
  mode, matching the information available in terminal mode.

## US-12: Same rules, same controls, either mode
**As a** player, **I want** identical gameplay rules and keybindings regardless of which
renderer I chose, **so that** switching modes doesn't mean relearning the game.

**AC:**
- Left/Right/Down/Up/Space/P/R/Q behave identically (per US-2/US-6/US-7 AC) in 3D mode as in
  terminal mode.
- Scoring, leveling, gravity curve, and game-over conditions are identical between modes —
  driven by the same engine, with no renderer-specific rule differences.
- A control legend/HUD is visible in 3D mode as well (parity with US-8), styled to fit the
  futuristic theme.

## US-13: Graceful fallback if 3D mode can't start
**As a** player on a machine without proper GPU support, **I want** a clear message and a
working game rather than a crash, **so that** I can still play.

**AC:**
- If GPU/graphics context initialization fails when `--renderer=3d` is requested (or chosen
  via picker), the game prints a clear, human-readable error explaining 3D mode is
  unavailable and automatically starts in terminal mode instead.
- The process does not panic or exit with an unhandled error/stack trace on this failure path.

## US-14: Closing the 3D window behaves like quitting (added by Smith, Gate 1)
**As a** player, **I want** clicking the window's OS close button to quit the game cleanly,
**so that** the 3D mode behaves like any other desktop application (Heuristic #2 Match
Between System and Real World / platform convention, #4 Consistency and Standards).

**AC:**
- Clicking the OS window close control ("X") in 3D mode exits the process cleanly — same
  clean-quit guarantee as pressing Q/Esc (US-6 AC), no panic/hang/orphaned process.
- No unsaved-progress prompt is required (the game has no persistence to lose — matches
  existing non-goals).

---

## Sprint 2 Fast-Follow (explicitly out of this sprint — logged for backlog)
- Persistent renderer preference (config file / env var default).
- Additional 3D themes/skins, user-selectable color schemes.
- Graphics settings (resolution, quality, fullscreen toggle).
- Mid-session renderer switching without restart.

## Sprint 2 Traceability
US-9<->Goal 2; US-10<->Goal 3,4 (regression guard); US-11<->Goal 1; US-12<->Goal 3,4;
US-13<->Goal 5; US-14<->Goal 1 (platform-consistent window behavior — added at Smith's
Gate 1 review, same pattern as Sprint 1's US-8).

---

# Sprint 3: Spatial 3D Box Tetris (TUI & Fancy GPU Modes)

**Owner:** Cypher (PM)
**Status:** Draft for Smith review (Gate 1)
**Date:** 2026-08-08

## US-15: Choose between 4 game/renderer modes at launch
**As a** player, **I want** to select from 4 distinct modes (2D Terminal, 2D Fancy GPU, 3D Box Terminal, 3D Box Fancy GPU) at startup, **so that** I can play classic 2D or spatial 3D Tetris in my preferred renderer environment.

**AC:**
- `cargo run -- --renderer=terminal` launches 2D Terminal Tetris.
- `cargo run -- --renderer=3d` launches 2D Fancy GPU Tetris.
- `cargo run -- --renderer=terminal_3d` (alias `tui_3d`) launches 3D Spatial Box Terminal Tetris.
- `cargo run -- --renderer=3d_box` (alias `blockout`) launches 3D Spatial Box Fancy GPU Tetris.
- `cargo run` with no flag presents an interactive startup picker displaying all 4 choices cleanly (1. Terminal 2D, 2. Fancy GPU 2D, 3. Terminal 3D Box, 4. Fancy GPU 3D Box).
- Up/Down + Enter navigates and selects options in the picker; Esc/Q quits cleanly.

## US-16: Core 3D Spatial Game Engine
**As a** player, **I want** a true 3D spatial grid engine with 3D polycube pieces falling down a 3D rectangular box/well, **so that** gameplay requires 3D spatial reasoning.

**AC:**
- A 3D box grid of size 5x5x10 (X=5 width, Y=5 depth, Z=10 height) is maintained.
- Standard 3D polycubes (tricubes/tetracubes) spawn centered at top Z=0.
- Gravity advances pieces down the Z axis.
- Collision detection validates piece placement against box boundaries (X, Y, Z) and existing locked 3D blocks.
- When a piece landed cannot move further down Z, it locks into the 3D grid.

## US-17: 3D Spatial Movement & 3D Rotation Controls
**As a** player, **I want** to move pieces across X and Y axes, drop them down Z, and rotate them around X, Y, and Z axes, **so that** I can maneuver pieces into complex 3D stack configurations.

**AC:**
- Arrow keys / WASD move the active piece across X (left/right) and Y (forward/backward) axes.
- Space performs hard drop down Z (instantly drops and locks).
- Down arrow performs soft drop down Z.
- Dedicated keys (e.g. `X`, `Y`, `Z` or `I`/`J`/`K`) perform 90-degree 3D rotations around pitch (X), yaw (Y), and roll (Z) axes.
- Rotations/moves resulting in collisions are rejected safely.

## US-18: 3D Layer Clearing & Scoring
**As a** player, **I want** full horizontal XxY layers to clear when filled and grant score, **so that** I am rewarded for filling 3D levels.

**AC:**
- When all 5x5=25 grid positions at a given Z level are filled with locked blocks, that Z layer clears.
- All layers above the cleared layer shift down Z by 1.
- Clearing 1, 2, 3, or 4 layers simultaneously awards exponentially scaling score points.
- Layer cleared count is tracked and updated in real-time.

## US-19: Terminal (TUI) 3D Box Renderer
**As a** terminal user, **I want** to play 3D Spatial Box Tetris inside a standard ANSI terminal using isometric wireframe rendering, **so that** I don't need a GPU to play 3D Tetris.

**AC:**
- Terminal 3D mode (`terminal_3d`) renders an isometric/axonometric wireframe representation of the 5x5x10 box in ANSI text.
- Grid depth and block positions are visually distinguishable using ASCII/ANSI character gradients/depth cues.
- Next piece preview, score, level, layers cleared, and 3D control legend are visible on screen in TUI.

## US-20: Fancy GPU 3D Box Renderer
**As a** player with a graphics card, **I want** a fully accelerated 3D Macroquad scene for 3D Box Tetris, **so that** I can experience glowing 3D polycubes and smooth 3D box projection.

**AC:**
- GPU 3D Box mode (`3d_box`) renders a 3D Macroquad viewport with an isometric/perspective camera framing the wireframe box well.
- 3D polycubes render as glowing 3D cubes in space.
- Full HUD and control legend are rendered on overlay panels.
- If GPU initialization fails, gracefully falls back to Terminal 3D mode with a message (per US-13 pattern).

---

## Sprint 3 Fast-Follow
- Custom box size selection (e.g. 3x3x10, 4x4x10, 6x6x12).
- Pentacube / extended piece sets toggle.

---

# Sprint 5: Two-Player Battle Mode (Local 1v1 & VS CPU)

**Owner:** Cypher (PM)
**Status:** Draft for Smith review (Gate 1)
**Date:** 2026-08-09

## US-24: Battle Mode Selection & Launch Options
**As a** player, **I want** to select between Single Player, Local 2-Player (1v1), or VS CPU mode at startup, **so that** I can play competitively against a friend or a computer opponent.

**AC:**
- Startup menu / CLI flag supports `--mode=single`, `--mode=2p_local`, and `--mode=vs_cpu`.
- Interactive mode picker lets players choose game mode (Single Player / Local 1v1 / VS CPU) and renderer mode.
- Dual-player keybindings in 2P local mode: Player 1 (A/D move, S soft drop, W rotate, Space hard drop), Player 2 (Left/Right move, Down soft drop, Up rotate, Enter hard drop). Legend displays controls clearly for both players.

## US-25: Side-by-Side Dual Board Rendering
**As a** player, **I want** to see both Player 1's and Player 2's (or CPU's) boards side-by-side with separate HUDs, **so that** I can track both my stack and my opponent's progress in real-time.

**AC:**
- Terminal (TUI) and Fancy GPU renderers display two independent boards side-by-side.
- Each board displays its own falling piece, next piece preview, score, lines cleared, and level.
- Clear visual labeling indicates Player 1 vs Player 2 / CPU.

## US-26: Garbage Line Attack Engine
**As a** competitive player, **I want** multi-line clears on my board to dump garbage lines into my opponent's board, **so that** I can pressure my opponent and disrupt their stack.

**AC:**
- Clearing multiple lines sends garbage lines to the opponent: 2 lines cleared = 1 garbage line, 3 lines cleared = 2 garbage lines, 4 lines (Tetris) = 4 garbage lines. (Single line clear = 0 garbage lines).
- Garbage lines push existing blocks upward from the bottom of the recipient's board.
- Each garbage line contains solid blocks across all columns except for 1 randomly placed hole.
- Incoming garbage queue / pending attack is applied to the opponent's board when their active piece locks.

## US-27: Autonomous CPU Opponent AI
**As a** solo player, **I want** to play against a computer-controlled opponent in VS CPU mode, **so that** I can practice battle mode without needing a second human player.

**AC:**
- In VS CPU mode, Player 2 is driven by an AI agent evaluating placement locations (evaluating stack height, holes, line clears, and surface roughness).
- CPU operates at a configurable drop speed / tick delay matching difficulty levels.
- CPU piece movement, rotation, and line clearing execute cleanly without crashing or freezing.

## US-28: Battle Match Win/Loss & Results Screen
**As a** battle mode player, **I want** a clear match victory notification when my opponent tops out, **so that** the winner of the battle is clearly celebrated.

**AC:**
- When one player's board tops out (new piece cannot spawn), that player is knocked out and the surviving player is declared WINNER.
- A match results overlay displays "PLAYER 1 WINS!" / "PLAYER 2 WINS!" / "CPU WINS!" alongside both final scores.
- Pressing `R` restarts the battle match; pressing `Q`/`Esc` returns to main menu or exits cleanly.

---

# Sprint 6: WebAssembly (WASM) Browser App Target

**Owner:** Cypher (PM)
**Status:** Approved (Gate 1 & 2 passed)
**Date:** 2026-08-09

## US-29: WebAssembly Compilation Target (`wasm32-unknown-unknown`)
**As a** web developer / player, **I want** the Tetris game to compile to WebAssembly (`wasm32-unknown-unknown`), **so that** the game engine and renderers run directly in web browsers without native desktop dependencies.

**AC:**
- Project compiles cleanly for `wasm32-unknown-unknown` target.
- Macroquad GPU 2D and 3D renderers build to WebAssembly without compilation errors.
- Automation in `Makefile` provides `make web` / `make wasm` targets.

## US-30: HTML5 Canvas Shell & Web Assets
**As a** browser player, **I want** an HTML5 canvas container and modern WebGL wrapper, **so that** I can launch Tetris directly from any web browser.

**AC:**
- `web/index.html` provides a responsive canvas layout, title, and styling.
- Miniquad/Macroquad JS glue loads and initializes the WASM bundle cleanly.
- Page handles viewport resizing and window focusing automatically.

## US-31: Local Development Web Server (`make serve`)
**As a** developer / QA, **I want** a `make serve` target to quickly launch a local HTTP server, **so that** I can test and verify the WebAssembly app locally in a browser.

**AC:**
- `make serve` starts a local HTTP server on port 8080 (or available port) serving the WASM web app.
- Automatically handles correct MIME types (`application/wasm`).

## US-32: Browser Controls & Full Game Mode Parity
**As a** player in a web browser, **I want** full control parity (keyboard controls, mode selection, battle mode, 3D viewports), **so that** the web app provides the exact same experience as native desktop.

**AC:**
- Keyboard input (WASD, Arrow keys, Space, Enter, XYZ rotation keys) translates cleanly in browser canvas.
- Single Player, VS CPU, and Local 2-Player battle modes work in WebAssembly canvas.

---

# Sprint 7: In-Game Menu System

**Owner:** Cypher (PM)
**Status:** Draft for Smith review (Gate 1)
**Date:** 2026-08-11

**Motivation:** US-32 claimed mode-selection parity in the browser, but the WASM entry point
(`main.rs`'s `#[cfg(target_arch = "wasm32")] fn main()`) hardcodes `GameMode::VsCpu` with no
selection mechanism — browser players cannot currently reach Single Player or Local 2-Player at
all. The existing mode/renderer picker (`picker.rs`) only covers the terminal-native launch path
(crossterm, pre-game, not compiled for `wasm32`), so it can't close this gap. This sprint adds a
menu system rendered *inside* the macroquad/GPU engine itself — the one code path shared by native
and WASM builds — so mode selection, pause, restart, and quit all work identically everywhere,
including the browser.

**Scope:** `gfx3d.rs` (primary — the WASM entry point) and `gfx3d_box.rs` (native GPU 3D box, same
engine family, kept at parity). The terminal renderers (`terminal.rs`, `terminal_3d.rs`) already
have equivalent pre-game selection via `picker.rs` and are out of scope for this sprint.

## US-33: In-Game Main Menu for Mode Selection
**As a** player launching the game — especially in the browser, where no pre-game picker exists —
**I want** an in-game main menu to choose Single Player, VS CPU, or Local 2-Player before the
match starts, **so that** I'm not locked into a single hardcoded mode.

**AC:**
- On launch, `gfx3d.rs` (and `gfx3d_box.rs`) show a menu screen with the 3 `GameMode` options and
  a clear "start" action, before any board is drawn.
- Menu is navigable with Up/Down (or W/S) + Enter, and renders identically on native and WASM
  builds (no platform-specific input or crossterm dependency).
- Selecting a mode starts a fresh `BattleState` in that mode.

## US-34: Pause Menu — Resume, Restart, Quit to Menu
**As a** player mid-match, **I want** to pause and choose Resume, Restart Match, or Quit to Main
Menu, **so that** I can back out of or restart a match without killing the browser tab or process.

**AC:**
- Pressing a dedicated key (e.g. Esc) during `GameState::Playing` opens a pause overlay; gameplay
  and gravity ticks stop while it's open.
- Resume returns to the match exactly as it was paused.
- Restart Match starts a new `BattleState` in the same mode that was active.
- Quit to Main Menu returns to the US-33 menu screen (never a hard process exit in the WASM build,
  since there is no OS process for a browser tab to return to).
- Pressing Esc again while the pause overlay is open resumes play (same key toggles it open/closed)
  — a player who pauses by reflex must be able to get back into the match the same way, without
  hunting for a different key.
- **[Smith, HCI #4/#5]** This pause menu formally supersedes the current instant, unconfirmed `Q`/
  `Esc`-to-quit and `R`-to-restart bindings in `gfx3d.rs`/`gfx3d_box.rs` — those exit/restart the
  match with zero confirmation today, which is itself an error-prevention gap this story is
  expected to close, not leave standing alongside a new menu. `R` during play opens the pause menu
  (pre-selected on Restart) rather than restarting instantly; the footer control legend must be
  updated to match the new bindings so it doesn't advertise dead behavior.

## US-35: Game Over → Restart / Main Menu Flow
**As a** player whose match just ended, **I want** the game-over screen to offer Restart and Main
Menu actions, **so that** I can immediately play again without reloading the page or restarting
the binary.

**AC:**
- The existing match-winner overlay gains two selectable actions: Restart (same mode) and Main
  Menu (returns to US-33).
- Keyboard-navigable the same way as US-33/US-34 (consistent input pattern across all three menus).
- No dead end: every terminal game state (win, loss, pause) has a visible, reachable way back to
  either another match or the main menu.

---

# Sprint 8 — Tech Debt: Normalize Architecture, Dedup Code, Remove Dead Parts, Fix Complexity

**Owner:** Cypher (PM) + Morpheus (Tech Lead) — Tier 2 fast-track (combined story + architecture)
**Status:** Draft for Smith review (combined Gate)
**Date:** 2026-08-11

**Motivation:** User-requested tech-debt sprint. All four items below were verified against the
current codebase (not assumed) before scoping: `cargo clippy --all-targets` confirmed clean, then
each dead-code candidate was grep-traced for zero callers, and each duplication candidate was
diffed to confirm near-identical content, before being written up as a story. This is a Tier 2
sprint — no end-user-facing behavior changes; acceptance criteria are proof-of-no-regression
(build/lint/test clean, visual/behavioral parity), not new UX.

**Explicitly out of scope (considered, deferred):** `game.rs`/`spatial_game.rs` (2D vs 3D game
state machines) and the full-file duplication between `gfx3d.rs`/`gfx3d_box.rs` (window setup,
HUD drawing, battle-mode loop) share structural shape but operate on genuinely different board
representations (flat 2D grid vs true 3D spatial). Merging them is a real architecture project,
not a small, safe, in-place refactor — it risks introducing gameplay bugs for a payoff this sprint
isn't sized for. Recommend as a future dedicated spike if pursued, not folded in here.

## US-36: Remove Verified Dead Code
**As a** maintainer, **I want** unused, `#[allow(dead_code)]`-suppressed functions deleted,
**so that** the codebase doesn't carry silently-unused surface area that misleads future readers
into thinking it's load-bearing.

**AC:**
- `gfx3d.rs::cell_world_pos()`, `gfx3d_box.rs::block_world_pos()`, and `terminal.rs::run()` are
  deleted along with their `#[allow(dead_code)]` attributes.
- `cargo build --all-targets`, `cargo clippy --all-targets`, and `cargo test` all pass clean with
  zero warnings after removal — a clean clippy pass with the `allow` gone is the actual proof these
  were dead, not just a grep that missed a caller.
- No behavior change: native and WASM builds run identically to before.

## US-37: Deduplicate the Shared Piece-Color Palette
**As a** maintainer, **I want** the byte-identical `piece_color()` palette in `gfx3d.rs` and
`gfx3d_box.rs` defined exactly once, **so that** a future color tweak can't be applied to one
renderer and silently forgotten in the other.

**AC:**
- One shared `piece_color(id: u8) -> Color` (macroquad) used by both `gfx3d.rs` and
  `gfx3d_box.rs`; the per-file copies are removed.
- Visual output is unchanged — same 7 tetromino colors, verified by running both renderers.
- `terminal.rs`'s own `piece_color` (ratatui `Color`, a different type/library) is explicitly out
  of scope — it's a different concern, not true duplication.

## US-38: Deduplicate `amain`'s Pause / Game-Over Menu Dispatch
**As a** maintainer, **I want** the near-identical `SingleScreen::Paused`/`SingleScreen::GameOver`
menu-action handling in `gfx3d.rs::amain` and `gfx3d_box.rs::amain` unified into one shared
implementation, **so that** a future menu behavior change (e.g. a new `MenuAction` variant) is
implemented once instead of twice in lockstep, with the risk of the two copies drifting apart.

**AC:**
- The shared dispatch logic (Resume / Restart / QuitToMenu handling, currently ~35 near-identical
  lines per file) lives in one place, parameterized over the one genuine per-renderer difference:
  how to construct a fresh game (`Game::new()` + tracking `active().y` vs `SpatialGame::new()` +
  tracking `active_piece.z`).
- Both `amain` functions call the shared logic. Existing pause/restart/quit/resume behavior is
  unchanged — Trin re-verifies via existing menu tests plus a live smoke pass of both renderers.
- `SingleScreen` stays where it already correctly lives (`menu.rs`) — this story unifies its
  *handling*, not its definition.

## US-39: Reduce Complexity of the `amain`/`abattle_main` Functions
**[Corrected 2026-08-11, mid-Phase-4 — see `neo.docs/state.md`]:** the original draft named
`run_app_async` as the target, based on a grep that missed `async fn`/`pub async fn` patterns and
silently mis-measured the gap between two small functions as one ~320-line function.
`run_app_async` is actually already small (~20 lines) and clean. The real god-functions —
confirmed via `grep -n "^async fn \|^pub async fn "` in both files — are `amain` (Single Player
loop, ~150 lines) and `abattle_main` (Battle loop, ~120 lines), same shape in `gfx3d.rs` and
`gfx3d_box.rs`. AC below is corrected to match; underlying motivation is unchanged.

**As a** maintainer, **I want** `amain`'s and `abattle_main`'s per-frame logic split into smaller,
named functions, **so that** the input/state-transition handling for each screen state and the
per-frame drawing can each be read and reasoned about independently instead of requiring the
whole ~120-150-line loop body held in your head at once.

**AC:**
- Each of `amain`/`abattle_main` (both files) has its per-iteration state-transition/input/tick
  logic extracted into one named function and its per-iteration drawing extracted into another,
  called once per loop iteration — the `loop { ... }` itself and its `next_frame().await` stay in
  place (the frame-boundary quit-to-menu pattern documented in `neo.docs/state.md` depends on the
  exact position of that yield point relative to the return, so the loop structure isn't touched).
- No behavior change — same control flow, same async yield points, all existing tests still pass.
- The split shape is consistent between `gfx3d.rs` and `gfx3d_box.rs` (same decomposition pattern
  applied to both, easier to keep in sync going forward).
- Sequenced last (Phase 4) since it's easiest to do cleanly once Phases 2-3 have already pulled the
  shared pieces (`piece_color`, menu dispatch) out of the function bodies being split.


