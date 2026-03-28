## 1. Import Session Model

- [x] 1.1 Extend the import-session state and planning inputs so a per-session device-directory override can be captured without mutating discovered `DeviceInfo`.
- [x] 1.2 Update archive-path planning to resolve the effective device directory name from `override -> detected display name`, reusing existing path normalization and validation rules.

## 2. Main Workflow UI

- [x] 2.1 Add a device-directory override text field to the main archive workflow and include contextual guidance that it changes only the destination folder name for the current import.
- [x] 2.2 Update the destination preview and confirmation screen so they display the effective device directory name and clearly show fallback behavior when no override is entered.

## 3. Regression Coverage

- [x] 3.1 Add or update unit tests covering archive planning with empty, explicit, and normalization-sensitive device-directory overrides.
- [x] 3.2 Add or update UI tests covering focus, editing, preview, and confirmation behavior for the new override field.
