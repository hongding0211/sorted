## Why

The archive flow still has two UX gaps around import readiness. The app can currently enter the confirmation screen even when no theme has been entered, and it also allows starting an import even when the final archive destination folder already exists. We need deterministic behavior so users never accidentally merge a new import into an existing archive folder, and confirmation is only reachable when the required import inputs are complete.

## What Changes

- Preserve normal file-copy behavior inside a newly created archive destination once an import is allowed to start.
- Block the import when the final archive destination folder already exists, and surface a clear user-facing error instead of entering confirmation or starting the copy.
- Prevent navigation into the confirmation screen when the theme field is empty, and keep focus/status feedback on the missing theme input in the main screen instead.
- Apply the conflict checks consistently in copy planning/execution and make confirmation-entry validation match the actual import prerequisites.
- Add regression coverage for rejecting pre-existing final destination folders and blocking confirmation when theme is missing.

## Capabilities

### New Capabilities
<!-- None. -->

### Modified Capabilities
- `themed-archive-import`: confirmation must require a non-empty theme before entry, and import planning must stop with an error when the final archive destination folder already exists.

## Impact

- Affected code: confirmation gating in `src/ui/app.rs`, copy planning/execution in `src/core/copy.rs`, and any UI status handling that surfaces validation or copy failures.
- Affected behavior: readiness checks before confirmation, plus preflight validation that prevents importing into an already existing final archive folder.
- Dependencies: no new external services are required; implementation should continue using the existing filesystem and TUI stack.
