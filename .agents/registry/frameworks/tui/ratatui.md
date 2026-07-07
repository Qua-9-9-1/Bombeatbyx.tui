# Ratatui & Crossterm Framework Guidelines

These rules dictate how terminal applications built with Ratatui and Crossterm must be structured to ensure correct rendering, performance, and clean separation of UI rendering from application state.

## 1. Clean Separation of UI Rendering
* **Isolate UI Layout from Logic:** Keep logic out of rendering calls. The code inside `Frame::render_widget` must strictly handle placement, styling, and text rendering. Never mutate global state or run network actions during a render tick.
* **Encapsulate Layout Customizations:** Define complex visual components (such as sidebars, status bars, or map displays) as modular helper functions or custom widgets.

## 2. Layouts and Constraints
* **Responsive Grid Layouts:** Utilize Ratatui's `Layout` and `Constraint` builder to organize screen elements dynamically. Do not hardcode coordinates or cell indexes.
* **Terminal Dimensions Safety:** Avoid assuming terminal sizes. Gracefully handle minimum sizing constraints by wrapping layouts in scrollable containers or truncating views cleanly if terminal dimensions are too small.

## 3. Terminal Safety & Raw Mode
* **Panic Hooks and Recovery:** Ensure raw mode is deactivated and the alternate screen is exited whenever a panic occurs. Register a global panic handler to restore terminal settings properly in case of a crash.
* **No Raw Standard Output Writes:** Never use `println!` or print directly to standard output while the alternate screen is active. Render all updates via Ratatui's frame buffering system.
