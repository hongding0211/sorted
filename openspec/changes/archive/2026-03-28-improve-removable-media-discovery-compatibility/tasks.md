## 1. Discovery Rules

- [x] 1.1 Refactor `src/platform/discovery.rs` so platform-specific device inclusion logic is expressed through focused helper functions.
- [x] 1.2 Update the Linux inclusion rule to accept mounted removable filesystems even when `sysinfo` reports an unknown disk kind.

## 2. Verification

- [x] 2.1 Add or update discovery tests that cover Linux-compatible removable mounted media with incomplete metadata.
- [x] 2.2 Run the relevant Rust test target to confirm removable-device discovery still behaves as expected across the updated rules.
