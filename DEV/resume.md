# Resume

## 当前设计

- Resume picker 默认进入 `All`，仍可切回 `Cwd`。
- session 仍按上游规则匹配当前 provider；不实现跨-provider 查询。
- source 范围包含 CLI、VS Code、exec 和 app-server。
- 首屏优先从 state DB 加载；空结果时回退扫描修复路径。
- `Ctrl+E` 展开模式默认开启，移动选中项时自动展开；再次按下可关闭。
- 自动展开预览在选择稳定 150ms 后加载，连续移动时只保留最后一个请求，避免预览阻塞分页。
- `resume --last` 和 fork picker 保持上游 provider、source 和 loader 语义。

state DB 快速路径只在首屏完全为空时回退扫描修复。若数据库已有部分记录、但另有 rollout
尚未入库，picker 不会主动做全量扫描；这是首屏速度优先的已知取舍。

## 实现边界

- `resume_scope.rs` 只定义 resume 的全来源范围和默认 All 行为。
- `resume_picker.rs` 负责 state DB 首屏、自动展开和 latest-wins 预览调度，继续使用上游 provider filter。
- 不修改测试、快照或 formatter 输出。
