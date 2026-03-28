## Why

当前归档目录的设备名段完全依赖自动探测得到的磁盘名称；当系统返回的卷标不准确、为空，或和用户想要保留的素材来源名称不一致时，用户只能接受错误目录名，影响归档结果的可读性和后续检索。为当前导入会话增加一个可选 override，可以在不破坏自动探测流程的前提下，让用户在导入前修正最终落盘目录名。

## What Changes

- 在归档导入流程中新增一个可选的“device directory override”输入，默认为空，留空时继续使用自动探测到的设备名。
- 当用户提供 override 时，归档预览、确认信息和最终输出目录都优先使用 override，而不是直接使用探测到的设备显示名。
- 明确 override 只影响当前导入会话生成的目标目录名，不改变设备发现结果、挂载路径或安全弹出行为。
- 为 override 的校验、预览和归档路径生成补充测试，确保空值回退、非法字符归一化和用户可见预览保持一致。

## Capabilities

### New Capabilities

### Modified Capabilities
- `removable-media-discovery`: 设备发现后的归档工作流需要允许用户在保留挂载路径与可用性状态的同时，手动覆盖用于归档目录生成的设备名。
- `tui-experience-polish`: 导入主流程需要为设备目录 override 提供可见的编辑、预览和确认反馈，避免用户误以为 override 会修改磁盘本身名称。

## Impact

- 受影响代码主要包括 `src/core/archive.rs`、`src/core/types.rs`、`src/core/copy.rs` 和 `src/ui/app.rs` 中的导入会话状态、预览文案与归档计划生成逻辑。
- 设备发现实现 `src/platform/discovery.rs` 仍然负责自动探测真实设备信息，但其输出会被导入会话层的 override 逻辑包装使用。
- 需要补充单元测试与 TUI 交互测试，覆盖 override 为空、override 生效、以及 override 含需归一化字符时的路径结果。
