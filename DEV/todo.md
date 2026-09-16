# 上游同步

先读 [README.md](README.md)、[resume.md](resume.md)、[ui.md](ui.md) 和 [cpa.md](cpa.md)，确认当前定制范围。

1. 检查远端、分支和工作区，保留用户未提交的修改，为 `dev` 创建备份分支。
2. 执行 `git fetch upstream --tags`，检查主线、发布分支与标签的变化。
3. 切换到 `main`，执行 `git merge --ff-only upstream/main`，再切回 `dev` 执行 `git rebase main`。
4. 冲突保留上游新结构，再接回文档列出的定制行为；上游接口变化时重新审核正确性和更低耦合方案。
5. 静态检查：`git diff --check main..dev --`、`git ls-files -u`、`git status --porcelain`、`git merge-base --is-ancestor main dev`，并用 `git range-diff` 比较 rebase 前后的定制提交。
6. 按改动范围选择最小验证：TUI 使用 `cargo check --locked -p codex-tui --lib`；App Server 使用 `cargo check --locked -p codex-app-server --lib`，均在 `codex-rs` 目录执行。遵守 README 的测试和格式化限制。
7. 仅在用户要求推送时，确认结果后执行 `git push --force-with-lease origin dev`。

结果只在对话中汇报，不创建 `upstream*.md` 或逐次同步报告。说明主要上游变化、冲突处理、定制兼容性和验证结果。

编译、运行和打包统一使用 README 中的 just 入口。Codex 与 Code Mode host 必须同时构建、同目录分发；V8 版本和下载配置由构建脚本处理。
