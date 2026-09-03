# TUI 界面定制

## 功能范围

本 fork 包含两项 TUI 界面修改：

- 输入框和远程图片行不渲染用户消息背景色。
- 底部使用固定的自定义状态栏。

## 输入区外观

TUI 不使用 `user_message_style()` 绘制输入框和远程图片行的背景。

本修改只影响 composer 输入区域：

- 保留原有 padding、尺寸和布局。
- 不改变输入、粘贴、附件和远程图片功能。
- 不改变历史消息的样式。
- 不改变菜单、弹窗和其他使用 `user_message_style()` 的界面。

实现位置：`codex-rs/tui/src/bottom_pane/chat_composer.rs`。

## 状态栏布局

状态栏左侧依次显示：

- Reasoning 等级、Fast 模式和模型名。
- 当前目录。
- Git 分支、修改文件数、新增行数和删除行数。
- Streaming 输出速度。
- Codex 5 小时和 weekly 额度。
- 额度 reset 剩余时间。

右侧显示当前 thread 的 context 已用百分比。

状态栏固定启用。上游 `/statusline` 配置和终端标题逻辑继续保留，`tui.status_line` 不改变本 fork 的底部布局。

## 状态栏数据

### Git

- 通过 App Server 的 workspace command 通道在后台查询。
- 按仓库根目录缓存，最多保存 16 个仓库；目录到仓库的映射最多保存最近解析的 64 个目录。
- 当前目录是否为仓库根目录在显示时比较，不写入共享的仓库缓存。
- Git 子进程超时为 2 秒。
- 查询失败时保留旧值；没有旧值时隐藏 Git 段。
- 渲染过程只读取缓存，不执行 Git 命令。

### 输出速度

- Streaming 期间统计输出 token 速度。
- 每 500ms 最多刷新一次显示。
- 使用最近 5 个样本计算平均速度。
- Turn 结束后保留最后一次速度。

### 额度

- 只读取已有的 `codex` rate-limit snapshot，不发起额外请求。
- 同一 SSE 响应包含多个额度族时，只把默认 `codex` 额度送入单快照会话状态；响应没有 `codex` 时保留模型专用额度。
- 同时存在 5 小时和 weekly 数据时显示两者。
- 只有 weekly 数据时只显示 weekly。
- 超过 15 分钟的 snapshot 不显示。
- 24 小时内的 reset 使用 `H:MM`，超过 24 小时按向上取整的天数显示。

### Context

- 右侧显示 context 已用百分比，保留一位小数。
- 复用上游按 thread 路由和恢复时重放的 token usage 通知，不维护第二份本地线程缓存。
- 切换 thread 时由上游事件管道恢复对应数值，不沿用其他会话的数据。

## 颜色

额度百分比、context 占用和 reset 倒计时使用连续 RGB 颜色。颜色在以下数值处达到指定颜色：

| 数值 | 颜色 |
|---:|---|
| 0% | 绿色 |
| 27.2% | 黄色 |
| 53.6% | 橙色 |
| 80% | 红色 |

区间内按 RGB 分量线性插值，80% 以上保持红色。显示文本可以取整，颜色按未取整的原始数值计算。

Reasoning 使用固定颜色：

| Reasoning | 颜色 |
|---|---|
| `minimal`、`low` | 绿色 |
| `medium` | 黄色 |
| `high` | 橙色 |
| `xhigh`、`max`、`ultra` | 红色 |
| 缺失或未知值 | 深灰色 |

其他固定颜色：

- Git 修改文件数为黄色。
- Git 新增行数为绿色。
- Git 删除行数为红色。
- 输出速度为青色。
- 模型名为浅蓝色。

## 实现位置

- `codex-rs/tui/src/bottom_pane/chat_composer.rs`
  - 移除输入框和远程图片行的背景。
  - 接入状态栏右侧 context 显示。

- `codex-rs/tui/src/chatwidget/custom_status_line.rs`
  - 负责状态栏布局、颜色、额度、context 和输出速度。

- `codex-rs/tui/src/chatwidget/custom_status_line/git_status.rs`
  - 负责 Git 查询和缓存。

- 其他 TUI 文件
  - 只保留状态更新、事件传递和 bottom pane 接线。

## 不包含的修改

- 不修改模型 context window。
- 不修改 `tool_suggest` 和 `remote_plugin` 的默认值。
- 不修改 Resume picker 和 Fork picker。
- 不修改遥测和 process log 行为。
- 不为额度增加网络请求。
- 不把 Git 查询放入渲染线程。

## Rebase 检查

上游修改 composer、footer、状态栏、rate limit、token usage 或 workspace command 后，重新检查对应接线。保留上游结构，只恢复本文件列出的界面行为。
