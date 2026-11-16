# Git Workflow & Commit Guidelines

These rules dictate how Git history is managed and how commit messages must be structured. The human developer retains full control over what reaches the remote repository.

## 1. The Execution Boundary (Strict Rule)
* **Commit only when asked:** The AI may run `git commit` only when the user explicitly asks it to commit (or asks it to "respect the commit rules" while delivering work). Otherwise it drafts the command for the user.
* **Never push:** The AI never runs `git push`, `git push --force`, `git rebase`, `git reset --hard` or any history-rewriting command unless the user explicitly requests that exact operation.

## 2. Commit Tags (Strict Taxonomy)
Every commit message must begin with exactly one of these uppercase tags enclosed in brackets:
* `[ADD]` - New features, files, or capabilities.
* `[EDIT]` - Modification of existing code, behavior, docs, or refactors.
* `[FIX]` - Repair of a bug or an illogical behavior.
* `[DEL]` - Removal of code, files, or deprecated features.

No other tag exists (`[REFACTOR]`, `[DOCS]`, `feat:`, `fix:` … are forbidden). Refactors and documentation use `[EDIT]`.

## 3. Message Format (Strict Rule)
* **English only**, whatever the language used in the conversation.
* **Single line, no body.** At most one short sentence; usually just a few words.
* Lowercase after the tag, no trailing period.
* Describe the result, not the process.

**Good:**
```text
[ADD] destructible bricks on map generation
[FIX] shield now protects for several beats
[EDIT] centralize match setup in common crate
[DEL] debug free shield for player 2
```

**Bad:**
```text
[ADD] Added bricks.                      <- past tense, trailing period
feat: bricks                             <- wrong tag format
[REFACTOR] split file                    <- tag does not exist
[ADD] ajout des briques                  <- not English
[ADD] bricks

Long body explaining why...              <- bodies are forbidden
```

## 4. Atomicity of Changes
* **One idea per commit:** Do not bundle unrelated changes. A gameplay rule, its UI display and its tests may share a commit; two independent rules may not.
* **Each commit must compile:** `cargo check --workspace` must pass at every commit so history stays bisectable.
