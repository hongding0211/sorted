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

## Install

Install the latest GitHub Release on macOS or Linux:

```bash
curl -fsSL https://github.com/hongding0211/sorted/releases/latest/download/sorted-install.sh | sh
```

Install a specific version:

```bash
curl -fsSL https://github.com/hongding0211/sorted/releases/download/v0.1.0/sorted-install.sh | sh -s -- v0.1.0
```

Override the install directory:

```bash
curl -fsSL https://github.com/hongding0211/sorted/releases/latest/download/sorted-install.sh | SORTED_INSTALL_DIR="$HOME/bin" sh
```

Prebuilt GitHub Release assets are currently published for Linux and Apple Silicon macOS. On Windows or Intel macOS, build from source with Cargo instead.

## Release

GitHub Actions publishes release artifacts for:

- `x86_64-unknown-linux-musl`
- `aarch64-apple-darwin`

To publish a release:

```bash
git tag v0.1.0
git push origin v0.1.0
```

You can also trigger the `Release` workflow manually in GitHub and provide a version tag such as `v0.1.0`.

Each release uploads:

- platform-specific compiled archives
- `sorted-install.sh`
- `sorted-checksums.txt`

See [docs/release-checklist.md](docs/release-checklist.md) for the first-release validation checklist.

### Cargo Release

Use `cargo-release` to bump the crate version and create the matching `v*` git tag that triggers the GitHub release workflow:

```bash
cargo release patch --no-publish --execute
```

Other common variants:

```bash
cargo release minor --no-publish --execute
cargo release major --no-publish --execute
```

Dry-run first to preview the next version and generated tag without changing git state:

```bash
cargo release patch --no-publish --no-push
```

This repository is configured so `cargo-release`:

- only allows releases from `main`
- creates tags in the form `v0.1.0`
- skips crates.io publishing
- uses `origin` as the push remote

## Keyboard

- `Ctrl+Q`: quit
- `Ctrl+R`: refresh devices
- `Ctrl+S`: open settings
- `Tab`: switch focus
- `Up` / `Down`: move selection
- `Left` / `Right`: collapse or expand the source tree
- `Enter`: confirm or save
- `Esc`: go back

## TODO

- [ ] Make tree-view pagination auto-scroll so lower entries remain visible when changing pages
- [ ] Add more convenient file navigation and file operation capabilities

## OpenSpec

This repository also includes OpenSpec artifacts and archived changes under
`openspec/`.
