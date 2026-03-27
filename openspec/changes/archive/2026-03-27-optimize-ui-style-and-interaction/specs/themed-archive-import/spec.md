## MODIFIED Requirements

### Requirement: Copy supported media into the resolved archive path
The system SHALL copy supported photo and video files from the selected source device into the resolved archive folder, SHALL report progress and completion status to the user, and SHALL present copy-state information with clear hierarchy so users can distinguish active progress, successful completion, and partial failure outcomes.

#### Scenario: Successful archive copy
- **WHEN** the user confirms an import from a readable device to a writable destination
- **THEN** the system creates the destination folders if needed, copies the selected media files into the device-specific archive folder, and shows a success-oriented completion summary when finished

#### Scenario: Copy failure is surfaced to the user
- **WHEN** a file copy fails during the archive process
- **THEN** the system reports the failure, indicates that the archive session did not complete successfully, and keeps failure details readable from the results view

## ADDED Requirements

### Requirement: Present a confirmation summary before import starts
The system SHALL show a confirmation view before the copy begins that summarizes the selected source, resolved destination, and next action so users can review the import before committing.

#### Scenario: Confirmation view summarizes import inputs
- **WHEN** the user advances from the main archive screen with a valid source and destination
- **THEN** the system opens a confirmation view that clearly presents the source folder, destination preview, and the action required to start or cancel the copy

### Requirement: Present copy progress with next-step clarity
The system SHALL display copy progress in a way that makes current activity, completion counts, and what the user can do next easy to understand.

#### Scenario: Copy progress emphasizes current state
- **WHEN** an archive copy is running
- **THEN** the system shows that the copy is in progress, includes a progress summary with completed versus total files, and indicates that the user must wait for the operation to finish before leaving the flow
