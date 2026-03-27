## ADDED Requirements

### Requirement: Present a clearer visual hierarchy across archive screens
The system SHALL present the archive TUI with consistent visual hierarchy so users can quickly distinguish the active screen, focused interaction region, primary session data, and secondary helper text.

#### Scenario: Main screen highlights the active work area
- **WHEN** the user is on the main archive screen
- **THEN** the system shows the source browser and archive summary as visually distinct sections and makes the currently focused region more prominent than surrounding content

#### Scenario: Secondary guidance is visually de-emphasized
- **WHEN** keyboard hints, helper copy, or explanatory text are shown on any archive screen
- **THEN** the system renders them in a style that remains readable but is less prominent than primary workflow data and actions

### Requirement: Provide contextual interaction guidance and feedback
The system SHALL present feedback and keyboard guidance that matches the user's current screen and interaction state rather than relying only on one generic global hint string.

#### Scenario: Guidance changes with the active screen
- **WHEN** the user enters settings, confirmation, copy progress, or results screens
- **THEN** the system shows screen-appropriate guidance for the actions that are currently available

#### Scenario: Feedback communicates outcome severity
- **WHEN** the system reports informational, success, warning, or error feedback during the archive workflow
- **THEN** the feedback is presented with wording and visual emphasis that makes the outcome severity easy to distinguish
