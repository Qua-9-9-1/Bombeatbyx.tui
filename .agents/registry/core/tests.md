# Global Testing Guidelines & Architecture

These rules dictate how automated tests must be written, regardless of the testing framework or language used. The absolute priority is to ensure tests are deterministic, behavior-driven, and resilient to refactoring.

## 1. The AAA Pattern (Strict Structure)
Every single test must strictly follow the Arrange-Act-Assert (or Given-When-Then) pattern. The AI must visually separate these three blocks using blank lines or comments.
* **Arrange:** Set up the exact state and mock dependencies required for the test.
* **Act:** Execute the single function or behavior being tested.
* **Assert:** Verify the outcome. 
* *Rule:* Never mix 'Act' and 'Assert' steps. Do not perform multiple distinct actions in a single test block.

## 2. Test Behavior, Not Implementation
* **Public Interfaces Only:** Tests must target the public API/methods of a module or class. Never write tests that specifically target private/internal methods. If an internal method is complex enough to warrant testing, it should be extracted into its own class or module.
* **Refactoring Resilience:** A test should fail if, and only if, the external behavior of the code changes. If refactoring internal logic causes a test to fail, the test was poorly written.

## 3. Strict Mocking Protocol
Over-mocking creates tests that pass even when the real system is broken.
* **Mock Boundaries, Not Domain:** ONLY mock external dependencies that cross architectural boundaries (e.g., HTTP clients, Database connections, File Systems, System Time).
* **Do Not Mock the Subject:** Never mock the actual system under test (SUT) or pure functions.
* **Prefer Fakes over Mocks:** When possible, prefer using lightweight in-memory implementations (Fakes) over strict interaction-based Mocks (e.g., an in-memory repository instead of a mocked `save()` method that expects to be called exactly once).

## 4. Determinism & Anti-Flakiness
Tests must produce the exact same result whether run locally, in CI/CD, in parallel, or ten years from now.
* **No Hardcoded Dates/Time:** Never use `Now()` or `CurrentTime()` inside the test logic or the system under test without controlling it. Always inject a mocked clock or pass specific timestamps.
* **No Randomness:** Never rely on randomly generated data (e.g., random UUIDs or random strings) for assertions unless the random seed is fixed. Use deterministic test data (e.g., `user-id-1`).

## 5. Test Naming Conventions
Test names must read like a specification. They must describe the scenario and the expected outcome.
* **Format:** Use a descriptive format like `MethodName_StateUnderTest_ExpectedBehavior` or `should_do_X_when_Y`.
* **Example (Bad):** `test_user_creation`
* **Example (Good):** `CreateUser_WhenEmailIsInvalid_ThrowsValidationError`

## 6. Coverage Constraints
* **Happy and Unhappy Paths:** For every feature tested, the AI must explicitly write tests for the "happy path" (successful execution) AND the expected "unhappy paths" (exceptions, bad inputs, unauthorized access).