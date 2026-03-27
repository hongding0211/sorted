## MODIFIED Requirements

### Requirement: Provide contextual interaction guidance and feedback
The system SHALL present feedback and keyboard guidance that matches the user's current screen and interaction state rather than relying only on one generic global hint string, and SHALL keep the active row visible when tree-based browsers contain more entries than fit in the viewport.

#### Scenario: Guidance changes with the active screen
- **WHEN** the user enters settings, confirmation, copy progress, or results screens
- **THEN** the system shows screen-appropriate guidance for the actions that are currently available

#### Scenario: Feedback communicates outcome severity
- **WHEN** the system reports informational, success, warning, or error feedback during the archive workflow
- **THEN** the feedback is presented with wording and visual emphasis that makes the outcome severity easy to distinguish

#### Scenario: Tree selection remains visible in long lists
- **WHEN** the user moves the selection through a source tree or settings tree that contains more rows than the panel can display at once
- **THEN** the system automatically scrolls the list so the currently selected row stays visible instead of disappearing above or below the viewport
