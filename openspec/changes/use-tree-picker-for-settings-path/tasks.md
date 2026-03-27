## 1. Settings Path Browser State

- [x] 1.1 Add settings-specific directory tree state and initialization logic so the settings screen can browse destination directories without mutating the main source browser state.
- [x] 1.2 Model the distinction between the persisted destination root, the currently browsed candidate directory, and the confirmed settings value used for save validation.

## 2. Settings Screen Interaction

- [x] 2.1 Update the settings screen layout to show the destination path browser, the current destination root value, and helper text that explains browse, confirm, and save actions.
- [x] 2.2 Implement settings-screen navigation for moving through directories and expanding or collapsing nodes with the same interaction model used by the main source browser.
- [x] 2.3 Implement the action that confirms the currently highlighted directory as the `Destination Root` without immediately persisting it to disk.

## 3. Validation And Regression Coverage

- [x] 3.1 Reuse existing destination-root validation and save logic for tree-selected paths, including clear error feedback when a confirmed path cannot be created or written.
- [x] 3.2 Add or update tests covering settings-tree navigation, separation between candidate and confirmed destination paths, and successful save/load behavior after selecting a destination through the settings browser.
