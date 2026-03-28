## Why

Archive imports currently leave users guessing about when a long-running copy will finish and whether it is safe to remove an external disk afterward. Adding real-time copy metrics and an explicit safe-eject flow reduces uncertainty during imports and helps users avoid data loss caused by removing media before the operating system has fully released it.

## What Changes

- Add a safe external-disk eject action that is available after archive work finishes or when the selected removable device is idle and ready to be removed.
- Surface device-eject success and failure states clearly so users know whether the disk can be unplugged.
- Expand copy progress details to show current transfer rate and estimated time remaining while an import is active.
- Record and display total copy duration after an import completes so users can understand how long the archive job took.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `removable-media-discovery`: Extend removable-device handling so the app can request a safe eject and report whether the selected external disk is ready to remove.
- `themed-archive-import`: Extend copy progress reporting so the UI shows live transfer rate, estimated completion time, and final elapsed duration.

## Impact

- Affected specs: `removable-media-discovery`, `themed-archive-import`
- Affected code: removable-device platform adapters, archive copy job progress model, TUI status and completion messaging
- User impact: clearer import progress feedback and a safer post-import workflow for external media removal
