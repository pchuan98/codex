# 开发入口

本 fork 保留上游目录结构，定制 CPA 启动、跨 Provider 恢复和 TUI 状态栏。

## 约束

- 不新增或修改测试、快照；禁止测试、formatter 和 snapshot 命令，包括 `cargo test`、nextest、`cargo fmt`、`rustfmt`、`cargo insta`。
- 只使用 `DEV/justfile` 的 run、pack、publish 入口及静态检查，不运行其他 just 命令。
- Rust 验证限于相关 crate 的最小范围 `cargo check` 或文档允许的 `cargo build`。
- 功能变更同步更新对应说明；定制逻辑优先放独立文件，减少上游接线。
- 同步上游使用 rebase，流程见 [todo.md](todo.md)。

## 常用命令

需要 PowerShell 7.4+、Node/npm 和 just。Windows 使用本机 MSVC Rust 工具链；Linux 使用同架构 Docker 构建环境。在 `DEV` 目录执行：

| 命令 | 用途 |
|---|---|
| `just run [Codex 参数...]` | 编译 Debug Codex 和 Code Mode host，通过 CPA 启动 |
| `just pack` | 打包本机 Debug 版本 |
| `just pack --release` | 打包本机 Release 版本 |
| `just publish npm` | 发布当前打包结果到 npm |
| `just publish gitea` | 发布当前打包结果到 Gitea |

例如 `just run resume <id>`、`just run --enable tui_workspace`。运行目录为 `DEV`，用 `-C <目录>` 指定工作区。

## 说明

- [package.md](package.md)：构建、产物和发布。
- [cpa.md](cpa.md)：CPA 地址、密钥及启动设置。
- [resume.md](resume.md)：跨 Provider 恢复规则与实现位置。
- [ui.md](ui.md)：输入区和固定状态栏。

`upstream` 为 `https://github.com/openai/codex.git`，`origin` 为 `https://github.com/pchuan98/codex.git`。本地 `main` 跟踪 `upstream/main`，定制提交保留在 `dev`，不使用 `origin/main` 作为上游基线。
