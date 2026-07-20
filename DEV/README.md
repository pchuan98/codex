# 必须知道

当前项目是在尽量维持上游 Codex 目录树的前提下进行简化和 TUI 定制的 fork。

## 注意事项

- 不需要主动处理测试和快照；不得修改测试代码。
- 功能变更时同步更新本目录下的自定义说明文档。
- 自定义功能优先放入新增文件，通过少量接线接入上游代码，降低 rebase 成本。
- 不主动运行 `just fmt`、全仓库 formatter、测试或快照更新。
- 与上游合并时使用 rebase，保持自定义提交位于最新上游提交之后。

## 自定义说明文档

- `defaults.md`：更保守的 analytics、feedback、OTEL、process log 和 feature 默认值。
- `resume.md`：resume picker 默认 All、全来源、state DB 优先加载和自动展开。
- `statusline.md`：自定义底部状态栏、Git 状态、输出速度、Codex 额度和上下文占用。
