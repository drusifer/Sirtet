# Decisions — Tetris (Rust)

Consolidated from CHAT.md and persona state files by Oracle at sprint close (2026-08-07).

## Product Scope (Cypher, Gate 1 approved by Smith)
- Single-player terminal Tetris, classic rules: 7 tetrominoes, 10x20 board, standard scoring
  (100/300/500/800 x level), level-up every 10 lines, basic (non-SRS) rotation, pause/restart.
- Explicitly out of scope: hold piece, ghost piece, SRS wall-kick, sound, persistent high
  scores, configurable keybindings. Logged as fast-follow backlog in docs/USER_STORIES.md.
- Smith added US-8 (on-screen control legend) at Gate 1 — the game has no menu, so the play
  screen is the only place a first-time player can discover the keys. Also pinned exact key
  bindings (Left/Right/Down/Up/Space/P/R/Q) which were originally left vague in the draft
  stories and would have made Trin's acceptance tests ambiguous.

## Architecture (Morpheus, Gate 2 approved by Smith)
- Lib crate (`tetris`) + thin `main.rs` binary, so the engine (board/piece/game) has zero
  terminal dependency and is fully unit-testable via `cargo test`.
- Terminal backend: `crossterm` directly, not `ratatui` — a fixed grid + a few side panels
  doesn't need a widget framework; direct cell rendering is simpler and easier to test.
- Piece randomizer: 7-bag (`rand` shuffle, refilled when empty) — fairness improvement over
  pure random, not a scope change (Smith confirmed no UX downside).
- Rotation: fixed 4-orientation cell tables per piece, in-place only, reject-on-collision, no
  wall-kick — matches US-2's AC exactly and PRD's non-goals.
- Gravity curve: `interval_ms = max(100, 1000 * 0.85^(level-1))` — monotonic, no cliff, floor
  at 100ms. Chosen so level 2 is ~15% faster than level 1, satisfying US-5's "visibly faster
  by level 2" AC without needing a hand-tuned lookup table.

## Implementation decisions made during the Phase Bloop
- `Board::is_area_free` takes raw `(i32,i32)` coordinates rather than a `Piece` type, so
  board.rs has zero dependency on piece.rs — game.rs is the only place that bridges them.
  This was a deliberate refinement of the architecture doc during Phase 1 implementation.
- `Board` grew `lock_cells`, `cell`, and `clear_full_lines` incrementally as later phases
  needed them (Phase 3 and Phase 4) rather than being fully speced upfront in Phase 1 — normal
  incremental growth, not scope creep, since Board's public surface stayed narrow and testable.
- `#[cfg(test)]`-gated `Board::test_fill_row` helper lets game.rs tests set up deterministic
  line-clear/game-over scenarios without depending on random piece placement, and compiles out
  of release builds entirely.
- `Piece::rotated_cw()` returns a copy rather than mutating in place; the caller (`Game`) is
  responsible for collision-checking before committing — keeps rotation rejection (US-2 AC)
  a one-line check at the call site instead of needing rollback logic.

## Process decisions
- Project had no Makefile before this sprint (bob-protocol scaffolding doesn't create one);
  Neo added one with build/test/run/release targets so `bobp make` tooling works going forward.

---

# Sprint 2 — Dual Renderer (Terminal + Accelerated 3D "Neon Grid")

Consolidated by Oracle at groom (2026-08-08). See docs/PRD.md, docs/USER_STORIES.md,
docs/ARCHITECTURE.md Sprint 2 addenda for full detail; this is the compressed decision log.

## Product Scope (Cypher, Gate 1 approved by Smith)
- Player chooses terminal (unchanged) or a GPU-accelerated futuristic 3D renderer, either
  via `--renderer=terminal|3d` or an interactive startup picker when no flag is given.
- Same engine, same rules, same keybindings in both modes (US-12 parity); terminal mode is
  a hard regression guard (US-10), not just "should still work."
- Smith added two items at Gate 1: the picker must be keyboard-navigable (Up/Down/Enter,
  Esc/Q quits cleanly) for consistency with in-game keybindings; and US-14, requiring the
  3D mode's OS window-close button to quit as cleanly as Q/Esc (platform convention).
- Out of scope: multiple 3D themes, mid-session renderer switching, persistent renderer
  preference, graphics settings UI (resolution/quality/fullscreen).

## Architecture (Morpheus, Gate 2 approved by Smith)
- Graphics crate: `macroquad` 0.4.x — chosen over `wgpu` (too low-level for one small scene)
  and `bevy` (full ECS engine, disproportionate scope for a fixed-camera falling-block game).
- No `Renderer` trait: two free functions (`terminal::run`, `gfx3d::run`) dispatched by a
  `match` in `main.rs` — only 2 backends, chosen once at startup, never swapped at runtime,
  so a trait would add indirection without benefit.
- CLI: hand-rolled parsing, single accepted syntax `--renderer=terminal|3d` — no `clap`
  dependency for two flags.
- Startup picker reuses the existing `crossterm` dependency (no GPU context exists yet at
  picker time); no new dependency needed.
- Module split: Sprint 1's `main.rs` body moved verbatim into `src/terminal.rs`; new
  `src/gfx3d.rs` for the 3D backend; `main.rs` shrinks to arg-parsing + picker + dispatch.
- 3D scene: fixed perspective camera, filled cells drawn as `draw_cube`, neon per-piece
  palette, cheap glow via a larger translucent backing cube (no bloom/post-processing pass).
- Smooth piece motion: the engine still ticks at its fixed gravity interval unchanged; the
  3D renderer lerps the active piece's rendered Y between tick positions using wall-clock
  time — animation is a rendering-layer-only concern.
- `Game::last_lines_cleared()`: one small additive, non-breaking accessor so the 3D renderer
  can trigger a line-clear effect without polling board diffs itself.
- Init-failure fallback (US-13): `catch_unwind` around the 3D attempt, with the default
  panic hook swapped for a no-op during that window so no raw backtrace reaches the player;
  on failure, print one line and start terminal mode instead.

## Implementation deviation from the architecture doc (disclosed, not silent)
- Decision #9 originally scoped `catch_unwind` to *init only*. In practice,
  `macroquad::Window::from_config` bundles window/GPU init and the entire game loop into one
  call with no way to separate them, so the shipped fallback wraps the *whole* 3D session
  instead. This still satisfies US-13's AC (no crash, clean message, terminal fallback) and
  arguably a stronger guarantee — the tradeoff is a genuine mid-session 3D bug would also
  silently fall back rather than surface visibly. Morpheus reviewed and accepted this at
  Phase 6's review rather than treating it as a violation of the architecture doc.
- Line-clear visual effect (US-11) ended up as a whole-scene flash rather than a per-row
  effect, since `last_lines_cleared()` only exposes a count, not which rows — a scope
  interpretation flagged for Smith's e2e review rather than silently overclaiming precision.

## Post-launch fix: 3D window never appeared on a real Wayland laptop (2026-08-08)
A user testing on real hardware (not this dev sandbox) reported `--renderer=3d` appearing
to hang with no window. Diagnosis (`wmctrl -l`, `xdotool getwindowgeometry`, `xwininfo`)
showed the window existed with correct 1024x768 geometry but `Map State: IsUnMapped` —
deterministically reproduced 3/3 times in the dev sandbox too once we knew what to check.
Root cause: `miniquad` defaults to X11/XWayland on Linux, and XWayland's window-mapping
handshake was silently failing on this compositor — the window was created and rendering
correctly underneath (confirmed via raw pixel capture, which doesn't require mapping) but
never actually shown, and no error/panic occurred (so `catch_unwind`'s fallback never
triggered — it only catches panics, not "window never becomes visible").

**Fix:** `gfx3d.rs`'s `window_conf()` now sets
`platform: miniquad::conf::Platform { linux_backend: LinuxBackend::WaylandOnly, .. }`,
forcing native Wayland instead of XWayland. Confirmed fixed directly on the reporting
user's hardware: the window is now visible, and all keyboard controls (movement, rotation,
hard drop, pause, restart, quit) work correctly — this also resolves the dev-sandbox
input-testing gap below, since real input now goes through native Wayland rather than the
XWayland/XTest path this sandbox couldn't exercise.

## Known limitation (dev-sandbox only — resolved for real users, see above)
- **Live keyboard input in 3D mode could not be verified end-to-end inside this development
  sandbox.** The sandbox cannot deliver synthetic (XTest/xdotool) key events to any X window
  at all — independently confirmed by both Neo and Trin using the standard `xev` tool (zero
  KeyPress events even against a plain test window). This was a sandbox-only gap, not a code
  defect, and is now moot for real users: the Wayland fix above was confirmed working,
  including full keyboard input, directly on a user's real laptop.

## Process decisions
- CHAT.md archived at Sprint 2's start-of-groom (70 messages): Sprint 1's complete history
  moved to `agents/chat_archive/CHAT-ARCHIVE-20260808.md` as one coherent unit rather than a
  literal 75%-by-line-count cut, since Sprint 1 was fully self-contained (planning through
  launch) and cutting mid-Sprint-2 would have separated active phase context from the
  retro/launch messages that reference it.
