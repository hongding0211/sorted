## Why

The current removable-device discovery logic is too strict on Linux and Linux-derived NAS environments. Some systems correctly expose mounted removable media, but report a disk kind that the app currently filters out, causing valid external storage to disappear from the archive workflow.

## What Changes

- Relax Linux removable-media discovery so mounted, readable removable filesystems are not excluded just because the platform reports an unknown disk kind.
- Refactor discovery filtering around platform-specific inclusion rules so future compatibility fixes can be added without spreading ad hoc checks through the discovery flow.
- Add behavioral coverage for Linux environments where removable media is mounted and readable but reported with incomplete or vendor-specific metadata.

## Capabilities

### New Capabilities

### Modified Capabilities
- `removable-media-discovery`: Linux discovery must include mounted removable filesystems even when the underlying platform reports incomplete disk-kind metadata, while preserving the existing requirement to exclude unreadable or unsupported sources.

## Impact

- Affected code: `src/platform/discovery.rs` and its tests.
- Affected behavior: removable-media enumeration on Linux and Linux-derived NAS platforms such as QNAP should surface mounted external storage more reliably.
- Dependencies: no new dependencies are required; the change continues to build on the existing `sysinfo`-based platform adapter.
