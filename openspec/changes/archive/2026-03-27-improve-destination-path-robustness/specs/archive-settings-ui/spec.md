## MODIFIED Requirements

### Requirement: Provide a settings workflow for archive preferences
The system SHALL provide a dedicated settings workflow in the TUI where users can view and update persisted archive preferences, and SHALL apply the same destination path resolution rules in settings that are used during preview and import.

#### Scenario: User opens the settings screen
- **WHEN** the user navigates to settings from the TUI
- **THEN** the system shows the current persisted archive preferences, including destination root and date format

#### Scenario: Saved settings are available in later sessions
- **WHEN** the user updates archive preferences in the settings workflow and saves them
- **THEN** the system persists the new values and reloads them in future sessions

#### Scenario: Home-directory shorthand is normalized on save
- **WHEN** the user enters a destination root beginning with `~/` in settings and saves it
- **THEN** the system resolves that path to the current user's home directory before persisting it

## ADDED Requirements

### Requirement: Validate destination root preferences before saving
The system SHALL validate destination root preferences before saving them and SHALL accept destination roots that do not yet exist when the path can be created successfully.

#### Scenario: Missing destination root is accepted when creatable
- **WHEN** the user enters a destination root that does not exist yet and its nearest existing parent directory can host a new folder
- **THEN** the system saves the setting instead of rejecting it for not already existing

#### Scenario: Invalid destination root is rejected on save
- **WHEN** the user enters a destination root that resolves to a file path or a location that cannot be created
- **THEN** the system blocks saving and shows a validation error that explains the problem
