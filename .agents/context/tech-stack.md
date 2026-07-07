# Technology Stack & Crate Architecture

This document describes the crates, dependencies, and internal architecture of the Bombeatbyx project.

## 1. Project Organization (Cargo Workspace)
The codebase is structured as a cargo workspace with three distinct crates:

### A. `common` (Core logic & Simulation)
* **Responsibility:** Implements the pure, deterministic game simulation logic, layouts, physics, and state machines.
* **Key Components:**
  - `Player` struct: Defined in [models.rs](file:///c:/Users/stage1/Documents/other/Bombeast.tui/common/src/game/models.rs). Holds all entity properties.
  - Spawn points optimization: Performed by `GameState::spawn_players` in [spawns.rs](file:///c:/Users/stage1/Documents/other/Bombeast.tui/common/src/game/state/spawns.rs) to select a dispersed subset of spawn points furthest apart.
* **Dependencies:** `serde`.

### B. `client` (Terminal UI & User Input)
* **Responsibility:** Draws user interfaces (panels, menus, maps), handles inputs, and manages settings.
* **Key Components:**
  - Player color assignment: strictly a client-side layout concern mapped dynamically from the player ID (`player.id`) rather than passing color strings from the server.
  - State reading: Utilizes the central `app.game_ctx.state.players` vector for both Lobby screens and active matches. Does not duplicate/maintain parallel player lists.
* **Dependencies:** `ratatui`, `crossterm`, `tokio`, `tokio-tungstenite` (WebSocket client), `serde_json`, `color-eyre`.

### C. `server` (Game Host & Synchronization)
* **Responsibility:** Runs the WebSocket-based web server, manages connection lobbies, processes client actions, and broadcasts authoritative state updates.
* **Key Components:**
  - Lobby state management and server update loops.
* **Dependencies:** `axum` (with `ws` feature), `tokio`, `tower-http`, `futures-util`.

## 2. Shared Utilities & Integration
- **Serialization:** Messages are serialized to JSON (via `serde` / `serde_json`) for client-server communication over WebSockets.
- **Async Runtime:** Both client and server run on top of the Tokio async runtime.
