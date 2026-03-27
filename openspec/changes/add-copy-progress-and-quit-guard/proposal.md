## Why

The archive flow already performs long-running copy work, but the in-progress screen only shows textual progress and still allows the global `Ctrl+Q` quit shortcut to terminate the app mid-copy. This makes it harder for users to judge remaining work and increases the risk of accidental interruption while media is being archived.

## What Changes

- Add a dedicated progress bar presentation to the archive copy screen so users can track copy completion visually in addition to file counts.
- Change quit handling during an active copy so pressing `Ctrl+Q` is intercepted instead of immediately closing the app.
- Show clear on-screen guidance when quit is blocked during copy, so users understand that they must wait for the operation to finish before leaving the flow.
- Add behavioral coverage for in-progress copy feedback and protected copy-session keyboard handling.

## Capabilities

### New Capabilities
<!-- None. -->

### Modified Capabilities
- `themed-archive-import`: copy progress must expose a visual progress bar and must block the global quit shortcut while a copy job is still running.

## Impact

- Affected code: `src/ui/app.rs` for copy-screen rendering, key handling, status messaging, and copy-state transitions.
- Affected behavior: the archive progress screen, global shortcut handling during active copy, and user guidance around blocked quit attempts.
- Dependencies: implementation should continue using the existing `ratatui` and `crossterm` stack without introducing new external services.
