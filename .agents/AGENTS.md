# Workspace Guidelines & Context for AI Agents (Bombeast.tui)

Welcome to the **Bombeast.tui** codebase. This file contains rules and technical details for future AI coding agents working on this workspace.

---

## 1. Project Overview
Bombeast TUI is a rhythm-based multiplayer grid-arena battle game written in Rust using `ratatui` and `crossterm`. Players move, drop bombs, and trigger actions strictly synchronized with a music rhythm/BPM beat.

The project is structured into three crates:
1. `common`: Core game simulation logic, layouts, physics, and state machines shared between client and server.
2. `client`: Terminal-based graphical user interface drawing panels, menus, and managing user settings/input loops.
3. `server`: Server framework managing game lobby states and broadcasting updates to game clients.

---

## 2. Architecture & Design Principles

### A. Shared Logic vs. Render Engine
- **Spawning and Grid Physics** MUST reside in `common`. The client should only handle inputs and display rendering.
- For example, player spawn point optimization (furthest-apart dispersion subset selection) is performed by `GameState::spawn_players` in `common/src/game/state.rs`.
- Do not maintain duplicate/parallel lists of players (e.g. separate client-side lobby lists). Utilize the central `app.game_ctx.state.players` vector for both Lobby screens and active matches.

### B. Player & Entity State
- The `Player` struct (in `common/src/game/models.rs`) holds all properties.
- **Player Colors**: The `color` property is assigned by the server (or locally mocked during offline lobby setups) and is preserved inside `Player::color` during match launch. The client interprets this string to style user nicknames in both the Lobby screen and the gameplay screen.
- **Emotes**: Players can trigger temporary gestures (keys `1` to `4` mapped to `👋`, `✌️`, `🖕`, `👍`). Emotes override player skins on the map for 1.5 seconds (`emote_until` field) and are intercepted instantly to bypass rhythm combo lockouts and spam delays.

### C. ASCII Fallback Rendering
- The game must remain completely playable on all terminals (even those lacking UTF-8 emoji support).
- Toggle `ascii_mode` in Settings to run fallbacks.
- **Fallback Mappings**:
  - Grid wall: `██` -> `##`
  - Brick wall: `░░` -> `[]`
  - Bomb: Animated -> `()`
  - Fire explosion: `💥`/`🔥` -> `##`/`**`/`::`
  - Alive player: Emoji -> Two-letter tag (`RO`, `CA`, `FR`, `FO`, `PE`)
  - Dead player: `💀` -> `XX`
  - Emotes: Emojis (`👋`, `✌️`, `🖕`, `👍`) -> Distinct two-letter tags avoiding double consecutive letters (`HI`, `VI`, `FU`, `OK`)

---

## 3. Development Workflow Checklist

1. **Keep Crates Synchronized**: If you modify structures in `common`, always verify both `client` and `server` compile.
2. **Run Compilation Check**: Propose or run:
   ```powershell
   cargo check --workspace
   ```
3. **Preserve Code Readability**: Do not group unrelated helper tasks back into `app.rs`. Leave `game.rs` and inputs processing modularized.
