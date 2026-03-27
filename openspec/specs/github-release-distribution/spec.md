# github-release-distribution Specification

## Purpose
TBD - created by archiving change add-github-release-distribution. Update Purpose after archive.

## Requirements
### Requirement: Build release artifacts for supported desktop targets
The system SHALL provide an automated GitHub-based release workflow that builds Sorted for the supported desktop targets on macOS, Linux, and Windows whenever a release tag is published or a maintainer triggers a release run manually.

#### Scenario: Tag starts a multi-platform release build
- **WHEN** a maintainer pushes a version tag that matches the repository's release convention
- **THEN** the GitHub workflow starts platform builds for the configured macOS, Linux, and Windows targets without requiring local manual packaging

#### Scenario: Maintainer triggers a release build manually
- **WHEN** a maintainer starts the release workflow through GitHub's manual dispatch entrypoint
- **THEN** the workflow accepts the requested release version context and builds the configured platform artifacts

### Requirement: Publish consistently named release packages
The system SHALL package each built binary into a downloadable archive with a versioned, target-specific filename and SHALL publish those archives to a GitHub Release associated with the requested version.

#### Scenario: Linux and macOS packages use tar.gz archives
- **WHEN** the workflow packages a Unix-like target
- **THEN** it emits a `.tar.gz` archive whose filename includes the Sorted version and target triple

#### Scenario: Windows packages use zip archives
- **WHEN** the workflow packages a Windows target
- **THEN** it emits a `.zip` archive whose filename includes the Sorted version and target triple

#### Scenario: Release assets are attached to GitHub Release
- **WHEN** all target packages complete successfully
- **THEN** the workflow uploads each package as an asset on the matching GitHub Release instead of leaving artifacts only in transient CI storage

### Requirement: Publish release metadata needed by installers
The system SHALL publish release metadata that lets automated installers locate and verify downloadable assets for a specific version or the latest version.

#### Scenario: Latest release can be discovered through GitHub metadata
- **WHEN** a client requests the repository's latest release metadata
- **THEN** the GitHub release information includes the version identifier and asset URLs for every published platform package

#### Scenario: Release publishes checksum information
- **WHEN** the workflow publishes a GitHub Release
- **THEN** it also uploads a checksum manifest that lists every packaged asset and its SHA256 digest
