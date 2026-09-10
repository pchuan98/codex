# CPA API 模式

## 目标

CPA 作为 OpenAI Responses API 反向代理使用。日常功能包括模型请求、工具调用、SSE 输出、远程压缩、会话恢复和额度显示。

本文件不把 API Key 认证改成 ChatGPT 登录认证。账户切换、工作区管理和额度重置不属于当前适配范围。

## 配置

用户级 `config.toml` 使用独立的 `cpa` Provider ID：

```toml
model_provider = "cpa"

[model_providers.cpa]
name = "OpenAI"
base_url = "https://codex.pchuan.top/v1"
env_key = "CPA_API_KEY"
wire_api = "responses"
```

`name = "OpenAI"` 使当前 Provider 能力判断启用 OpenAI V2 远程压缩。Provider ID 仍为 `cpa`，会话记录不会写成 `openai`。

在启动 Codex 的 PowerShell 中设置密钥：

```powershell
$env:CPA_API_KEY = '你的 CPA Key'
```

密钥不写入仓库或 `config.toml`。

CPA 启动器还会使用上游公开配置项关闭 Analytics、Feedback 和 OTEL Statsig metrics。该行为与 CPA Provider 开关相互独立；详情见 `DEV/telemetry.md`。

## 已确认的接口

CPA 已确认支持：

- `POST /v1/responses`。
- `POST /v1/responses/compact`。
- Responses 工具调用。
- SSE 流式响应。
- `x-codex-*` 额度响应头。

Upstream update (2026-09-10): commits `3dc1e2a58` and `1ac689cc7` route remote compaction through streamed `/responses` requests containing `compaction_trigger` items and remove the legacy `/responses/compact` implementation. The previously confirmed `/responses/compact` response is no longer sufficient evidence of CPA compatibility. CPA support for the new streamed compaction protocol still requires verification; this sync did not send live requests to CPA.

## Experimental context management in API mode

The fork allows API providers to activate history/notes without a ChatGPT login:

```toml
[features.context_management]
experimental_mode = true
```

Keep the CPA provider configuration above, including `name = "OpenAI"`. Activation
still requires model metadata with `supports_experimental_context = true` and
respects managed restrictions on the token-budget feature. The bundled catalog
currently declares this capability for `gpt-6-astra`. Other model IDs are not
automatically opted in.

API mode skips the ChatGPT subscription and `/backend-api/codex` URL checks.
History/notes requests reuse the active model provider's base URL and credential
resolution, including `CPA_API_KEY`; no ChatGPT credential is synthesized. For
the example base URL, requests target `/v1/alpha/history/v2/*` and
`/v1/alpha/notes/v2/*`. The server must implement these routes and the model's
context-management protocol. Client activation alone does not establish end-to-end
compatibility. ChatGPT login mode retains the upstream subscription eligibility.

Implementation: `core/src/session/token_budget.rs` activates the token budget and
history/notes; `ext/history-notes/src/extension.rs` registers the extension without
requiring Codex-backend login. Both retain the OpenAI provider-name check and
exclude AWS providers. Recheck these gates and provider authentication during
future upstream rebases.

Initial code validation was static only; no tests, snapshots, or formatters were
run or modified. During the subsequent September 10 deployment session, the
refreshed CPA catalog advertised the capability and live history/notes requests
completed successfully. This confirms the observed session path, not all
protocol or multi-account behavior.

## 会话恢复

Resume picker 不按 Provider 过滤。切换到 `cpa` 后仍显示其他 Provider 创建的本地会话。

模型、Reasoning 和 Provider 的恢复优先级沿用上游，不额外强制覆盖为当前配置。

TUI 的“不限制 Provider”必须发送空数组：

```text
model_providers = []
```

App Server 将 `model_providers = None` 解释为当前 Provider，因此不能用 `None` 表示全部 Provider。

Fork picker、直接恢复和 `--last` 保留上游规则。

## 额度

CPA 的同一个 Responses 响应可能包含多套额度：

- `codex`：套餐总额度。
- `codex_bengalfox`：GPT-5.3-Codex-Spark 模型专用额度。

上游账户额度读取支持按 `limit_id` 返回多份快照，但模型响应事件仍写入单个 `latest_rate_limits`。为避免 CPA 的模型专用额度覆盖套餐总额度，SSE 响应头包含 `codex` 时只把该快照送入会话事件；没有 `codex` 时保留服务端返回的其他额度。

这个选择只影响模型响应产生的本地单快照状态，不修改服务端额度、计费和限流。通过账户额度接口获得的多额度数据仍沿用上游处理。

额度来自模型响应头。新进程需要先完成一次模型请求，`/status` 和状态栏才有当前额度数据。

## 相关代码

- `codex-rs/codex-api/src/rate_limits.rs`：解析默认额度和模型专用额度，并确保默认 `codex` 快照最后写入。
- `codex-rs/codex-api/src/sse/responses.rs`：在单快照会话事件前优先选择 `codex`。
- `codex-rs/tui/src/resume_picker.rs`：Resume picker 的 Provider 查询范围。
- `codex-rs/tui/src/chatwidget/custom_status_line.rs`：底部状态栏只接受 `codex` 快照用于紧凑展示。

## 上游同步检查

上游修改以下行为后重新检查本适配：

- Provider capability 和远程 compact 判断。
- Responses SSE 额度事件和 `latest_rate_limits` 保存结构。
- App Server 的 `model_providers` 空数组和缺省值语义。
- Resume 恢复模型配置的优先级。

如果模型响应事件也改为按 `limit_id` 保存多份快照，再删除 SSE 的单快照筛选。
