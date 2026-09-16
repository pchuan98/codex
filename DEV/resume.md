# Resume Picker

## 功能范围

本 fork 只修改 Resume 的 Provider 行为：

- Resume picker 不按 Provider 过滤会话。
- 切换 API Provider、CPA 或 OAuth 登录方式后，仍能看到本地历史会话。
- 本地恢复时，历史 Provider 与当前 Provider 不同则使用当前配置的 Provider、模型和 Reasoning，避免历史 Provider 已移除或模型不兼容导致恢复失败。
- 同 Provider 的恢复优先级沿用上游；显式模型配置覆盖继续优先。

## 查询语义

App Server 将 `model_providers = None` 解释为当前 Provider，因此“不限制 Provider”必须发送空数组：

```text
model_providers = []
```

列表查询修改只应用于 Resume picker。跨 Provider 恢复同时覆盖本地 picker、`resume <id>` 和 `--last`。
远程工作区、已加载线程的重连及 Fork 保持上游规则。

## 复用的上游行为

以下功能完全使用上游实现：

- Resume picker 默认的 `Cwd`/`All` 范围和 CLI 参数。
- 会话来源过滤；不会默认包含非交互会话。
- Preview 手动展开和 `Ctrl+E` 行为。
- Preview、Page 和 Transcript 请求调度。
- State DB 首屏加载和 Store 回退。
- Session 列表分页、搜索和归档。
- Fork picker、远程工作区及已加载线程的重连。

## 实现位置

- `codex-rs/tui/src/resume_picker.rs`
  - Resume 查询使用独立的 `ProviderFilter::AllProviders`，映射为 `model_providers = []`。
  - 保留上游 `ProviderFilter::Any` 的缺省查询语义，避免影响远程 Fork picker。
- `codex-rs/tui/src/app_server_session/rollout_history.rs`
  - 默认恢复本地会话前，通过 `thread/read` 读取 Provider 元数据，不加载 turns。
  - Provider 不同时复用 `OverrideFromCurrentConfig`；相同时保留历史模型设置。
  - 在 `resume_thread_with_permission_overrides` 中处理，保留上游权限恢复入口；目标 Provider 复用上游 `history_model_provider` 的显式选择和服务端默认值解析，再明确写入恢复请求，避免上游省略隐式 Provider 时丢失跨 Provider 切换。
  - 不改写历史文件或持久化 Provider 配置；读取失败时保留错误，不盲目切换。

## Rebase 检查

2026-10-08：恢复入口保留上游的工具传输配置校验，再执行本地跨 Provider 判断。上游新增的历史分页和恢复行为继续沿用。

上游修改 App Server 的 `model_providers` 缺省/空数组语义、`thread/read` Provider 元数据或恢复设置优先级后，重新审核 Resume 接线。不要重新实现 picker 的范围、来源、Preview 或快捷键行为。
