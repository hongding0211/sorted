## MODIFIED Requirements

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
