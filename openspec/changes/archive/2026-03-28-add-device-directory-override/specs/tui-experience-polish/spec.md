## ADDED Requirements

### Requirement: Expose and explain the archive device-directory override in the main workflow
The system SHALL expose a dedicated text input for the device-directory override in the main archive workflow, SHALL make it clear that the override only changes the destination folder name for the current import session, and SHALL surface the effective value in confirmation feedback before copy begins.

#### Scenario: User edits the device-directory override
- **WHEN** the user focuses the device-directory override field on the main archive screen
- **THEN** the system accepts text input for the override and presents guidance that the value affects the archive folder name rather than the disk label

#### Scenario: Confirmation shows the effective device directory name
- **WHEN** the user opens the import confirmation after entering an override
- **THEN** the system shows the destination preview and device-directory value that will be used for the archive output

#### Scenario: Empty override is presented as fallback behavior
- **WHEN** the user opens the import confirmation without entering a device-directory override
- **THEN** the system makes it clear that the archive output will use the detected device name
