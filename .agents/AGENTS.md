# Agent Guidelines Index (Bombeatbyx.tui)

Welcome to the Bombeatbyx TUI workspace guidelines index. This file acts as the main entry point router mapping all project-specific context and reusable technology rules.

## 1. Project Context
* [Business Context](./context/business-context.md) - Gameplay rules, rhythm synchronization, emotes, and ASCII fallback rendering mappings.
* [Tech Stack](./context/tech-stack.md) - Crate workspace structure, dependencies, and state architecture.
* [Objectives](./context/objectives.md) - Compilation commands, file organization guidelines, and developer workflow checklists.

## 2. Technology Registry

### Languages
* [Rust Rules](./registry/languages/rust.md) - Toolchain requirements, error handling constraints, memory borrowing, and type safety guidelines.

### Frameworks & Libraries
* [Axum Rules](./registry/frameworks/backend/axum.md) - Route design, extractor order, WebSockets, state safety, and HTTP response handling.
* [Ratatui Rules](./registry/frameworks/tui/ratatui.md) - TUI rendering loop, responsive layouts, alternate screen restoration, and Crossterm terminal events.
* [Tokio Rules](./registry/libraries/system/tokio.md) - Async execution constraints, preventing runtime blockages, and channel communications.

### Formats & Tooling
* [JSON Format](./registry/formats/json.md) - Best practices for data serialization and API request/response formats.
* [TOML Format](./registry/formats/toml.md) - Structure and syntax formatting rules for Cargo configuration.
* [Rust Testing](./registry/tooling/testing/rust-testing.md) - Writing unit tests, integration tests, and running benchmarks.

### Infrastructure
* [CI/CD Rules](./registry/infrastructure/ci-cd.md) - Pipeline architecture, dependency caching, parallel jobs, and artifact immutability.

### Architectures
* [Client-Server Rules](./registry/architectures/client-server.md) - State synchronization, authoritative server guidelines, and protocol design.

### Core Guidelines
* [Global Architecture](./registry/core/architecture.md) - File sizes, modularity limits, naming conventions, and SOLID design patterns.
* [Global Documentation](./registry/core/docs.md) - Comment guidelines, README patterns, and documentation structures.
* [Global Git Workflow](./registry/core/git-workflow.md) - Branch naming, commit messages, and PR processes.
* [Global Security](./registry/core/security.md) - Secret management, inputs sanitization, and dependency scanning.
* [Global Testing](./registry/core/tests.md) - Test categories, assertions, and mock boundaries.