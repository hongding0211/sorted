# themed-archive-import Specification

## Purpose
TBD - created by archiving change add-cross-platform-media-archive-tui. Update Purpose after archive.
## Requirements
### Requirement: Remember archive destination settings
The system SHALL let the user set a destination root directory for archived media, SHALL resolve supported home-directory shorthand in that destination before validation or use, and SHALL persist the resolved directory for reuse in future sessions.

#### Scenario: Saved destination reused on next launch
- **WHEN** the user previously saved a valid destination root directory
- **THEN** the system preloads that resolved directory as the default archive destination in the next session

#### Scenario: Home-directory shorthand is resolved before import
- **WHEN** the configured destination root starts with `~/`
- **THEN** the system resolves it against the current user's home directory before building the archive destination or starting the import

#### Scenario: Missing destination root can be created
- **WHEN** the configured destination root does not yet exist but its nearest existing parent directory is valid for directory creation
- **THEN** the system allows the import to proceed and creates the destination root before copying media

#### Scenario: Destination must be valid before import
- **WHEN** the configured destination root resolves to a non-directory path, cannot be created, or cannot be written
- **THEN** the system blocks the import and prompts the user to choose a valid writable destination

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
The system SHALL copy supported photo and video files from the selected source device into the resolved archive folder, SHALL report progress and completion status to the user, SHALL render a visual progress bar in the status area while the copy is active, and MUST intercept the global quit shortcut while an archive copy job is still running so the copy can stop safely without immediately exiting the app. The system MUST reject a copy attempt when the final archive destination folder already exists.

#### Scenario: Successful archive copy
- **WHEN** the user confirms an import from a readable device to a writable destination
- **THEN** the system creates the destination folders if needed and copies the selected media files into the device-specific archive folder

#### Scenario: Existing final archive destination blocks import
- **WHEN** the resolved final archive destination folder already exists before copy starts
- **THEN** the system blocks the import before copy starts and reports that the destination already exists

#### Scenario: Copy failure is surfaced to the user
- **WHEN** a file copy fails during the archive process
- **THEN** the system reports the failure and indicates that the archive session did not complete successfully

#### Scenario: Active copy shows visual progress
- **WHEN** the archive copy is in progress and completed-file updates are available
- **THEN** the system returns to the main archive view and shows a visual progress bar in the status area that reflects copied versus total files alongside textual progress details

#### Scenario: Quit shortcut is blocked during copy
- **WHEN** the user presses `Ctrl+Q` while an archive copy job is still running
- **THEN** the system intercepts the shortcut, requests that the copy stop at a safe boundary, returns to the main archive view, and reports in the status area that the copy was interrupted

### Requirement: Present a confirmation summary before import starts
The system SHALL show a confirmation view before the copy begins that summarizes the selected source, resolved destination, and next action so users can review the import before committing. The system MUST require a non-empty theme before allowing navigation into that confirmation view.

#### Scenario: Confirmation view summarizes import inputs
- **WHEN** the user advances from the main archive screen with a valid source, a valid destination, and a non-empty theme
- **THEN** the system opens a confirmation view that clearly presents the source folder, destination preview, and the action required to start or cancel the copy

#### Scenario: Missing theme blocks confirmation
- **WHEN** the user attempts to advance from the main archive screen without entering a theme
- **THEN** the system stays on the main archive screen, focuses the theme input, and warns that a theme is required before continuing

### Requirement: Present copy progress with next-step clarity
The system SHALL display copy progress in a way that makes current activity, completion counts, and what the user can do next easy to understand.

#### Scenario: Copy progress emphasizes current state
- **WHEN** an archive copy is running
- **THEN** the system returns to the main archive view and shows a visual progress bar in the status area that reflects copied versus total files alongside textual progress details

#### Scenario: Quit shortcut safely interrupts active copy
- **WHEN** the user presses `Ctrl+Q` while an archive copy job is still running
- **THEN** the system intercepts the shortcut, requests that the copy stop at a safe boundary, returns to the main archive view, and reports in the status area that the copy was interrupted

