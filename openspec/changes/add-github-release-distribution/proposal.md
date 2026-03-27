## Why

Sorted 已经具备跨平台运行能力，但当前缺少一套稳定、可重复的发布分发流程，导致用户难以直接获取 macOS、Linux、Windows 的可执行产物。现在补齐基于 GitHub 的 release 流程和安装脚本，可以把“能开发”推进到“能交付、能安装、能升级”。

## What Changes

- 增加基于 GitHub Actions 的多平台构建与发布流程，为 macOS、Linux、Windows 生成可下载的 release 产物。
- 增加 GitHub Release 发布约定，包括 tag 驱动的发布、产物命名、校验信息和最新版本发现方式。
- 增加面向终端用户的安装脚本，支持从 GitHub 下载当前平台对应的最新 release 并安装到本地。
- 增加 README 中的安装与发布说明，明确用户安装入口和维护者发布入口。

## Capabilities

### New Capabilities
- `github-release-distribution`: 通过 GitHub Actions 和 GitHub Releases 构建、打包并分发 Sorted 的多平台发布产物。
- `release-install-bootstrap`: 通过 shell 安装脚本发现并下载 GitHub 上的最新 release，为当前主机安装合适的 Sorted 可执行文件。

### Modified Capabilities

## Impact

- 受影响代码和目录包括 `.github/workflows/`、发布辅助脚本目录、README，以及可能新增的打包元数据文件。
- 受影响系统包括 GitHub Actions、GitHub Releases 和终端环境中的 `curl`/`tar`/`unzip` 等常见安装依赖。
- 需要定义跨平台二进制命名、压缩格式、tag 触发策略与安装目标路径等发布约定。
