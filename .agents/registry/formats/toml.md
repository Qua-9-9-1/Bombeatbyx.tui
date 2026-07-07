# TOML (Tom's Obvious, Minimal Language) : Best Practices & Architecture

## 1. Type Strictness
- **Design Philosophy:** Unlike YAML, TOML is strongly typed. An array cannot contain mixed data types.
- **Rule:** Ensure consistency in your arrays. If a configuration list requires mixed types, you must redesign your data structure to use arrays of tables (`[[table]]`) or distinct keys.

## 2. Hierarchical Grouping
- **Table Usage:** Use tables (`[section]`) to group related configuration keys logically. Do not use deep inline objects (`{ key = value }`) as they defeat the purpose of TOML's flat readability.
- **Sorting:** Alphabetize keys within their respective tables to make visual scanning easier as the configuration file grows.

## 3. String Definitions
- **Literal Strings:** Use single quotes (`'C:\Users\path'`) for literal strings where escape sequences (like `\`) should be ignored. This is mandatory for regex patterns or Windows file paths.
- **Multi-line Strings:** Use triple quotes (`"""`) for long descriptions or embedded scripts to maintain natural line breaks.