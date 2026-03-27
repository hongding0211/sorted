## 1. Shared Destination Path Resolution

- [x] 1.1 Add a shared destination-root resolver in core config logic that expands leading `~` and returns the resolved `PathBuf`
- [x] 1.2 Update destination validation to accept missing roots when their nearest existing parent can support directory creation, while rejecting files and non-creatable locations
- [x] 1.3 Persist resolved destination roots so saved settings, reloaded settings, and downstream callers use the same path value

## 2. Integrate Resolved Paths Through Import Flow

- [x] 2.1 Update settings save and validation flow in the TUI to use the shared resolver and surface clearer destination errors
- [x] 2.2 Update archive planning and preview generation to build paths from the resolved destination root instead of the raw user input
- [x] 2.3 Ensure copy execution creates any missing destination root directories before creating archive subfolders

## 3. Regression Coverage

- [x] 3.1 Add unit tests for `~` expansion and destination validation success/failure cases in core config logic
- [x] 3.2 Add archive or copy flow tests that verify missing destination roots are created and resolved home-directory paths produce the expected archive root
- [x] 3.3 Update or add any user-facing documentation snippets that describe acceptable destination-root input behavior
