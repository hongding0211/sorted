## 1. Copy Progress UI

- [ ] 1.1 Update the copying screen in `src/ui/app.rs` to render a visual progress bar derived from `CopyProgress`.
- [ ] 1.2 Keep textual progress details alongside the progress bar so copied and total file counts remain visible during the archive job.
- [ ] 1.3 Adjust copy-screen help or status text to explain that the screen remains active until the copy finishes.

## 2. Protected Quit Handling

- [ ] 2.1 Change global `Ctrl+Q` handling so it is intercepted while a copy job is active instead of returning the app quit signal.
- [ ] 2.2 Show a clear status message when a quit attempt is blocked during copy so the user understands why the app did not exit.

## 3. Verification

- [ ] 3.1 Add or update tests covering copy-screen progress rendering inputs and active-copy quit handling.
- [ ] 3.2 Manually verify that `Ctrl+Q` still quits normally outside the copying state and is blocked only while copy work is in progress.
