## MODIFIED Requirements

### Requirement: Copy supported media into the resolved archive path
The system SHALL copy supported photo and video files from the selected source device into the resolved archive folder, SHALL report progress and completion status to the user, SHALL render a visual progress bar in the status area while the copy is active, SHALL show the current transfer rate and estimated time remaining while the copy is active, SHALL report the total elapsed time after the copy finishes, and MUST intercept the global quit shortcut while an archive copy job is still running so the copy can stop safely without immediately exiting the app. The system MUST reject a copy attempt when the final archive destination folder already exists.

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

#### Scenario: Active copy shows transfer rate and ETA
- **WHEN** the archive copy is in progress and enough progress samples exist to estimate throughput
- **THEN** the system shows the current transfer rate and estimated time remaining in the status area alongside copy progress

#### Scenario: Completed copy shows elapsed duration
- **WHEN** the archive copy completes successfully
- **THEN** the system reports completion and includes how much total time the archive job took

#### Scenario: Quit shortcut is blocked during copy
- **WHEN** the user presses `Ctrl+Q` while an archive copy job is still running
- **THEN** the system intercepts the shortcut, requests that the copy stop at a safe boundary, returns to the main archive view, and reports in the status area that the copy was interrupted

### Requirement: Present copy progress with next-step clarity
The system SHALL display copy progress in a way that makes current activity, completion counts, transfer speed, estimated remaining time, and post-copy outcome easy to understand.

#### Scenario: Copy progress emphasizes current state
- **WHEN** an archive copy is running
- **THEN** the system returns to the main archive view and shows a visual progress bar in the status area that reflects copied versus total files alongside textual progress details

#### Scenario: Copy progress includes timing metrics
- **WHEN** an archive copy is running and timing estimates are available
- **THEN** the status area shows the current transfer rate and estimated completion time in addition to copied progress

#### Scenario: Completion message includes duration
- **WHEN** an archive copy finishes successfully
- **THEN** the status area replaces the remaining-time estimate with a completion summary that includes the total elapsed duration

#### Scenario: Quit shortcut safely interrupts active copy
- **WHEN** the user presses `Ctrl+Q` while an archive copy job is still running
- **THEN** the system intercepts the shortcut, requests that the copy stop at a safe boundary, returns to the main archive view, and reports in the status area that the copy was interrupted
