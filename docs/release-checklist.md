# Release Checklist

Use this checklist before creating the first public GitHub Release for Sorted.

## Local Validation

1. Confirm the package script can build a versioned archive from a compiled binary:
   `scripts/package-release.sh v0.1.0 x86_64-unknown-linux-gnu tar.gz target/debug/sorted dist`
2. Confirm the installer script parses and validates successfully:
   `sh -n scripts/install.sh`
3. Confirm the package script parses successfully:
   `sh -n scripts/package-release.sh`
4. Run the test suite:
   `cargo test`

## First Release

1. Push the current branch to GitHub.
2. Create and push a semantic version tag such as `v0.1.0`.
3. Wait for the `Release` GitHub Actions workflow to finish for all configured targets.
4. Open the GitHub Release and verify these assets exist:
   - `sorted-v0.1.0-x86_64-unknown-linux-gnu.tar.gz`
   - `sorted-v0.1.0-x86_64-pc-windows-msvc.zip`
   - `sorted-v0.1.0-x86_64-apple-darwin.tar.gz`
   - `sorted-v0.1.0-aarch64-apple-darwin.tar.gz`
   - `sorted-install.sh`
   - `sorted-checksums.txt`
5. Test the installer on macOS or Linux:
   `curl -fsSL https://github.com/hongding0211/sorted/releases/latest/download/sorted-install.sh | sh`
6. If a release asset is missing or malformed, delete the GitHub Release, fix the workflow, and publish the tag again.
