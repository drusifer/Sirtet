# Task Board — Tetris (Rust) Sprint 5 (Two-Player Battle Mode)

**Maintained by:** Mouse (SM)
**Status:** Sprint 5 Complete (All 5 phases implemented and verified)
**Date:** 2026-08-09

Cycle per phase: Neo implements (TDD) -> Trin UAT -> Morpheus review -> next phase.

---

## Phase 1 — Battle Engine & Mode Wrapper (`src/battle.rs`)
- [x] 1.1 Create `src/battle.rs` defining `GameMode` (`Single`, `TwoPlayerLocal`, `VsCpu`), `MatchWinner`, and `BattleState` wrapping P1 and P2 `Game` instances.
- [x] 1.2 Add unit tests in `src/battle.rs` covering dual engine initialization, tick update, and match winner determination when a board tops out.
**Stories:** US-24, US-28

## Phase 2 — Garbage Attack Mechanic (`src/board.rs`, `src/game.rs`)
- [x] 2.1 Implement `push_garbage_lines(count, rng)` in `src/board.rs` to shift stack up and insert bottom rows with 1 random hole.
- [x] 2.2 Add pending garbage queue in `src/game.rs` and route multi-line clear attacks (2 lines = 1 garbage, 3 = 2, 4 = 4) to opponent on piece lock.
- [x] 2.3 Add unit tests verifying garbage calculation, stack shift, hole generation, and attack queue application.
**Stories:** US-26

## Phase 3 — Autonomous CPU Opponent AI (`src/cpu_ai.rs`)
- [x] 3.1 Create `src/cpu_ai.rs` containing `CpuAgent` with heuristic placement evaluation (aggregate height, holes, bumpiness, line clears).
- [x] 3.2 Integrate CPU AI tick controller into `BattleState` for `VsCpu` mode.
- [x] 3.3 Add unit tests in `src/cpu_ai.rs` verifying candidate placement scoring and move selection.
**Stories:** US-27

## Phase 4 — Dual Board Renderers & Mode Launcher (`src/terminal.rs`, `src/gfx3d.rs`, `src/cli.rs`, `src/picker.rs`)
- [x] 4.1 Update `src/cli.rs` and `src/picker.rs` for `--mode=single|2p_local|vs_cpu` flags and mode selection.
- [x] 4.2 Update `src/terminal.rs` and `src/gfx3d.rs` to render dual boards side-by-side with separate HUDs and victory screen overlay.
**Stories:** US-24, US-25, US-28

## Phase 5 — Audit & Quality Gates
- [x] 5.1 Run full unit test suite (`bobp make test`) ensuring 100% pass across battle engine, garbage attack, CPU AI, and existing renderers (61/61 tests passing).
- [x] 5.2 Run Clippy lint gate (`bobp make lint`) ensuring 0 warnings across all targets.
- [x] 5.3 Build clean release binary (`bobp make release`).
**Stories:** US-28

---

# Previous Sprints

## Sprint 1 — Terminal Tetris Core
- [x] Board, Piece, Game engine, Crossterm TUI (33/33 tests passing)

## Sprint 2 — Dual Renderer: Terminal + Accelerated 3D ("Neon Grid")
- [x] CLI parser, 3D macroquad renderer, fallback wrapper (38/38 tests passing)

## Sprint 3 — Spatial 3D Box Tetris (TUI & Fancy GPU Modes)
- [x] 3D spatial game engine (`spatial_game.rs`), 4-way menu picker, 3D TUI wireframe renderer (`terminal_3d.rs`), 3D Fancy GPU renderer (`gfx3d_box.rs`), Tinkercad ViewCube gizmo, translucent wall shading (51/51 tests passing)

## Sprint 4 — Tech Debt & Refactoring
- [x] Consolidation of camera/ViewCube (`camera.rs`) and visual FX (`fx.rs`) across renderers (56/56 tests passing)
