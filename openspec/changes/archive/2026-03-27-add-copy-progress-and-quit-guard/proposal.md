## Why

The archive flow already performs long-running copy work, but the in-progress screen only shows textual progress and still allows the global `Ctrl+Q` quit shortcut to terminate the app mid-copy. This makes it harder for users to judge remaining work and increases the risk of accidental interruption while media is being archived.

## What Changes

- Add a dedicated progress bar presentation to the status area so users can track copy completion visually after confirming the import.
- Change quit handling during an active copy so pressing `Ctrl+Q` requests a safe stop instead of immediately closing the app mid-copy.
- Add behavioral coverage for in-progress copy feedback and protected copy-session keyboard handling.

## Capabilities

### New Capabilities
<!-- None. -->

### Modified Capabilities
- `themed-archive-import`: copy progress must expose a visual progress bar in the status area and must intercept the global quit shortcut to stop copy work safely while a copy job is still running.

## Impact

- Affected code: `src/ui/app.rs` and `src/core/copy.rs` for status-area rendering, key handling, safe-stop signaling, and copy-state transitions.
- Affected behavior: the main archive screen status area, global shortcut handling during active copy, and graceful interruption of in-flight archive work.
- Dependencies: implementation should continue using the existing `ratatui` and `crossterm` stack without introducing new external services.
