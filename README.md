# Sirtet

A single-player terminal Tetris game written in Rust, using `crossterm` for the TUI.

## Run

```
cargo run
```

## Test

```
cargo test
```

## Controls

| Key | Action |
|-----|--------|
| Left / Right | Move |
| Down | Soft drop |
| Up | Rotate |
| Space | Hard drop |
| P | Pause |
| R | Restart |
| Q / Esc | Quit |

## Docs

- [docs/PRD.md](docs/PRD.md) — product vision, scope, success criteria
- [docs/USER_STORIES.md](docs/USER_STORIES.md) — user stories with acceptance criteria
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — module layout, key technical decisions
- [docs/DECISIONS.md](docs/DECISIONS.md) — consolidated decision log
- [task.md](task.md) — sprint phase breakdown and status

## Source layout

- `src/board.rs` — playfield grid, collision, line clearing (pure logic, no I/O)
- `src/piece.rs` — the 7 tetrominoes and their rotation states (pure logic, no I/O)
- `src/game.rs` — game engine: movement, gravity, scoring, level, state machine (pure logic, no I/O)
- `src/main.rs` — crossterm terminal UI: rendering and input, calls into the engine only

The engine (`board`/`piece`/`game`) has zero terminal dependency and is fully covered by
`cargo test`. `main.rs` is a thin rendering/input adapter, verified by manual testing.

## Team process

This project uses the [bob-protocol](agents/AGENTS.md) multi-persona workflow. See
`agents/CHAT.md` for the team communication log and `agents/*.docs/` for persona state.

## License

GPLv3 — see [LICENSE](LICENSE).
