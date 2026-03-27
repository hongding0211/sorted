## 1. Project Setup

- [x] 1.1 Initialize a Rust binary project for the TUI and add core dependencies for terminal UI, configuration persistence, async/task execution, and filesystem handling
- [x] 1.2 Create the initial module structure for `core`, `platform`, and `ui` layers with shared domain types such as `DeviceInfo`, archive settings, and import session state
- [x] 1.3 Define a config file schema and platform-specific config directory resolution for macOS, Windows, and Linux, including persisted destination root and date format settings

## 2. Removable Media Discovery

- [x] 2.1 Implement a cross-platform discovery interface that returns mounted removable devices with display name, mount path, and availability state
- [x] 2.2 Add platform adapters or detection logic for macOS, Windows, and Linux and normalize their output into the shared `DeviceInfo` model
- [x] 2.3 Add refresh handling and unavailable-device validation so stale or unreadable devices cannot be used for imports

## 3. Archive Planning And Copy Execution

- [x] 3.1 Implement destination root persistence and loading so the remembered `dist` directory is prefilled on startup
- [x] 3.2 Implement date format persistence, validation, and archive path generation for `dist/<theme><formatted-date>/<device-name>/`
- [x] 3.3 Implement media file discovery, destination directory creation, and file copy execution with progress and failure reporting

## 4. TUI Workflow

- [x] 4.1 Build a settings screen for editing the persisted destination root and date format, including validation and a rendered date preview
- [x] 4.2 Build the main TUI flow for selecting a device, entering a theme, and previewing the resolved archive path with the saved settings
- [x] 4.3 Add a confirmation step before copy starts and surface copy progress, completion, and error states in the interface
- [x] 4.4 Add manual refresh and recovery actions so users can respond to device insertion, removal, or destination errors without restarting the app

## 5. Validation

- [x] 5.1 Add unit tests for config persistence, date format validation, path normalization, and archive path generation rules
- [x] 5.2 Add integration or smoke tests for copy planning and failure handling against temporary directories
- [ ] 5.3 Verify the end-to-end archive workflow on macOS, Windows, and Linux with representative removable-media scenarios
