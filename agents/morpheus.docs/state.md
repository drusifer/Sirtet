# Agent State — Morpheus (Tech Lead)

## Context
### Recent Decisions
- Sprint 8 (Tech Debt) architecture, Tier 2 fast-track combined with Cypher: extracted dedup
  targets get a home in the existing `menu.rs` (not a new module) — `piece_color` (US-37) and a
  new `handle_single_screen()` (US-38) both live there, same "one shared DRY home, zero
  per-renderer duplication" rationale as Sprint 7's `camera.rs::apply_preset_hotkeys()`.
  `handle_single_screen`'s exact generic shape (closure-based reset vs a trait) left to Neo — AC
  only requires the duplication gone, not a specific mechanism; closure is likely simplest since
  `Game`/`SpatialGame` don't share a trait today. US-39's `run_app_async` split is mechanical,
  sequenced last so it acts on the body *after* Phases 2-3 already pulled shared pieces out. Full
  writeup in `docs/ARCHITECTURE.md` Sprint 8 section.
- Sprint 7 architecture (both gates passed, both Stage-1 gates): shared `Menu`/`MenuAction`
  widget in `src/menu.rs`, `AppScreen{MainMenu,Playing,Paused,GameOver}` per renderer,
  `run_app(initial_mode: Option<GameMode>)`. Full design in `docs/ARCHITECTURE.md`.
- Phase 1 code review: **PASS.** `move_selection`/`confirm` are correctly split from the
  macroquad-dependent `update`/`draw` (matches the architecture's testability requirement),
  `pause_menu_restart_selected()` reuses `pause_menu()` via struct-update syntax instead of
  duplicating the option list. No SOLID violations, no smells worth flagging for a widget this
  size. Trin's non-blocking wrap-around test-coverage note (game_over_menu untested at length 2)
  acknowledged but not worth blocking on — logic is length-agnostic by construction.

### Key Findings
- None new.

### Important Notes
- None.

## Current Task
**Status:** All 4 Sprint 8 phases reviewed and passed. Handed to Oracle for Stage 3 close.
**Assigned to:** Morpheus -> (idle until Sprint 8 retro)
**Started:** 2026-08-11

### Task Description
Sprint 8 (Tech Debt) Phase Bloop: review each phase after Trin's UAT.

### Progress
- [x] Wrote Sprint 8 architecture decisions (`docs/ARCHITECTURE.md`).
- [x] Sprint plan (`task.md`, 4 phases) — reviewed via Mouse's combined-turn plan, no objections.
- [x] Phase 1 review: PASS.
- [x] Phase 2 review: PASS (`piece_color` landed in `menu.rs` per architecture doc, net -22 lines).
- [x] Phase 3 review: PASS. `resolve_menu_action(action, &mut quit_to_menu) -> bool` — closure/bool
      approach (not a generic trait) was the right call for a 2-branch difference this small.
      Trin's UAT caught a real gap (new pure logic shipped with no tests) on round 1; Neo's fix
      (5 tests + a guard test on `game_over_menu()`'s options) closed it in one retry — Fix Bloop
      worked as designed, no anti-loop concern.
- [x] **Mid-sprint US-39 scoping correction:** Neo caught that `run_app_async` (the story's
      original target) is already ~20 lines and clean — an early grep during Cypher/Morpheus's
      initial investigation missed `async fn`/`pub async fn` patterns and silently merged 3
      functions' line spans into what looked like one ~320-line function. Real target confirmed
      independently: `amain`/`abattle_main` (~120-150 lines each, both files). Updated
      `USER_STORIES.md`/`task.md`/`ARCHITECTURE.md` before Neo implemented — correct handling of
      a mid-implementation scope correction (flag, verify, fix docs, then proceed), not a silent
      redirect.
- [x] Phase 4 review: PASS. Same 4-function shape in both files (`amain_update`/`amain_draw`/
      `abattle_update`/`abattle_draw`), matches corrected US-39, mechanical only. **All 4 phases
      of Sprint 8 now reviewed and passed.**

### Blockers
None.

## Next Steps
### Immediate Next Action
Sprint 8's Phase Bloop is complete. Nothing for Morpheus until Stage 3's `*sprint retro`, where
Morpheus's retro focus (per the sprint skill) is architecture decisions made + anything to
revisit — worth flagging then: the `&mut`-heavy 10-param `amain_update` signature (both files) is
a known minor smell accepted to avoid introducing a new bundling type mid-tech-debt-sprint: fine
for now, but if this file needs touching again, consider whether a small context struct would
pay for itself.

### Waiting On
Oracle (Stage 3 groom), then Smith (`*user test`), then the full-team retro.
