# Task Board — Tetris (Rust) Sprint 8 (Tech Debt)

**Maintained by:** Mouse (SM)
**Status:** Complete — all 4 phases implemented, UAT'd, and reviewed. Live GUI smoke pass
outstanding (this environment has no display to drive macroquad) — flagged to Smith for Stage 3.
**Date:** 2026-08-11

Cycle per phase: Neo implements -> Trin UAT -> Morpheus review -> next phase. Tier 2 (Minor/Tech
Debt) sprint per `AGENT.md` rule 10 — planning fast-tracked (Cypher+Morpheus combined, Smith+Mouse
combined), phase Bloop runs the normal way. All 4 phases are refactor-only: no user-facing
behavior change, so each phase's "done" bar is build/lint/test clean + a live smoke pass, not new
functional tests.

## Phase 1 — Remove Verified Dead Code
- [x] 1.1 Deleted `gfx3d.rs::cell_world_pos()`, `gfx3d_box.rs::block_world_pos()`,
  `terminal.rs::run()` and their `#[allow(dead_code)]` attributes (all three confirmed zero
  callers via grep before scoping).
- [x] 1.2 Verified: `cargo build --all-targets`, `cargo clippy --all-targets`, `cargo test` all
  clean with the `allow` attributes gone (proves they were actually dead, not just unreferenced by
  grep).
**Stories:** US-36

## Phase 2 — Dedup Shared Piece-Color Palette
- [x] 2.1 Moved the byte-identical `piece_color(id: u8) -> Color` (macroquad) out of `gfx3d.rs`
  and `gfx3d_box.rs` into one shared `pub fn` in `menu.rs`; both renderers import it.
- [x] 2.2 Verified via diff (shared fn is byte-identical to what was removed). **Live visual
  smoke pass not done** — this environment has no display to run the GPU renderers; flagged for
  Smith at Stage 3.
**Stories:** US-37

## Phase 3 — Dedup `amain`'s Pause/Game-Over Menu Dispatch
- [x] 3.1 Extracted the near-identical `SingleScreen::Paused`/`SingleScreen::GameOver` handling
  (~35 lines/file) from `gfx3d.rs::amain` and `gfx3d_box.rs::amain` into a shared `menu::
  resolve_menu_action()`, parameterized over the one real difference (game-reset + active-piece-
  position tracking: `Game`/`.active().y` vs `SpatialGame`/`.active_piece.z`) via a local
  `restart` bool at each call site.
- [x] 3.2 Both `amain` functions call the shared function; Trin re-verified existing menu tests +
  added 5 new unit tests for `resolve_menu_action` (round-1 UAT reject, fixed same session).
  **Live smoke pass (pause/resume/restart/quit-to-menu) not done** — no display in this
  environment; flagged for Smith at Stage 3.
**Stories:** US-38

## Phase 4 — Reduce `amain`/`abattle_main` Complexity
**[Corrected 2026-08-11]:** originally scoped as `run_app_async` (~320 lines) — that was a
mis-measurement (grep missed `async fn`). `run_app_async` is actually ~20 lines and already
clean. Real target, confirmed via `grep -n "^async fn \|^pub async fn "`: `amain` (~150 lines) and
`abattle_main` (~120 lines), same shape in both files.
- [x] 4.1 Split each of `gfx3d.rs`/`gfx3d_box.rs`'s `amain` and `abattle_main` into a named
  per-iteration update function and a named per-iteration draw function, called once per loop
  body. The `loop { ... next_frame().await ... }` structure itself stayed untouched (verified —
  exactly one `next_frame().await` per function, same position) — it carries the frame-boundary
  quit-to-menu pattern (see `neo.docs/state.md`), which depends on the exact position of the
  yield point. Same decomposition shape in both files.
- [x] 4.2 Verified: no behavior change, all 76 tests pass, clippy 0 warnings, native+wasm builds
  clean.
**Stories:** US-39

**Deferred (out of scope, flagged for a future dedicated spike if pursued):** `game.rs`/
`spatial_game.rs` state-machine duplication and `gfx3d.rs`/`gfx3d_box.rs` whole-file duplication
(window setup, HUD drawing, battle loop) — different board representations make this a real
architecture project, not a safe small-phase refactor.

---

# Previous Sprints

## Sprint 7 — In-Game Menu System (Complete — user-verified live, 2D+3D, native+WASM)

## Phase 1 — Shared `Menu` Widget (`src/menu.rs`)
- [x] 1.1 Create `src/menu.rs`: `MenuAction` enum, `Menu` struct with `main_menu()`/`pause_menu()`/
  `game_over_menu()` constructors, `.update() -> Option<MenuAction>` (Up/Down or W/S navigate +
  wrap-around, Enter confirms), `.draw(screen_w, screen_h)` (2D overlay via `draw_text`/
  `draw_rectangle`, same primitives as the existing HUD).
- [x] 1.2 `src/lib.rs`: add `pub mod menu;`. Unit tests for selection wrap-around and
  `MenuAction` resolution (headless — no macroquad window needed).
**Stories:** US-33, US-34, US-35 (shared foundation)

## Phase 2 — `gfx3d.rs` Integration (WASM entry point — primary target)
- [x] 2.1 Replace `run_battle(battle: BattleState)` with `run_app(initial_mode: Option<GameMode>)`
  in `gfx3d.rs`: add `AppScreen{MainMenu,Playing,Paused,GameOver}`, wire `MainMenu` mode selection,
  Esc toggle pause (Resume pre-selected) / `R` opens pause (Restart pre-selected) — replacing the
  old instant Q/Esc-quit and instant R-restart, `GameOver` entered on `battle.winner` with
  Restart/Main Menu actions.
- [x] 2.2 `src/main.rs`: wasm `fn main()` calls `gfx3d::run_app(None)`, drop hardcoded
  `GameMode::VsCpu`; `run_gfx3d_with_fallback` calls `gfx3d::run_app(Some(battle.mode))`.
  `web/index.html`: update footer control legend to match new bindings.
**Stories:** US-33, US-34, US-35 (gfx3d.rs)

## Phase 3 — `gfx3d_box.rs` Integration (native parity)
- [x] 3.1 Mirror Phase 2's `AppScreen`/`run_app(initial_mode)` pattern in `gfx3d_box.rs`.
- [x] 3.2 `src/main.rs`: `run_gfx3d_box_with_fallback` calls `gfx3d_box::run_app(Some(battle.mode))`.
**Stories:** US-33, US-34, US-35 (gfx3d_box.rs)

## Phase 4 — WASM Multi-Renderer Support + Live Bug Fixes (user-driven, ad hoc)
Added mid-sprint at the user's direct request, then hardened through live testing on the running
dev server rather than a formal Trin UAT pass:
- [x] 4.1 `src/menu.rs`: `RendererKind`, `OptionsScreen` (combined radio-button Game/Players
  screen), shared `run_until_choice()` on both `Menu` and `OptionsScreen`.
- [x] 4.2 `src/main.rs`: wasm-only `wasm_app_main()` orchestrator owning the single
  `Window::from_config`, dispatching to `gfx3d::run_match`/`gfx3d_box::run_match`.
- [x] 4.3 Single Player (`amain` in both renderers) migrated from its old bespoke P/R/Q pause to
  the same `SingleScreen`/`Menu` pattern as Battle mode (shared `SingleScreen` enum in `menu.rs`).
- [x] 4.4 Bug fix: menu-chain frame-boundary bug — a confirming Enter press was still "just
  pressed" on the very next screen's first poll, auto-confirming its default option. Fixed in
  `Menu`/`OptionsScreen::run_until_choice()` AND in `abattle_main`/`amain`'s own quit-to-menu
  return paths (all 4: both renderers x both modes).
- [x] 4.5 Camera: fixed 2D board top-clipping (`default_2d_fancy()` distance/target), added 1-5
  camera angle presets (`OrbitCamera::apply_preset_hotkeys()`, single shared table/method — no
  per-renderer duplication), preset 5 tuned to a safe near-vertical top-down angle with a named
  `MAX_PRESET_PITCH` guard against the fixed-up-vector gimbal singularity.
**Stories:** US-33 (extended scope), US-34, US-35

## Sprint 6 — WebAssembly Browser Target
- [x] WASM target config (`wasm32-unknown-unknown`), `web/index.html` canvas shell +
  `mq_js_bundle.js` glue, `make web`/`make wasm`/`make serve` automation (59/59 tests passing)

## Sprint 1 — Terminal Tetris Core
- [x] Board, Piece, Game engine, Crossterm TUI (33/33 tests passing)

## Sprint 2 — Dual Renderer: Terminal + Accelerated 3D ("Neon Grid")
- [x] CLI parser, 3D macroquad renderer, fallback wrapper (38/38 tests passing)

## Sprint 3 — Spatial 3D Box Tetris (TUI & Fancy GPU Modes)
- [x] 3D spatial game engine (`spatial_game.rs`), 4-way menu picker, 3D TUI wireframe renderer (`terminal_3d.rs`), 3D Fancy GPU renderer (`gfx3d_box.rs`), Tinkercad ViewCube gizmo, translucent wall shading (51/51 tests passing)

## Sprint 4 — Tech Debt & Refactoring
- [x] Consolidation of camera/ViewCube (`camera.rs`) and visual FX (`fx.rs`) across renderers (56/56 tests passing)

## Sprint 5 — Two-Player Battle Mode (Local 1v1 & VS CPU)
- [x] Dual board side-by-side rendering, reciprocal garbage attacks, autonomous CPU AI, 4-renderer battle support (61/61 tests passing)
