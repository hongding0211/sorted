## MODIFIED Requirements

### Requirement: Remember archive destination settings
The system SHALL let the user set a destination root directory for archived media, SHALL resolve supported home-directory shorthand in that destination before validation or use, and SHALL persist the resolved directory for reuse in future sessions.

#### Scenario: Saved destination reused on next launch
- **WHEN** the user previously saved a valid destination root directory
- **THEN** the system preloads that resolved directory as the default archive destination in the next session

#### Scenario: Home-directory shorthand is resolved before import
- **WHEN** the configured destination root starts with `~/`
- **THEN** the system resolves it against the current user's home directory before building the archive destination or starting the import

#### Scenario: Missing destination root can be created
- **WHEN** the configured destination root does not yet exist but its nearest existing parent directory is valid for directory creation
- **THEN** the system allows the import to proceed and creates the destination root before copying media

#### Scenario: Destination must be valid before import
- **WHEN** the configured destination root resolves to a non-directory path, cannot be created, or cannot be written
- **THEN** the system blocks the import and prompts the user to choose a valid writable destination
