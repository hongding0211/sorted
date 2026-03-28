## 1. Tree Navigation Logic

- [x] 1.1 为 source tree 的左键处理补充“选中未展开子节点时回到最近父节点并折叠父层”的逻辑。
- [x] 1.2 为 settings tree 的左键处理补充与 source tree 一致的父层回退和折叠逻辑。
- [x] 1.3 在树重建后按目标父路径重新定位选中索引，避免回退后选中错位。

## 2. Interaction Copy

- [x] 2.1 更新主界面 source tree 的帮助文案，说明左键既可折叠当前节点，也可回到父层。
- [x] 2.2 更新设置页 destination tree 的帮助文案，保持与主界面一致的导航描述。

## 3. Verification

- [x] 3.1 为 source tree 增加左键折叠当前节点和左键回退父层的测试。
- [x] 3.2 为 settings tree 增加对应的父层回退与折叠测试。
- [x] 3.3 运行相关测试，确认树形浏览、设置页选择和已有展开/折叠行为没有回退。
