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
