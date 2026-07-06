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

## 4. Coding Standards & Code Style Rules

AI agents working on this codebase MUST strictly adhere to the following code quality and structural guidelines:

### A. Code Style & Readability
- **No Comments**: Do not write comments inside the code. The code must be self-explanatory through clean naming conventions.
- **Indentation Limit**: Maximum of 4 levels of nested indentations. If a logic path requires a 5th level, isolate and extract that chunk into a dedicated helper function.

### B. Function Design & Sizing
- **Length Constraint**: Functions must average between 20 and 40 lines of code. 
- **Oversizing Rule**: You may exceed 40 lines only if absolutely necessary for continuous logic. Otherwise, break down sequential steps into explicit sub-functions.

### C. Clean Architecture & File Organization
- **File Size & Directories**: If a single directory or logical module exceeds 2 core source files, group them into a dedicated sub-folder representing that specific module or component.
- **Strict File Segmentation**: Functions placed inside a file must strictly align with the filename's single responsibility.
  - *Example*: Network event handling has absolutely nothing to do in an `app.rs` file.
  - *Example*: Core game simulation logic must never be mixed with user interface or application wiring files.
- **Cohesive Grouping**: Group functions in separate files based on their domain context, ensuring clean separation of concerns across the workspace.

### D. Rust Specifics & Async (Tokio)
- **Do Not Block the Async Runtime**: Never use blocking operations (`std::thread::sleep`, synchronous file I/O, or heavy CPU-bound loops) inside Tokio async tasks. Use `tokio::time::sleep` or `tokio::task::spawn_blocking` if unavoidable.
- **Strict Compile-Time Warnings**: All crates must compile without warnings. Do not leave unused imports, dead code, or unhandled `Result` values (`unwrap()` is strictly forbidden unless paired with a proper `.expect("context")`).

### E. Network & State Synchronization (Client/Server)
- **Single Source of Truth**: The Server is the ultimate authority. The Client must never update the global game state directly based on local inputs; it must send an event, wait for the server's broadcast, and apply the state update uniformly.
- **Deterministic Simulation**: All logic inside the `common` crate must be deterministic. Avoid using `SystemTime::now()` or random number generators (`rand`) directly inside core physics/grid simulation unless the seed is strictly synchronized by the server.

### F. Terminal UI (Ratatui & Crossterm) Safety
- **No Direct Terminal State Corruption**: Never raw-print (`println!`) to `stdout` while the terminal alternate screen is active. Every visual update must transit through Ratatui's `Frame::render_widget`.
- **Cross-Platform Compatibility**: Avoid platform-specific system calls (like Windows-only or Linux-only utilities). Use abstraction layers provided by `crossterm` and Rust's standard library to ensure the client binary runs seamlessly across Linux, macOS, and Windows.