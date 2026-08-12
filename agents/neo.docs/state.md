# Agent State — Neo (SWE)

## Context
### Recent Decisions
- **Sprint 8 US-39 scoping correction (2026-08-11):** the story as written ("split the ~320-line
  `run_app_async` functions") was based on a measurement error during Cypher/Morpheus's initial
  tech-debt investigation — an early `grep -n "^fn \|^pub fn "` pattern doesn't match `async fn` or
  `pub async fn`, so `run_app_async`, `run_match`, `abattle_main`, and `amain` were all invisible
  to that grep. The gap between the last function it DID see (`window_conf`/`run_app`) and the
  next one (`handle_playing_input`) got misread as "one ~320-line `run_app_async` body," when
  actually `run_app_async` is only ~20 lines (already clean — just a menu-choice loop calling
  `run_match`) and the real complexity lives in `amain` (~150 lines) and `abattle_main` (~120
  lines) in between. Confirmed via direct `grep -n "^async fn \|^pub async fn "` in both files —
  same shape in `gfx3d.rs` and `gfx3d_box.rs`. Flagged to Morpheus before implementing rather than
  silently redirecting the work; the underlying goal (split an oversized game-loop function into
  named, readable pieces, no behavior change) is unchanged, only the target function names were
  wrong. **Lesson: when grepping for function definitions in async Rust, always include `async fn`
  and `pub async fn` patterns, not just `fn`/`pub fn` — an incomplete grep silently merges adjacent
  functions into one apparent giant one.**
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
- [x] Sprint 7 committed and pushed (confirmed: `git log` shows `1478bc8`).

## Sprint 8 (Tech Debt) — Phase 3 complete (Phases 1-2 also complete)
**Status:** Phases 1-3 done, Phase 3 handed to Trin for UAT.
**Assigned to:** Neo -> Trin
**Started:** 2026-08-11

### Task Description
Phase 1: delete 3 verified-dead functions. Phase 2: extract shared `piece_color` into `menu.rs`.
Phase 3: dedup `amain`'s Paused/GameOver menu-action dispatch.

### Progress
- [x] Phase 1: deleted `cell_world_pos`/`block_world_pos`/`terminal.rs::run()` + their
      `#[allow(dead_code)]` attrs. Passed UAT + Morpheus review.
- [x] Phase 2: `piece_color(id) -> Color` moved to `menu.rs` as `pub fn`, both GPU renderers'
      local copies removed, both now `use tetris::menu::{piece_color, ...}`. Passed UAT + review.
- [x] Phase 3: added `menu::resolve_menu_action(action: MenuAction, quit_to_menu: &mut bool) ->
      bool` in `menu.rs`. Both `amain`'s `Paused`/`GameOver` arms now call it instead of a
      duplicated `match action { ... }` block (was ~15 lines x 2 arms x 2 files = ~60 lines;
      shared resolver is 10 lines once).
      **Behavior-preservation reasoning (important — this phase touches control flow, not just a
      pure helper):**
      - `Paused`'s Escape-key toggle-back-to-Playing check is untouched — it's a separate `if`
        before `menu.update()` is even called, not part of what got extracted.
      - `GameOver`'s arms never had an explicit `Resume` case (folded into
        `Resume | StartMode(_) => {}` as a no-op) while `Paused`'s did (`Resume => screen =
        Playing`). The shared resolver treats `Resume` the same as `Restart`
        ("transition to Playing", with `Restart` additionally triggering a reset via a local
        `restart` bool at the call site). This is a **safe unification, not a silent behavior
        change**, because `Menu::game_over_menu()`'s `options` list (checked directly in
        `menu.rs`) only ever contains `Restart`/`QuitToMenu` — `menu.update()` can only return an
        action that's in `options`, so `GameOver` can structurally never produce `Resume` in the
        first place. The old `Resume | StartMode(_) => {}` arm was dead code in practice; unifying
        its treatment doesn't change any reachable behavior.
      - `StartMode(_)` still resolves to "stay" (`false`) in both arms, same as before.
- [x] 71/71 tests, 0 clippy warnings, native+wasm clean (same pre-existing 7 wasm warnings as
      Phase 2, confirmed unrelated).
- [x] Trin UAT round 1: REJECTED — `resolve_menu_action` shipped with zero unit tests despite
      being pure, headless-testable logic. Fixed: added 5 tests (Resume/Restart/QuitToMenu/
      StartMode cases) + a guard test asserting `game_over_menu()` never offers Resume/StartMode
      (codifies the exact invariant the dedup's safety argument depends on). 76/76 tests now
      (was 71), 0 clippy warnings. Re-handed to Trin.
- [x] Trin UAT round 2: PASS (76/76 re-verified). Live-smoke-test gap noted, not blocking (see
      above — pre-existing, not new to Phase 3).
- [x] Morpheus review (Phase 3): PASS.
- [x] **US-39 scoping correction (see Recent Decisions at top of this file):** `run_app_async` was
      the wrong target (~20 lines, already clean) — real target is `amain`/`abattle_main` (~120-
      150 lines each). Flagged to Morpheus, confirmed independently, docs corrected
      (`USER_STORIES.md`/`task.md`/`ARCHITECTURE.md`) before implementing.
- [x] Phase 4: split `amain` and `abattle_main` in **both** `gfx3d.rs` and `gfx3d_box.rs` into
      `*_update()` (per-iteration input/tick/menu-transition) and `*_draw()` (per-iteration
      drawing + camera/viewcube update) functions, called once each per loop iteration. The
      `loop { ... next_frame().await ... }` shell is untouched in all 4 — verified by diffing that
      only the body between `let mut quit_to_menu = false;` and `next_frame().await` changed.
      Params passed via `&mut`/`&` rather than a new bundling struct (10 params on the largest
      `*_update`, 8 on `*_draw`) — consistent with the architecture doc's "no new types" decision
      for this sprint; used `#[allow(clippy::too_many_arguments)]` where the count exceeds
      clippy's default of 7 (only on the 2 functions that actually need it — `abattle_update` has
      just 5-6 params and doesn't need the allow, kept it clean).
      Two implementation bugs caught by `bobp make build` before I claimed done (both were type
      mistakes writing the new signatures, not logic errors): `prev_active_y`/`prev_active_z` are
      `i32`/`i8` (piece grid coordinates), not `f32` as I first assumed from the variable name;
      and `landing_fx`/`banner` needed `&mut` in `amain_draw` since their own `.draw()`/
      `.draw_2d_shockwave()` methods take `&mut self` (they track internal FX timer state).
- [x] 76/76 tests, 0 clippy warnings, native+wasm builds clean (same 7 pre-existing wasm warnings,
      reconfirmed unrelated).
- [ ] Trin UAT (Phase 4) — **this is Sprint 8's last phase.** A pass here hands off to Oracle for
      Stage 3 close (`*ora groom`), not another phase.

### Blockers
None. Sprint 8's only open item across all 4 phases is the live-GUI-smoke-test gap (flagged
repeatedly above, first surfaced in Phase 3) — that's a Smith/user follow-up at Stage 3, not a
per-phase blocker.

### Important Notes
Mid-Phase-2 I ran `git stash` / `git stash pop` to A/B-test whether some wasm warnings were
pre-existing vs newly introduced — the pop hit a conflict on `agents/CHAT.md` (a build-log line
appended after the stash). Recovered cleanly by discarding that one regenerable log line and
re-popping; no work was lost, but this is a reminder to prefer `git diff`/`git worktree` over
`git stash` for A/B comparisons in a repo with a live auto-appending chat log — the log itself is
a moving target that stash isn't well suited to.

## Next Steps
### Immediate Next Action
Waiting on Trin's Phase 4 UAT — Sprint 8's last phase. If resuming cold and Trin has passed it,
there's no Phase 5: the sprint moves to Stage 3 close (Oracle `*ora groom` → Smith `*user test` →
`*sprint retro` → Cypher `*pm launch`). The live-GUI-smoke-test gap (flagged since Phase 3, not
resolvable in this environment) is the one open item Smith/user should close before or during
Stage 3's end-to-end test.

### Waiting On
Trin.
