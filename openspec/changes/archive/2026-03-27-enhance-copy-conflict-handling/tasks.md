## 1. Confirmation Readiness

- [x] 1.1 Update `src/ui/app.rs` so advancing from the main import screen refuses to enter confirmation when the theme is blank after trimming.
- [x] 1.2 Reuse the existing status/focus feedback on the main screen so a missing theme points the user back to the theme input immediately.

## 2. Copy Conflict Policy

- [x] 2.1 Add a final-destination existence guard so planning/execution stops when the resolved archive destination folder already exists.
- [x] 2.2 Reuse that guard in both confirmation-entry validation and copy start so the rule stays consistent if filesystem state changes.

## 3. User-Facing Failure Reporting

- [x] 3.1 Ensure the existing-destination failure uses a stable, understandable error message that the existing UI can present without ambiguity.
- [x] 3.2 Verify that the app blocks before any copy work starts when the final archive directory already exists.

## 4. Verification

- [x] 4.1 Add or update UI-focused tests covering the main-screen advance path when theme is missing.
- [x] 4.2 Add or update tests covering rejection when the resolved final archive destination folder already exists.
- [x] 4.3 Remove or replace outdated same-name-root coverage so tests reflect the final rule.
- [x] 4.4 Run the relevant Rust test target to confirm the new confirmation-gating and existing-destination handling behavior passes end to end.
