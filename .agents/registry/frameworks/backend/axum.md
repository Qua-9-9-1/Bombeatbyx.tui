# Axum Framework Guidelines

These rules dictate how Axum-based web applications and API servers should be designed to ensure maximum cleanliness, correctness, and reliability.

## 1. Route Handlers & Extractors
* **Extractor Order:** Axum requires extractors to be in a specific order. Always place route path/query extractors first, then state extractors, and finally the request body/payload extractors.
* **Avoid Handlers with Excessive Arguments:** Keep handler signatures concise. If you need more than 3-4 extractors, group them using custom structs or helper middleware.
* **Return Typed Responses:** Prefer returning explicit types that implement `IntoResponse` (such as `Json<T>`, `StatusCode`, or custom enum errors) rather than raw strings or untyped structures.

## 2. State Management & Thread Safety
* **Use Thread-Safe State:** Pass sharing states via Axum's `State` extractor using thread-safe types, usually wrapped in `Arc` (e.g., `State(state): State<Arc<AppState>>`).
* **Encapsulate State Operations:** Avoid raw, direct access/mutation to mutexes inside routes. Provide cohesive, synchronized methods on the state struct to run state transformations cleanly.

## 3. WebSocket Integration
* **Graceful Lifecycle Management:** Manage WebSocket connection channels safely. Clean up internal tasks when WebSocket loops terminate or when connection drops.
* **Channel Framing:** Use structured message framing (e.g. JSON strings or custom binary headers) over the WebSocket channel. Always wrap incoming messages in safe parser validation before processing.

## 4. Error Handling
* **Custom Error Types:** Map internal server errors to custom domain error enums that implement `IntoResponse`. This prevents leaking internal implementation details to clients while ensuring clear HTTP/transport error mapping.
* **Never Panic in Handlers:** Ensure all edge cases return `Result` rather than triggering a panic, as a thread panic can compromise server state predictability.
