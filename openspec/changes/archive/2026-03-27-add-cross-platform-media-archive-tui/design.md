## Context

The project introduces a new Rust TUI application for creators who archive media from removable devices across macOS, Windows, and Linux. The workflow combines three concerns that must work together: platform-specific removable media discovery, a responsive terminal interface for selecting devices and archive metadata, a settings experience for remembered archive preferences, and reliable filesystem copying into a stable destination layout.

The current repository does not yet contain application code, so this design can establish a clean architecture from the start. The main constraints are cross-platform support, safe handling of large media copies, and folder naming rules that remain valid across filesystems.

## Goals / Non-Goals

**Goals:**
- Build a Rust TUI that runs on macOS, Windows, and Linux with a shared application core.
- Detect attached removable storage devices and present them with a stable device label and mount path.
- Persist the default archive destination so users do not need to re-enter `dist` every session.
- Let users configure the date format used in archive folder names from inside the TUI.
- Copy photos and videos into `dist/<theme><formatted-date>/<device-name>/` with normalized names.
- Keep the implementation modular so future features such as duplicate detection, dry-run previews, and rename rules can build on the same core.

**Non-Goals:**
- Automatic background import immediately when a device is inserted.
- Asset deduplication, checksum verification, or resumable copy in the first version.
- Metadata extraction from EXIF, camera models, or creation timestamps.
- Mobile-device sync over MTP/PTP when the device does not expose a filesystem mount.

## Decisions

### Use a layered Rust architecture
The codebase should separate into `core` (archive rules, copy planning, config, validation), `platform` (device discovery adapters), and `ui` (TUI state and interaction flow). This keeps most logic platform-agnostic and limits OS-specific code to device enumeration.

Alternatives considered:
- A single binary module with inline platform checks. Rejected because device discovery and UI state would become tightly coupled.
- Separate binaries per OS. Rejected because behavior would drift and increase maintenance cost.

### Use a polling-based removable media abstraction
The first version should use a polling loop that periodically refreshes visible removable mounts instead of relying on platform-native hotplug events. Polling is easier to make consistent across macOS, Windows, and Linux and is sufficient for an interactive TUI where users are already watching the session.

Alternatives considered:
- Native event subscriptions per platform. Rejected for v1 because APIs differ widely and add more platform-specific complexity.
- Manual path entry only. Rejected because it weakens the core value of device-driven archiving.

### Persist configuration in a user-scoped config file
The application should store the remembered `dist` directory, date format preference, and small UI defaults in a per-user config location, using platform conventions where possible. The stored schema should be explicit and versioned so future updates can migrate cleanly.

Alternatives considered:
- Environment variables only. Rejected because it is inconvenient for interactive users.
- Storing config in the current working directory. Rejected because archive behavior should follow the user, not the shell location.

### Normalize archive path components before planning copy operations
Theme names and device names should be normalized into filesystem-safe path segments before any copy begins. The date component should be rendered from a user-configurable format string that is validated before it is saved or used. The application should generate a copy plan first, show the resolved destination, and only then execute the copy. This reduces the chance of partial copies into malformed paths.

Alternatives considered:
- Writing directly while computing paths on the fly. Rejected because it makes validation and preview harder.
- Preserving raw device labels verbatim. Rejected because labels may contain unsupported characters or separators on some filesystems.

### Provide a dedicated settings screen in the TUI
The TUI should include a dedicated settings flow where users can review and edit the persisted destination root and date format. Keeping these preferences in a separate screen avoids overloading the import form and gives validation errors a clear home.

Alternatives considered:
- Editing preferences only inline on the import screen. Rejected because persistent settings and one-off import fields have different mental models.
- Requiring manual config-file edits. Rejected because it undermines the usability of a terminal-first tool.

### Treat copy execution as a foreground job with progress reporting
Copying should happen as a foreground task within the TUI, with visible progress and explicit completion or failure reporting. The executor should create needed directories, stream file copies in chunks, and continue to the next file only when the current one completes.

Alternatives considered:
- Spawning detached background workers. Rejected because a terminal tool should keep session state visible and understandable.
- Shelling out to system copy commands. Rejected because behavior and error handling would vary by platform.

## Risks / Trade-offs

- [Removable media detection may differ by OS] -> Define a narrow internal `DeviceInfo` model and implement platform adapters behind that contract.
- [Polling can miss very brief mount changes or feel slightly delayed] -> Use a short refresh interval and expose a manual refresh action in the TUI.
- [Long copy operations may block the UI if executed synchronously] -> Run the copy loop through an async task or worker thread and send progress updates back to the UI state.
- [Folder-name normalization can surprise users] -> Show the resolved theme and device folder names before confirming the import.
- [Invalid date format strings can generate confusing archive names] -> Validate date format input in the settings screen and preview the rendered result before saving.
- [Large imports increase risk of partial archives when interrupted] -> Create the destination tree deterministically and present per-file errors clearly so interrupted sessions can be retried.

## Migration Plan

Because this is a new application, no production migration is needed. Implementation should start with the shared domain model and config schema, then add device discovery adapters, then the settings workflow and archive path formatter, then the main TUI workflow, and finally end-to-end verification on macOS, Windows, and Linux. If a release proves unstable on one platform, the fallback is to disable that adapter behind a feature flag while keeping the shared archive workflow intact on the others.

## Open Questions

- Which exact media file extensions should be included by default in v1?
- Should the date component use the import date or the device file capture date when naming the theme folder?
- Should duplicate filenames inside the same destination folder be skipped, overwritten, or renamed automatically?
