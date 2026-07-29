# Business & Gameplay Context (Bombeast.tui)

This document describes the high-level goals, game rules, and accessibility modes of the Bombeast rhythm-battle game.

## 1. Project & Game Overview

**Bombeast** is a **multiplayer rhythm-based bomb arena** that fuses two genre classics:

| Inspiration | What it brings |
| :--- | :--- |
| **Bomberman** | Grid arena, bomb placement, chain explosions, destructible bricks, power-up bonuses, player elimination |
| **Crypt of the NecroDancer** | **Every action must be taken in sync with the music beat** — moving, placing a bomb, or using an item outside the beat window is rejected or penalised |

The primary goal is to **beat other players in real-time multiplayer** by leveraging both strategic bomb placement and rhythm precision. A player who masters the beat gains combo bonuses; a player who misses beats loses momentum.

## 2. Core Gameplay Mechanics

### 2.1 Rhythm Synchronization (Critical Rule)
* **All player actions** (movement, bomb drop, item use) are **gated by the active BPM beat**.
* An action submitted outside the beat window is rejected: the server validates the beat index of each incoming `Action` message.
* Maintaining consecutive on-beat actions builds a **combo multiplier** (tracked per player in `Player.combo`).
* Missing a beat resets the combo. The `BeatAccuracy` enum (`Perfect`, `Good`, `Miss`, `Waiting`) reflects the precision of each action.
* Beat timing is authoritative on the server; the client displays a rhythm indicator to guide the player.

### 2.2 Bomberman-Style Arena
* The grid is a rectangular map with:
  - **Walls** (impassable, indestructible — border + even-coordinate pillars)
  - **Bricks** (destructible obstacles that can hide power-up bonuses)
  - **Empty cells** (walkable)
* Players place **bombs** that explode in cross-shaped blasts after a fuse delay.
* Explosion range and maximum simultaneous bombs are upgradeable via collected **bonuses**.
* A player hit by an explosion loses a life (`Player.lives`). When all lives are spent, the player is eliminated (or becomes a spectator, depending on room settings).
* **Sudden death** mode progressively shrinks the arena to force confrontation.

### 2.3 Game Modes
| Mode | Win Condition |
| :--- | :--- |
| **Deathmatch** | Last player (or team) with lives remaining wins |
| **Score** | First player to reach `target_score` wins |
| Both modes support an optional **time limit** (`time_limit_mins`). |

### 2.4 Power-up Bonuses
* Bonuses spawn on the grid at a configurable interval (`bonus_every` beats).
* Collected bonuses are tracked in `Player.collected_bonuses`.
* Available bonus types increase bomb range, bomb count, movement speed, or grant special items.
* The `second_item` slot allows a player to hold one special item (e.g., Shield).

### 2.5 Shield
* A shield (`shield_until_beat`) protects the player from the next explosion within that beat window.

### 2.6 Emotes
* Players can trigger temporary gestures mapped to keys `1` to `4`:
  - `1` → `👋` (Wave)
  - `2` → `✌️` (Peace)
  - `3` → `🖕` (Middle Finger)
  - `4` → `👍` (Thumbs Up)
* Emotes override the player's skin on the grid map for 1.5 seconds (`emote_until` field in the player state).
* Emotes bypass rhythm combo lockouts and spam delays, rendering immediately on trigger.

## 3. Multiplayer Architecture (Connection Modes)

Bombeast is **multiplayer-first**. Solo play (offline bots) exists only as a development aid (`is_local_dev_bots`). All real gameplay goes through the WebSocket server.

### Mode A — Online Server (Hosted)
* The client connects to a **remote, publicly hosted server** over WebSocket.
* Rooms can be marked **public** (visible in the room browser) or **private** (join by code).
* Room discovery uses the `GetRooms` / `LobbyUpdate` message flow.

### Mode B — Local Server (Player-Hosted)
* Any player can spawn a **local server process** directly from the client (`server_process: Option<Child>` in `App`).
* The client starts the server binary as a child process, then connects to it via WebSocket on `localhost`.
* Rooms created this way can be **LAN-visible** (`is_lan: true`) so players on the same network can join.
* When the hosting player disconnects or the client exits, the child server process is terminated (`App::stop_local_server` called on `Drop`).

### Room Lifecycle
1. Host creates a room (`CreateRoom`) → server assigns a room code.
2. Other players join (`JoinRoom`) → all clients receive `LobbyUpdate`.
3. Players toggle ready (`ToggleReady`); host starts the game (`StartGame`).
4. Server broadcasts `GameStarted { initial_state }` → clients enter `InGame`.
5. Each action is sent as `ClientMessage::Action(action)` and validated beat-server-side.
6. Server broadcasts authoritative `GameState` snapshots every tick.
7. Game ends → server sends `GameStopped { victory_state }` → clients return to lobby.

### Host Privileges
* **Transfer host** to another player (`TransferHost`).
* **Kick / Ban** a player (`KickPlayer`, `BanPlayer`).
* **Update room settings** before game start (`UpdateSettings`).
* **Stop a running game** (`StopGame`).

## 4. ASCII Fallback Rendering

To maintain compatibility on terminals lacking UTF-8 emoji support, the game allows toggling `ascii_mode` in settings. The rendering fallback mappings are:

| Element | Emoji / Unicode | ASCII Fallback | Description |
| :--- | :--- | :--- | :--- |
| **Grid Wall** | `██` | `##` | Impassable grid boundaries |
| **Brick Wall** | `░░` | `[]` | Destructible obstacles |
| **Bomb** | Animated | `()` | Active bomb ticking |
| **Fire Explosion** | `💥` / `🔥` | `##` / `**` / `::` | Visual blast states |
| **Alive Player** | Emoji (dynamic) | Two-letter tag | Tags: `RO`, `CA`, `FR`, `FO`, `PE` |
| **Dead Player** | `💀` | `XX` | Defeated player state |
| **Emote 1 (Wave)** | `👋` | `HI` | Avoids consecutive double letters |
| **Emote 2 (Peace)** | `✌️` | `VI` | Avoids consecutive double letters |
| **Emote 3 (Finger)** | `🖕` | `FU` | Avoids consecutive double letters |
| **Emote 4 (Thumbs)** | `👍` | `OK` | Avoids consecutive double letters |
