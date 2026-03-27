## ADDED Requirements

### Requirement: Provide a settings workflow for archive preferences
The system SHALL provide a dedicated settings workflow in the TUI where users can view and update persisted archive preferences.

#### Scenario: User opens the settings screen
- **WHEN** the user navigates to settings from the TUI
- **THEN** the system shows the current persisted archive preferences, including destination root and date format

#### Scenario: Saved settings are available in later sessions
- **WHEN** the user updates archive preferences in the settings workflow and saves them
- **THEN** the system persists the new values and reloads them in future sessions

### Requirement: Validate date format preferences before saving
The system SHALL let the user configure the date format used in archive folder names and SHALL validate that format before saving it.

#### Scenario: Valid date format is accepted
- **WHEN** the user enters a supported date format pattern in settings
- **THEN** the system saves the format and shows a preview of the rendered date output

#### Scenario: Invalid date format is rejected
- **WHEN** the user enters an unsupported or invalid date format pattern
- **THEN** the system blocks saving and shows a validation error that explains the problem
