## Context

Sorted 当前是一个跨平台 Rust CLI/TUI 项目，但仓库中还没有可复用的 CI/CD 发布流程，也没有标准化的安装入口。发布分发这次变更会同时触达构建脚本、GitHub Actions、GitHub Releases、README 和用户本地安装路径，属于跨模块的交付能力建设。

维护者希望尽量把分发流程集成到 GitHub 上，并且为终端用户提供一个“从 GitHub 下载最新 release 并安装”的脚本体验。这意味着设计既要考虑维护者如何稳定地产出版本，也要考虑用户如何在 macOS、Linux、Windows 之外的 shell 环境里得到一致的安装体验。

## Goals / Non-Goals

**Goals:**
- 让维护者通过 Git tag 或手动触发方式在 GitHub 上生成多平台 release 产物。
- 为 macOS、Linux、Windows 提供一致的二进制命名和压缩打包约定。
- 提供一个 shell 安装脚本，用 GitHub Releases API 自动发现最新版本并下载当前平台对应产物。
- 让 README 中明确发布和安装入口，降低首次使用门槛。

**Non-Goals:**
- 不在本次设计中引入 Homebrew、Scoop、AUR、Winget 等平台原生包管理渠道。
- 不在本次设计中处理代码签名、公证或商店分发。
- 不改变 Sorted 现有运行逻辑、配置格式或 TUI 交互。

## Decisions

### 使用 GitHub Actions 作为唯一发布编排入口
选择 GitHub Actions，是因为用户明确希望集成在 GitHub 上，且它能直接与 tag、Release、artifact 和仓库权限模型配合。相比自建 CI 或额外 SaaS，GitHub Actions 能以更低维护成本完成构建矩阵、产物上传和 release 发布。

备选方案：
- 本地手动构建并上传 Release：实现简单，但不可重复且容易遗漏平台。
- 第三方发布工具链：功能更强，但会引入额外集成成本和维护负担。

### 使用目标平台构建矩阵并统一产物命名
发布流程将基于平台矩阵分别构建：
- `x86_64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`
- `aarch64-apple-darwin`
- `x86_64-apple-darwin`

每个平台产物统一使用带版本号和目标三元组的文件名，例如：
- `sorted-v1.2.3-x86_64-unknown-linux-gnu.tar.gz`
- `sorted-v1.2.3-x86_64-pc-windows-msvc.zip`

这样做能让安装脚本仅通过版本号和平台映射计算下载地址，也方便用户手动识别。

备选方案：
- 仅发布裸二进制：下载后平台差异不明显，也不方便附带 README 或 LICENSE。
- 使用平台各自命名规范但不统一模板：会增加安装脚本的分支复杂度。

### 安装脚本通过 GitHub Releases API 发现最新版
安装脚本将通过 GitHub API 读取 `latest` release 元数据，解析当前 shell 所在平台与 CPU 架构，选择匹配产物下载并安装到默认目录。这样可以满足“像从 GitHub 上下载最新 release” 的诉求，同时避免把固定版本硬编码进脚本。

安装默认策略：
- 默认安装到 `~/.local/bin`，如果该目录不存在则创建。
- 若 `SORTED_INSTALL_DIR` 已设置，则优先使用该目录。
- 若用户传入显式版本参数，则优先下载该版本；否则下载 latest。

备选方案：
- 抓取 release HTML 页面：实现脆弱，结构变化风险高。
- 将最新版本硬编码在脚本中：每次发版都需同步改脚本，不符合自动化目标。

### 发布工作流生成校验信息并附带安装脚本入口
每个 release 除平台压缩包外，还应发布 SHA256 校验清单，并在 README 中提供基于 `curl ... | sh` 或“先下载脚本再执行”的安装入口。虽然校验不一定在脚本首版强制执行，但 release 至少要输出校验信息，为后续增强留出路径。

备选方案：
- 不生成校验信息：上线更快，但发布可信度较弱。
- 在首版安装脚本中强制校验：更安全，但会显著增加跨平台 shell 兼容逻辑。

## Risks / Trade-offs

- [跨平台构建失败率较高] → 先覆盖仓库当前最可行的目标三元组，并把矩阵定义集中在 workflow 中，便于后续增减目标。
- [GitHub API 限流或匿名访问失败] → 安装脚本允许用户通过环境变量覆盖仓库地址和版本，并保留直接下载指定 URL 的兜底方式。
- [shell 安装脚本在极简系统依赖不全] → 在脚本启动阶段明确检查 `curl`、`tar`、`unzip` 等依赖并给出可操作错误。
- [Windows 用户无法直接执行 shell 脚本] → 发布能力本身仍覆盖 Windows 二进制分发，本次安装脚本以 POSIX shell 终端为目标，README 需要明确 Windows 手动下载方式。

## Migration Plan

1. 新增发布 workflow、打包脚本和安装脚本。
2. 在 GitHub 仓库中创建首个语义化版本 tag，验证 workflow 能生成所有 release 产物。
3. 更新 README 安装与发布说明，让用户入口切换到 GitHub Releases。
4. 如发布流程出现严重问题，可通过删除错误 tag/release、修复 workflow 后重新发布来回滚；应用运行逻辑不受影响。

## Open Questions

- 是否需要在首版就支持 Linux ARM64 与 Windows ARM64。
- 是否要把安装脚本发布为独立固定 URL，例如仓库内 `scripts/install.sh` 的 raw 地址。
- 后续是否要扩展到 Homebrew / Scoop 等生态集成。
