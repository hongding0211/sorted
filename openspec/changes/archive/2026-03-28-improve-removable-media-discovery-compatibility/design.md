## Context

The discovery layer currently uses a shared `sysinfo::Disks` snapshot and then applies lightweight platform filters to decide which entries become `DeviceInfo`. This is simple, but the Linux branch currently couples two different signals: whether the device is removable and whether `sysinfo` can classify the disk kind. On Linux-derived NAS systems, mounted removable media may still surface with an unknown disk kind, which causes otherwise valid archive sources to be dropped.

The code already keeps platform-specific behavior inside `src/platform/discovery.rs`, so this is a good place to make the Linux rule more tolerant without changing the rest of the archive flow. The main constraint is that we still need to exclude unreadable or non-mounted sources and preserve macOS-specific behavior that already relies on `/Volumes`.

## Goals / Non-Goals

**Goals:**
- Detect mounted removable filesystems more reliably on Linux and Linux-derived NAS platforms.
- Keep platform-specific discovery rules localized so future compatibility work is easy to extend.
- Preserve the existing `DeviceInfo` contract and downstream UI/archive behavior.
- Add tests that cover mounted removable devices with incomplete metadata.

**Non-Goals:**
- Introduce hotplug event subscriptions or background polling changes.
- Add MTP/PTP support for devices without mounted filesystems.
- Replace `sysinfo` with a new cross-platform discovery dependency.

## Decisions

### Encapsulate per-platform inclusion rules in focused helpers
Discovery should continue to take a `sysinfo::Disk` list, but platform-specific inclusion decisions should be expressed through small helper functions instead of one monolithic conditional. This keeps the adapter readable and makes future exceptions or heuristics easier to add without affecting other platforms.

Alternatives considered:
- Keep a single `should_include_disk` conditional and tweak the Linux branch inline. Rejected because the current issue shows that platform-specific nuance will keep growing.
- Split each OS into its own file immediately. Rejected for now because the current logic is still small enough to keep in one module.

### Prefer removable + mounted/readable filesystem checks over disk-kind classification on Linux
Linux discovery should treat `is_removable()` plus a valid mount-point directory as the primary inclusion signal. Disk kind metadata is useful when present, but it should not be a hard requirement because NAS and embedded environments may expose removable media through generic or unknown types.

Alternatives considered:
- Continue requiring a non-unknown disk kind. Rejected because it excludes valid mounted removable media on QNAP-like environments.
- Trust mount-point shape alone and ignore removability. Rejected because that risks surfacing non-removable bind mounts or internal storage mounted in custom paths.

### Preserve post-discovery availability validation as a separate concern
The app should keep building `DeviceInfo` and then validating availability from the mount path, rather than folding availability checks into Linux-specific heuristics. This preserves the existing contract between discovery and the UI, and avoids creating platform-specific `DeviceAvailability` behavior.

Alternatives considered:
- Encode more Linux-specific status inside `DeviceAvailability`. Rejected because the UI only needs to know whether the source can be browsed and imported.

## Risks / Trade-offs

- [Some Linux environments may still misreport removability] -> Keep the rule centered on mounted removable filesystems for now and add targeted tests so future adjustments have a safe place to land.
- [Different NAS vendors may mount external media under vendor-specific paths] -> Rely on `sysinfo` mount points rather than hard-coded Linux mount roots.
- [Platform logic may still grow over time] -> Keep inclusion logic behind helper functions so moving to per-OS modules remains straightforward if complexity increases.

## Migration Plan

No data migration is required. Implementation updates the discovery adapter and its tests only. If the revised Linux heuristic proves too broad, rollback is limited to restoring the previous inclusion rule in `src/platform/discovery.rs`.

## Open Questions

- Do we want additional Linux-specific heuristics later for environments that fail to set `is_removable()` correctly but still mount media under known external-storage roots?
