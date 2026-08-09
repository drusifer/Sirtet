# Task Board — Tetris (Rust) Sprint 4 (Tech Debt & Refactoring)

**Maintained by:** Mouse (SM)
**Status:** Phase 1 ready — Fast-Track Tier 2 Sprint 4 Plan Approved
**Date:** 2026-08-08

Cycle per phase: Neo implements (TDD) -> Trin UAT -> Morpheus review -> next phase.

---

## Phase 1 — Camera & ViewCube Consolidation (`src/camera.rs`)
- [ ] 1.1 Create `src/camera.rs` containing shared `OrbitCamera`, `ViewCubeGizmo`, vertex rotation math, and default camera initializers.
- [ ] 1.2 Update `src/gfx3d.rs` and `src/gfx3d_box.rs` to consume `camera.rs`.
- [ ] 1.3 Add unit tests in `src/camera.rs` covering vertex projection, rotation bounds, and home reset state.
**Stories:** US-22 (Shared camera & ViewCube)

## Phase 2 — Visual FX & Banner Consolidation (`src/fx.rs`)
- [ ] 2.1 Create `src/fx.rs` containing `CameraShake`, `LandingFx`, `LayerClearFx`, and `ScoreBanner` overlay rendering logic.
- [ ] 2.2 Update `src/gfx3d.rs` and `src/gfx3d_box.rs` to consume `fx.rs`.
- [ ] 2.3 Add unit tests in `src/fx.rs` for FX timer decay and banner formatting.
**Stories:** US-21 (Shared FX helper)

## Phase 3 — Audit & Quality Gates
- [ ] 3.1 Run full unit test suite (`bobp make test`) ensuring 100% pass across engine, camera, and FX.
- [ ] 3.2 Run Clippy lint gate (`bobp make lint`) ensuring 0 warnings across all targets.
- [ ] 3.3 Build clean release binary (`bobp make release`).
**Stories:** US-23 (Codebase audit & zero tech debt)

---

# Previous Sprints

## Sprint 1 — Terminal Tetris Core
- [x] Board, Piece, Game engine, Crossterm TUI (33/33 tests passing)

## Sprint 2 — Dual Renderer: Terminal + Accelerated 3D ("Neon Grid")
- [x] CLI parser, 3D macroquad renderer, fallback wrapper (38/38 tests passing)

## Sprint 3 — Spatial 3D Box Tetris (TUI & Fancy GPU Modes)
- [x] 3D spatial game engine (`spatial_game.rs`), 4-way menu picker, 3D TUI wireframe renderer (`terminal_3d.rs`), 3D Fancy GPU renderer (`gfx3d_box.rs`), Tinkercad ViewCube gizmo, translucent wall shading (51/51 tests passing)
