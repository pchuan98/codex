# 本地会话恢复

## 行为

- 本地 Resume picker、按名称恢复和 `--last` 可发现不同 Provider、登录方式创建的历史。
- 历史 Provider 与当前 Provider 不同时，使用当前配置的 Provider、模型和 Reasoning 继续旧聊天，包含显式 `-m` 的情况。
- 同 Provider 沿用上游恢复优先级：保留历史模型，显式模型参数可以覆盖。历史模型不可用时，用 `resume <id> -m <可用模型>` 或恢复后 `/model` 切换。
- 远程、Cloud、Fork、侧聊天和已加载会话重连沿用上游规则。
- 不改写历史文件或持久化 Provider 配置；读取失败时保留错误。

## 实现

| 位置 | 职责 |
|---|---|
| `tui/src/app_server_session/resume_provider.rs` | 仅为普通本地恢复解析当前目标 Provider |
| `tui/src/app_server_session/rollout_history.rs` | 保留维护锁和 unread 清除顺序，复用锁后的单次元数据读取判断跨 Provider，并明确写入请求 |
| `tui/src/resume_picker.rs`、`tui/src/lib.rs` | 仅本地 Resume 关闭 `current_account_only`；列表 Provider 查询复用上游 `ProviderFilter::All` |
| `app-server/src/resume_provider.rs` | 本地服务允许显式覆盖到当前有效 Provider |
| `app-server/src/config_manager.rs` | 调用策略后完整执行上游配置和组织策略校验 |

表中路径均相对 `codex-rs/`。App Server 的本地策略只在进程内服务和本地 managed daemon 开启；独立 stdio、WebSocket 服务保持上游策略。

Provider 查询的空数组表示全部，省略则使用服务端默认值。跨 Provider 请求必须明确写目标 Provider，才能使用当前模型和 Reasoning 而不回填历史设置。

## 维护边界

扩大历史发现范围不会绕过账户、后端归属和组织策略检查。启用上游跨账户恢复检查后，不符合条件的历史仍可能只能查看。

上游修改 Provider 查询语义、`thread/read` 元数据或恢复优先级时，检查上述接线。列表分页、范围、搜索、预览、归档和权限流程继续复用上游。
