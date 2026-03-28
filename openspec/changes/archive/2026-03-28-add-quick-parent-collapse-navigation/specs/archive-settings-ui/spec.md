## MODIFIED Requirements

### Requirement: Provide a settings workflow for archive preferences
The system SHALL provide a dedicated settings workflow in the TUI where users can view and update persisted archive preferences, SHALL make the currently editable field obvious, SHALL present preview and save guidance with clear visual separation from the editable values, and SHALL provide a tree-structured destination directory browser in settings that uses the same expand, collapse, selection, and quick parent-backtracking model as the main source browser.

#### Scenario: User opens the settings screen
- **WHEN** the user navigates to settings from the TUI
- **THEN** the system shows the current persisted archive preferences, including destination root and date format

#### Scenario: Focused field is visually distinguished
- **WHEN** the user moves focus between editable settings fields
- **THEN** the system highlights the focused field more prominently than unfocused fields and keeps helper guidance visible for how to browse, confirm, edit, or save

#### Scenario: User browses destination directories as a tree
- **WHEN** the destination root field is focused in settings
- **THEN** the system shows a tree-structured directory browser that lets the user move through directories and expand or collapse nodes using the same interaction model as the main source browser

#### Scenario: Left navigation returns to the parent directory level in settings
- **WHEN** the user presses left on a settings tree row that is inside an expanded parent directory but is not itself expanded
- **THEN** the system moves the selection to the nearest visible parent directory and collapses that parent so the child level just left is folded away

#### Scenario: Candidate directory does not overwrite settings until confirmed
- **WHEN** the user moves through directories in the settings tree without confirming a new destination root
- **THEN** the system keeps the previously selected destination root as the active settings value and makes the browsing state distinguishable from the saved value

#### Scenario: Saved settings are available in later sessions
- **WHEN** the user updates archive preferences in the settings workflow and saves them
- **THEN** the system persists the new values and reloads them in future sessions
