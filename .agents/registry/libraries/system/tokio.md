# Tokio Async Runtime Guidelines

These rules dictate how asynchronous applications built with Tokio must manage runtime scheduling, tasks, and data synchronization to prevent blocking and deadlock scenarios.

## 1. Preventing Reactor Starvation
* **Never Block Async Tasks:** Never execute CPU-bound loops, blocking filesystem operations, or synchronous sleeping (`std::thread::sleep`) inside async context handlers. These operations block executor threads and starve other tasks.
* **Isolate Blocking Work:** Wrap unavoidable blocking library calls or heavy computations in `tokio::task::spawn_blocking` to execute them on a dedicated pool, or yield execution using `tokio::task::yield_now()` periodically.
* **Use Async Alternatives:** Always use `tokio::time::sleep` for delays and Tokio's asynchronous IO and synchronization types instead of their standard library synchronous counterparts.

## 2. Structured Task Lifecycle & Spawning
* **Bounded Lifetime Control:** Track spawned tasks using handles. Avoid creating unmonitored background tasks (detached futures) that cannot be cancelled or monitored for failure.
* **Graceful Cancellation:** Implement clean cancellation triggers utilizing `select!` macro, `tokio::sync::watch` signals, or `CancellationToken` patterns to terminate tasks cleanly on application exit.

## 3. Safe Concurrency & Message Passing
* **Select Channel Types Wisely:** Use bounded channels (e.g. `tokio::sync::mpsc::channel`) to prevent unbounded memory growth. Choose the correct channel type for the domain:
  - `mpsc`: Multi-producer, single-consumer.
  - `oneshot`: Single-producer, single-consumer, one-off signal.
  - `broadcast`: Multi-producer, multi-consumer broadcast pattern.
  - `watch`: Single-producer, multi-consumer state observation.
* **Minimize Lock Contention:** Keep critical sections inside mutex guards (`tokio::sync::Mutex` or standard `Mutex`) as small as possible. Never hold a synchronous mutex guard across an `.await` boundary, as it will cause deadlocks or compile errors.
