# TUI 定制

## 输入区和布局

- 输入框、远程图片行和未聚焦侧栏输入预览不填充背景，保留上游布局、图片缩略图、选择与输入操作。
- 底部固定显示 Reasoning、Fast、模型、当前目录、Git 状态和输出速度。
- 右侧显示当前 thread 的 context 已用百分比，按最新上下文 token 数除以窗口大小计算，保留一位小数，不扣除固定基线；窗口未知或无效时显示 `--.-%`。
- 隐藏 `← for agents`、`? for shortcuts` 和 `? shortcuts` 提示；左方向键入口、`?` 帮助和用户重映射仍有效，其他提示保留。
- 隐藏 transcript 复制完成提示（包括终端转发）；复制进行中、忙碌和失败反馈保留。
- 固定布局与颜色不受 `/statusline`、其预设、`tui.status_line` 和 Status line colors 开关控制；不启动 MCP 状态栏读取，避免扩展异步结果覆盖固定状态栏。终端标题及其他界面沿用上游。
- 只读历史和纯语音界面不显示状态栏和 context。

## 数据与刷新

| 数据 | 规则 |
|---|---|
| Git | 通过 workspace command 后台查询，单命令超时 2 秒，每 5 秒刷新；仓库或状态查询失败、输出达到 512 KiB 时保留旧值；仅增删行查询失败、格式无效或输出超限时更新分支及文件数，行数显示未知 |
| Git 缓存 | 每个 widget 最多 16 个仓库、64 个目录映射、16 个在途查询；oneshot 结果归属发起查询的 widget，pending 项不淘汰 |
| Git 统计 | 修改数包含未跟踪文件；增删行数在仓库根目录执行 `git diff HEAD --numstat`，只统计已跟踪内容，即使开启 `diff.relative`，进入子目录也不会缩小统计范围；detached 显示短 commit，无 HEAD 时行数显示未知 |
| 输出速度 | 按实际 answer、plan、reasoning delta 的字符数估算 token；500ms 刷新、最近 5 个样本平均，turn 结束保留最后值；仅接收通过上游过滤的实时通知，历史回放和完整文本回填不计入 |
| Context | 使用当前 thread 最新上下文用量与窗口大小，未聚焦侧栏按自身通知更新；等待恢复用量时显示未知，左侧状态保留上游快照语义 |

## Context 状态与恢复

百分比为 `最新上下文 token 数 / 上下文窗口大小 × 100`，限制在 0% 至 100%。例如 8,000 / 100,000 显示 8.0%；累计会话消耗不参与计算。原始用量仍沿用上游数据，不修改模型预算、窗口大小或压缩阈值。

Resume、fork 和 paginated 子会话重放已有用量；metadata-only 路径缺少归属时只读取最近一轮的元数据，不为此加载消息和工具输出。后端实现见 [resume.md](resume.md)。

Side 创建期间提前到达的 token usage 通知，与启动通知一起暂存，待子会话归属安装后再路由，避免有效用量被当作未知会话丢弃。

每个 thread 的 event store 单独保存一份最新用量，不占历史事件缓冲区；事件淘汰或会话元数据刷新不会丢失它。切换界面时从该状态生成一份用量快照，回退历史时清除旧用量，等待后端更新。

## 刷新与布局

状态变化只标记待刷新，在绘制或侧栏快照前统一重建。Git 查询完成通知和 5 秒定时刷新驱动缓存更新。渲染只读取缓存，不执行 Git 命令。inline、fullscreen 和未聚焦侧栏复用上游 `StatusLine` 的测量、换行及布局缓存；只为右侧 context 预留宽度，空间不足时优先显示 context。模式和快捷键提示使用上游独立提示行，不再与 context 拼接。

## 颜色

Context 的颜色使用连续 RGB 渐变，按未取整的百分比计算。

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

- `chatwidget/custom_status_line.rs`：固定布局、颜色和速度。
- `chatwidget/custom_status_line/git_status.rs`：后台查询和 widget 自有缓存。
- `chatwidget/custom_status_line/context.rs`：聚焦输入区和侧栏共用的 context 计算。
- `bottom_pane/custom_status_line.rs`：为右侧 context 预留空间，复用上游状态栏测量和绘制。
- `bottom_pane/chat_composer/custom_status_line.rs`：保存右侧 context，独立于模式提示。
- `bottom_pane/chat_composer.rs`、`app/side_pane.rs`：输入区背景与布局接线。
- `app/side_companion.rs`：未聚焦侧栏的 context 通知更新。
- `app/thread_context_usage.rs`：独立于历史事件缓冲区的会话用量状态。

上游修改 composer、footer、token usage 或 workspace command 时检查这些接线，保留选区、图片、粘贴和多窗格行为。上游 `BranchChanges` 统计分支已提交差异，不能替代这里的工作区统计。输出速度依赖实际流式 delta。

## Fullscreen 输入性能

Fullscreen 的按键分发跳过释放事件的快捷键扫描，导航键分类每次按键只计算一次；先筛选实际匹配的快捷键，再按需计算一次输入上下文，普通粘贴字符跳过上下文中的快捷键冲突检查。保留现有自定义快捷键、PageUp/PageDown 和 transcript 交互优先级；不依赖禁用语音快捷键。
