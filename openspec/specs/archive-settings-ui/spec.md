# archive-settings-ui Specification

## Purpose
TBD - created by archiving change add-cross-platform-media-archive-tui. Update Purpose after archive.
## Requirements
### Requirement: Provide a settings workflow for archive preferences
The system SHALL provide a dedicated settings workflow in the TUI where users can view and update persisted archive preferences, SHALL make the currently editable field obvious, SHALL present preview and save guidance with clear visual separation from the editable values, and SHALL provide a tree-structured destination directory browser in settings that uses the same expand, collapse, and selection model as the main source browser.

#### Scenario: User opens the settings screen
- **WHEN** the user navigates to settings from the TUI
- **THEN** the system shows the current persisted archive preferences, including destination root and date format

#### Scenario: Focused field is visually distinguished
- **WHEN** the user moves focus between editable settings fields
- **THEN** the system highlights the focused field more prominently than unfocused fields and keeps helper guidance visible for how to browse, confirm, edit, or save

#### Scenario: User browses destination directories as a tree
- **WHEN** the destination root field is focused in settings
- **THEN** the system shows a tree-structured directory browser that lets the user move through directories and expand or collapse nodes using the same interaction model as the main source browser

#### Scenario: Candidate directory does not overwrite settings until confirmed
- **WHEN** the user moves through directories in the settings tree without confirming a new destination root
- **THEN** the system keeps the previously selected destination root as the active settings value and makes the browsing state distinguishable from the saved value

#### Scenario: Saved settings are available in later sessions
- **WHEN** the user updates archive preferences in the settings workflow and saves them
- **THEN** the system persists the new values and reloads them in future sessions

### Requirement: Validate destination root preferences before saving
The system SHALL validate destination root preferences before saving them, SHALL accept destination roots that do not yet exist when the path can be created successfully, and SHALL apply the same validation rules whether the destination root was chosen through tree browsing or retained from an existing saved value.

#### Scenario: Missing destination root is accepted when creatable
- **WHEN** the user confirms a destination root that does not exist yet and its nearest existing parent directory can host a new folder
- **THEN** the system saves the setting instead of rejecting it for not already existing

#### Scenario: Invalid destination root is rejected on save
- **WHEN** the user confirms or retains a destination root that resolves to a file path or a location that cannot be created
- **THEN** the system blocks saving and shows a validation error that explains the problem

### Requirement: Validate date format preferences before saving
The system SHALL let the user configure the date format used in archive folder names, SHALL validate that format before saving it, and SHALL present preview or validation feedback near the settings workflow so the user can understand the result before leaving the screen.

#### Scenario: Valid date format is accepted
- **WHEN** the user enters a supported date format pattern in settings
- **THEN** the system shows a preview of the rendered date output and allows the settings to be saved

#### Scenario: Invalid date format is rejected
- **WHEN** the user enters an unsupported or invalid date format pattern
- **THEN** the system blocks saving and shows a validation error that explains the problem without requiring the user to infer it from unrelated status text
