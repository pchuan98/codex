# Resume Picker

## 功能范围

本 fork 只修改 Resume 的 Provider 行为：

- Resume picker 不按 Provider 过滤会话。
- 切换 API Provider、CPA 或 OAuth 登录方式后，仍能看到本地历史会话。
- 模型、Reasoning 和 Provider 的恢复优先级沿用上游，不再强制覆盖为当前配置。

## 查询语义

App Server 将 `model_providers = None` 解释为当前 Provider，因此“不限制 Provider”必须发送空数组：

```text
model_providers = []
```

该行为只应用于 Resume picker。Fork picker、直接恢复和 `--last` 保持上游规则。

## 复用的上游行为

以下功能完全使用上游实现：

- Resume picker 默认的 `Cwd`/`All` 范围和 CLI 参数。
- 会话来源过滤；不会默认包含非交互会话。
- Preview 手动展开和 `Ctrl+E` 行为。
- Preview、Page 和 Transcript 请求调度。
- State DB 首屏加载和 Store 回退。
- Session 列表分页、搜索和归档。
- Fork picker、直接恢复及 `--last`。

## 实现位置

- `codex-rs/tui/src/resume_picker.rs`
  - Resume 查询使用独立的 `ProviderFilter::AllProviders`，映射为 `model_providers = []`。
  - 保留上游 `ProviderFilter::Any` 的缺省查询语义，避免影响远程 Fork picker。

## Rebase 检查

上游修改 App Server 的 `model_providers` 缺省/空数组语义后，重新审核 Resume 查询接线。不要重新实现模型配置恢复、picker 的范围、来源、Preview 或快捷键行为。
