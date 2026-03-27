## Context

Sorted currently stores `destination_root` as a `PathBuf` and validates it by requiring the path to already exist and be a directory. That behavior is reused in settings save, destination preview, and import planning, so the same limitation appears throughout the app. This is causing a poor user experience for common paths such as `~/Desktop/temp`, because the TUI captures the literal `~` string and Rust `PathBuf` does not expand it automatically.

The requested change cuts across configuration, archive planning, preview rendering, and copy execution. We need one shared interpretation of destination paths so the path shown in settings and confirmation is the same path the importer will create and copy into.

## Goals / Non-Goals

**Goals:**
- Resolve home-directory shorthand like `~/Desktop/temp` into an absolute filesystem path before validation or use.
- Allow a missing destination root to be created automatically when its nearest existing ancestor is a writable directory.
- Keep path behavior consistent across settings save, preview generation, import planning, and copy execution.
- Surface clearer errors for paths that cannot be created, point to a file, or resolve outside valid filesystem expectations.
- Add tests that lock in behavior for success and failure cases.

**Non-Goals:**
- Adding shell-like expansion beyond leading `~` for the current user.
- Building a file picker or changing the overall settings UX.
- Introducing automatic fallback destinations when the configured path is invalid.
- Changing archive subfolder naming rules beyond the existing theme/device normalization.

## Decisions

### Resolve destination input through a shared normalization step

We will introduce a destination-path resolution step in core configuration logic that:
- trims empty input,
- expands a leading `~` to the current user's home directory,
- converts the result into the `PathBuf` used by the rest of the app.

Settings save, preview generation, and import planning will all use this same resolver before validation. This avoids divergent rules between UI and core logic.

Alternative considered:
- Expand `~` only in the TUI input handler. Rejected because persisted config, tests, and non-UI callers would still bypass the behavior and drift from import planning.

### Validate creatability instead of requiring pre-existence

Validation will no longer reject every missing destination root. Instead it will:
- reject empty paths,
- reject paths whose existing target is not a directory,
- for missing paths, walk upward to the nearest existing ancestor and verify that ancestor is a directory and writable/creatable,
- preserve actionable failures when no valid ancestor exists or permissions prevent creation.

This matches the copy flow, which already creates the final archive root tree with `create_dir_all`, while extending that convenience to the configured destination root itself.

Alternative considered:
- Keep strict pre-existence checks and only create the final archive leaf. Rejected because it preserves the current friction and does not solve the reported edge case.

### Persist the resolved absolute path

When a user saves settings with `~/...`, the app will persist the resolved absolute path rather than the literal shorthand. This keeps later sessions independent from whether the original input was shell-like and ensures previews/imports operate on a stable path value.

Alternative considered:
- Preserve the raw user input and resolve it on every use. Rejected because the UI currently edits the stored `PathBuf` as text, and keeping a raw-plus-resolved dual representation would add complexity without a strong user benefit for this app.

### Add focused regression coverage at the config and archive levels

Tests will cover:
- `~` expansion into the current home directory,
- accepting missing-but-creatable destination roots,
- rejecting paths whose existing target is a file,
- rejecting missing paths when no creatable directory ancestor exists,
- building archive previews/plans from resolved paths.

This keeps the contract close to the shared core logic rather than relying only on UI tests.

## Risks / Trade-offs

- [Writable-path detection can vary by platform] -> Favor creation-based validation of the nearest existing ancestor where possible, and keep tests targeted to behavior under temp directories rather than OS-specific permission assumptions.
- [Persisting the resolved absolute path loses the original `~` shorthand] -> Accept this trade-off for simpler semantics and more predictable reloading across sessions.
- [Changing validation semantics may affect existing error messages] -> Update tests and specs to define the new expected behavior explicitly so user-facing regressions are intentional.
- [Preview may still hide some permission failures that only appear at copy time] -> Reuse the same resolution and validation path during save, preview, and plan construction so most failures are detected before copy begins.
