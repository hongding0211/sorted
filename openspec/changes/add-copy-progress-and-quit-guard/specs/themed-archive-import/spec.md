## MODIFIED Requirements

### Requirement: Copy supported media into the resolved archive path
The system SHALL copy supported photo and video files from the selected source device into the resolved archive folder, SHALL report progress and completion status to the user, SHALL render a visual progress bar while the copy is active, and MUST block the global quit shortcut while an archive copy job is still running.

#### Scenario: Successful archive copy
- **WHEN** the user confirms an import from a readable device to a writable destination
- **THEN** the system creates the destination folders if needed and copies the selected media files into the device-specific archive folder

#### Scenario: Copy failure is surfaced to the user
- **WHEN** a file copy fails during the archive process
- **THEN** the system reports the failure and indicates that the archive session did not complete successfully

#### Scenario: Active copy shows visual progress
- **WHEN** the archive copy is in progress and completed-file updates are available
- **THEN** the system shows a visual progress bar that reflects copied versus total files alongside textual progress details

#### Scenario: Quit shortcut is blocked during copy
- **WHEN** the user presses `Ctrl+Q` while an archive copy job is still running
- **THEN** the system keeps the app open, keeps the copy screen active, and informs the user that quitting is unavailable until the copy completes
