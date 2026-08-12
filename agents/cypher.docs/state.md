# Agent State — Cypher (Product Manager)

## Context
### Recent Decisions
- Sprint 8 (Tech Debt) scoped Tier 2 fast-track, combined with Morpheus in one turn: US-36 (remove
  verified dead code: `cell_world_pos`, `block_world_pos`, `terminal.rs::run`), US-37 (dedup
  `piece_color`), US-38 (dedup `amain`'s Paused/GameOver menu dispatch — ~35 near-identical lines
  per renderer), US-39 (split the ~320-line `run_app_async` god-functions). All 4 verified against
  actual code (grep-traced zero callers, diffed near-identical blocks) before writing stories —
  not assumed from the user's general ask. Explicitly deferred: `game.rs`/`spatial_game.rs` and
  `gfx3d.rs`/`gfx3d_box.rs` whole-file duplication — too large/risky for small-phase tech debt,
  different board representations, real gameplay-bug risk if merged. Smith approved same turn
  (Tier 2), Mouse planned phases and handed to Neo. See `docs/USER_STORIES.md` Sprint 8 section.
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
**Status:** Sprint 8 fully planned and handed to Neo. Cypher's part is done for this sprint.
**Assigned to:** Cypher -> (idle until Sprint 8 retro/launch, Stage 3)
**Started:** 2026-08-11

### Task Description
Sprint 8 (Tech Debt) planning, Tier 2 fast-track: combined story+architecture writing with
Morpheus in one turn, Smith+Mouse gated/planned in the next turn, straight to Neo.

### Progress
- [x] US-36: Remove Verified Dead Code
- [x] US-37: Deduplicate Shared Piece-Color Palette
- [x] US-38: Deduplicate `amain`'s Pause/Game-Over Menu Dispatch
- [x] US-39: Reduce Complexity of `run_app_async`
- [x] Smith combined gate — approved
- [x] Mouse phase breakdown — `task.md` Sprint 8, 4 phases
- [x] Handed to Neo (`*swe impl phase-1`)

### Blockers
None.

## Next Steps
### Immediate Next Action
Nothing until Stage 3 (Oracle groom -> Smith e2e -> retro -> Cypher `*pm launch sprint-8`).
Cypher's retro focus per the sprint skill: story quality / AC accuracy — worth checking after the
fact whether "no behavior change" ACs were sufficient/testable in practice, or needed Trin to add
extra verification beyond what was written.

### Waiting On
Neo (Phase Bloop, phases 1-4), then Oracle/Smith close.
