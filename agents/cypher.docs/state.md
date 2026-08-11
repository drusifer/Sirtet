# Agent State — Cypher (Product Manager)

## Context
### Recent Decisions
- Sprint 7 scoped: In-Game Menu System (main menu, pause, game-over flow).
- Root motivation: US-32 claimed WASM mode-selection parity but `main.rs`'s wasm entry
  hardcodes `GameMode::VsCpu` — browser players have no way to reach Single Player or
  Local 2-Player at all. `picker.rs` (the existing mode picker) is terminal-only/pre-game,
  not compiled for `wasm32`, so it can't close this gap.
- Scoped Sprint 7 to `gfx3d.rs` (primary — the WASM entry point) and `gfx3d_box.rs` (native
  GPU 3D box, same engine family). Terminal renderers excluded — they already have
  equivalent pre-game selection via `picker.rs`.

### Key Findings
- Sprint 6 (WASM) closed 2026-08-09, but its render/CPU-AI code had unfixed bugs (getrandom/
  wasm-bindgen mismatch, CPU instant-drop, visual halo artifact) discovered and fixed by Neo
  in the current session (commit `d8899ee`) before this sprint started.

### Important Notes
- `docs/USER_STORIES.md` had pre-existing uncommitted edits from Sprint 6 at session start;
  Sprint 7's stories were appended on top, not conflicting.

## Current Task
**Status:** Stories drafted (US-33, US-34, US-35), written to `docs/USER_STORIES.md`. Awaiting
Smith's Gate 1 review.
**Assigned to:** Cypher -> Smith
**Started:** 2026-08-11

### Task Description
Sprint 7 planning, Stage 1 Step 1: write user stories + AC for the in-game menu system
(mode select, pause/restart/quit, game-over flow) requested by the user.

### Progress
- [x] US-33: In-Game Main Menu for Mode Selection
- [x] US-34: Pause Menu — Resume, Restart, Quit to Menu
- [x] US-35: Game Over → Restart / Main Menu Flow
- [ ] Smith Gate 1 review (`*user review`)
- [ ] Morpheus architecture (Gate 2)
- [ ] Mouse phase breakdown

### Blockers
None — awaiting Smith.

## Next Steps
### Immediate Next Action
Smith: `*user review` the 3 stories in `docs/USER_STORIES.md` (Sprint 7 section) against HCI
principles — is the menu flow (main → play → pause/game-over → menu) coherent and free of
dead ends? Must post explicit `*user approve` or `*user reject`.

### Waiting On
Smith.
