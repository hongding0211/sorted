## Why

Photographers and creators often finish a shoot with media spread across SD cards, cameras, phones, and other removable devices, then lose time to repetitive manual archiving work. A cross-platform TUI tool can standardize this workflow by detecting inserted media sources and importing assets into a predictable archive structure with remembered destination settings.

## What Changes

- Add a Rust-based cross-platform TUI application that runs on macOS, Windows, and Linux.
- Detect newly attached removable storage devices, with emphasis on common camera and card-reader workflows.
- Let users configure and persist a default destination (`dist`) directory for future archive sessions.
- Add a settings interface in the TUI for editing remembered archive preferences.
- Let users enter a theme or project name for each import session, such as a trip or shoot title.
- Let users configure the date format used in archive folder names.
- Copy photo and video assets from a selected external device into a dated archive path using the pattern `dist/<theme><formatted-date>/<device-name>/`.
- Normalize device labels into safe folder names so archives remain portable across filesystems.

## Capabilities

### New Capabilities
- `removable-media-discovery`: Detect removable media across supported desktop platforms and expose device identity and mount information to the TUI.
- `archive-settings-ui`: Provide a settings workflow in the TUI for editing and persisting archive preferences such as destination root and date format.
- `themed-archive-import`: Persist archive destination settings and copy media into a themed, date-stamped folder structure grouped by source device.

### Modified Capabilities
- None.

## Impact

- Introduces a new Rust codebase for the TUI application and platform-specific device detection layer.
- Requires configuration persistence for saved destination directories, date format preferences, and archive defaults.
- Depends on cross-platform terminal UI, filesystem, and removable-device detection libraries.
- Defines the archive path contract that future features, such as previews or duplicate handling, must preserve.
