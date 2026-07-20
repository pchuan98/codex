# Fork Defaults

## 当前设计

这个文件记录本 fork 直接改写的默认行为。目标是减少默认启动时的外部连接、本地 process log 记录和不需要的界面负担。

这些默认值不依赖用户配置兜底，也不新增公开开关：

- 默认不启用 analytics。
- 默认不启用 feedback。
- 默认不把 OTEL metrics 发到 Statsig。
- 默认不启用 `tool_suggest`。
- 默认不启用 `remote_plugin`。
- TUI 输入框不渲染自定义背景色，保留原有 padding 和布局。
- TUI 和 app-server 不再把 process tracing logs 挂到 SQLite log layer。

注意：这不是 no-SQL mode。Codex 仍会初始化 state runtime，并可能创建 `state_5.sqlite`、`logs_2.sqlite`、`goals_1.sqlite`、`memories_1.sqlite`。这些文件属于 session metadata、resume、goals、memories、agent graph 等本地状态，不和 process log layer 混在一起处理。

## 实现位置

- `codex-rs/core/src/config/mod.rs`
  - `analytics_enabled` 在没有显式配置时固定为 `Some(false)`。
  - `feedback_enabled` 在没有显式配置时默认为 `false`。

- `codex-rs/config/src/types.rs`
  - `OtelConfig::default().metrics_exporter` 从 `Statsig` 改成 `None`。

- `codex-rs/core/src/config/otel.rs`
  - 缺省 `metrics_exporter` 解析为 `None`，避免配置缺失时重新落回 Statsig。

- `codex-rs/features/src/lib.rs`
  - `ToolSuggest` 默认关闭。
  - `RemotePlugin` 默认关闭。

- `codex-rs/tui/src/bottom_pane/chat_composer.rs`
  - 不再用 `user_message_style()` 给输入框和远程图片行渲染背景色。
  - 不改 layout padding，输入区仍按原有 inset 计算。

- `codex-rs/tui/src/lib.rs`
  - 保留 `log_db` 参数管道给 embedded app-server，但本进程固定传 `None`。
  - 不再注册 SQLite log layer。

- `codex-rs/app-server/src/lib.rs`
  - 不再注册 SQLite log layer。
  - `MessageProcessorArgs.log_db` 固定为 `None`。

## Rebase notes

- 不要重新添加 `--log-to-sqlite`。它只控制 process log layer，不能阻止 state runtime 创建 SQLite 文件，容易造成误解。
- 如果以后要做 no-SQL mode，需要单独设计 state runtime 的可选初始化和所有调用方 fallback。
- 不要把这些默认行为扩散成新的公开配置或 CLI 参数；当前目标是 fork 默认更保守。
- 不要主动运行大范围 formatter、快照更新或测试；需要检查时只跑最小范围 `cargo check -p codex-tui --bin codex-tui`。

## Verification

- 在 `codex-rs` 下做编译检查：`cargo check -p codex-tui --bin codex-tui`。
