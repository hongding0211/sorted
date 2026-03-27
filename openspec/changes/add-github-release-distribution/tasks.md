## 1. Release Workflow Foundation

- [x] 1.1 Add a GitHub Actions release workflow that builds Sorted on the configured macOS, Linux, and Windows targets from version tags or manual dispatch.
- [x] 1.2 Add packaging steps or helper scripts that produce versioned archives with target-specific filenames for each platform.
- [x] 1.3 Upload packaged artifacts and a SHA256 checksum manifest to the matching GitHub Release.

## 2. Install Bootstrap

- [x] 2.1 Add a POSIX shell install script that resolves the requested or latest GitHub Release version and maps the current OS and CPU architecture to a supported asset.
- [x] 2.2 Implement archive download, extraction, executable placement, install-directory override, and PATH guidance in the install script.
- [x] 2.3 Add clear prerequisite and unsupported-platform error handling for missing tools or unmatched release assets.

## 3. Documentation And Validation

- [x] 3.1 Update README with GitHub Release installation instructions, manual download guidance, and maintainer release steps.
- [x] 3.2 Verify the workflow, package naming, and install script behavior against the spec scenarios using local dry runs where possible and a documented first-release checklist.
