## MODIFIED Requirements

### Requirement: Detect removable storage devices across supported desktop platforms
The system SHALL discover removable storage devices on macOS, Windows, and Linux, SHALL expose each detected device to the TUI with a stable display name, mount path, and availability state, and SHALL communicate discovery loading and empty states clearly during browsing and refresh.

#### Scenario: Device list shown in the TUI
- **WHEN** the user opens or refreshes the archive session and removable devices are available
- **THEN** the system shows each currently detected removable storage device with its label and mount location in a browsable device list

#### Scenario: Discovery is still loading
- **WHEN** device discovery or directory expansion is still in progress
- **THEN** the system shows that the content is loading and avoids presenting the area as if it were already complete or empty

#### Scenario: No devices are available
- **WHEN** no removable storage devices are detected after refresh completes
- **THEN** the system presents an explicit empty-state message that tells the user no removable devices were found

### Requirement: Exclude unsupported or unavailable sources
The system SHALL ignore sources that are not readable mounted filesystems, SHALL prevent users from starting an archive import from an unavailable device, and SHALL make unavailable state messaging clear during source browsing.

#### Scenario: Non-readable source is filtered out
- **WHEN** a device cannot be read or does not expose a mounted filesystem path
- **THEN** the system does not offer that source as an import candidate

#### Scenario: Device becomes unavailable before import
- **WHEN** the user selects a device that is removed before copy starts
- **THEN** the system blocks the import and reports that the selected device is no longer available in a way that is clearly distinguishable from normal informational feedback
