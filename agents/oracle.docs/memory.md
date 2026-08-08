# Project Memory

This file serves as a consolidated index of project-wide decisions, historical context, and key milestones. It is maintained by the Oracle and reviewed by all agents to ensure consistency.

## Project Context
- **Project Name:** tetris
- **Start Date:** 2026-08-07
- **Key Objectives:** Single-player terminal Tetris in Rust — classic rules, TUI via
  crossterm, fully unit-tested engine. See docs/PRD.md for full scope.

## Major Decisions
| Date | Decision | Rationale | Consequences |
|------|----------|-----------|--------------|
| 2026-08-07 | Lib crate + thin bin split | Engine must be unit-testable independent of terminal | 33 unit tests cover all game logic; main.rs untested by cargo test, verified via manual PTY smoke tests instead |
| 2026-08-07 | crossterm, not ratatui | Grid game doesn't need a widget framework | Direct cell-positioned rendering in main.rs |
| 2026-08-07 | 7-bag randomizer | Fairness over pure random | Piece distribution is uniform per 7-piece window |
| 2026-08-07 | Basic rotation, no wall-kick | Matches PRD non-goals / US-2 AC | Simpler fixed rotation tables, no kick-table maintenance |
| 2026-08-07 | Gravity: max(100, 1000*0.85^(level-1)) | Needed a monotonic, no-cliff curve, visibly faster by level 2 | No lookup table to maintain; floor guarantees playability |

See docs/DECISIONS.md for the full write-up of scope, architecture, and implementation
decisions with rationale.

## Repository Structure Memory
- `agents/`: Contains persona-specific documentation and state.
- `docs/`: PRD.md, USER_STORIES.md, ARCHITECTURE.md, DECISIONS.md — product/tech docs.
- `src/`: `lib.rs` (board/piece/game modules, zero terminal deps), `main.rs` (crossterm TUI).
- `task.md`: Single source of truth for the current sprint (maintained by Mouse) — 7 phases,
  all complete as of 2026-08-07.
- `Makefile`: build/test/run/release targets, added this sprint (none existed before).
- `README.md`: project entry point with ToC, added at sprint close.

## Sprint History
- **Sprint 1 (2026-08-07):** Full Tetris implementation, planning through close. 7 phases,
  33 unit tests, clean clippy, manually PTY-verified TUI. See agents/mouse.docs/sprint_log.md.
