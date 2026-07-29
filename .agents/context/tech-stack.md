# Technology Stack & Crate Architecture

This document describes the crates, dependencies, and internal architecture of the Bombeast project.

## 1. Project Organization (Cargo Workspace)

The codebase is structured as a Cargo workspace with three distinct crates:

### A. `common` (Core logic & Simulation)
* **Responsibility:** Implements the **pure, deterministic game simulation** — grid layout, player physics, beat/rhythm logic, bomb explosions, bonus spawning, and all state machines shared between client and server.
* **Key Types:**
  - [`GameState`](file:///c:/Users/stage1/Documents/other/Bombeast.tui/common/src/game/state/mod.rs) — Authoritative snapshot of all game data (grid, players, timers, BPM, mode, countdowns).
  - [`GameContext`](file:///c:/Users/stage1/Documents/other/Bombeast.tui/common/src/game/mod.rs) — Wraps `GameState` with a rhythm clock (`RhythmEngine`) and exposes `tick_game_logic()`.
  - [`Player`](file:///c:/Users/stage1/Documents/other/Bombeast.tui/common/src/game/models.rs) — All per-player properties (lives, combo, beat accuracy, bonuses, items, emotes).
  - `GameMode` — `Deathmatch` | `Score`.
  - `BeatAccuracy` — `Perfect` | `Good` | `Miss` | `Waiting`.
  - `GameAction` — The exhaustive enum of all actions a player can submit in a beat.
  - `ClientMessage` / `ServerMessage` — Serializable WebSocket protocol messages.
* **Spawn logic:** `GameState::spawn_players` in [spawns.rs](file:///c:/Users/stage1/Documents/other/Bombeast.tui/common/src/game/state/spawns.rs) selects a dispersed subset of spawn points furthest apart.
* **Dependencies:** `serde` (+ `serde_json` for message serialization shared with consumers).

---

### B. `client` (Terminal UI & User Input)
* **Responsibility:** Renders all TUI screens (main menu, lobby, in-game map, pause, victory), handles keyboard input, manages client settings, and maintains the WebSocket connection to the server.
* **Application States (`AppState`):**
  - `MainMenu` → `HostModal` | `JoinRoomMenu` | `SettingsMenu`
  - `HostModal` → `Lobby` (after room creation)
  - `JoinRoomMenu` → `Lobby` (after joining)
  - `Lobby` → `InGame` (on `GameStarted`)
  - `InGame` → `PauseMenu` | `VictoryScreen`
* **Server Connection Modes (in `App`):**
  - **Online mode:** `network.is_multiplayer = true`, WebSocket connects to a remote hosted server URL.
  - **Local mode:** `server_process: Option<Child>` — the client spawns the `server` binary as a child process, then connects to it on `localhost`. Killed automatically on `App::drop`.
* **Key Concerns:**
  - Player color assignment is a **client-side** concern derived from `player.id` — never sent from the server.
  - State reading uses `app.game_ctx.state.players` as the single source of truth for both lobby and in-game screens. No parallel player lists.
  - In **multiplayer mode**, the server tick drives `GameState`; the client only calls `ctx.rhythm.tick_logic()` locally for rhythm display.
  - In **local/offline mode**, the client runs the full `ctx.tick_game_logic()` loop itself.
* **Dependencies:** `ratatui`, `crossterm`, `tokio`, `tokio-tungstenite` (WebSocket client), `serde_json`, `color-eyre`.

---

### C. `server` (Game Host & Synchronization)
* **Responsibility:** Runs the WebSocket-based game server, manages connection rooms, validates incoming client actions (including **beat-window verification**), and broadcasts authoritative `GameState` updates to all peers.
* **Key Components:**
  - `SharedState` — `Arc<Mutex<ServerState>>` containing all active `Room`s keyed by room code.
  - `Room` — Holds `peers` (map of player ID → `PeerHandle`), `game_ctx: Option<GameContext>`, `room_settings`, `host_id`, `in_game` flag, and a ban list.
  - Message handlers in `server/src/websockets/rooms/handlers/`:
    - `handle_create_room` — Validates and registers a new room with optional `is_public` / `is_lan` flags.
    - `handle_join_room` — Adds a peer, rejects banned players, broadcasts `LobbyUpdate`.
    - `handle_start_game` / `handle_stop_game` — Host-gated game lifecycle transitions.
    - `handle_action` — **Validates beat window** before applying `GameAction` to `GameState`.
    - `handle_transfer_host`, `handle_kick_player`, `handle_ban_player` — Host-only privilege actions.
  - `disconnect_peer` — Cleans up peer on WebSocket close; reassigns host; stops game if < 2 active players.
* **Dependencies:** `axum` (with `ws` feature), `tokio`, `tower-http`, `futures-util`, `serde_json`.

---

## 2. Shared Utilities & Integration

| Concern | Mechanism |
| :--- | :--- |
| **Serialization** | JSON via `serde` / `serde_json` over WebSocket frames |
| **Async Runtime** | `tokio` on both client and server |
| **Rhythm Clock** | `RhythmEngine` in `common` — provides current beat index and beat window boundaries |
| **Grid Encoding** | `Vec<Cell>` in `GameState`; indexed as `grid[y * width + x]` |
| **Room Discovery (LAN)** | UDP broadcast on port `27315` (client binds on startup to receive LAN server announcements) |

## 3. WebSocket Protocol Summary

```
CLIENT                          SERVER
  |-- GetRooms ----------------> |
  |<-- RoomList --------------- |
  |-- CreateRoom(public, lan) -> |
  |<-- LobbyUpdate ------------ |
  |-- JoinRoom(code, name, skin) |
  |<-- LobbyUpdate ------------ |
  |-- ToggleReady -----------> |
  |-- UpdateSettings ---------> |  (host only)
  |-- StartGame -------------> |  (host only)
  |<-- GameStarted(state) ----- |
  |-- Action(GameAction) -----> |  (beat-validated server-side)
  |<-- StateUpdate(GameState) - |
  |<-- GameStopped(victory) --- |
  |-- LeaveLobby ------------> |
```
