# 遥测与日志配置

Codex 源码中的遥测和日志行为全部沿用上游。CPA 启动器不注入 Analytics、Feedback 或 OTEL 关闭参数。

需要关闭可配置的遥测时，将 `DEV/config.toml.example` 中对应段合并到实际使用的 `<CODEX_HOME>/config.toml`；未设置 `CODEX_HOME` 时为 `~/.codex/config.toml`。不要覆盖已有配置或重复 TOML 表。

```toml
[analytics]
enabled = false

[feedback]
enabled = false

[otel]
metrics_exporter = "none"
```

配置由上游 Codex 直接加载，与 CPA Provider 是否启用无关。`cpa.toml` 只存 CPA 启动器设置，不存 Codex 的遥测配置。

上游没有公开关闭 SQLite process tracing log layer 的配置项。因此本 fork 不再关闭该 layer，process logs 与状态数据库正常使用上游实现；`metrics_exporter` 只控制 metrics，不关闭其他日志 exporter。
