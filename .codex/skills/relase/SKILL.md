---
name: relase
description: Archive and sync a completed OpenSpec change, then commit, push, and run cargo release. Use when the user wants to finish implementation work and publish a Rust release from this repo.
license: MIT
compatibility: Requires openspec CLI, git, and cargo-release.
metadata:
  author: OpenAI
  version: "1.0"
---

Finish and publish a release workflow for this repository.

The skill name is intentionally `relase` to match the user's preferred trigger name, but the Rust release command is `cargo release`.

**Use this skill when**
- The user wants a full release flow, not just one step.
- The repo has a completed OpenSpec change that should be archived as part of the release.
- The user wants Codex to handle archive, sync, commit, push, and `cargo release` in sequence.

**Default workflow**

1. **Inspect repo state first**

   Run:
   ```bash
   git status --short
   openspec list --json
   ```

   Summarize:
   - uncommitted changes
   - active OpenSpec changes
   - whether the repo is ready for archive/release work

2. **Archive the OpenSpec change**

   If there is one obvious active change, use it.
   If multiple active changes exist, ask the user which one to archive.

   Before archiving:
   - check `openspec instructions apply --change "<name>" --json`
   - confirm tasks are complete

   Then archive the change using the project's OpenSpec archive workflow.

3. **Sync specs if needed**

   If the archived or archiving change has delta specs under `openspec/changes/<name>/specs/`, sync them into `openspec/specs/` before or during archive handling.

   Verify the resulting spec tree is in the expected post-archive state.

4. **Review git diff before publish**

   Run:
   ```bash
   git status --short
   git diff --stat
   ```

   Summarize the release scope.
   If the worktree contains unrelated changes, pause and ask before including them in the release commit.

5. **Commit intentionally**

   Stage only the files relevant to the release.
   Use a clear non-amended commit message.

   Good default pattern:
   ```text
   chore: prepare release
   ```

   If the repo changes suggest a more specific message, prefer that.

6. **Push the branch**

   Push the current branch to `origin`.
   Report the branch name and push result.

7. **Run `cargo release`**

   Before running it:
   - inspect `Cargo.toml` and current version if relevant
   - explain what level of release is about to happen if the command requires a bump choice

   Use the repo's existing release conventions.
   If `cargo release` needs a version level or extra flags and the correct choice is not obvious from repo context, pause and ask the user briefly.

8. **Show a concise release summary**

   Include:
   - archived change name and archive path
   - whether specs were synced
   - commit hash/message
   - pushed branch
   - `cargo release` result

**Guardrails**
- Do not archive an OpenSpec change with incomplete tasks unless the user explicitly wants that.
- Do not include unrelated dirty files in the release commit without confirming.
- Do not amend commits unless the user asks.
- Prefer the existing OpenSpec skills/workflows for archive behavior instead of re-inventing archive logic.
- Prefer non-interactive git commands.
- If `cargo release` would make irreversible remote changes and the exact release target is unclear, stop and ask one focused question.
