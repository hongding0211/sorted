# removable-media-discovery Specification

## Purpose
TBD - created by archiving change add-cross-platform-media-archive-tui. Update Purpose after archive.
## Requirements
### Requirement: Detect removable storage devices across supported desktop platforms
The system SHALL discover removable storage devices on macOS, Windows, and Linux, SHALL expose each detected device to the TUI with a stable display name, mount path, and availability state, and SHALL communicate discovery loading and empty states clearly during browsing and refresh. On Linux, the system SHALL treat a mounted, readable removable filesystem as a detectable device even when the underlying platform reports incomplete or unknown disk-kind metadata.

#### Scenario: Device list shown in the TUI
- **WHEN** the user opens or refreshes the archive session and removable devices are available
- **THEN** the system shows each currently detected removable storage device with its label and mount location in a browsable device list

#### Scenario: Discovery is still loading
- **WHEN** device discovery or directory expansion is still in progress
- **THEN** the system shows that the content is loading and avoids presenting the area as if it were already complete or empty

#### Scenario: No devices are available
- **WHEN** no removable storage devices are detected after refresh completes
- **THEN** the system presents an explicit empty-state message that tells the user no removable devices were found

#### Scenario: Linux removable media uses unknown disk-kind metadata
- **WHEN** a Linux environment reports a removable mounted filesystem with a valid mount path but an unknown disk kind
- **THEN** the system still includes that filesystem in the removable device list

### Requirement: Exclude unsupported or unavailable sources
The system SHALL ignore sources that are not readable mounted filesystems, SHALL prevent users from starting an archive import from an unavailable device, and SHALL make unavailable state messaging clear during source browsing.

#### Scenario: Non-readable source is filtered out
- **WHEN** a device cannot be read or does not expose a mounted filesystem path
- **THEN** the system does not offer that source as an import candidate

#### Scenario: Device becomes unavailable before import
- **WHEN** the user selects a device that is removed before copy starts
- **THEN** the system blocks the import and reports that the selected device is no longer available in a way that is clearly distinguishable from normal informational feedback

### Requirement: Safely eject removable storage devices from the archive workflow
The system SHALL let the user request a safe eject for a selected removable storage device when that device is idle, SHALL delegate the eject request to a platform-appropriate mechanism, and SHALL report whether the device is ready to remove.

#### Scenario: Safe eject succeeds after archive work completes
- **WHEN** the user requests eject for the selected removable storage device after no archive copy is actively using it
- **THEN** the system performs a safe eject request through the operating system and reports that the device is ready to remove

#### Scenario: Safe eject is unavailable during active copy
- **WHEN** the selected removable storage device is still the source of an active archive copy and the user requests eject
- **THEN** the system rejects the request without attempting OS eject and reports that the device cannot be ejected while copying is in progress

#### Scenario: Safe eject failure is surfaced clearly
- **WHEN** the operating system rejects the safe eject request or returns an error for the selected removable storage device
- **THEN** the system keeps the device available in the UI and reports that it is not yet safe to remove the disk
