## 1. Safe Eject Flow

- [x] 1.1 Add a platform-level safe-eject adapter for removable devices and normalize success, unsupported, and failure outcomes for the app layer.
- [x] 1.2 Prevent eject requests while the selected device is the source of an active archive copy and surface a clear “cannot eject during copy” status.
- [x] 1.3 Refresh removable-device state after a successful eject so the device list and status area reflect that the disk is ready to remove or no longer mounted.

## 2. Copy Progress Metrics

- [x] 2.1 Extend the archive copy progress model to track copied bytes, timing samples, smoothed transfer rate, and total elapsed duration.
- [x] 2.2 Compute and publish estimated time remaining only when enough progress data exists to produce a stable estimate.
- [x] 2.3 Preserve final elapsed duration in the completed copy result so the UI can show how long the archive job took.

## 3. TUI Status and Validation

- [x] 3.1 Update the main archive status area to show transfer rate and estimated time remaining while copy is active, and replace ETA with elapsed duration after completion.
- [x] 3.2 Add a user-triggered safe-eject action and success or failure messaging to the removable-device workflow.
- [x] 3.3 Add or update tests covering active-copy eject blocking, successful and failed eject reporting, live progress metrics, and completion duration messaging.
