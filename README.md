# Bombeatbyx

A multiplayer **rhythm bomb arena** played in the terminal.

Bombeatbyx mixes a Bomberman-style arena (bombs, bricks, power-ups, chain explosions) with the core rule of rhythm dungeon crawlers: **every action must land on the beat**. Mashing keys does not work: off-beat inputs are rejected, on-beat inputs build a combo that multiplies your score and powers up your bombs.

![Bombeatbyx cover](cover.avif)

## Features

* Real-time multiplayer over WebSocket: online server or LAN rooms hosted from the client.
* **Solo vs Bots** with 1 to 7 AI opponents and three difficulty levels.
* Destructible bricks hiding power-ups, chain reactions, kill credit.
* Combo multiplier, *Fever* mode, sudden death shrinking arena.
* Deathmatch and Score modes, optional time limit.
* Full emoji rendering with an ASCII fallback for limited terminals.

## Quick Start

### Option 1: Prebuilt release

1. Download the archive for your OS from the [Releases page](https://github.com/Qua-9-9-1/Bombeatbyx.tui/releases).
2. Extract it. Keep `client` and `server` in the **same folder** (the client starts the server itself when you host a LAN game).
3. Run `client` (`client.exe` on Windows) in a terminal of at least **102 x 42** characters.

### Option 2: From source

Requires Rust **1.85 or newer** (edition 2024).

```bash
cargo run --release -p client
```

### Option 3: With Docker (no local Rust needed)

Useful when Rust is not installed, or when the machine blocks unsigned executables.

```powershell
docker run --rm -it -v "${PWD}:/app" -w /app -e TERM=xterm-256color rust:1.88 cargo run --release -p client
```

Use Windows Terminal (or any modern terminal) for correct emoji rendering. See [Development](docs/development.md#building-with-docker) for a faster, cached setup.

## Documentation

| Guide | Content |
| :--- | :--- |
| [How to Play](docs/playing.md) | Controls, rules, rhythm, bonuses, scoring, game modes, bots |
| [Hosting & Networking](docs/hosting.md) | Online server, LAN games, ports, troubleshooting |
| [Development](docs/development.md) | Project layout, build, tests, architecture, contribution rules |

## Project Layout

```
common/   Shared game simulation (rules, rhythm, AI, protocol messages)
client/   Terminal UI (ratatui) and WebSocket client
server/   Authoritative WebSocket game server (axum)
.agents/  Context and rules for AI coding agents
docs/     Player and developer documentation
```
