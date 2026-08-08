# Agent State

## Context
### Recent Decisions
None yet.

### Key Findings
- Full sprint engine (board/piece/game) is at 33 unit tests, all logic-level ACs (US-1
  through US-7 non-rendering parts) covered. main.rs is deliberately untested by cargo test
  (thin adapter per architecture) and was instead verified via PTY smoke tests across two
  phases (6 and 7).

### Important Notes
None new.

## Current Task
**Status:** Sprint 2 Phase 6 (FINAL phase) complete, handed to Trin for UAT.
**Assigned to:** Neo (self) -> Trin
**Started:** 2026-08-08

### Task Description (Phase 6)
Implement Sprint 2 Phase 6 (task.md): init-failure fallback for 3D mode (US-13) + full
integration/regression pass across both renderers — the last phase of the sprint.

### Progress (Phase 6)
- [x] Phase 5 passed Trin's UAT (PASS-WITH-CAVEAT) + Morpheus review.
- [x] `main.rs` restructured: `run_terminal()` and `run_gfx3d_with_fallback()` helpers.
      The fallback wraps `gfx3d::run(Game::new())` in `std::panic::catch_unwind`, with the
      default panic hook swapped for a no-op for the duration (restored after) so a 3D
      failure doesn't dump a raw backtrace — matches the fallback wording Smith approved
      at Gate 2 ("3D rendering unavailable on this system — starting terminal mode instead.")
- [x] Scope deviation from decision #9, disclosed in task.md and here rather than silently
      diverging: the architecture doc wanted `catch_unwind` scoped to *init only*, but
      `macroquad::Window::from_config` bundles init + the whole game loop into one call with
      no way to separate them. Wrapping the whole session still satisfies US-13's literal AC
      and is arguably a stronger guarantee; the tradeoff (a genuine mid-session 3D bug would
      also silently fall back rather than crash visibly) is called out for Morpheus/Smith.
- [x] Verified the fallback actually works, not just compiles: temporarily added
      `panic!(...)` as the first line of `gfx3d::run`, confirmed via PTY test that the
      fallback message prints, terminal mode starts and is fully playable (CONTROLS legend
      renders, `q` exits cleanly with code 0), then reverted the temporary panic — no test
      scaffolding left in the shipped code.
- [x] Full regression: `cargo test` (34 engine + 4 cli), `cargo clippy --all-targets`,
      `cargo build --release` all clean
- [x] Smoke-tested all 4 entry paths in one pass: `--renderer=terminal` (PTY, US-1..US-8
      still pass), `--renderer=bogus` (clear error, exit 1), `--renderer=3d` (window opens,
      runs full duration under `timeout` without panicking), no-flag picker -> Enter ->
      terminal mode -> clean quit
- [x] task.md fully checked off (all 6 phases), added a "Known limitations" section
      documenting the input-verification gap and the decision #9 scope deviation for
      Oracle/Smith at sprint close
- [x] Posted handoff to CHAT.md @Trin *qa uat phase-6 (final phase)

### Blockers
None. Sprint's open items (live 3D input verification) are correctly Smith's Stage 3 job,
not a Neo/Trin blocker.

### Oracle Consultations
None yet this sprint — due next at Stage 3 groom.

## Prior Task (Phase 5 — for reference, superseded by Phase 6 above)
**Status:** Sprint 2 Phase 5 complete, handed to Trin for UAT. IMPORTANT CAVEAT below —
keyboard input in 3D mode could not be automatically verified in this sandbox; flagging
loudly rather than claiming false confidence.
**Assigned to:** Neo (self) -> Trin
**Started:** 2026-08-08

### Task Description
Implement Sprint 2 Phase 5 (task.md): smooth piece-motion interpolation between gravity
ticks, keyboard input wired to Game methods, line-clear flash effect, window-close handling.

### Progress
- [x] Phase 4 passed UAT + Morpheus review, no changes needed since.
- [x] 5.1 Motion: track the active piece's row at the start of each gravity interval
      (`anim_from_y`) and lerp its rendered Y toward the post-tick row over the interval
      (decision #7). Any player-initiated move/rotate/drop/restart snaps `anim_from_y` to
      the new position immediately (no lerp) so input still feels instant — only natural
      gravity fall is smoothed. A lock event (new piece spawns higher than the old one was)
      is detected by `y_after <= y_before` and snaps rather than gliding backward.
- [x] Input: `is_key_pressed` for Left/Right/Down/Up/Space/P/R mapped 1:1 to the same Game
      methods terminal.rs uses (move_left/move_right/soft_drop/rotate/hard_drop/
      toggle_pause/restart); Q/Escape break the frame loop, ending `amain` -> `run` ->
      `main` cleanly (US-14/quit parity with terminal mode, US-12 control parity)
- [x] 5.2 Line-clear FX: `last_lines_cleared()` checked right after each `tick()`; on >0, a
      ~300ms full-scene white flash fades out, intensity scaled by lines cleared (brighter
      for a tetris). NOTE: the engine only exposes a *count* (decision #8), not which rows,
      so this is a whole-scene flash, not a per-row effect — documented as a deliberate,
      honest scope interpretation of "visual effect on the cleared row(s)" (US-11 AC),
      flagging for Smith's e2e review rather than overclaiming per-row precision.
- [x] `cargo build`/`test` (34+4)/`clippy --all-targets` all clean
- [x] Window-close (US-14): relies on macroquad's own frame-loop semantics per
      ARCHITECTURE.md decision #10 — not independently re-verified this phase (see caveat).

### IMPORTANT CAVEAT — could not verify keyboard input end-to-end in this sandbox
Spent significant effort trying to drive real input against the 3D window via `xdotool`
(key/keydown+hold/keyup, windowfocus/windowactivate with and without `--sync`) and capturing
results via X11 screenshots, then via a file-based debug log inside `gfx3d.rs` recording
every `get_keys_pressed()` result. **Zero key events were ever recorded**, even with the
window properly focused/activated first. To rule out a macroquad-specific issue, I ran the
same `xdotool key` commands against a plain `xev` window (the standard X11 event-debugging
tool) — **`xev` also logged zero KeyPress events**, proving this is a sandbox-wide
limitation on synthetic (XTest) key delivery, not something specific to our window or code.
Earlier in this phase I mistakenly read some screenshots as showing multiple hard-dropped
pieces stacked at the bottom of the board and reported (informally, not yet handed to Trin)
that Space was working — on reflection that was very likely a single naturally-spawned
multi-row piece (S/Z/L/J/T all span 2 rows) near its spawn position, not evidence of
successful input. I'm correcting that here before handoff rather than letting it stand.
**Net result: input wiring is verified by code review + structural parity with terminal.rs's
already-proven key mapping, and by the fact every `Game` method it calls has full unit-test
coverage — but the actual keypress-to-screen path in 3D mode has NOT been exercised
end-to-end by an automated test in this environment.** Debug logging code has been removed
from gfx3d.rs (no debug scaffolding shipped). This needs real manual play (a physical
keyboard, not synthetic X events) to close the gap — flagging for Smith's Stage 3 *user test
explicitly, and for Trin now in case Trin's environment can do better.

### Blockers
None blocking further phases, but see caveat above — recommend Trin attempt independent
input verification and escalate to Morpheus/Smith if also unable, rather than silently
inheriting my inconclusive result.

### Oracle Consultations
None yet this sprint.

## Next Steps
### Immediate Next Action
Wait for Trin's Phase 5 UAT verdict (including an attempt at independent input
verification), then Morpheus's review, then start Phase 6 (init-failure fallback + full
integration pass).

### Waiting On
N/A — see current task below, Phase 6 (final phase) complete.

### Planned Work
None — all 6 phases implemented, awaiting Trin's Phase 6 UAT then Morpheus's final review.

---
*Last updated: 2026-08-07 23:21*
