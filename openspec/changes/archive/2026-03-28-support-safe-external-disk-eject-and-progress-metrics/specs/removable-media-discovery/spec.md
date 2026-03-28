## ADDED Requirements

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
