## Why

The current TUI already supports the core archive flow, but the visual hierarchy and interaction feedback are still fairly bare. Users must infer focus, status transitions, and next actions from dense text blocks, which makes settings edits, device browsing, and import confirmation feel more error-prone than they need to.

## What Changes

- Refresh the TUI presentation so primary actions, focused fields, important status messages, and long-running states are easier to scan at a glance.
- Improve interaction feedback across the main flow, including clearer empty/loading/error states, stronger confirmation and progress views, and more actionable keyboard guidance.
- Make settings and import-related screens feel more consistent by standardizing layout rhythm, section emphasis, and validation messaging.
- Add implementation tasks and behavioral specs that cover both visual polish and user-facing interaction expectations.

## Capabilities

### New Capabilities
- `tui-experience-polish`: define cross-screen presentation, status feedback, and keyboard guidance requirements for the archive TUI.

### Modified Capabilities
- `archive-settings-ui`: settings screens must provide clearer focus, validation, preview, and save feedback.
- `themed-archive-import`: import confirmation, in-progress copy, and results views must present information with stronger hierarchy and user guidance.
- `removable-media-discovery`: device discovery views must communicate loading, empty, and unavailable states more clearly during browsing and refresh.

## Impact

- Affected code: `src/ui/app.rs` and supporting UI modules, with potential touch points in validation and status-message plumbing.
- Affected behavior: screen layout, focus styling, status and error messaging, confirmation/progress/results presentation, and keyboard affordances in the TUI.
- Dependencies: no new external services are expected; implementation should build on the existing `ratatui` and `crossterm` stack.
