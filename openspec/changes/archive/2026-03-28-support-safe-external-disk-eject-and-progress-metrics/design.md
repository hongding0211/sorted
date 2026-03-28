## Context

Sorted already has two pieces that make this change feasible: a removable-media discovery layer that normalizes detected external disks into `DeviceInfo`, and an archive copy pipeline that emits progress updates back into the TUI. What is missing is a post-copy device action that coordinates with the operating system before the user unplugs the disk, plus richer progress metrics that help users understand how fast the job is moving and how much longer it will take.

This work crosses platform adapters, copy-progress state, and the shared status rendering path. The design needs to preserve the current asynchronous copy workflow, keep platform-specific eject behavior behind a narrow interface, and avoid showing misleading ETA data when the copy rate is not yet stable.

## Goals / Non-Goals

**Goals:**
- Let the app request a safe eject for the selected removable device after the archive flow is no longer actively copying from it.
- Surface clear success, failure, and unsupported-eject feedback so users know whether it is safe to unplug the disk.
- Extend copy progress state to include current transfer rate, estimated time remaining, and final elapsed duration.
- Keep the richer progress UI in the existing main status area instead of introducing a separate progress screen.

**Non-Goals:**
- Supporting unsafe force-unmount behavior when the operating system rejects a safe eject request.
- Adding background hotplug listeners that automatically remove devices from the UI without refresh.
- Implementing byte-perfect instantaneous bandwidth telemetry; a stable user-facing estimate is more important than sub-second precision.

## Decisions

### Add a platform eject adapter behind the existing removable-media abstraction

Safe eject should live next to device discovery in the platform layer. The app already relies on platform-specific code to identify removable media, so the same boundary is the right place to translate a selected `DeviceInfo` into an OS-level eject request. The TUI and archive flow should only need a single outcome model such as success, failed with message, or unsupported.

This keeps shell commands or system APIs out of UI code and lets each platform choose the least risky eject mechanism available.

Alternatives considered:
- Trigger platform eject commands directly from the TUI event handler. Rejected because it would spread OS branching into the UI layer.
- Add eject behavior to the copy worker. Rejected because eject is a device-management action, not part of file transfer execution.

### Gate safe eject on an idle device state instead of allowing it during active copy

The app should only offer or execute safe eject when the selected removable device is not participating in an active archive import. During copy, the source disk is still in use and any eject request is either guaranteed to fail or risks confusing the user. The simplest rule is to treat the active copy session as a lock on the selected source device and defer eject until the job completes or fails.

Alternatives considered:
- Allow eject during copy and depend on the OS to reject it. Rejected because it creates avoidable failure noise and invites unsafe user behavior.
- Automatically eject immediately after success. Rejected because users may want to inspect the disk or run another import before removing it.

### Compute progress metrics from byte totals and smoothed timestamps

Current progress is file-count oriented, which is good for coarse completion but not enough for ETA. The copy pipeline should accumulate copied bytes and timestamped samples so the UI can derive a smoothed transfer rate and an estimated remaining duration from the remaining bytes. Once the copy completes, the session should retain a start/end duration summary for the final success state.

Using byte totals plus smoothing avoids the misleading jumps that come from extrapolating ETA from file counts alone, especially when large media files dominate the workload.

Alternatives considered:
- Derive ETA only from completed file count. Rejected because file sizes vary too much for a trustworthy estimate.
- Show raw instantaneous byte rate from every callback. Rejected because it will oscillate heavily and make ETA feel noisy.

### Reuse the shared status panel for both in-progress and completed timing details

The status area already presents progress and completion messages, so it should remain the single surface for live rate, ETA, and final elapsed time. During copy, the panel can show the progress bar, copied counts, transfer rate, and estimated completion time. After completion, it can swap ETA for a concise “completed in …” summary while preserving the final success or interruption message.

Alternatives considered:
- Introduce a dedicated modal or detail pane for advanced metrics. Rejected because the new information is useful at a glance and does not justify extra navigation.

## Risks / Trade-offs

- [Smoothed rate can lag behind sudden throughput changes] -> Favor slightly conservative ETA updates so values stay readable instead of twitchy.
- [Platform eject support differs across macOS, Windows, and Linux] -> Normalize outcomes into a small result enum and keep platform-specific error text available for diagnostics.
- [Users may expect eject to disappear devices immediately from the list] -> Refresh discovery after a successful eject so the UI converges with the operating system state.
- [ETA may be unavailable at the very beginning of a copy] -> Show a placeholder until enough byte/timestamp samples exist to produce a stable estimate.

## Migration Plan

No data migration is required. The change updates runtime behavior in the platform discovery/eject adapter, copy progress model, and TUI rendering only. Rollback is limited to removing the eject action and reverting to the previous progress summary fields.

## Open Questions

- Which platform eject mechanism is already most consistent with the existing dependency set: native APIs already in use, or shelling out to standard system tools where necessary?
- Should the post-copy elapsed time remain visible until the next archive session starts, or only in the latest completion message?
