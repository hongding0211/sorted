## ADDED Requirements

### Requirement: Allow a per-import override for the archive device directory name
The system SHALL let the user provide an optional device-directory override for the currently selected removable device during an import session, SHALL fall back to the detected device display name when the override is empty, and SHALL use the effective value consistently for archive-path planning without changing the underlying device identity, mount path, or availability state.

#### Scenario: Empty override keeps the detected device name
- **WHEN** the user does not provide a device-directory override before confirming an import
- **THEN** the system uses the detected removable-device display name for the archive directory segment

#### Scenario: Override replaces the archive directory segment
- **WHEN** the user provides a non-empty device-directory override for the selected removable device
- **THEN** the system uses the override instead of the detected display name when building the destination preview and final archive path

#### Scenario: Override does not rewrite discovered device identity
- **WHEN** a device-directory override is present for the selected device
- **THEN** the system continues to validate availability, mounted path, and eject behavior against the originally discovered removable device
