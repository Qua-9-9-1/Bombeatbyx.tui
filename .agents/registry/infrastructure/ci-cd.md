# CI/CD & Pipeline Architecture Guidelines

These rules dictate how continuous integration and deployment pipelines (e.g., GitHub Actions, GitLab CI) must be written. The absolute priority is pipeline speed, deterministic execution, and strict secret boundary management.

## 1. Caching & Execution Speed
* **Mandatory Dependency Caching:** Every pipeline that installs dependencies (npm, pip, cargo, nuget) MUST implement caching mechanisms for those package managers. Redownloading the entire internet on every commit is strictly forbidden.
* **Parallelization:** Independent jobs (e.g., Linting, Unit Tests, Security Scans) must run in parallel, never sequentially. Only deployment or build jobs that depend on test success should be sequential.

## 2. Security & Triggers
* **No Secrets in PRs:** Workflows triggered by `pull_request` (especially from external forks) must NEVER have access to production secrets or deployment credentials.
* **Pin Actions to SHAs:** When using third-party actions/plugins (e.g., `uses: actions/checkout@v4`), prefer pinning them to a specific commit SHA rather than a floating version tag to prevent supply-chain attacks if the action repository is compromised.

## 3. Immutability of Artifacts
* **Build Once, Deploy Many:** The code must be compiled and packaged (e.g., into a Docker image or a zip file) exactly once during the CI phase. The CD phase must take this exact immutable artifact and promote it across environments (Staging -> Production). Never recompile the code during the deployment phase.