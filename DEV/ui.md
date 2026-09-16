# TUI 定制

## 输入区和布局

- 输入框、远程图片行和未聚焦侧栏输入预览不填充背景，保留上游布局、图片缩略图、选择与输入操作。
- 底部固定显示：Reasoning、Fast、模型、当前目录、Git 状态、输出速度、5 小时及 weekly 额度、reset 倒计时。
- 右侧显示当前 thread 的 context 已用百分比，保留一位小数。
- 隐藏 `← for agents`、`? for shortcuts` 和 `? shortcuts` 提示；左方向键入口、`?` 帮助和用户重映射仍有效，其他提示保留。
- 固定布局与颜色不受 `/statusline`、其预设、`tui.status_line` 和 Status line colors 开关控制。终端标题及其他界面沿用上游。

## 数据与刷新

| 数据 | 规则 |
|---|---|
| Git | 通过 workspace command 后台查询，单命令超时 2 秒，每 5 秒刷新；失败或输出达到 512 KiB 时保留旧值 |
| Git 缓存 | 每个 widget 最多 16 个仓库、64 个目录映射、16 个在途查询；oneshot 结果归属发起查询的 widget，pending 项不淘汰 |
| Git 统计 | 修改数包含未跟踪文件；增删行数用 `git diff HEAD --numstat`，只统计已跟踪内容；detached 显示短 commit，无 HEAD 时行数显示未知 |
| 输出速度 | 按实际 answer、plan、reasoning delta 的字符数估算 token；500ms 刷新、最近 5 个样本平均，turn 结束保留最后值；仅接收通过上游过滤的实时通知，历史回放和完整文本回填不计入 |
| 额度 | 读取原始 rate-limit snapshot，不额外请求；账户读取只取响应主 snapshot，滚动通知按到达顺序更新；超过 15 分钟隐藏 |
| Reset | 24 小时内显示 `H:MM`，更长时间向上取整为天数 |
| Context | 复用上游按 thread 路由和恢复时重放的 token usage；未聚焦侧栏按自身通知更新，左侧状态保留上游快照语义 |

状态变化只标记待刷新，在绘制或侧栏快照前统一重建；Git 完成通知及 5 秒定时刷新驱动缓存更新和倒计时。渲染只读取缓存，不执行 Git 命令。inline 和 fullscreen 共用上游 `status_line_layout` 和测量缓存；宽度不足时缩短模式提示，再不足则只显示 context。

## 颜色

额度、context 和 reset 使用连续 RGB 渐变，按未取整的数值计算：

| 数值 | 颜色 |
|---:|---|
| 0% | 绿色 |
| 27.2% | 黄色 |
| 53.6% | 橙色 |
| 80% 及以上 | 红色 |

| 内容 | 颜色 |
|---|---|
| Reasoning `minimal` / `low` | 绿色 |
| Reasoning `medium` | 黄色 |
| Reasoning `high` | RGB 橙色 |
| Reasoning `xhigh` / `max` / `ultra` | 红色 |
| Reasoning 缺失或未知 | 深灰色 |
| Git 修改数 / 新增行 / 删除行 | 黄色 / 绿色 / 红色 |
| 输出速度 / 模型名 | 青色 / 浅蓝色 |

## 实现与维护

路径均相对 `codex-rs/tui/src/`：

- `chatwidget/custom_status_line.rs`：固定布局、颜色、额度和速度。
- `chatwidget/custom_status_line/git_status.rs`：后台查询和 widget 自有缓存。
- `chatwidget/custom_status_line/context.rs`：聚焦输入区和侧栏共用的 context 计算。
- `bottom_pane/chat_composer/custom_status_line.rs`：右侧 context 与模式提示组合。
- `bottom_pane/chat_composer.rs`、`app/side_pane.rs`：输入区背景与布局接线。
- `app/side_companion.rs`：未聚焦侧栏的 context 通知更新。

上游修改 composer、footer、rate limit、token usage 或 workspace command 时检查这些接线，保留选区、图片、粘贴和多窗格行为。上游 `BranchChanges` 统计分支已提交差异，不能替代这里的工作区统计；额度需要原始 reset timestamp，速度需要实际流式 delta。
