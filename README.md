# Sirtet

A single-player Tetris game written in Rust, with a choice of two renderers: a classic
terminal UI (`crossterm`) or a GPU-accelerated, futuristic "Neon Grid" 3D mode (`macroquad`).

## Run

```
cargo run                       # shows a startup picker to choose a renderer
cargo run -- --renderer=terminal
cargo run -- --renderer=3d
```

If `--renderer=3d` fails to start (e.g. no GPU/display available), the game prints a
message and falls back to terminal mode automatically instead of crashing.

## Test

```
cargo test
```

## Controls

Identical in both renderers:

| Key | Action |
|-----|--------|
| Left / Right | Move |
| Down | Soft drop |
| Up | Rotate |
| Space | Hard drop |
| P | Pause |
| R | Restart |
| Q / Esc | Quit |

The startup picker (shown when no `--renderer` flag is given) uses Up/Down to select and
Enter to confirm; Esc/Q quits without starting a game.

## Docs

- [docs/PRD.md](docs/PRD.md) — product vision, scope, success criteria (Sprint 1 + Sprint 2)
- [docs/USER_STORIES.md](docs/USER_STORIES.md) — user stories with acceptance criteria
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — module layout, key technical decisions
- [docs/DECISIONS.md](docs/DECISIONS.md) — consolidated decision log, including known
  limitations (see: live 3D keyboard input verification)
- [task.md](task.md) — sprint phase breakdown and status

## Source layout

- `src/board.rs` — playfield grid, collision, line clearing (pure logic, no I/O)
- `src/piece.rs` — the 7 tetrominoes and their rotation states (pure logic, no I/O)
- `src/game.rs` — game engine: movement, gravity, scoring, level, state machine (pure logic, no I/O)
- `src/cli.rs` — `--renderer` flag parsing (pure logic, unit-tested)
- `src/picker.rs` — crossterm-based startup renderer picker (shown when no flag is given)
- `src/terminal.rs` — crossterm terminal UI: rendering and input, calls into the engine only
- `src/gfx3d.rs` — macroquad 3D UI: neon-themed rendering, motion, line-clear FX, input
- `src/main.rs` — thin dispatcher: parses args, runs the picker if needed, launches the
  chosen renderer (with an init-failure fallback from 3D to terminal mode)

The engine (`board`/`piece`/`game`) has zero terminal or graphics dependency and is fully
covered by `cargo test`, along with `cli.rs`. `terminal.rs`/`picker.rs`/`gfx3d.rs`/`main.rs`
are thin I/O adapters, verified by manual testing (PTY for terminal mode/picker, a real
display for 3D mode).

**Wayland note:** `--renderer=3d` forces native Wayland (`LinuxBackend::WaylandOnly`)
instead of `miniquad`'s XWayland default. On at least one real compositor, XWayland's
window-mapping handshake silently failed — the window was created and rendering correctly
underneath but never actually shown, with no error printed. Native Wayland was confirmed
working (rendering + full keyboard controls) on real hardware; see docs/DECISIONS.md for
the diagnosis. If `--renderer=3d` doesn't show a window on your system, please file it.

## Team process

This project uses the [bob-protocol](agents/AGENTS.md) multi-persona workflow. See
`agents/CHAT.md` for the team communication log (older history archived under
`agents/chat_archive/`) and `agents/*.docs/` for persona state.

## License

GPLv3 — see [LICENSE](LICENSE).
