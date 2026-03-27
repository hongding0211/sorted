# themed-archive-import Specification

## Purpose
TBD - created by archiving change add-cross-platform-media-archive-tui. Update Purpose after archive.
## Requirements
### Requirement: Remember archive destination settings
The system SHALL let the user set a destination root directory for archived media and SHALL persist that directory for reuse in future sessions.

#### Scenario: Saved destination reused on next launch
- **WHEN** the user previously saved a destination root directory
- **THEN** the system preloads that directory as the default archive destination in the next session

#### Scenario: Destination must be valid before import
- **WHEN** the configured destination root is missing or cannot be written
- **THEN** the system blocks the import and prompts the user to choose a writable destination

### Requirement: Build themed archive paths from session metadata
The system SHALL create destination paths using the pattern `dist/<theme><formatted-date>/<device-name>/`, where `theme` comes from the user input, `formatted-date` comes from the archive session date rendered with the saved date format preference, and `device-name` comes from the detected source device after filesystem-safe normalization.

#### Scenario: Import path generated from theme and device
- **WHEN** the user chooses destination `dist`, enters theme `xxx travel`, and selects a device named `EOS R6`
- **THEN** the system resolves the destination folder as `dist/xxx travel<formatted-date>/EOS R6/` using the current archive date, the saved date format, and normalized path segments

#### Scenario: Unsafe characters are normalized
- **WHEN** the entered theme or device label contains characters unsupported by the target filesystem
- **THEN** the system replaces or removes unsafe characters before creating the archive path

#### Scenario: Archive path uses configured date format
- **WHEN** the saved date format is `%Y-%m-%d`
- **THEN** the system renders the archive date segment using that format when building the destination folder

### Requirement: Copy supported media into the resolved archive path
The system SHALL copy supported photo and video files from the selected source device into the resolved archive folder and SHALL report progress and completion status to the user.

#### Scenario: Successful archive copy
- **WHEN** the user confirms an import from a readable device to a writable destination
- **THEN** the system creates the destination folders if needed and copies the selected media files into the device-specific archive folder

#### Scenario: Copy failure is surfaced to the user
- **WHEN** a file copy fails during the archive process
- **THEN** the system reports the failure and indicates that the archive session did not complete successfully

