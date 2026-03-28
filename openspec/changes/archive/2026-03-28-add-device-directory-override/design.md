## Context

当前导入流程里，`DeviceInfo.display_name` 既承担设备发现后的展示用途，也被 `build_archive_plan` 直接拿来生成最终归档目录名。这个耦合让“探测到什么名字，就必须落什么目录”成为默认行为；一旦操作系统返回的卷标不准确，用户在确认导入前没有任何修正空间。

现有 UI 已经有 source tree 和 theme 输入两个主流程交互点，确认弹窗也会展示 destination preview，因此最小改动路径不是去改设备发现本身，而是在导入会话层增加一个可选 override，并让预览与归档计划统一消费它。核心约束是：override 只能替换目标目录名，不能改变设备选择、挂载路径验证、设备可用性判断或安全弹出逻辑。

## Goals / Non-Goals

**Goals:**
- 为当前导入会话增加一个可选设备目录名 override，留空时继续回退到自动探测的设备名。
- 让 destination preview、确认弹窗和最终 `ArchivePlan` 对 override 使用一致的优先级规则。
- 保持设备发现、设备列表展示、设备校验和拷贝执行逻辑的主体不变，只在会话建模和路径生成处插入 override。
- 增加覆盖空 override、有效 override 和归一化后的 override 路径结果的测试。

**Non-Goals:**
- 不尝试修改操作系统看到的磁盘卷标，也不写回设备元数据。
- 不为每个磁盘建立持久化 override 映射，也不在设置页新增全局配置项。
- 不改变已有 theme、source 选择或 destination root 设置的保存语义。

## Decisions

### Store the override in import-session state instead of persisted settings
override 的语义是“本次导入想要使用的目录名”，而不是应用级偏好，因此最合适的归属是 `ImportSession`。这样可以与 `theme`、`selected_source` 一样，在进入一次导入流程时编辑、在刷新设备或结束导入后自然重置，避免引入新的配置 schema 和跨设备映射问题。

备选方案是把 override 放进 `ArchiveSettings` 并持久化到 `config.toml`。这个方案适合全局默认值，但会引入“一个 override 应该对应哪个设备”的额外建模成本，也容易让用户误以为配置会长期改写某块磁盘的名字，因此不采用。

### Resolve the effective archive directory name at planning time
真正用于生成目录的值应当在 `build_archive_plan` 或其上游计划构建时统一决议：若 override 非空，则使用 override；否则回退到 `DeviceInfo.display_name`。这样预览和实际执行都走同一套规划逻辑，不会出现 UI 预览显示 override、但 copy 实际仍落到原设备名目录的分叉。

备选方案是只在 UI 里拼接预览字符串，而让底层继续使用 `display_name`。这个方案实现表面更快，但会制造用户可见结果和真实写盘结果不一致的严重风险，因此不采用。

### Keep device discovery immutable and treat override as presentation-plus-output metadata
设备发现仍然产出真实的 `DeviceInfo`，包括 `id`、`display_name`、`mount_path` 和 `availability`。override 不应反向修改 discovery 结果，而是作为导入会话里的补充字段，在确认弹窗和归档计划中被读取。这能保证刷新设备、重新选择 source、请求 eject 等依旧围绕真实设备信息工作。

备选方案是直接覆写 `DeviceInfo.display_name`。这个方案会把“用户临时修正目录名”和“系统探测到的设备标签”混成一个字段，使后续状态提示、刷新行为和调试日志更难解释，因此不采用。

### Add explicit UI copy that the override affects only the destination folder name
override 容易被理解成“重命名磁盘”，所以主界面帮助文案和确认弹窗都需要明确说明它只改变 destination preview / archive path，不会修改磁盘本身名称。这样可以减少用户对行为范围的误判。

备选方案是只增加一个输入框，不额外解释。这个方案虽然节省界面空间，但会把行为语义留给用户猜测，在涉及文件归档路径时不够稳妥，因此不采用。

## Risks / Trade-offs

- [新增输入字段会让主界面焦点切换更复杂] -> 复用现有焦点模型，把 override 作为和 theme 同级的文本输入区域，并补充帮助文案与焦点切换测试。
- [用户可能输入空白或大量非法路径字符] -> 沿用现有 `normalize_path_component` 规则，并在 planning 阶段对归一化后为空的结果继续报错。
- [override 只做会话级状态，用户重新连接同一设备后仍需再次输入] -> 先满足最直接的修正目录名场景，未来若确实需要长期映射，再单独提一个持久化 change。
- [确认页与主界面如果展示不同字段，容易让用户怀疑是否真的生效] -> 在两个位置都展示 override 或其回退结果，并让 preview 直接来自统一的计划构建逻辑。

## Migration Plan

这次变更不涉及配置文件迁移，也不需要修改已有归档目录结构。部署时只需保证旧配置仍能正常加载；如果实现回滚，只需删除导入会话中的 override 字段并恢复原有 preview / planning 逻辑。

## Open Questions

- override 输入是放在主界面与 `theme` 并列，还是放在确认弹窗中二次编辑；实现时建议优先选择主界面编辑，以便用户在进入确认前就能看到完整 preview。
