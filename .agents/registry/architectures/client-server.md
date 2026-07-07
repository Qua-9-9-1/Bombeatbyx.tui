# Client-Server Architecture Guidelines

These rules outline best practices for structuring client-server application models, establishing clear communication protocols, and maintaining decoupled responsibilities.

## 1. Single Source of Truth
* **Authoritative Server:** The server maintains complete authority over the game state, physics, and validation. The client must never locally update global state in anticipation of actions; it must send an action request, wait for the server broadcast, and apply the response uniformly.
* **Deterministic Simulation:** All logic running on both the client and the server (shared simulation libraries) must be entirely deterministic. Do not introduce unsynchronized random seeds, OS-specific times, or hardware differences to core calculations.

## 2. Decoupled Interface & Networking
* **Strict Crate Boundaries:** Decouple networking protocols and user interfaces from the core logic structures. Ensure visual layouts and rendering code reside entirely within the client crate.
* **Abstract Messaging:** Communicate using structured, serialized message definitions. Avoid tight coupling between networking channels and domain entities.
* **Heartbeats and Health checks:** Implement ping-pong signals or heartbeats to detect connection drops early and manage state cleanup immediately.
