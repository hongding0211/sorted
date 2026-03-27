# Sorted

Sorted is a cross-platform Rust TUI for importing media from removable devices
into a predictable archive structure.

## What It Does

- Detects removable devices on macOS, Windows, and Linux
- Lets you browse a source tree and pick a specific folder instead of copying an entire disk
- Persists the destination root and date format in a user-scoped config file
- Builds archive paths using:

```text
<destination>/<theme><formatted-date>/<device-name>/
```

- Normalizes output directory names so spaces become `_`
- Shows confirmation, copy progress, and copy results in the TUI

## Run

```bash
cargo run
```

## Test

```bash
cargo test
```

## Keyboard

- `Ctrl+Q`: quit
- `Ctrl+R`: refresh devices
- `Ctrl+S`: open settings
- `Tab`: switch focus
- `Up` / `Down`: move selection
- `Left` / `Right`: collapse or expand the source tree
- `Enter`: confirm or save
- `Esc`: go back

## OpenSpec

This repository also includes OpenSpec artifacts and archived changes under
`openspec/`.
