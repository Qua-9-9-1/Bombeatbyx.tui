# Global Architecture & Code Structure Guidelines

This document defines the absolute rules for code structure, architectural patterns, and naming conventions. These rules apply universally regardless of the programming language or application domain.

## 1. Modularity & Size Limits
To prevent monolithic design and maintain readability, enforce the following limits:
* **File Size:** Maximum 300 lines per file. If a file exceeds this limit, extract the logic into smaller, domain-specific modules.
* **Function Size:** Maximum 20-40 lines per function.
* **Indentation Limits:** Maximum 3 levels of nested logic (e.g., `if` inside a `for` inside a `function`). Extract deep logic into helper functions.

## 2. Naming Conventions
Names must convey intent clearly without requiring comments.
* **Directories & Files:** Use `kebab-case` for directories. For files, default to `kebab-case` unless the specific language standard strictly dictates otherwise.
* **Types, Classes, and Interfaces:** Use `PascalCase`.
* **Variables & Methods:** Use the language standard (`camelCase` or `snake_case`).
* **Booleans:** Must be prefixed with affirmative verbs like `is_`, `has_`, `can_`, or `should_`.
* **Avoid Abbreviations:** Use complete words. Single-letter variables are forbidden except for standard iterators (`i`, `j`, `x`, `y`) in short scopes.

## 3. Core Design Principles
* **Separation of Concerns (SoC):** Strictly separate Core Logic, User/System Interface, and State Management.
* **Single Responsibility Principle (SRP):** Every module, class, or function must have one, and only one, reason to change. 
* **No Magic Numbers or Strings:** Hardcoded values are strictly forbidden. Extract them into clearly named constant variables or Enums at the top of the file.
* **DRY (Don't Repeat Yourself):** If a block of logic appears in multiple places, extract it into a shared utility function.

## 4. Control Flow & Error Handling
* **Early Returns (Guard Clauses):** Avoid deep `if/else` branching. Handle errors and invalid states at the very beginning of the function and return immediately.
* **Fail Fast:** Do not swallow errors silently. Catch exceptions where they occur, log them properly, and propagate them if the system cannot safely recover.

## 5. Predictability & State Management
* **Prefer Immutability:** Variables and data structures should default to being immutable. Favor creating new data structures over mutating existing ones to prevent unpredictable state changes.
* **Explicit State Mutations:** If state mutation is strictly necessary, it must be encapsulated within dedicated, highly visible functions. Avoid global mutable state entirely.

## 6. Functional Purity & Side Effects
* **Maximize Pure Functions:** Write functions that, given the same inputs, always return the same output without affecting the outside system state.
* **Isolate Side Effects:** Any operation that interacts with the outside world (I/O, file system, network, terminal rendering) must be pushed to the boundaries of the application. The core algorithmic logic must remain independent of side effects.

## 7. Loose Coupling
* **Depend on Abstractions:** Modules should interact through clear interfaces or parameter passing, not by directly hardcoding dependencies on other concrete modules. This ensures code remains testable and interchangeable.