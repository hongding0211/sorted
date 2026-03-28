# tui-experience-polish Specification

## Purpose
TBD - created by archiving change optimize-ui-style-and-interaction. Update Purpose after archive.
## Requirements
### Requirement: Present a clearer visual hierarchy across archive screens
The system SHALL present the archive TUI with consistent visual hierarchy so users can quickly distinguish the active screen, focused interaction region, primary session data, and secondary helper text.

#### Scenario: Main screen highlights the active work area
- **WHEN** the user is on the main archive screen
- **THEN** the system shows the source browser and archive summary as visually distinct sections and makes the currently focused region more prominent than surrounding content

#### Scenario: Secondary guidance is visually de-emphasized
- **WHEN** keyboard hints, helper copy, or explanatory text are shown on any archive screen
- **THEN** the system renders them in a style that remains readable but is less prominent than primary workflow data and actions

### Requirement: Provide contextual interaction guidance and feedback
The system SHALL present feedback and keyboard guidance that matches the user's current screen and interaction state rather than relying only on one generic global hint string, SHALL keep the active row visible when tree-based browsers contain more entries than fit in the viewport, and SHALL support quick parent-level back navigation in tree browsers so users can leave a nested folder level and collapse it with a single left-navigation action.

#### Scenario: Guidance changes with the active screen
- **WHEN** the user enters settings, confirmation, copy progress, or results screens
- **THEN** the system shows screen-appropriate guidance for the actions that are currently available

#### Scenario: Feedback communicates outcome severity
- **WHEN** the system reports informational, success, warning, or error feedback during the archive workflow
- **THEN** the feedback is presented with wording and visual emphasis that makes the outcome severity easy to distinguish

#### Scenario: Tree selection remains visible in long lists
- **WHEN** the user moves the selection through a source tree or settings tree that contains more rows than the panel can display at once
- **THEN** the system automatically scrolls the list so the currently selected row stays visible instead of disappearing above or below the viewport

#### Scenario: Left navigation collapses the selected expanded folder
- **WHEN** the user presses left while the selected source tree folder is expanded
- **THEN** the system collapses that folder and keeps the selection on the same folder row

#### Scenario: Left navigation returns to the parent level
- **WHEN** the user presses left while the selected source tree row is inside an expanded parent folder but the selected row itself is not expanded
- **THEN** the system moves selection to the nearest visible parent folder and collapses that parent so the child level just left is folded away

### Requirement: Expose and explain the archive device-directory override in the main workflow
The system SHALL expose a dedicated text input for the device-directory override in the main archive workflow, SHALL make it clear that the override only changes the destination folder name for the current import session, and SHALL surface the effective value in confirmation feedback before copy begins.

#### Scenario: User edits the device-directory override
- **WHEN** the user focuses the device-directory override field on the main archive screen
- **THEN** the system accepts text input for the override and presents guidance that the value affects the archive folder name rather than the disk label

#### Scenario: Confirmation shows the effective device directory name
- **WHEN** the user opens the import confirmation after entering an override
- **THEN** the system shows the destination preview and device-directory value that will be used for the archive output

#### Scenario: Empty override is presented as fallback behavior
- **WHEN** the user opens the import confirmation without entering a device-directory override
- **THEN** the system makes it clear that the archive output will use the detected device name
