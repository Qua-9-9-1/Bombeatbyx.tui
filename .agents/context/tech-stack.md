# Technology Stack & Crate Architecture

This document describes the crates, dependencies, and internal architecture of the Bombeast project.

## 1. Project Organization (Cargo Workspace)

The codebase is structured as a Cargo workspace with three distinct crates:

### A. `common` (Core logic & Simulation)
* **Responsibility:** Implements the **pure, deterministic game simulation** — grid layout, player physics, beat/rhythm logic, bomb explosions, bonus spawning, and all state machines shared between client and server.
* **Key Types:**
  - `GameState` (`common/src/game/state/mod.rs`) — Authoritative snapshot of all game data (grid, players, timers, BPM, mode, countdowns, `current_beat`). Server-only fields (`pending_drops`, `rng`) are `#[serde(skip)]`.
  - `GameContext` (`common/src/game/context.rs`) — Wraps `GameState` with a rhythm clock (`RhythmEngine`), exposes `new_match()`, `tick_game_logic()` and `shift_clock()`.
  - `Player` (`common/src/game/models.rs`) — Per-player properties. **Always build one with `Player::new(id, name, skin, lives)`**, never with a struct literal; new fields need `#[serde(default)]`.
  - `RoomSettings` — Lobby configuration; new fields need `#[serde(default)]` because it is persisted in the client `config.json`.
  - `GameMode` — `Deathmatch` | `Score`. `BeatAccuracy` — `Perfect` | `Great` | `Ok` | `Miss` | `Waiting`.
  - `GameAction` — The exhaustive enum of all actions a player can submit in a beat.
  - `GameRng` (`common/src/game/spawns.rs`) — Seedable PRNG. Use `GameRng::seeded` in tests, `from_time` in game code.
  - `ClientMessage` / `ServerMessage` — Serializable WebSocket protocol messages.
* **Module map (`common/src/game/`):**
  | File | Responsibility |
  | :--- | :--- |
  | `rules.rs` | All gameplay tuning constants |
  | `models.rs`, `bonuses.rs`, `skins.rs` | Data types (re-exported from `models`) |
  | `bombs.rs` | Bomb placement, fuse ticking, blasts, chain reactions, brick drops |
  | `physics.rs` | Movement, facing, bonus pickup, dash |
  | `state/actions.rs` | Action dispatch, combo bookkeeping, items, emotes |
  | `state/combat.rs` | Deaths, kill credit, points |
  | `state/match_flow.rs` | Match setup and end conditions |
  | `state/spawns.rs` | Spawns, respawns, brick generation, random bonuses |
  | `state/sudden_death.rs` | Shrinking arena |
  | `ai/` | Bot brain (danger map, pathfinding, goals, simulation tests) |
* **Tests:** large test modules live in sibling `*_tests.rs` files (`#[path = "..."] mod tests;`) to keep source files under 300 lines.
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
  - In **local/offline mode** (solo vs bots), the client runs the full `ctx.tick_game_logic()` loop itself and calls `update_bots()` on each beat.
  - Lobby rows are indexed by the `ROW_*` constants in `client/src/screens/lobby.rs`; never use raw cursor numbers.
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
  |-- CreateRoom(public, lan, name, skin) -> |
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
  |<-- Ping { rtt_ms } -------- |  (every 3 s, carries the measured RTT)
  |-- Pong -------------------> |
```

## 4. Network Rules

| Concern | Rule |
| :--- | :--- |
| **Server address** | Default `127.0.0.1:27300` (the server's default port), editable in Settings. Old `:3000` configs are migrated on load. |
| **Settings trust** | `UpdateSettings` always goes through `RoomSettings::sanitized()`: an invalid size or BPM would panic the server. |
| **Rename** | `JoinRoom` from a peer already in a room is a profile update: only its own room, invalid values ignored, never a disconnect. |
| **Heartbeat** | Server pings every 3 s and drops a peer after 3 unanswered pings. The client drops the link after 12 s of silence and gives up connecting after 5 s. |
| **Latency** | RTT is smoothed (EWMA) per peer in `Room.latencies`. Actions are judged at `reception - RTT/2`, capped by `MAX_LATENCY_COMPENSATION` (150 ms). |
| **Rhythm sync** | On the first snapshot of each beat, the client calls `RhythmEngine::align(beat, RTT/2 + half tick, 25 ms tolerance)`. |
| **Prediction** | The client applies its own action immediately and keeps it over server snapshots only for `RTT + 150 ms` (`client/src/network/sync.rs`). |
| **Broadcast** | `GameStateUpdate` is sent only when the serialized state changed (`Room.last_broadcast_state`). |
| **Host change** | Always via `Room::set_host`, which updates the running game and broadcasts `HostTransferred`. |
| **LAN hosting** | The client looks for the `server` binary next to its own executable first (release zip layout), then in `target/`. |
