# Development Guide

## 1. Requirements

* Rust **1.85+** (the workspace uses edition 2024), installed with [rustup](https://rustup.rs/).
* On Windows, either the MSVC toolchain (Visual Studio Build Tools) or the self-contained GNU toolchain.
* Optional: Docker, to build and run without a local toolchain.

## 2. Workspace

| Crate | Role | Main dependencies |
| :--- | :--- | :--- |
| `common` | Pure, deterministic game simulation shared by client and server: rules, rhythm clock, AI, protocol messages | `serde` |
| `client` | Terminal UI, input handling, WebSocket client, offline solo mode | `ratatui`, `crossterm`, `tokio`, `tokio-tungstenite` |
| `server` | Authoritative WebSocket server, rooms, LAN announcements | `axum`, `tokio` |

Key modules of `common/src/game/`:

| Path | Responsibility |
| :--- | :--- |
| `rules.rs` | Every gameplay tuning constant (fuse, points, combo steps, settings bounds) |
| `models.rs`, `bonuses.rs`, `skins.rs` | Data types (`Player`, `Cell`, `RoomSettings`, bonuses, skins) |
| `rhythm.rs` | Beat clock, accuracy judging, latency-aware timing, clock alignment |
| `context.rs` | `GameContext`: game state + rhythm clock, the per-tick loop |
| `bombs.rs`, `physics.rs` | Bombs, blasts, chain reactions, movement, pickups |
| `state/` | Actions, combat and scoring, match flow, spawns, sudden death |
| `ai/` | Bot brain: danger map, pathfinding, goals, simulation tests |

Detailed rules and conventions for contributors (and AI agents) live in [`.agents/`](../.agents/AGENTS.md).

## 3. Build and Run

```bash
cargo build --workspace                 # debug build of everything
cargo run -p client                     # play (debug)
cargo run --release -p client           # play (optimized)
cargo run --release -p server -- 27300  # dedicated server
```

The client needs a terminal of at least **102 x 42** characters.

## 4. Tests

```bash
cargo test --workspace
cargo check --workspace --all-targets   # must report zero warnings
```

* Game rules are unit-tested in `common`, next to the code or in sibling `*_tests.rs` files.
* `common/src/game/ai/simulation_tests.rs` plays deterministic headless bot matches: keep them green when tuning the AI or the rules.
* Tests must be deterministic: use `GameRng::seeded(...)`, never the clock, for randomness.

## 5. Building with Docker

When no Rust toolchain is available (or the OS blocks unsigned executables), build and play inside a Linux container. Keep the build cache in a git-ignored folder so later builds are incremental:

```powershell
# Build (first run takes several minutes)
docker run --rm -v "${PWD}:/app" -w /app `
  -e CARGO_HOME=/app/.toolchain/cargo-linux `
  -e CARGO_TARGET_DIR=/app/.toolchain/linux-target `
  rust:1.88 cargo build --release --workspace

# Play in the current terminal
docker run --rm -it -v "${PWD}:/app" -w /app `
  -e CARGO_HOME=/app/.toolchain/cargo-linux `
  -e CARGO_TARGET_DIR=/app/.toolchain/linux-target `
  -e TERM=xterm-256color -e LANG=fr_FR.UTF-8 `
  rust:1.88 /app/.toolchain/linux-target/release/client
```

* `.toolchain/` is listed in `.gitignore`: never commit toolchains or build outputs.
* `LANG` only selects the default key preset (`fr_*` gives ZQSD).
* Inside Docker, only the solo mode and LAN games within the container work out of the box; publish ports (`-p 27300:27300`) to play with other machines.

## 6. Architecture Notes

* **Authoritative server:** the server ticks every room every 16 ms with `GameContext::tick_game_logic()` and broadcasts the state only when it changed. Clients never decide game outcomes.
* **Offline solo:** the client runs the same `tick_game_logic()` locally and drives the bots with `common::game::ai::decide_bot_action` on each beat.
* **Protocol:** JSON-serialized `ClientMessage` / `ServerMessage` enums (`common/src/messages.rs`) over a WebSocket at `/ws`. See [`.agents/context/tech-stack.md`](../.agents/context/tech-stack.md) for the message flow and the network rules.
* **Compatibility:** `Player`, `GameState` and `RoomSettings` are serialized (and `RoomSettings` is persisted in `config.json`): new fields need `#[serde(default)]`.

## 7. Contributing

### Branches

Work on a dedicated branch named `<Author>/<short-topic>` (for example `Quentin.D/network-fixes`), push it, then open a pull request towards `main`.

### Commit messages

* English, a single line, a few words (one sentence at most).
* Always start with one of these tags:

| Tag | Use |
| :--- | :--- |
| `[ADD]` | New feature or file |
| `[EDIT]` | Change of existing behavior, refactor, documentation |
| `[FIX]` | Bug fix |
| `[DEL]` | Removal |

```text
[ADD] sudden death shrinking arena
[FIX] fire kills on contact and shield lasts several beats
```

### Checklist before committing

* `cargo test --workspace` passes and `cargo check` shows no warnings.
* New tuning values go into `common/src/game/rules.rs`.
* Files stay under 300 lines; split them otherwise.
* Gameplay changes are reflected in [How to Play](playing.md) and `.agents/context/business-context.md`.

## 8. Releases

Pushing a tag `v*` runs `.github/workflows/release.yml`: it builds `client` and `server` for Linux and Windows and publishes `bombeatbyx-linux.zip` and `bombeatbyx-windows.zip` on the GitHub release.

```bash
git tag v0.2.0
git push origin v0.2.0
```
