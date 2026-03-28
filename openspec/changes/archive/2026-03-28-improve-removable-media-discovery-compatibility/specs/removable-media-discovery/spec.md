## MODIFIED Requirements

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
