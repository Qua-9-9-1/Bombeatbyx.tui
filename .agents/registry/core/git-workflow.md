# Git Workflow & Commit Guidelines

These rules dictate how Git history is managed and how commit messages must be structured. The absolute priority is that the human developer retains full control over the execution of Git commands.

## 1. The Execution Boundary (Strict Rule)
* **Never Execute Commits:** The AI is strictly forbidden from executing `git commit`, `git push`, or any command that permanently alters the remote or local history without explicit instruction. 
* **Role as Drafter:** When asked to create a commit, the AI must only output the `git commit -m "..."` command as a text block for the user to copy, paste, and execute themselves.

## 2. Custom Commit Taxonomy
The project uses a strict, custom tag-based system instead of standard Conventional Commits. Every commit message must begin with one of the following exact uppercase tags enclosed in brackets:
* `[ADD]` - For new features, files, or capabilities.
* `[EDIT]` - For modifying existing code or behavior.
* `[FIX]` - For repairing bugs or errors.
* `[DEL]` - For removing code, files, or deprecated features.
* `[REFACTOR]` - For restructuring code without changing its external behavior.
* `[DOCS]` - For updates to documentation or comments.

## 3. Mandatory Message Structure
A commit message consists of a tagged title and a mandatory descriptive body.
* **The Body is Mandatory:** Never generate a single-line commit message. The body must explain the *why* and the *how* of the changes.
* **Multiple Tags:** If a single commit legitimately covers multiple distinct types of changes, each tag must be placed on a new line in the commit message title area.

**Example of Single Tag:**
```text
[ADD] User authentication module
```

Implemented JWT-based login and registration to secure the API endpoints.

Example of Multiple Tags:

```text
[FIX] Null pointer in user profile
[EDIT] Update profile schema validation
```

Resolved the crash occurring when a user had no avatar. 
Modified the schema to make the avatar URL explicitly optional.

## 4. Atomicity of Changes
* **One Idea Per Commit:** Do not bundle unrelated changes into a single commit (e.g., do not format the entire codebase and add a new feature at the same time). If the AI suggests changes, it must logically separate them into distinct, atomic commit proposals.