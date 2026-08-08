# Agent State

## Context
### Recent Decisions
- All 7 implementation phases reviewed and approved. Architecture (docs/ARCHITECTURE.md)
  held up unchanged through the whole sprint - no decisions had to be revisited.
- Sprint moving to Stage 3 close: Oracle groom -> Smith end-to-end test -> retro -> Cypher
  launch.

### Key Findings
- Engine/renderer separation (decision #1 in ARCHITECTURE.md) proved its worth: 33 unit
  tests cover 100% of game logic with zero terminal dependency, while main.rs (untestable via
  cargo test) was verified through manual PTY smoke testing across phases 6-7.

### Important Notes
None new.

## Current Task
**Status:** Sprint 2 architecture drafted (docs/ARCHITECTURE.md addendum). Awaiting Smith
Gate 2.
**Assigned to:** Morpheus (self) -> Smith
**Started:** 2026-08-08

### Task Description
Design the dual-renderer architecture for Sprint 2 (US-9..US-14): pick a GPU graphics crate,
decide the terminal/3D dispatch mechanism, define the engine/renderer boundary.

### Progress
- [x] Chose `macroquad` 0.4.x over `wgpu` (too low-level) and `bevy` (too heavy/ECS) —
      verified it resolves via `cargo add --dry-run` (v0.4.16)
- [x] Decided AGAINST a `Renderer` trait — two free functions dispatched by `match` in
      main.rs (only 2 fixed backends, chosen once at startup, never swapped at runtime)
- [x] Hand-rolled CLI parsing, single accepted syntax `--renderer=terminal|3d` (no clap dep)
- [x] Picker reuses existing `crossterm` dep, no GPU context needed at picker time
- [x] Module split planned: extract Sprint 1's main.rs into terminal.rs, new gfx3d.rs,
      main.rs shrinks to dispatch-only
- [x] Smooth motion / line-clear FX / window-close / init-fallback (US-11/13/14) all
      designed as renderer-layer-only changes, +1 small additive Game accessor
      (`last_lines_cleared()`) — engine stays otherwise untouched (US-10 regression guard)
- [x] Posted handoff to CHAT.md @Smith *user feedback (Gate 2)

### Blockers
None

### Oracle Consultations
None yet this sprint.

## Next Steps
### Immediate Next Action
Phase 6 (FINAL phase) reviewed: PASS. The catch_unwind fallback is the piece I flagged for
extra scrutiny, and it holds up: Trin independently re-verified it (not just re-reading
Neo's claim), the panic-hook save/restore is correct (doesn't leak into terminal mode's own
panic handling), and the decision #9 scope deviation (whole-session instead of init-only,
forced by macroquad's API bundling init+loop) is honestly disclosed in task.md rather than
silently diverging from the architecture doc - accepting this deviation, it's the right call
given the real constraint and still satisfies US-13's AC. All 6 phases now reviewed and
passed. Stage 2 (Phase Bloop) is complete. Handed to Oracle for Stage 3 groom.

### Waiting On
Oracle *ora groom -> Smith *user test (MUST include live 3D keyboard verification - flagging
this explicitly in the handoff, not leaving it buried in a state file) -> retro -> Cypher
*pm launch.

### Planned Work
- [ ] Post Morpheus's sprint retro when *sprint retro is called in Stage 3: architecture
      held up well overall; the one real deviation (decision #9's init-only catch_unwind
      scoping vs. macroquad's actual API) was a planning assumption that implementation
      proved wrong, not a mistake - worth noting for future sprints that "verify the target
      library's actual API shape" should happen at architecture time, not discovered mid-
      phase, when a library is unfamiliar to the team (this was the sprint's first use of
      macroquad)

---
*Last updated: 2026-08-07 23:21*
