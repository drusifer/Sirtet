# Sprint Log — Tetris (Rust)

## Sprint 1 (2026-08-07)
**Goal:** Playable single-player terminal Tetris in Rust (see docs/PRD.md).
**Gates cleared:** Smith Gate 1 (stories, approved w/ US-8 added) + Gate 2 (architecture, approved).

**Phases (task.md):**
1. Project scaffold + Board core
2. Piece definitions + rotation
3. Game engine: movement, gravity, lock, next-queue
4. Line clear, scoring, leveling
5. Game over, restart, pause
6. Terminal UI (main.rs)
7. Integration & smoke test

Each phase sized 1-3 tasks per sprint skill rule (context-overflow avoidance).
Cycle: Neo implements -> Trin UAT -> Morpheus review -> next phase.

## Sprint 2 (2026-08-08)
**Goal:** Dual renderer — terminal (unchanged) + GPU-accelerated futuristic 3D mode,
selectable at launch via `--renderer` flag or an interactive picker (see docs/PRD.md
Sprint 2 Addendum).
**Gates cleared:** Smith Gate 1 (US-9..US-14, approved w/ picker-keyboard-nav + US-14
window-close additions) + Gate 2 (architecture: macroquad, no Renderer trait, hand-rolled
CLI parsing, init-only catch_unwind fallback — approved).

**Phases (task.md):**
1. Scaffold: macroquad dep + extract main.rs into terminal.rs + CLI parsing
2. Startup picker (crossterm)
3. Engine accessor: `Game::last_lines_cleared()` (additive only)
4. 3D renderer scaffold: static scene (board/piece/HUD as cubes, neon theme)
5. 3D motion, input, line-clear effects, window-close handling
6. Fallback (catch_unwind) + full integration + regression pass

6 phases, 2-3 tasks each. Phase order follows the dependency chain: extraction/parsing must
land before the picker can dispatch to it; the engine accessor (pure logic, independently
testable) is decoupled early so Phase 4-5's 3D work can consume it; 3D scene comes before 3D
motion/effects; fallback + regression is last since it depends on both renderers existing.
