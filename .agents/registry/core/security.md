# Global Security & Secrets Guidelines

These rules dictate how the AI must handle authentication, authorization, and sensitive data across all languages and frameworks. The absolute priority is "Zero Trust" and preventing data leaks.

## 1. Secrets Management (Zero Hardcoding)
* **The Absolute Ban:** NEVER hardcode API keys, database connection strings, JWT secrets, passwords, or any sensitive tokens directly in the source code.
* **Environment Variables:** All secrets must be loaded dynamically from environment variables at runtime. 
* **Mock Secrets for Examples:** If generating boilerplate code or test files that require a key, strictly use obvious dummy values (e.g., `YOUR_API_KEY_HERE` or `super-secret-test-key-do-not-use-in-prod`).

## 2. Input Handling & Injection Prevention
* **Never Trust User Input:** Treat all incoming data (HTTP bodies, query parameters, headers, file uploads) as potentially malicious.
* **ORM / Parameterized Queries Only:** Never concatenate strings to build SQL queries. Always use the framework's ORM or strictly parameterized queries to prevent SQL Injection.
* **Output Encoding:** Before rendering any user-generated content in a web interface (HTML, React, Vue), ensure it is properly encoded or sanitized to prevent Cross-Site Scripting (XSS).

## 3. Cryptography & Passwords
* **Modern Algorithms:** Never use deprecated algorithms like MD5 or SHA-1 for hashing. 
* **Password Hashing:** Always use Argon2, bcrypt, or scrypt for hashing user passwords before database storage. Never store plain-text passwords.