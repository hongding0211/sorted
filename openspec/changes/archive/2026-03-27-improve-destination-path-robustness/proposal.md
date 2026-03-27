## Why

The current destination path flow is brittle: saving settings rejects paths that do not already exist, and paths entered with `~` are treated literally instead of resolving to the user's home directory. This makes common archive destinations such as `~/Desktop/temp` fail unexpectedly and creates friction in a workflow that should be forgiving.

## What Changes

- Expand user-entered destination roots like `~/Desktop/temp` into an absolute path before validation, preview, persistence, and copy execution.
- Allow the app to create a missing destination root when the configured parent path is valid, instead of requiring users to pre-create the folder manually.
- Tighten destination validation and error handling so invalid, non-creatable, or non-writable paths fail with clearer guidance while successful paths behave consistently across settings, preview, and copy flows.
- Add regression coverage for home-directory shorthand, missing destination roots, and failure cases around invalid parent paths or non-directory destinations.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `themed-archive-import`: destination roots must support home-directory shorthand, creatable missing directories, and clearer validation before import.
- `archive-settings-ui`: destination settings validation and preview must use the same resolved path semantics that imports use.

## Impact

- Affected code: `src/core/config.rs`, `src/core/archive.rs`, `src/core/copy.rs`, `src/ui/app.rs`, and relevant tests.
- Affected behavior: persisted destination settings, confirmation preview, import validation, and destination directory creation.
- Dependencies: no new external services are required; implementation may reuse existing standard-library and current crate dependencies for path resolution and filesystem checks.
