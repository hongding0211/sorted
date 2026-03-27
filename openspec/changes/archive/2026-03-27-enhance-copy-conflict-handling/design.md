## Context

The current import flow has two separate readiness gaps. In the main screen, `confirm_or_advance` only validates source selection before switching to the confirmation screen, so users can reach confirmation with an empty theme and only get bounced back later when `start_copy` runs. Separately, the app will proceed even if the final archive folder it intends to create already exists, which risks merging a new import into an old archive destination the user may have meant to keep isolated.

This change cross-cuts pre-confirmation validation, core copy planning, error reporting, and regression tests. We need one explicit readiness policy so confirmation is only reachable with required inputs and the import is blocked whenever the final archive destination already exists.

## Goals / Non-Goals

**Goals:**
- Require a non-empty theme before the app transitions from the main screen into confirmation.
- Define one clear preflight rule for destination conflicts during archive import.
- Detect when the final archive destination folder already exists and return a user-facing failure that explains why the import is blocked.
- Add targeted tests that distinguish missing-theme gating and pre-existing final-destination rejection.

**Non-Goals:**
- Adding interactive conflict resolution or per-file overwrite prompts.
- Changing destination-root validation rules unrelated to import readiness.
- Introducing automatic rename behavior when the final archive destination already exists.

## Decisions

### Validate theme presence before entering confirmation

The main-screen advance action will treat theme entry as a prerequisite for confirmation, just like source selection. If the theme is blank after trimming, the app will stay on the main screen, move focus to the theme field, and show the existing warning there instead of allowing a round-trip into confirmation.

Alternative considered:
- Keep the current `start_copy` guard only. Rejected because it allows an avoidable screen transition that implies the import is ready when it is not.

### Block pre-existing final archive destinations during planning

The copy pipeline will validate the resolved `archive_root` before confirmation and before execution. If that exact final destination directory already exists, planning will fail with a stable, understandable error message so the user is stopped before confirmation proceeds into a misleading import.

Alternative considered:
- Check only at file-write time. Rejected because the conflict is about import readiness, not an individual file, and should be blocked before the copy session begins.

### Keep the guard close to existing archive-plan resolution

The existing planning path already resolves destination semantics and is used by both confirmation preview and copy startup. The new existence guard belongs there so the UI and runtime share the same rule without duplicating destination-path logic in multiple places.

Alternative considered:
- Add a UI-only existence check. Rejected because `start_copy` would still need to reimplement the same validation and could drift from what confirmation shows.

## Risks / Trade-offs

- [Users who relied on checking confirmation before typing a theme will now be stopped earlier] -> Accept the tighter gate because theme is already required for a valid archive path, and focus the input immediately so the next step is obvious.
- [Users can no longer intentionally merge into an existing final archive folder] -> Accept the stricter rule because the clarified requirement is to block whenever that final folder already exists.
- [Filesystem state can change between confirmation and copy start] -> Re-run the same plan validation when copy starts so a folder created in the meantime is still blocked.
