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

All tuning values live in `common/src/game/rules.rs` (re-exported by `models`). Never hardcode them elsewhere.

### 2.1 Rhythm Synchronization (Critical Rule)
* **All player actions** (movement, bomb drop, item use) are **gated by the active BPM beat**: one action per beat (`last_acted_beat`), plus a 100 ms anti-spam lockout.
* `BeatAccuracy` = `Perfect` / `Great` / `Ok` / `Miss` / `Waiting`, evaluated from the distance to the nearest beat. A `Miss` consumes the beat, does nothing and resets the combo.
* **Combo** (`Player.combo`): +1 for each `Perfect`/`Great` action (`Ok` keeps it). An on-beat action that fails (bumping a wall) still keeps the groove but consumes the beat.
* **Idle reset (NecroDancer rule):** letting a beat pass without acting resets the combo (`GameState::reset_idle_combos`, called when the beat window closes).
* **Multiplier:** rhythm points are multiplied by `1 + combo / COMBO_STEP` (max `MAX_COMBO_MULTIPLIER`).
* **Fever:** from `FEVER_COMBO`, newly placed bombs get `+FEVER_BONUS_RANGE` range.
* Beat timing is authoritative on the server; `GameState.current_beat` is broadcast so clients render beat-based effects (shields) correctly.

### 2.2 Bomberman-Style Arena
* Walls (border + even/even pillars), **bricks** (generated at `BRICK_DENSITY_PERCENT`, never within `SPAWN_SAFE_RADIUS` of a spawn) and empty cells.
* Bombs explode after `BOMB_FUSE_BEATS` beats. Range and pierce are **locked at placement** (stored in `Cell::Bomb`).
* Blasts stop at walls, at bricks (unless pierce) and at other bombs, which **chain-explode in the same tick**.
* A destroyed brick may hide a bonus (`BRICK_BONUS_DROP_PERCENT`), revealed when the fire on its cell dies out (`GameState.pending_drops`).
* Fire is lethal on contact: deaths are checked **every tick**, not only on beats.
* Each `Cell::Explosion` stores its `owner_id` for kill credit.
* Death: lose a life, lose all power-ups and the combo, respawn after `RESPAWN_DELAY_SECS` with `RESPAWN_PROTECTION_BEATS` of invulnerability.
* **Sudden death** (`state/sudden_death.rs`): after 2 min (or half the time limit), one cell per beat turns into a wall, spiralling inward and keeping a 3x3 center. Crushed players lose a life.

### 2.3 Game Modes & Scoring
| Mode | Win Condition |
| :--- | :--- |
| **Deathmatch** | Last player with lives remaining wins |
| **Score** | First player to reach `target_score` wins |
| Both modes support an optional **time limit** (`time_limit_mins`). |

Match setup and end detection are centralized in `common` (`GameContext::new_match`, `GameState::update_match_end`). Client and server must not reimplement them.

| Event | Points |
| :--- | :--- |
| Kill another player | `+KILL_POINTS` (and `Player.kills += 1`) |
| Self-kill | `-SELF_KILL_PENALTY` |
| Brick destroyed | `+BRICK_POINTS` |
| Bonus picked up | `+BONUS_PICKUP_POINTS` |
| Successful on-beat action | `accuracy points x combo multiplier` |

### 2.4 Power-up Bonuses
Bonuses spawn every `bonus_every` beats and from bricks; weighted rarity in `BonusType::roll`.

| Bonus | Effect |
| :--- | :--- |
| 💣 BombQty | +1 simultaneous bomb |
| 🔥 BombRange | +1 blast range |
| 🔪 Pierce | Blasts go through bricks |
| 💖 ExtraLife (rare) | +1 life, up to `MAX_LIVES` (never lowers a higher count) |
| 🛡️ Shield (item) | Invulnerable for `SHIELD_DURATION_BEATS` beats |
| 🧨 Detonator (item) | Instantly explodes all own bombs (kept if none placed) |
| 👟 Dash (item) | Moves up to `DASH_DISTANCE` cells in the facing direction |

Items go in the single `second_item` slot and are used with the spell key (`E`). Picking a new item replaces the current one.

### 2.5 AI Bots & Solo Mode
* **Solo vs Bots** is a real game mode (main menu), configured in the lobby with `RoomSettings.bot_count` (1-7) and `bot_difficulty` (Easy / Normal / Hard).
* The AI lives in `common/src/game/ai/` and is pure: `decide_bot_action(state, bot_id, difficulty, rng)` returns one action per beat.
  - `danger.rs`: beats-until-fire per cell, including chain reactions and hypothetical bombs.
  - `pathfinding.rs`: BFS returning the first move towards a goal.
  - `goals.rs`: escape, bomb placement (only with a verified escape route), detonator use, then bonus > enemy > brick targets.
* Difficulty profiles (`ai::profile`) tune idle/panic chances, aggression, item use and rhythm accuracy.
* `ai/simulation_tests.rs` runs deterministic headless bot matches; keep them green when tuning the AI.

### 2.6 Emotes
* Players can trigger temporary gestures mapped to keys `F1` to `F4`:
  - `1` → `👋` (Wave)
  - `2` → `✌️` (Peace)
  - `3` → `🖕` (Middle Finger)
  - `4` → `👍` (Thumbs Up)
* Emotes override the player's skin on the grid map for 1.5 seconds. `emote_until` is not serialized: the authoritative side clears `active_emote` when it expires (`GameState::tick_emotes`).
* Emotes bypass rhythm combo lockouts and spam delays, rendering immediately on trigger.

## 3. Multiplayer Architecture (Connection Modes)

Bombeast is **multiplayer-first**, with an offline **Solo vs Bots** mode (`App.is_solo_mode`) where the client runs the full simulation and drives the AI bots itself. While a solo game is paused, `GameContext::shift_clock` freezes every wall-clock deadline.

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
