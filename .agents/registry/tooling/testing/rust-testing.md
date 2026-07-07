# Rust Native Testing (Cargo) : Best Practices & Architecture

## 1. Unit Tests Isolation (Conditional Compilation)
- **Location:** Place unit tests in the same file as the source code, at the bottom.
- **Encapsulation:** Always wrap unit tests inside a dedicated module decorated with `#[cfg(test)]`. This ensures the test code and its dependencies (like mock libraries) are completely omitted from the final production binary compilation.
- **Pattern:**

```rust
#[cfg(test)]
mod tests {
    use super::*; // Import private functions from the outer scope
    
    #[test]
    fn test_pure_function() { 
        // Test logic here
    }
}
```

## 2. Integration Tests Structure
- **Location:** Place integration tests outside the `src/` directory, in a dedicated root folder named `tests/`.
- **Public API Boundary:** Treat files in `tests/` as external crates. They can only test the public API of your library (`pub fn`). Do not attempt to test private internals here.

## 3. Error Handling in Tests
- **Avoid Panics:** Instead of using `.unwrap()` blindly inside tests (which causes uninformative panics), allow your test functions to return a `Result<(), Box<dyn std::error::Error>>`.
- **Clean Failure:** This allows using the `?` operator inside tests. If a function fails, Cargo reports the exact error variant neatly without tearing down the test harness layout.

## 4. Parallelism & Shared State
- **Thread Safety:** Cargo runs tests in parallel using multiple threads by default. 
- **Rule:** If tests modify global state, static variables, or environment variables (`std::env::set_var`), you must run the tests sequentially using `cargo test -- --test-threads=1` or guard the state with a shared `std::sync::Mutex`.