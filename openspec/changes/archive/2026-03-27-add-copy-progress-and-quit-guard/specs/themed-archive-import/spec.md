## MODIFIED Requirements

### Requirement: Copy supported media into the resolved archive path
The system SHALL copy supported photo and video files from the selected source device into the resolved archive folder, SHALL report progress and completion status to the user, SHALL render a visual progress bar in the status area while the copy is active, and MUST intercept the global quit shortcut while an archive copy job is still running so the copy can stop safely without immediately exiting the app.

#### Scenario: Successful archive copy
- **WHEN** the user confirms an import from a readable device to a writable destination
- **THEN** the system creates the destination folders if needed and copies the selected media files into the device-specific archive folder

#### Scenario: Copy failure is surfaced to the user
- **WHEN** a file copy fails during the archive process
- **THEN** the system reports the failure and indicates that the archive session did not complete successfully

#### Scenario: Active copy shows visual progress
- **WHEN** the archive copy is in progress and completed-file updates are available
- **THEN** the system returns to the main archive view and shows a visual progress bar in the status area that reflects copied versus total files alongside textual progress details

#### Scenario: Quit shortcut is blocked during copy
- **WHEN** the user presses `Ctrl+Q` while an archive copy job is still running
- **THEN** the system intercepts the shortcut, requests that the copy stop at a safe boundary, returns to the main archive view, and reports in the status area that the copy was interrupted
