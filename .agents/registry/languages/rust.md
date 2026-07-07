# Rust Language Guidelines

These rules dictate how Rust code must be written to ensure maximum memory safety, concurrency safety, and idiomatic correctness. The Rust compiler and its toolchain are the ultimate source of truth.

## 1. Toolchain & Quality Enforcement
* **Clippy is Absolute:** The code must compile with absolutely zero warnings from `cargo clippy`. Treat all Clippy warnings as hard errors.
* **Formatting:** All code must strictly adhere to the standard `cargo fmt` style. Do not invent custom formatting.

## 2. Error Handling (Strict Constraints)
Rust provides powerful algebraic data types for error handling. Bypassing them causes panics and undefined system states.
* **No Panics:** The use of `.unwrap()` and `.expect()` is strictly forbidden in production code. 
* **Explicit Propagation:** Always use the `?` operator to propagate errors up the call stack.
* **Use `Result` and `Option`:** Handle all edge cases using `match` or `if let` bindings. Never assume a value exists.

## 3. Ownership, Borrowing & Memory
* **Minimize Cloning:** The `.clone()` method is a performance penalty. Always prefer passing references (`&T` or `&mut T`) and managing lifetimes correctly. Only use `.clone()` if ownership is fundamentally required and restructuring the borrowing is impossible.
* **Immutable by Default:** Leverage Rust's default immutability. Only use `mut` when absolutely mathematically necessary.
* **Avoid `Rc` / `Arc` when possible:** Do not default to reference counting to solve lifetime issues. Restructure the code hierarchy to respect single ownership first.

## 4. Type System & Idiomatic Patterns
* **The Newtype Pattern:** Use tuple structs to create distinct types for values that share the same underlying representation (e.g., `struct UserId(u64);` instead of just passing `u64`). This prevents accidental parameter swapping.
* **Enums for State:** Represent all mutually exclusive states using Enums rather than relying on boolean flags or discrete integer values.
* **Derive Traits:** Liberally derive standard traits (`Debug`, `Clone`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`) on structs and enums to ensure interoperability with the ecosystem.

## 5. Concurrency & Async (Tokio)
* **Never Block the Async Runtime:** If using Tokio or another async runtime, never use synchronous I/O operations (like `std::fs` or `std::thread::sleep`). Always use their async equivalents.
* **Send + Sync:** Ensure data shared across thread boundaries explicitly implements the `Send` and `Sync` traits safely.