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
