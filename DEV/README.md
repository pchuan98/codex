# 必须知道

当前项目是在尽量维持上游 Codex 目录树的前提下进行简化和 TUI 定制的 fork。

## 注意事项

- 不处理测试和快照；不得新增或修改测试代码、snapshot 基线或生成的 snapshot 文件。
- 功能变更时同步更新本目录下的自定义说明文档。
- 自定义功能优先放入新增文件，通过少量接线接入上游代码，降低 rebase 成本。
- 禁止运行任何 `just ...` 命令，不限于 `just fmt`、`just test`、`just fix` 和 schema/snapshot 命令。
- 禁止运行测试、formatter 和 snapshot 更新，包括 `cargo test`、nextest、`cargo fmt`、`rustfmt` 与 `cargo insta`。
- 需要验证时只运行相关文档明确允许的最小范围 `cargo build` 或 `cargo check`。
- 与上游合并时使用 rebase，保持自定义提交位于最新上游提交之后。

## Git 远端与分支

- `upstream` 指向官方仓库 `https://github.com/openai/codex.git`。
- 本地 `main` 跟踪 `upstream/main`，只用于保存最新官方主线。
- `origin` 指向 fork 仓库 `https://github.com/pchuan98/codex.git`。
- 本地 `dev` 跟踪 `origin/dev`，fork 提交只保留在 `dev`。
- 不使用 `origin/main` 作为上游基线。

首次配置缺失的官方远端和本地 `main` 时执行：

```powershell
git remote add upstream https://github.com/openai/codex.git
git fetch upstream main
git branch --track main upstream/main
```

以后同步官方主线时执行：

```powershell
git fetch upstream main
git switch main
git merge --ff-only upstream/main
git switch dev
git rebase main
```

rebase 会改写 `dev` 的提交历史。确认结果后，如需更新 fork 的远端分支，执行
`git push --force-with-lease origin dev`。

## 本地二进制

- Windows 下 `cx.bat` 当前运行 `codex-rs/target/debug/codex-copy.exe`，不是 Cargo 默认更新的 `codex.exe`。
- `cargo build` 成功不代表 `codex-copy.exe` 已更新；用 `cx` 验证前必须先比较两个文件的修改时间。
- 优先直接运行新生成的 `codex.exe` 验证。覆盖 `codex-copy.exe` 或修改 `cx.bat` 是单独操作，不能在普通 build 后默认执行。

## 自定义说明文档

- `telemetry.md`：CPA 遥测参数和 process tracing log layer。
- `resume.md`：Resume picker 的跨 Provider 查询和恢复配置。
- `ui.md`：输入区外观和自定义底部状态栏。
- `cpa.md`：CPA API 模式的 Provider 配置、远程压缩、会话恢复和额度处理。
- `package.md`：CPA npm 包、版本计算和启动方式。

## Debug 测试入口

使用 CPA Provider 运行当前源码的 Debug 版本：

```powershell
.\DEV\test.bat <CPA_API_KEY>
```

脚本只为当前进程设置 `CPA_API_KEY`，然后在 `codex-rs` 中执行 `cargo run --bin codex --`。Provider 和遥测参数与 CPA 启动器一致，不修改持久化配置。
