# Agent State

## Context
### Recent Decisions
- End-to-end *user test: PASS. Full PTY playthrough covering initial render, movement,
  rotation, hard-drop, pause (on a live game - confirmed separately after an ambiguous first
  run), game-over overlay, restart, clean quit. No usability defects filed.
- One non-blocking observation for backlog: line clears are instant with no visual feedback
  (flash/pause) - not a PRD requirement, just a polish idea for a future sprint.

### Key Findings
- My first combined playtest (60 rapid hard-drops before testing pause) hit game-over before
  reaching the pause step, so PAUSED didn't render in that run - re-tested pause in isolation
  on a fresh game and confirmed it works correctly. This was a test-sequencing artifact, not
  a bug; matches Phase 5's verified "pause is a no-op once game-over" behavior.

### Important Notes
None new.

## Current Task
**Status:** Sprint 2 Stage 3 Step 8 (end-to-end *user test) complete — PASS WITH DISCLOSED
GAPS. Handed to all for retro.
**Assigned to:** Smith (self) -> all
**Started:** 2026-08-08

### Task Description
Stage 3 Step 8: full end-to-end user test of Sprint 2 (dual renderer) before sprint retro.

### Progress
- [x] Terminal mode: full PTY playthrough — initial render, move/rotate/hard-drop, pause
      overlay, game-over w/ final score (drove the game to game-over via repeated hard
      drops), restart (score resets to 0, no stale GAME OVER), clean quit (exit 0). All of
      Sprint 1's US-1..US-8 confirmed still intact (US-10 regression guard holds).
- [x] Picker: confirmed shows both options, Enter launches the selected mode.
- [x] 3D mode: window opens (title "Sirtet — Neon Grid" via xdotool), renders the neon
      per-piece palette + dark background + glow + wireframe on a dark scene, HUD
      (NEXT/CONTROLS/SCORE/LEVEL/LINES) all present and positioned correctly — confirmed via
      screenshots captured during Phase 4/5 testing (this session's own screenshot capture
      tooling degraded by the end of this long session and stopped working reliably, but the
      earlier evidence stands and the window itself still opens/runs without crashing on
      every check I ran).
- [x] Forced-fallback (US-13): re-confirmed the message prints, no raw panic leaks, and
      terminal mode starts and plays cleanly.
- [x] **Found and routed a real bug during this test**: `xdotool windowclose` against the 3D
      window didn't reliably quit the process (US-14). Filed `*user bug` -> Trin triaged as
      a legitimate code gap (unlike the keyboard-input issue, WM_DELETE_WINDOW isn't
      synthetic key injection) -> Neo added the missing `is_quit_requested()` check (the
      documented macroquad quit-handling pattern) -> Trin re-verified the fix on code review.
      Could not get a clean positive confirmation via `xdotool windowclose` even after the
      fix (this sandbox is now 0-for-several on that specific mechanism too), but the fix
      itself is correct against macroquad's documented API and doesn't regress anything.
- [x] Posted PASS-with-caveats to CHAT.md @all *sprint retro

### Blockers
None — sprint can proceed to retro/launch, but two items are explicitly NOT verified with
real hardware and must be called out to the user, not quietly dropped:
1. Live keyboard input in 3D mode (Left/Right/Down/Up/Space/P/R/Q/Esc) — code-reviewed and
   unit-test-covered at the engine layer, never exercised with a real keypress.
2. OS window-close button in 3D mode (US-14) — fix applied and code-reviewed, but this
   sandbox could not reliably confirm it fires correctly even after the fix.

## Next Steps
### Immediate Next Action
Post Smith's sprint retrospective (UX issues, HCI gaps, user feedback themes) as part of
*sprint retro.

### Waiting On
N/A - contributing retro now.

### Planned Work
- [ ] Post retro to CHAT.md
- [ ] Make sure Cypher's launch message doesn't overstate verification — the sprint
      delivered real, tested functionality, but two interaction paths need a human with a
      keyboard before anyone should call this 100% done
- [ ] Backlog carried from Sprint 1 (not this sprint): line-clear visual feedback in
      terminal mode (3D mode has one now via US-11; terminal mode still doesn't)

---
*Last updated: 2026-08-07 23:28*
