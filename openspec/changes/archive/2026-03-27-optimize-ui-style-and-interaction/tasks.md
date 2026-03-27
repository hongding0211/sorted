## 1. Shared UI foundations

- [x] 1.1 Extract shared ratatui styling helpers for focused panels, semantic status states, labels, and secondary helper text.
- [x] 1.2 Add screen-aware keyboard guidance so each major screen can render the actions relevant to the current context.
- [x] 1.3 Refine app feedback state as needed so informational, success, warning, and error messages can be rendered with clearer emphasis than the current single plain-text treatment.

## 2. Main workflow screen polish

- [x] 2.1 Rework the main archive screen layout to better separate source browsing, archive summary, and next-step readiness.
- [x] 2.2 Improve removable-device browsing states so loading, empty, and unavailable conditions are explicitly communicated in the source area.
- [x] 2.3 Redesign the confirmation screen to present a clearer review summary before copy starts.
- [x] 2.4 Update copy progress and results screens to prioritize outcome state, counts, and readable failure details.

## 3. Settings and validation experience

- [x] 3.1 Update the settings screen so the focused field, editable values, preview, and save guidance have clearer visual separation.
- [x] 3.2 Surface date-format and destination validation feedback closer to the settings workflow so save failures are easier to understand.

## 4. Verification

- [x] 4.1 Add or update tests that cover screen-state messaging and validation behavior affected by the UI feedback changes.
- [x] 4.2 Manually verify the full keyboard-driven flow across main, settings, confirmation, copy progress, and results screens to confirm the new hierarchy and guidance remain consistent.
