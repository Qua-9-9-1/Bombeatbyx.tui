# Clean Architecture (Hexagonal) Guidelines

This project strictly follows the Clean Architecture / Ports and Adapters paradigm. The absolute priority is the Dependency Rule: source code dependencies must point ONLY inward, toward the Domain layer.

## 1. Directory Structure (The Layers)
The project MUST be divided into the following strict layers (either as separate folders or distinct sub-projects/crates):
* `Domain/`: The core business logic, entities, and domain interfaces.
* `Application/`: Use cases, orchestrators, and application interfaces (Ports).
* `Infrastructure/`: Database access, external API clients, file system adapters.
* `Presentation/` (or `UI/API/`): Controllers, Routes, Web framework integrations.

## 2. The Dependency Rule (Strict Prohibition)
* **Inward Flow Only:** `Domain` knows nothing. `Application` knows only `Domain`. `Infrastructure` and `Presentation` know `Application` and `Domain`.
* **The Absolute Ban:** NEVER import a framework, a database ORM (e.g., EF Core, Prisma), or an external library into the `Domain` or `Application` layers. These layers must remain pure language code.

## 3. Ports and Adapters
* **Dependency Inversion:** If the `Application` layer needs to fetch data from a database, it must define an Interface/Trait (the Port, e.g., `IUserRepository`). The `Infrastructure` layer implements this interface (the Adapter).
* **Execution:** At runtime, the Dependency Injection container injects the concrete Adapter into the Application layer. The Application layer never instantiates the Adapter directly.

## 4. DTOs vs. Domain Entities
* **No Entity Leaks:** Domain Entities (e.g., a `User` class with business rules) must NEVER be directly returned to the HTTP response or saved directly via an ORM if it compromises their purity.
* **Mapping:** Use Data Transfer Objects (DTOs) at the boundaries. The `Presentation` layer maps HTTP Requests to Application Commands. The `Infrastructure` layer maps Database Models to Domain Entities.