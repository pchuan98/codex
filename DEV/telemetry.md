# 遥测与进程日志

## 功能范围

通过 CPA 启动器运行 Codex 时固定注入以下配置：

```text
-c analytics.enabled=false
-c feedback.enabled=false
-c otel.metrics_exporter='none'
```

这些配置使用上游公开配置项，不修改 Codex 的默认值。`cpa-set provider false` 只关闭 CPA Provider，仍会注入上述遥测配置。

Rust 代码只保留上游暂时没有公开开关的修改：

- TUI 不把 process tracing logs 写入 SQLite log layer。
- App Server 不把 process tracing logs 写入 SQLite log layer。

直接运行未附带这些 `-c` 参数的 `codex.exe` 时，Analytics、Feedback 和 OTEL metrics 使用上游默认值。

## SQLite 边界

关闭 process tracing log layer 不会停用 Codex 的状态数据库。Codex 仍会初始化 state runtime，并可能创建：

- `state_5.sqlite`
- `logs_2.sqlite`
- `goals_1.sqlite`
- `memories_1.sqlite`

这些数据库继续保存 session metadata、resume、goals、memories 和 agent graph 等状态。当前修改不实现 no-SQL mode。

## 实现位置

- `DEV/scripts/npm/bin/cpa.js`
  - 无论 CPA Provider 是否启用，始终注入三项公开遥测配置。

- `DEV/test.bat`
  - Debug 测试入口注入相同配置。

- `codex-rs/tui/src/startup_orchestration.rs`
  - 不注册 SQLite process log layer。
  - 保留 embedded App Server 的 `log_db` 参数管道，但固定为 `None`。

- `codex-rs/app-server/src/lib.rs`
  - 不注册 SQLite process log layer。
  - 将 `MessageProcessorArgs.log_db` 固定为 `None`。

## 不包含的修改

- 不修改 Analytics、Feedback 或 OTEL 的上游默认值。
- 不关闭 `tool_suggest` 或 `remote_plugin`。
- 不修改模型 context window。
- 不增加新的公开遥测配置项。
- 不改变 resume、fork、provider、source 或 loader 行为。

## Rebase 检查

上游提供 SQLite process tracing 的公开关闭选项后，优先改为由 CPA 启动器注入配置，并删除 TUI/App Server 初始化补丁。上游调整 Analytics、Feedback 或 OTEL 配置键时，同步修改 `cpa.js` 和 `DEV/test.bat`。
