## MODIFIED Requirements

### Requirement: Provide a settings workflow for archive preferences
The system SHALL provide a dedicated settings workflow in the TUI where users can view and update persisted archive preferences, SHALL make the currently editable field obvious, and SHALL present preview and save guidance with clear visual separation from the editable values.

#### Scenario: User opens the settings screen
- **WHEN** the user navigates to settings from the TUI
- **THEN** the system shows the current persisted archive preferences, including destination root and date format

#### Scenario: Focused field is visually distinguished
- **WHEN** the user moves focus between editable settings fields
- **THEN** the system highlights the focused field more prominently than unfocused fields and keeps helper guidance visible for how to edit or save

#### Scenario: Saved settings are available in later sessions
- **WHEN** the user updates archive preferences in the settings workflow and saves them
- **THEN** the system persists the new values and reloads them in future sessions

### Requirement: Validate date format preferences before saving
The system SHALL let the user configure the date format used in archive folder names, SHALL validate that format before saving it, and SHALL present preview or validation feedback near the settings workflow so the user can understand the result before leaving the screen.

#### Scenario: Valid date format is accepted
- **WHEN** the user enters a supported date format pattern in settings
- **THEN** the system shows a preview of the rendered date output and allows the settings to be saved

#### Scenario: Invalid date format is rejected
- **WHEN** the user enters an unsupported or invalid date format pattern
- **THEN** the system blocks saving and shows a validation error that explains the problem without requiring the user to infer it from unrelated status text
