# CHAT_sprint8 — Sprint Archive

## Summary

Sprint 8 (Tech Debt): removed 3 verified-dead functions, deduped piece_color and amain's Paused/GameOver menu dispatch into menu.rs, split amain/abattle_main into update+draw functions in both gfx3d.rs and gfx3d_box.rs. Included a mid-sprint US-39 scope correction (run_app_async was already clean; real target was amain/abattle_main) and one Fix Bloop retry (Trin caught missing tests on new resolve_menu_action logic). 76/76 tests, 0 clippy warnings throughout. Live GUI smoke test still outstanding - flagged for Smith at Stage 3, environment has no display to drive macroquad.

```mermaid
sequenceDiagram
    autonumber
    participant Cypher
    participant All
    participant Neo
    participant User
    participant Smith
    participant Morpheus
    participant Mouse
    participant Trin
    participant Oracle
    Note over Cypher,Oracle: 📅 2026-08-09
    Cypher->>All: "pm launch — *pm launch sirtet-sprint-6. WebAssembly<br/>(WASM) browser target released!"
    Note over Cypher,Oracle: 📅 2026-08-10
    Neo->>User: "init — Neo online (cold start). Last state:<br/>Sprint 5 battle mode complete — 61/61<br/>tests passing, clippy clean, ready for<br/>new work. Awaiting task assignment."
    Note over Cypher,Oracle: 📅 2026-08-11
    Neo->>User: "fix — WASM renderer fixed: 2 root causes. (1)<br/>direct getrandom dep needed wasm-bindgen<br/>glue our loader can't provide -> swapped<br/>to macroquad::rand everywhere, dropped<br/>rand+getrandom. (2) mq_js_bundle.js<br/>bundled a quad-net plugin that<br/>ReferenceErrors under strict mode<br/>(unused, no networking in this game)…"
    User->>All: "request — *sprint An in game menu system for<br/>choosing the game mode 1/2 player Quit<br/>Restart, etc..."
    Cypher->>Smith: "pm handoff — Sprint 7 stories ready: US-33 (in-game<br/>main menu, mode select), US-34 (pause:<br/>resume/restart/quit-to-menu), US-35<br/>(game-over: restart/main-menu). Root<br/>gap: WASM entry hardcodes VsCpu, no<br/>picker exists in-browser. Scope:<br/>gfx3d.rs (wasm entry) + gfx3d_box.rs.<br/>Full AC in docs/USER_STORIES.md Sprint<br/>7…"
    Smith->>Morpheus: "user approve — *user approve. US-33/34/35 approved.<br/>Amended US-34 directly (co-authored AC,<br/>no reject round-trip): existing instant<br/>Q/Esc-quit and R-restart in<br/>gfx3d.rs/gfx3d_box.rs must be superseded<br/>by the pause menu, not left standing<br/>alongside it; Esc toggles pause<br/>open/closed. Full text in<br/>docs/USER_STORIES.…"
    Morpheus->>Smith: "lead handoff — Sprint 7 architecture recorded<br/>(docs/ARCHITECTURE.md addendum): shared<br/>Menu/MenuAction widget (src/menu.rs)<br/>reused by gfx3d.rs+gfx3d_box.rs, driven<br/>by an AppScreen{MainMenu,Playing,Paused,<br/>GameOver} state machine per renderer.<br/>run_battle(battle) -> run_app(); mode<br/>now chosen in-menu. Native --mode b…"
    Smith->>Morpheus: "user reject — *user reject REASON: run_app() silently<br/>discards an explicit --mode flag for<br/>3d/3d-box (Nielsen #1/#9 - no feedback).<br/>FIX: run_app(initial_mode:<br/>Option<GameMode>) - Some(mode) skips<br/>MainMenu; None (wasm, or no --mode)<br/>shows it. Details in<br/>smith.docs/state.md. @Morpheus *lead<br/>arch sprint-7 revise"
    Morpheus->>Smith: "lead handoff — Revised per Smith's reject:<br/>run_app(initial_mode: Option<GameMode>).<br/>Some(mode) (CLI/picker) skips MainMenu<br/>-> Playing directly; None (wasm entry,<br/>or no --mode) shows MainMenu. run_app<br/>builds its own BattleState internally<br/>either way - single construction path.<br/>docs/ARCHITECTURE.md updated. @Smith…"
    Smith->>Mouse: "user approve — *user approve. run_app(initial_mode:<br/>Option<GameMode>) resolves the Gate 2<br/>concern cleanly - no dropped flags, no<br/>double-asking, no dead ends in<br/>MainMenu->Playing->{Paused,GameOver}.<br/>Sprint 7 arch approved. @Mouse *sm plan<br/>sprint-7"
    Mouse->>Morpheus: "sm handoff — Sprint 7 broken into 3 phases in<br/>task.md: P1 shared src/menu.rs widget<br/>(foundation), P2 gfx3d.rs integration<br/>(WASM entry, primary target), P3<br/>gfx3d_box.rs integration (native parity,<br/>mirrors P2). No Tank/deploy scope.<br/>@Morpheus *lead review sprint plan"
    Morpheus->>Neo: "lead handoff — Sprint plan approved - phase order<br/>matches the architecture addendum<br/>(menu.rs dependency first, gfx3d.rs<br/>prioritized as the WASM entry point).<br/>@Neo *swe impl phase-1"
    Neo->>Trin: "swe handoff — Phase 1 complete: src/menu.rs<br/>(MenuAction, Menu with main/pause/pause-<br/>restart-preselected/game-over<br/>constructors, move_selection/confirm<br/>pure+unit-tested, update/draw macroquad<br/>wrappers). 65/65 tests, clippy 0<br/>warnings. @Trin *qa uat phase-1"
    Trin->>Morpheus: "qa handoff — UAT phase-1 PASS. Independently re-ran<br/>test+lint (65/65, 0 warnings). Verified<br/>each Menu constructor against<br/>US-33/34/35 AC directly - all match,<br/>incl. pause_menu_restart_selected for<br/>Smith's R-pre-selects-Restart amendment.<br/>Minor non-blocking note in<br/>trin.docs/state.md. @Morpheus *lead<br/>review phas…"
    Morpheus->>Neo: "lead handoff — Phase 1 review PASS - clean split of<br/>pure logic from macroquad-dependent I/O,<br/>DRY reuse in<br/>pause_menu_restart_selected, no smells.<br/>@Neo *swe impl phase-2"
    Neo->>Trin: "swe handoff — Phase 2 complete: gfx3d.rs<br/>AppScreen/BattleScreen state machine,<br/>run_app(initial_mode) replacing<br/>run_battle, main.rs wasm entry +<br/>run_gfx3d_with_fallback wired,<br/>web/index.html legend updated. 65/65<br/>tests, 0 lint warnings, native+wasm<br/>build clean. Scope note: amain (Single<br/>Player) reachable from men…"
    Neo->>Trin: "swe handoff — Added 3D Tetris to WASM per user request<br/>- required a renderer-choice step<br/>(menu.rs: RendererKind, renderer_menu,<br/>run_until_choice shared across all menu<br/>screens now) + full AppScreen<br/>integration for gfx3d_box.rs (Phase 3,<br/>mirrors gfx3d.rs's Phase 2). New main.rs<br/>wasm_app_main() owns the single Win…"
    Neo->>All: "swe handoff — Sprint 7 (in-game menu system) complete,<br/>live-verified by user across 2D/3D +<br/>native/WASM. Extended mid-sprint to WASM<br/>multi-renderer support + 5 rounds of<br/>live-bug fixes (frame-boundary double-<br/>input bug, camera clipping, presets).<br/>71/71 tests, 0 lint warnings. Full recap<br/>in neo.docs/state.md and t…"
    User->>All: "request — Tech Debt sprint requested: normalize<br/>arch, dedup code, remove dead parts, fix<br/>complexity. Starting Stage 1 planning<br/>(Tier 2 fast-track)."
    Cypher->>Smith: "pm handoff — Sprint 8 (Tech Debt) stories +<br/>architecture drafted together (Tier 2<br/>fast-track): US-36 dead code removal,<br/>US-37 dedup piece_color, US-38 dedup<br/>amain menu dispatch, US-39 split<br/>run_app_async. All 4 verified against<br/>actual code (grep-traced callers, diffed<br/>duplicates) before scoping - not<br/>assumed. G…"
    Smith->>Mouse: "user approve — *user approve. Reviewed US-36..39<br/>against HCI/no-regression bar: all 4 ACs<br/>are refactor-only (build/lint/test clean<br/>+ explicit 'no behavior change' + live<br/>smoke pass), zero new/changed user-<br/>facing bindings or flows - correct scope<br/>for a tech-debt sprint. Deferred<br/>Game/SpatialGame merge is the right…"
    Mouse->>Neo: "sm handoff — Sprint 8 planned: 4 small phases (dead<br/>code removal -> dedup piece_color -><br/>dedup amain menu dispatch -> split<br/>run_app_async), each 1-2 tasks, task.md<br/>updated (Sprint 7 archived to Previous<br/>Sprints). @Neo *swe impl phase-1"
    Neo->>All: "resume — Resuming as Neo. Phase 1: delete<br/>gfx3d.rs::cell_world_pos(),<br/>gfx3d_box.rs::block_world_pos(),<br/>terminal.rs::run() + their<br/>#[allow(dead_code)] attrs."
    Neo->>Trin: "swe handoff — Phase 1 complete: deleted<br/>gfx3d.rs::cell_world_pos(),<br/>gfx3d_box.rs::block_world_pos(),<br/>terminal.rs::run() + their<br/>#[allow(dead_code)] attrs. 71/71 tests<br/>pass, clippy 0 warnings, no allow-<br/>suppressed dead code left anywhere in<br/>these 3 files. @Trin *qa uat phase-1"
    Trin->>All: "resume — Resuming as Trin. Phase 1 UAT: verify<br/>the 3 deleted functions were genuinely<br/>dead (not just build-clean) and no<br/>regression."
    Trin->>Morpheus: "qa handoff — Phase 1 UAT: PASS. Independently re-ran<br/>make test (71/71) and make lint (0<br/>warnings). Grepped<br/>cell_world_pos/block_world_pos repo-wide<br/>- zero remaining references anywhere<br/>(not just in the 2 files touched).<br/>terminal.rs's Game/GameMode imports<br/>still used elsewhere in the file (no<br/>unused-import warni…"
    Morpheus->>All: "resume — Resuming as Morpheus. Phase 1 code<br/>review: confirm the 3 deletions are<br/>clean removals with no architectural<br/>concerns."
    Morpheus->>Neo: "lead handoff — Phase 1 review: PASS. Pure deletions (19<br/>lines removed, 0 added, 3 files) - no<br/>architectural concerns, matches the<br/>story exactly. @Neo *swe impl phase-2"
    Neo->>Trin: "swe handoff — Phase 2 complete: extracted<br/>piece_color(id) into menu.rs as a shared<br/>pub fn, removed the byte-identical local<br/>copies from gfx3d.rs and gfx3d_box.rs,<br/>both now import it. 71/71 tests, 0<br/>clippy warnings, native+wasm builds<br/>clean (7 pre-existing wasm-target<br/>dead_code warnings on<br/>run_app/window_conf are…"
    Trin->>All: "resume — Resuming as Trin. Phase 2 UAT: verify<br/>piece_color dedup - same colors, no<br/>orphaned imports."
    Trin->>Morpheus: "qa handoff — Phase 2 UAT: PASS. Confirmed zero<br/>remaining piece_color definitions in<br/>gfx3d.rs/gfx3d_box.rs (grep), shared<br/>menu.rs version is byte-identical to<br/>what was removed (colors unchanged).<br/>71/71 tests, 0 clippy warnings.<br/>@Morpheus *lead review phase-2"
    Morpheus->>All: "resume — Resuming as Morpheus. Phase 2 review."
    Morpheus->>Neo: "lead handoff — Phase 2 review: PASS. Matches<br/>architecture doc exactly - piece_color<br/>landed in menu.rs (not a new module),<br/>both renderers import it, net -22 lines.<br/>@Neo *swe impl phase-3"
    Neo->>All: "resume — Resuming as Neo. Phase 3: dedup amain's<br/>Paused/GameOver menu dispatch."
    Neo->>Trin: "swe handoff — Phase 3 complete: shared<br/>menu::resolve_menu_action() dedups<br/>amain's Paused/GameOver dispatch in both<br/>renderers. 71/71 tests, 0 clippy<br/>warnings, native+wasm clean. Full<br/>behavior-preservation reasoning + a<br/>flagged caveat (no live GUI smoke test<br/>possible in this environment) in<br/>neo.docs/state.md. @Tri…"
    Trin->>All: "resume — Resuming as Trin. Phase 3 UAT - closest<br/>look of the 4 phases per Morpheus's<br/>flag."
    Trin->>Neo: "qa reject — Phase 3 UAT: REJECT (fixable, not a<br/>design issue). resolve_menu_action() is<br/>new pure logic with zero macroquad<br/>dependency but shipped with no unit<br/>tests, unlike the rest of menu.rs. Need<br/>4 cases: Resume->true, Restart->true,<br/>QuitToMenu->false+sets bool,<br/>StartMode->false. Full reasoning in<br/>trin.docs…"
    Neo->>All: "resume — Resuming as Neo. Adding<br/>resolve_menu_action unit tests per<br/>Trin's reject."
    Neo->>Trin: "swe handoff — Fixed: added 5 tests for<br/>resolve_menu_action<br/>(Resume/Restart/QuitToMenu/StartMode<br/>cases) plus a guard test asserting<br/>game_over_menu() never offers<br/>Resume/StartMode - codifies the safety<br/>invariant the whole Phase 3 dedup relies<br/>on, so if that ever changes tests catch<br/>it immediately. 76/76 tests (was…"
    Trin->>All: "resume — Resuming as Trin. Phase 3 UAT retry."
    Trin->>Morpheus: "qa handoff — Phase 3 UAT: PASS on retry. 76/76 (5 new<br/>tests independently re-verified,<br/>including the game_over_menu invariant<br/>guard). @Morpheus lead review phase-3.<br/>Live GUI smoke test still outstanding<br/>for Stage 3 - noted in my state, not<br/>blocking."
    Morpheus->>All: "resume — Resuming as Morpheus. Phase 3 review."
    Morpheus->>Neo: "lead handoff — Phase 3 review: PASS. Closure/bool<br/>approach instead of a generic trait was<br/>the right call - avoids machinery the<br/>dedup didn't need, matches my note that<br/>the exact mechanism was Neo's call.<br/>resolve_menu_action's doc comment<br/>correctly explains WHY unifying<br/>Resume/GameOver is safe (not just what<br/>it do…"
    Neo->>All: "resume — Resuming as Neo. Phase 4: split<br/>run_app_async into smaller named<br/>functions."
    Neo->>Morpheus: "swe ask — Correction before Phase 4: run_app_async<br/>is already small (~20 lines) - original<br/>US-39 scoping was wrong (an early grep<br/>missed async fn, mis-attributing<br/>amain/abattle_main's spans to it). Real<br/>god-functions are amain/abattle_main<br/>(~120-150 lines each, both files). Same<br/>underlying goal, different ta…"
    Morpheus->>All: "resume — Resuming as Morpheus. Reviewing Neo's<br/>Phase 4 scoping correction."
    Morpheus->>Neo: "lead approve — Confirmed independently via grep -n<br/>'async fn|pub async fn' in both files -<br/>Neo's correction is right. Approved:<br/>Phase 4 now targets amain/abattle_main<br/>(not run_app_async). Updating<br/>US-39/task.md/ARCHITECTURE.md to match,<br/>then handing back to Neo to implement."
    Morpheus->>Neo: "lead handoff — US-39/task.md/ARCHITECTURE.md updated to<br/>the corrected target<br/>(amain/abattle_main). @Neo swe impl<br/>phase-4"
    Neo->>Trin: "swe handoff — Phase 4 complete: split<br/>amain/abattle_main (corrected target)<br/>into per-iteration update+draw fns, both<br/>files. Loop shell untouched (preserves<br/>frame-boundary fix). 76/76 tests, 0<br/>clippy warnings, native+wasm clean. Last<br/>phase of Sprint 8. Full notes in<br/>neo.docs/state.md. @Trin qa uat phase-4"
    Trin->>All: "resume — Resuming as Trin. Phase 4 UAT - last<br/>phase of Sprint 8."
    Trin->>Morpheus: "qa handoff — Phase 4 UAT: PASS. Independently re-ran<br/>make test (76/76) and make lint (0<br/>warnings). Confirmed loop shells<br/>untouched in all 4 functions (grep for<br/>next_frame().await - exactly one per<br/>function, same position). This is the<br/>last phase of Sprint 8. @Morpheus lead<br/>review phase-4"
    Morpheus->>All: "resume — Resuming as Morpheus. Phase 4 review -<br/>final phase of Sprint 8."
    Morpheus->>Oracle: "lead handoff — Phase 4 review: PASS. Same 4-function<br/>shape in both files (amain_update/amain_<br/>draw/abattle_update/abattle_draw),<br/>matches corrected US-39, mechanical only<br/>per the architecture decision. All 4<br/>Sprint 8 phases now reviewed and passed.<br/>@Oracle ora groom sprint-8"
    Oracle->>All: "resume — Resuming as Oracle. Sprint 8 groom."
```
