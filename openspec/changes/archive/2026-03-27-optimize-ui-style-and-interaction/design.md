## Context

Sorted already has a functional archive TUI with device browsing, settings editing, confirmation, copy progress, and results screens. The current UI is implemented mostly as straightforward `Paragraph` and `List` blocks in `src/ui/app.rs`, with a single global help string and a single status message area reused for every kind of feedback.

That simplicity makes the code approachable, but it also means several user-facing concerns are blended together:
- focus is conveyed mainly by yellow borders or bold text,
- loading, empty, validation, and success states all share the same status area,
- confirmation, progress, and results screens present dense text without much hierarchy,
- keyboard guidance is present but not contextual to the active screen or field.

This change touches multiple screens and capabilities, so we need a design that keeps visual polish consistent without overcomplicating the current ratatui-based architecture.

## Goals / Non-Goals

**Goals:**
- Introduce a more intentional visual hierarchy for primary content, focused controls, and actionable system feedback.
- Make empty, loading, validation, success, and failure states easier to distinguish during the archive workflow.
- Keep interaction patterns consistent across main, settings, confirmation, progress, and results screens.
- Preserve keyboard-first operation while making guidance more contextual and easier to scan.
- Implement the polish within the existing terminal UI stack and app state model.

**Non-Goals:**
- Replacing ratatui with another UI framework.
- Adding mouse interaction or a full form/widget library abstraction.
- Redesigning archive business logic, device discovery, or copy planning behavior beyond user-facing presentation and flow guidance.
- Introducing theme customization or user-selectable color schemes in this change.

## Decisions

### Define a small shared presentation system for the TUI

We will introduce a lightweight set of shared presentation helpers for block styles, emphasis colors, semantic statuses, and reusable text treatments instead of styling each screen ad hoc. This gives the app a consistent visual language while keeping implementation localized to UI rendering code.

This shared system should cover:
- focused vs. unfocused containers,
- semantic emphasis for info, warning, error, and success messages,
- reusable section titles and inline labels,
- helper text treatment for keyboard hints and secondary explanations.

Alternative considered:
- Continue tweaking styles inline inside each draw function. Rejected because it would make cross-screen polish inconsistent and hard to maintain as more states are added.

### Split global feedback into clearer contextual surfaces

The current single `status_message` string is useful, but it is doing too much. We will keep a global status area for short-lived app feedback while also giving each screen richer local feedback where it matters most:
- the source/device area should show explicit loading, empty, and unavailable guidance,
- settings should render validation and preview feedback close to editable values,
- confirmation should summarize what is about to happen and how to proceed,
- copy progress and results should emphasize outcome state before details.

This keeps the app model simple while improving clarity without requiring a large state-management rewrite.

Alternative considered:
- Replace the current status model with a full notification/event center. Rejected because it adds complexity beyond the needs of this TUI and is not required to deliver the requested UX improvement.

### Rework screen layouts around scannable sections instead of dense paragraphs

We will refactor the main screens to use clearer groupings, spacing, and summary blocks so users can identify the current step quickly. In practice this means:
- the main screen should better separate source browsing from archive summary and next-step readiness,
- settings should visually isolate editable fields, live preview, and save guidance,
- confirmation should present a concise checklist-style review,
- progress and results should promote outcome, counts, and next actions over raw text dumps.

Alternative considered:
- Add more modal overlays for each step. Rejected because the current navigation is already screen-based; clearer composition inside existing screens is enough and avoids extra interaction complexity.

### Keep keyboard-first interactions, but scope guidance to the active context

The app already supports a strong keyboard workflow. Instead of adding many new shortcuts, we will improve discoverability by tailoring on-screen guidance to the current screen and focus. Shared shortcuts can stay global, while screen-specific actions should be shown only when relevant.

Alternative considered:
- Expand the shortcut set substantially. Rejected because more shortcuts would increase cognitive load and would not address the main issue, which is clarity rather than capability.

### Prefer incremental state extensions over a major app-model rewrite

Where richer UI feedback needs new data, we will add narrow state structures or helper methods near the existing `App` model rather than introducing a separate presentation layer. This keeps implementation aligned with the current architecture and reduces regression risk.

Alternative considered:
- Introduce a full presenter/view-model split. Rejected because the codebase is still compact, and the requested change can be delivered with focused refactoring.

## Risks / Trade-offs

- [More UI states can make rendering logic harder to follow] -> Mitigate by extracting small rendering helpers and semantic style utilities instead of growing each draw function inline.
- [Terminal color/styling choices may not look identical across environments] -> Favor strong layout hierarchy and label wording first, with color acting as reinforcement rather than the sole signal.
- [Richer feedback may require touching several flows in `App`] -> Keep behavioral changes narrow and covered by targeted tests around status, validation, and screen transitions.
- [Improved results/progress presentation could expose long failure details awkwardly] -> Provide a concise summary first and then render detailed failures in a wrapped, scroll-friendly text block if needed.
