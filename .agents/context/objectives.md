# Development Objectives & Workflows

This document outlines the standard development workflows and objectives for contributing to the Bombeast project.

## 1. Crate Synchronization
Whenever structs or logic inside the `common` crate are modified, ensure that both the `client` and `server` crates are checked to prevent API mismatch.

## 2. Compilation and Code Validation
* Run a full workspace compilation check prior to committing or testing code:
  ```powershell
  cargo check --workspace
  ```
* All crates must compile **without warnings**. Avoid unused imports, dead code, or unhandled `Result`/`Option` returns.
* Run `cargo test --workspace` before committing. Gameplay changes must come with unit tests in `common`; AI changes must keep `ai/simulation_tests.rs` green.
* If no local Rust toolchain is available, use Docker (edition 2024 needs Rust >= 1.85):
  ```powershell
  docker run --rm -v "${PWD}:/app" -v bombeast-target:/app/target -w /app rust:1.88 cargo test --workspace
  ```

## 3. Code Organization and Modularization
* **Preserve Code Readability:** Helper tasks must not be grouped back into `app.rs`. Leave `game.rs` and the input event loops separate and modular.
* **Single Responsibility:** Align file contents strictly with their naming context (e.g. network events do not belong in application wiring files).

## 4. Security Constraints
* **No secret exposure:** Server URLs, room codes, and player tokens must never be logged in plain text in production paths.
* **Input sanitization:** All `ClientMessage` variants received by the server must be validated before mutating shared state. Never trust client-supplied data for authoritative game logic.
* **No privilege escalation:** Only the room host (`host_id`) may execute host-only actions (`StartGame`, `StopGame`, `UpdateSettings`, `TransferHost`, `KickPlayer`, `BanPlayer`). The server must verify host identity on every such request.
* **Dependency hygiene:** Keep third-party dependencies minimal and up to date. Avoid pulling crates with broad permission scopes for narrow tasks.

## 5. Game Rule Constraints
* **Beat validation is server-authoritative:** Never accept a `ClientMessage::Action` without verifying the beat index against the server-side rhythm clock. Clients may predict locally but the server result is canonical.
* **State broadcast integrity:** After any game tick that mutates `GameState`, the server must broadcast the updated state to **all peers** in the room without omission.
* **Room isolation:** Operations on one room's `GameContext` or peer list must never bleed into another room's state. Use the room code as the sole key.
* **Graceful disconnect:** When a peer disconnects mid-game, the server must reassign host if needed and terminate the game if fewer than two active (non-spectator) players remain, then broadcast `GameStopped`.
* **Spectator separation:** Spectators (`is_spectator: true`) must not receive `GameStarted` as active players and must not be able to submit `Action` messages that affect the game.

## 6. File Organization Checklist (Before Committing)
- [ ] `cargo check --workspace` passes with zero warnings and `cargo test --workspace` is green.
- [ ] Gameplay tuning values added to `common/src/game/rules.rs`, not hardcoded.
- [ ] `business-context.md` updated when a game rule changes.
- [ ] Commit messages follow `registry/core/git-workflow.md` (English, one line, `[ADD]`/`[EDIT]`/`[FIX]`/`[DEL]`).
- [ ] Modified `common` structs reflected in both `client` and `server` usage sites.
- [ ] No game logic added directly in `app.rs` — belongs in `game.rs` or a dedicated module.
- [ ] No network handling added in UI/render files.
- [ ] All new server message handlers placed in `server/src/websockets/rooms/handlers/`.
- [ ] Security-sensitive paths (host-only actions, beat validation) include explicit guard checks.
