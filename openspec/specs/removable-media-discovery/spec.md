# removable-media-discovery Specification

## Purpose
TBD - created by archiving change add-cross-platform-media-archive-tui. Update Purpose after archive.
## Requirements
### Requirement: Detect removable storage devices across supported desktop platforms
The system SHALL discover removable storage devices on macOS, Windows, and Linux and expose each detected device to the TUI with a stable display name, mount path, and availability state.

#### Scenario: Device list shown in the TUI
- **WHEN** the user opens or refreshes the archive session
- **THEN** the system shows each currently detected removable storage device with its label and mount location

#### Scenario: Newly attached device appears after refresh
- **WHEN** a removable storage device is attached while the TUI is running and detection is refreshed
- **THEN** the system includes the new device in the selectable device list

### Requirement: Exclude unsupported or unavailable sources
The system SHALL ignore sources that are not readable mounted filesystems and SHALL prevent users from starting an archive import from an unavailable device.

#### Scenario: Non-readable source is filtered out
- **WHEN** a device cannot be read or does not expose a mounted filesystem path
- **THEN** the system does not offer that source as an import candidate

#### Scenario: Device becomes unavailable before import
- **WHEN** the user selects a device that is removed before copy starts
- **THEN** the system blocks the import and reports that the selected device is no longer available

