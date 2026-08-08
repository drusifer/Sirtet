# Agent State

## Context
### Recent Decisions
- Groomed docs at sprint close: created docs/DECISIONS.md (consolidated scope/architecture/
  implementation decisions), updated agents/oracle.docs/lessons.md (RNG-test-flakiness lesson,
  PTY-smoke-test-harness lesson, Write-tool-block workaround), updated
  agents/oracle.docs/memory.md (major decisions table, repo structure, sprint history).
  Created root README.md (was missing) with ToC, run/test instructions, controls table.

### Key Findings
- CHAT.md at 25 real messages (well under the 50-100 archive threshold) - no archiving
  needed this sprint.

### Important Notes
None new.

## Current Task
**Status:** Sprint 2 groom complete, handed to Smith for end-to-end user testing.
**Assigned to:** Oracle (self) -> Smith
**Started:** 2026-08-08

### Task Description
Stage 3 Step 7 (Sprint 2): groom documentation before Smith's final end-to-end test and
sprint retro.

### Progress
- [x] CHAT.md archived: 70 messages, over Sprint 1's 25 but within the 50-100 threshold —
      archived Sprint 1's complete history (lines 1-245) to
      agents/chat_archive/CHAT-ARCHIVE-20260808.md as one coherent unit (not a literal 75%
      line cut, since that would've split Sprint 2's still-active phase context) + regenerated
      agents/CHAT.diagram.md via `bobp chat-diagram`
- [x] docs/PRD.md, docs/USER_STORIES.md, docs/ARCHITECTURE.md status headers updated from
      "Draft for review" to "Approved" (both gates cleared)
- [x] docs/DECISIONS.md: added a full Sprint 2 section (product scope, architecture, the
      disclosed decision #9 deviation, and — critically — the live-3D-input-verification
      known limitation flagged prominently, not buried)
- [x] agents/oracle.docs/lessons.md: added 2 new lessons — sandbox can't deliver synthetic
      X11 input at all (verified via xev, not just our code), and "verify third-party
      library API shape at architecture time" (the macroquad Window::from_config surprise)
- [x] agents/oracle.docs/memory.md: updated decisions table, repo structure, and sprint
      history with Sprint 2's entry (including the open item carried to Stage 3)
- [x] README.md: rewritten for the dual-renderer reality — run instructions for both modes,
      updated source layout (cli.rs/picker.rs/terminal.rs/gfx3d.rs), known-limitation note
- [x] Posted handoff to CHAT.md @Smith *user test — explicitly called out that the 3D
      keyboard-input verification is not optional, it's the one thing this sprint could not
      close without a human at a real keyboard

### Blockers
None

## Next Steps
### Immediate Next Action
Available if Smith needs historical/documentation context during end-to-end testing.

### Waiting On
Smith *user test (MUST include live 3D input) -> retro -> Cypher *pm launch.

### Planned Work
- [ ] Post Oracle's sprint retro (documentation gaps, decisions not recorded) when
      *sprint retro is called — likely note: the input-verification gap should probably be
      flagged as a testable acceptance criterion earlier next time (at story-writing time),
      not discovered as a sandbox limitation mid-implementation

---
*Last updated: 2026-08-07 23:25*
