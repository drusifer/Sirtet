# Agent State — Neo (SWE)

## Context
### Recent Decisions
- Sprint 7 (In-Game Menu System) is functionally complete and live-verified by the user across
  both renderers (2D Neon Grid, 3D Spatial Box) and both build targets (native, WASM). See
  `task.md` Sprint 7 for the full phase breakdown, including the ad hoc Phase 4 work added mid-
  sprint at the user's direct request (WASM multi-renderer support) plus several bugs the user
  found through live play rather than a formal Trin UAT pass.
- Key architecture: `src/menu.rs` holds everything menu-related, shared by `gfx3d.rs` and
  `gfx3d_box.rs` — `Menu`/`MenuAction` (pause/game-over/main menus), `OptionsScreen`
  (combined renderer+mode radio-button screen), `SingleScreen`/per-file `BattleScreen` (screen
  state machines), all with `run_until_choice()`/`update()`/`draw()` split so selection logic is
  unit-testable without a live macroquad window.
- `main.rs`'s wasm-only `wasm_app_main()` owns the single `Window::from_config` call and
  dispatches to `gfx3d::run_match(mode)` / `gfx3d_box::run_match(mode)` — this is why both
  renderers expose `run_match` (single-match, returns quit-to-menu bool) alongside their
  standalone `run_app` (native CLI/picker entry, still opens its own window).
- **Recurring bug class fixed twice this sprint, worth remembering:** any async loop that
  `return`s immediately upon detecting a confirming Enter press (before that iteration's
  `next_frame().await`) leaves the "just pressed" flag live for whatever screen runs next,
  which then instantly auto-confirms its own default option. Fixed in `Menu`/`OptionsScreen::
  run_until_choice()` (menu.rs) and in `abattle_main`/`amain`'s own quit-to-menu paths (all 4:
  gfx3d.rs x2, gfx3d_box.rs x2) — pattern is: set a local flag instead of returning immediately,
  let the loop's normal draw+`next_frame().await` run once more, THEN check the flag and return.
  **If a new early-return path is ever added to these loops, apply the same pattern.**
- Camera: `OrbitCamera::apply_preset_hotkeys()` (camera.rs) is the single, DRY home for 1-5
  camera-angle presets — called once from inside `update()`, so every renderer gets presets for
  free with zero per-call-site duplication. `MAX_PRESET_PITCH` (1.55 rad) guards against the
  fixed `up: (0,1,0)` vector's gimbal-lock singularity at true vertical (±π/2) — respect this
  bound if presets are ever added/changed.

### Key Findings
- 71/71 tests passing, clippy 0 warnings, native `cargo build`/`bobp make wasm` both clean (all
  verified via `bobp make`, not raw cargo — this project's Makefile is the required entry point
  per the `make` skill).
- Still outstanding / not done this sprint: Trin/Morpheus/Smith never ran a formal Stage-2/3
  Bloop pass — the user tested live instead and reported bugs directly, which is how the
  frame-boundary bug and the camera clipping were actually found. If a future session wants to
  close this out "by the book," a retrospective/Stage-3 close was skipped.
- `agents/CHAT.md` and most `agents/*.docs/state.md` files (Cypher, Morpheus, Smith, Mouse, Trin)
  reflect the Stage-1 planning gates (both passed) but predate all of Phase 4's ad hoc work — they
  are stale relative to this file and `task.md`. This file and `task.md` are the source of truth
  for where Sprint 7 actually landed.

### Important Notes
- None.

## Current Task
**Status:** Sprint 7 complete and live-verified. Session ending — user requested commit + push
before context clear.
**Assigned to:** Neo -> (next session, cold start)
**Started:** 2026-08-11

### Task Description
Full Sprint 7 delivery: in-game menu system (main menu, pause, game-over) across all 4
renderer/mode combinations, extended mid-sprint to WASM multi-renderer support at the user's
request, plus five rounds of live-bug-report-driven fixes (frame-boundary double-input bug,
board top-clipping, camera presets, top-down preset tuning, quit-to-menu bug).

### Progress
- [x] All of `task.md` Sprint 7 Phases 1-4 (see that file for the itemized list).
- [x] `bobp make test` — 71/71. `bobp make lint` — 0 warnings. Native + WASM builds clean.
- [x] Dev server (`bobp make serve`) has been running on :8080 throughout — user tested every
      change live in-browser.
- [ ] Commit game-related changes and push (in progress as of this state save).

### Blockers
None.

## Next Steps
### Immediate Next Action
If resuming cold: `git log --oneline -3` to confirm the Sprint 7 commit landed and was pushed.
Then check `bobp make serve` — it was running throughout this session in the background; a new
session won't have that process, so if the user wants to keep testing live, run
`bobp make serve` again first.

If the user wants Sprint 7 formally closed out (Trin UAT retrospective, Smith end-to-end pass,
Cypher launch announcement per the `/sprint` skill's Stage 3), that step was skipped in favor of
direct live testing — flag this rather than assuming it already happened.

### Waiting On
User.
