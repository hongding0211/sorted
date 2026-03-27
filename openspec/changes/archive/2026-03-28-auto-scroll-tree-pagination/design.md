## Context

Sorted 的主界面 source browser 和设置页 destination browser 都会把树状目录扁平化后交给 `ratatui::widgets::List` 渲染。当前实现只保存“选中了第几项”，没有保存“列表当前从哪一行开始显示”，因此当条目数超过面板可视高度时，用户继续向下移动选中项会让当前行滑出视口。

## Goals / Non-Goals

**Goals:**
- 让两个树列表在选中项超出当前可视窗口时自动调整滚动偏移。
- 保持现有上下移动、左右展开折叠和设置页确认逻辑不变。
- 让自动滚动行为足够简单、可预测，并且容易通过单元测试覆盖。

**Non-Goals:**
- 不新增 PageUp/PageDown 或其他新的导航快捷键。
- 不修改目录树扁平化规则，也不改变异步目录加载机制。
- 不重做现有的列表样式体系。

## Decisions

### Persist per-list scroll offsets in app state
两个树列表都需要记住自己的 offset，这样重绘、切屏和树结构展开后可以延续上一次视口位置，而不是每次都从顶部重新计算。

### Compute the next offset from selection and viewport height
自动滚动本质上只需要三个输入: 当前 offset、当前选中行、当前面板可见高度。将这部分提炼成纯函数后，可以单独验证“选中项在上边界外”“选中项在下边界外”“选中项已经可见”等情况。

### Use stateful ratatui list rendering
列表改为通过 `ListState` 渲染，这样 UI 层仍然沿用现有 `List` 组件，只是把选中项和 offset 一起传入。

## Risks / Trade-offs

- 可视高度依赖当前面板尺寸，如果边框或布局发生变化，offset 计算要和实际可用行数保持一致。
- source tree 和 settings tree 各自维护 offset，必须避免切换页面时误用另一份状态。

## Validation

- 为 offset 计算增加单元测试，覆盖边界内、边界上方、边界下方三类情况。
- 运行完整 `cargo test`，确认现有树导航与设置页回归测试仍然通过。
