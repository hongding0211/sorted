## Context

Sorted already tracks copy progress through `CopyProgress` updates emitted from the copy worker and rendered in `src/ui/app.rs`. That means the app has enough state to report progress, but the current presentation is limited to a text line and the global key handler still treats `Ctrl+Q` as an unconditional app exit, even while copy work is active in the background.

This change is small in scope but touches both rendering and input handling on a long-running workflow. The design needs to preserve the current asynchronous copy model, improve progress clarity, and prevent accidental termination while media files are still being archived.

## Goals / Non-Goals

**Goals:**
- Render a visual progress bar in the main status area using the existing copy progress state.
- Intercept `Ctrl+Q` while a copy session is still active so the app can stop the copy safely before quitting.
- Surface concise progress and stop-in-flight feedback without forcing a dedicated copy screen.
- Keep the implementation localized to the current TUI app model and event loop.

**Non-Goals:**
- Adding pause, cancel, or resume controls for copy jobs.
- Changing the copy execution engine or introducing task cancellation primitives.
- Redesigning unrelated archive, settings, or device discovery workflows.

## Decisions

### Render progress from copy state inside the status area

The app already receives `copied_files` and `total_files` updates from the copy worker. We will derive the progress ratio directly from that state and render it as a compact progress bar plus supporting text inside the shared status panel after the user confirms the import.

This keeps progress UI synchronized with the existing background copy channel and avoids introducing a second progress data structure that could drift from the actual worker state.

Alternative considered:
- Track byte-level progress or per-file subprogress. Rejected because the current copy callback only reports file-level completion, and byte-level tracking would require a significantly deeper rewrite of the copy engine.

### Turn global quit into a safe-stop request during active copy

We will treat `copy_updates.is_some()` as the source of truth that a protected copy session is in progress. While that state is active, `Ctrl+Q` will no longer return the app-level quit signal immediately. Instead, the handler will flip a shared cancellation flag, let the worker stop at the next file boundary, and then exit once the copy thread reports completion.

This approach preserves the current global shortcut structure while limiting the behavioral change to the risky window where quitting could interrupt archiving.

Alternative considered:
- Disable `Ctrl+Q` outright until the copy completes. Rejected because it removes the user's ability to interrupt a long-running copy session on purpose.

### Reuse existing status surfaces instead of a dedicated copy screen

When the user confirms the import, the app should return to the main archive view and let the shared status panel carry the active progress bar, progress summary, and current-file detail. When the user requests stop via `Ctrl+Q`, the same status area can show that shutdown is in progress.

This keeps the UX compact and avoids creating a separate in-progress screen that pulls attention away from the main archive context.

Alternative considered:
- Show a dedicated confirmation dialog for blocked quit attempts. Rejected because the user cannot confirm an actual quit while copying anyway, so a modal would add friction without unlocking a meaningful choice.

## Risks / Trade-offs

- [File-count progress can feel coarse for very large files] -> Mitigate by pairing the progress bar with copied/total file counts so users still get a trustworthy completion indicator.
- [Stopping is only checked between files, not mid-file] -> Mitigate by treating file boundaries as the safe interruption point and making that behavior deterministic in tests.
- [Input handling could become inconsistent if copy-active state is inferred from the wrong flag] -> Mitigate by deriving the guard from the same receiver/cancellation state used to drive copy lifecycle updates and covered by targeted tests.
