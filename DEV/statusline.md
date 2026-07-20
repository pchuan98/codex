# 状态栏

## 当前设计

底部状态栏始终使用本 fork 的固定布局并默认开启；终端标题继续使用上游实现。
上游 `/statusline` 配置代码继续保留，但 `tui.status_line` 的选择不改变底部固定布局。

左侧依次显示：

- reasoning、Fast 模式和裸模型名。
- 当前目录。
- Git 分支，以及 dirty 文件数、插入和删除行数。
- streaming 输出速度。
- Codex 5h 和 weekly 额度。

额度数据只读取现有的 `codex` rate-limit snapshot，不请求额外接口。只有 weekly 数据时，仅显示 weekly 使用率和 reset：24 小时内使用 `H:MM`，超过 24 小时按向上取整的天数显示，例如 `1% 2d`。

右侧显示当前 thread 的 context 已用百分比。token usage 按 thread 缓存，切换 thread 时恢复对应值，避免沿用其他会话的数据。

## 性能策略

Git 查询通过当前 app-server 的 workspace command 通道在后台执行，并按仓库根目录缓存，最多保留
16 个仓库，渲染路径只读取缓存。这样 embedded 和 remote workspace 会在各自实际执行环境中读取
Git 状态。失败时保留旧值；没有旧值时隐藏 Git 段。Git 子进程有 2 秒超时，避免大型或异常仓库
留下后台进程。

额度 reset 时间直接使用 rate-limit snapshot 的 Unix 时间戳，不反向解析展示文本。超过上游
15 分钟 stale 阈值的额度数据会从状态栏隐藏。输出速度在 streaming 期间更新，并在 turn 结束后
保留最后一次测量值。

状态栏不包含 provider 前缀、自定义 provider 限额、DeepSeek 余额或缓存率，也不读取 `api.toml`。

## 实现边界

- `custom_status_line.rs` 负责布局、额度 reset、context 和输出速度。
- `custom_status_line/git_status.rs` 负责异步 Git 查询和缓存。
- `thread_token_usage.rs` 负责按 thread 保存 token usage。
- 其他文件只保留必要的事件和 bottom pane 接线。
