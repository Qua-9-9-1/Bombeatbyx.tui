# Development Objectives & Workflows

This document outlines the standard development workflows and objectives for contributing to the Bombeatbyx project.

## 1. Crate Synchronization
Whenever structs or logic inside the `common` crate are modified, ensure that both the `client` and `server` crates are checked to prevent API mismatch.

## 2. Compilation and Code Validation
* Run a full workspace compilation check prior to committing or testing code:
  ```powershell
  cargo check --workspace
  ```
* All crates must compile without warnings. Avoid unused imports, dead code, or unhandled `Result`/`Option` returns.

## 3. Code Organization and Modularization
* **Preserve Code Readability:** Helper tasks must not be grouped back into `app.rs`. Leave `game.rs` and the input event loops separate and modular.
* **Single Responsibility:** Align file contents strictly with their naming context (e.g. network events do not belong in application wiring files).
