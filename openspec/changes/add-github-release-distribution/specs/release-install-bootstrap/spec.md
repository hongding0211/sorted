## ADDED Requirements

### Requirement: Install latest compatible release from GitHub
The system SHALL provide a shell-based install bootstrap that discovers the latest Sorted GitHub Release by default, selects the package compatible with the current operating system and CPU architecture, and installs the Sorted executable on the local machine.

#### Scenario: Installer selects latest release for current platform
- **WHEN** a user runs the install script without a version argument on a supported macOS or Linux shell environment
- **THEN** the script resolves the latest GitHub Release metadata, selects the matching asset for the current platform and architecture, and downloads it for installation

#### Scenario: Installer accepts explicit version override
- **WHEN** a user provides a specific version to the install script
- **THEN** the script downloads the matching versioned release asset instead of the latest release

### Requirement: Install into a predictable executable path
The system SHALL install the Sorted executable into a predictable local directory, SHALL support an explicit install directory override, and SHALL surface post-install guidance when the directory is not already on PATH.

#### Scenario: Default install path is created automatically
- **WHEN** the default install directory does not already exist
- **THEN** the script creates it before placing the Sorted executable there

#### Scenario: User overrides install directory
- **WHEN** a user sets the supported install-directory override for the script
- **THEN** the script installs the Sorted executable into that directory instead of the default location

#### Scenario: PATH guidance is shown after install
- **WHEN** the install completes but the target directory is not present in the current PATH
- **THEN** the script prints clear guidance for adding that directory before the user runs `sorted`

### Requirement: Fail clearly on unsupported environments or missing tools
The system SHALL stop before installation when the host platform is unsupported or when required download and archive tools are unavailable, and SHALL explain the blocking condition clearly.

#### Scenario: Unsupported platform is rejected
- **WHEN** the install script runs on an OS or CPU architecture that has no matching release package
- **THEN** the script exits without partial installation and tells the user that the platform is not yet supported

#### Scenario: Required tool is missing
- **WHEN** the install script needs a tool such as `curl`, `tar`, or `unzip` and it is unavailable in the environment
- **THEN** the script exits before download and reports which prerequisite is missing
