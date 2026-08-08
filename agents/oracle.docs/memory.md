# Project Memory

This file serves as a consolidated index of project-wide decisions, historical context, and key milestones. It is maintained by the Oracle and reviewed by all agents to ensure consistency.

## Project Context
- **Project Name:** tetris (game name: "Sirtet")
- **Start Date:** 2026-08-07
- **Key Objectives:** Single-player Tetris in Rust with two selectable renderers: the
  original terminal TUI (crossterm) and a GPU-accelerated futuristic 3D mode (macroquad,
  added Sprint 2). Fully unit-tested engine, decoupled from both renderers. See docs/PRD.md
  for full scope (both sprints).

## Major Decisions
| Date | Decision | Rationale | Consequences |
|------|----------|-----------|--------------|
| 2026-08-07 | Lib crate + thin bin split | Engine must be unit-testable independent of terminal | 33 unit tests cover all game logic; main.rs untested by cargo test, verified via manual PTY smoke tests instead |
| 2026-08-07 | crossterm, not ratatui | Grid game doesn't need a widget framework | Direct cell-positioned rendering in main.rs |
| 2026-08-07 | 7-bag randomizer | Fairness over pure random | Piece distribution is uniform per 7-piece window |
| 2026-08-07 | Basic rotation, no wall-kick | Matches PRD non-goals / US-2 AC | Simpler fixed rotation tables, no kick-table maintenance |
| 2026-08-07 | Gravity: max(100, 1000*0.85^(level-1)) | Needed a monotonic, no-cliff curve, visibly faster by level 2 | No lookup table to maintain; floor guarantees playability |
| 2026-08-08 | macroquad for 3D rendering, no Renderer trait | Smallest crate for one small GPU scene; only 2 fixed backends, chosen once at startup | terminal.rs (moved verbatim) + new gfx3d.rs, dispatched by a plain match in main.rs |
| 2026-08-08 | catch_unwind fallback scoped to the whole 3D session, not init-only | macroquad::Window::from_config bundles init+loop, no way to separate them | Deviation from the original architecture doc, disclosed; a genuine mid-session 3D bug would also silently fall back rather than crash visibly |

See docs/DECISIONS.md for the full write-up of scope, architecture, and implementation
decisions with rationale (both sprints).

## Repository Structure Memory
- `agents/`: Contains persona-specific documentation and state. Older chat history archived
  under `agents/chat_archive/` (Sprint 1's full history archived at Sprint 2's groom).
- `docs/`: PRD.md, USER_STORIES.md, ARCHITECTURE.md, DECISIONS.md — product/tech docs, each
  now has a Sprint 1 section and a Sprint 2 addendum.
- `src/`: `lib.rs` (board/piece/game modules, zero I/O deps), `cli.rs` (renderer-flag
  parsing), `picker.rs` (crossterm startup picker), `terminal.rs` (crossterm TUI, Sprint 1's
  main.rs moved verbatim), `gfx3d.rs` (macroquad 3D UI, Sprint 2), `main.rs` (thin dispatcher).
- `task.md`: Single source of truth for the current sprint (maintained by Mouse) — Sprint 1's
  7 phases + Sprint 2's 6 phases, all complete as of 2026-08-08.
- `Makefile`: build/test/run/release targets, added Sprint 1 (none existed before).
- `README.md`: project entry point with ToC, updated each sprint.

## Sprint History
- **Sprint 1 (2026-08-07):** Full Tetris implementation, planning through close. 7 phases,
  33 unit tests, clean clippy, manually PTY-verified TUI. See agents/mouse.docs/sprint_log.md
  and agents/chat_archive/CHAT-ARCHIVE-20260808.md for the full chat history.
- **Sprint 2 (2026-08-08):** Dual renderer — terminal (regression-guarded) + GPU-accelerated
  futuristic 3D mode via macroquad, selectable via `--renderer` flag or a startup picker.
  6 phases, 34 engine + 4 cli unit tests, clean clippy/release build. **Open item carried to
  Stage 3:** live 3D-mode keyboard input needs verification with a real keyboard — this
  sandbox cannot deliver synthetic X11 input at all (see lessons.md). See
  agents/mouse.docs/sprint_log.md for phase breakdown.
