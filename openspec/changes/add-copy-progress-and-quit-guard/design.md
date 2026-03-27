## Context

Sorted already tracks copy progress through `CopyProgress` updates emitted from the copy worker and rendered on the `Screen::Copying` view in `src/ui/app.rs`. That means the app has enough state to report progress, but the current presentation is limited to a text line and the global key handler still treats `Ctrl+Q` as an unconditional app exit, even while copy work is active in the background.

This change is small in scope but touches both rendering and input handling on a long-running workflow. The design needs to preserve the current asynchronous copy model, improve progress clarity, and prevent accidental termination while media files are still being archived.

## Goals / Non-Goals

**Goals:**
- Render a visual progress bar on the copy screen using the existing copy progress state.
- Block `Ctrl+Q` while a copy session is still active, without changing the shortcut's behavior on other screens.
- Surface clear feedback when a quit attempt is intercepted during copy so users understand why the app stays open.
- Keep the implementation localized to the current TUI app model and event loop.

**Non-Goals:**
- Adding pause, cancel, or resume controls for copy jobs.
- Changing the copy execution engine or introducing task cancellation primitives.
- Redesigning unrelated archive, settings, or device discovery workflows.

## Decisions

### Render progress from copy state instead of introducing a new tracking model

The app already receives `copied_files` and `total_files` updates from the copy worker. We will derive the progress ratio directly from that state and render it as a dedicated progress bar plus supporting text on the copy screen.

This keeps progress UI synchronized with the existing background copy channel and avoids introducing a second progress data structure that could drift from the actual worker state.

Alternative considered:
- Track byte-level progress or per-file subprogress. Rejected because the current copy callback only reports file-level completion, and byte-level tracking would require a significantly deeper rewrite of the copy engine.

### Gate global quit behavior on active copy state

We will treat `copy_updates.is_some()` or the active copying screen state as the source of truth that a protected copy session is in progress. While that state is active, `Ctrl+Q` will no longer return the app-level quit signal. Instead, the handler will set an explicit status message that explains quitting is unavailable until the copy finishes.

This approach preserves the current global shortcut structure while limiting the behavioral change to the risky window where quitting could interrupt archiving.

Alternative considered:
- Disable all global shortcuts during copy. Rejected because only quit is safety-critical here; broad shortcut suppression would make the app feel inconsistent and could hide useful non-destructive guidance updates.

### Reuse existing status/help surfaces for blocked quit feedback

When the user presses `Ctrl+Q` during copy, the app will update the existing feedback text so the blocked action is immediately visible. The copy screen help text should also explicitly say that quit is unavailable while copying.

This keeps the UX understandable without adding a modal or new transient notification system for a single guarded action.

Alternative considered:
- Show a dedicated confirmation dialog for blocked quit attempts. Rejected because the user cannot confirm an actual quit while copying anyway, so a modal would add friction without unlocking a meaningful choice.

## Risks / Trade-offs

- [File-count progress can feel coarse for very large files] -> Mitigate by pairing the progress bar with copied/total file counts so users still get a trustworthy completion indicator.
- [Guarding only `Ctrl+Q` leaves forced terminal termination outside app control] -> Mitigate by documenting and messaging the guarded shortcut clearly; handling OS-level kills is out of scope for this change.
- [Input handling could become inconsistent if copy-active state is inferred from the wrong flag] -> Mitigate by deriving the guard from the same state used to drive the copying screen lifecycle and covered by targeted tests.
