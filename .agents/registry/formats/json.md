# JSON (JavaScript Object Notation) : Best Practices & Architecture

## 1. Schema Validation
- **Never Trust Raw Data:** Do not blindly parse and consume JSON payloads from external sources.
- **Rule:** Always validate incoming JSON against a strict schema (e.g., JSON Schema, Zod in TS, or Serde in Rust) before processing it. Reject the payload at the boundary if it lacks required fields or contains unexpected types.

## 2. Security & Constraints
- **Payload Limits:** Always enforce maximum payload sizes on your web server to prevent DOS attacks via massive JSON files.
- **Nesting Limits:** Limit the maximum depth of JSON parsing. Deeply nested JSON (e.g., `{"a":{"b":{"c":...}}}`) can cause Stack Overflow exceptions and crash the server during recursive parsing.

## 3. Naming Conventions & Types
- **Key Casing:** Choose a single naming convention for keys across your entire system and enforce it (typically `camelCase` for JSON, or `snake_case` if mimicking database columns directly). Never mix them.
- **Date Standardization:** JSON does not have a native Date type. Always serialize dates and times as **ISO 8601** formatted strings (e.g., `2023-10-25T14:30:00Z`). Do not use Unix timestamps unless strictly required for performance, as they lack timezone context.
- **Numbers:** JSON does not distinguish between floats and integers. For financial data or massive identifiers (like Snowflake IDs or large integers), serialize them as Strings to prevent precision loss in parsers like JavaScript's `Number`.

## 4. Error Handling
- **Parsing Boundaries:** JSON parsing is inherently unsafe. Always wrap native parsing calls (like `JSON.parse()`) in a `try/catch` block, or use safe unwrap methods, to prevent malformed data from crashing the application thread.