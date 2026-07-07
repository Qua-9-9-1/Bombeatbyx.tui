# Global Documentation Guidelines

These rules dictate how code, architectures, and decisions must be documented. The absolute priority is to provide context ("Why") rather than paraphrasing the code ("What"), ensuring maximum maintainability.

## 1. The Core Philosophy: "Why" over "What"
* **No Paraphrasing:** Never write comments that simply translate the code into English. Code explains *how* it does something; comments must explain *why* it does it.
* **Business Context:** Use comments exclusively to explain non-obvious business rules, mathematical formulas, or bug workarounds that forced a specific technical choice.

## 2. Inline Comments vs. Docblocks
* **Docblocks for Public APIs:** Every exposed function, class, or interface must have a formal docblock (e.g., JSDoc, Rustdoc, Python Docstring) detailing parameters, return types, and potential exceptions.
* **Sparse Inline Comments:** Use inline comments strictly for complex algorithms or regex explanations. If a block of code requires a massive inline comment to be understood, the code must be refactored into a smaller, well-named function instead.

## 3. Language & Formatting
* **English Only:** All comments, documentation, and commit messages must be written in English to guarantee universal readability and avoid encoding issues.
* **Markdown Standard:** All external documentation files (`README.md`, architecture guides) must use strict Markdown formatting without arbitrary HTML tags.

## 4. Macro-Architecture & ADRs (Architecture Decision Records)
* **Documenting Decisions:** Any significant change to the macro-architecture (e.g., choosing a new framework, changing a database paradigm) must not be hidden in commit messages. It must be documented using the ADR format.
* **ADR Structure:** An Architecture Decision Record must clearly state:
    1. **Context:** The specific technical or business problem.
    2. **Decision:** The exact solution chosen.
    3. **Consequences:** The mathematical, architectural, or workflow impact (pros and cons) of this choice.