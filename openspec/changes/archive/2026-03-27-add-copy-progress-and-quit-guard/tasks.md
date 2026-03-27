## 1. Status Progress UI

- [x] 1.1 Update the main archive status area in `src/ui/app.rs` to render a visual progress bar derived from `CopyProgress` after confirmation.
- [x] 1.2 Keep textual progress details alongside the progress bar so copied and total file counts remain visible during the archive job.
- [x] 1.3 Return to the main archive view immediately after confirmation and keep copy progress visible through status updates there.

## 2. Safe Quit Handling

- [x] 2.1 Change global `Ctrl+Q` handling so it requests a safe copy stop while a copy job is active instead of returning the app quit signal immediately.
- [x] 2.2 Wire a cancellation signal into copy execution so the app can stop at a safe file boundary and exit after the running copy has halted.

## 3. Verification

- [x] 3.1 Add or update tests covering copy-screen progress rendering inputs and active-copy quit handling.
- [ ] 3.2 Manually verify that `Ctrl+Q` still quits normally outside the copying state and is blocked only while copy work is in progress.
