# CPA API 模式

CPA 启动器使用独立的 `cpa` Provider ID，并注入以下上游公开配置：

```toml
model_provider = "cpa"

[model_providers.cpa]
name = "OpenAI"
base_url = "https://codex.pchuan.top/v1"
env_key = "CPA_API_KEY"
wire_api = "responses"
```

`name = "OpenAI"` 用于上游 Provider 能力识别；Provider ID 仍为 `cpa`。API Key 认证不会变成 ChatGPT 登录认证。Responses API、工具调用、SSE 和远程压缩使用上游实现，是否可用由 CPA 服务端决定。

## 会话恢复

Resume picker 不按 Provider 过滤。本地跨 Provider 恢复使用当前 Provider、模型和 Reasoning，同 Provider 恢复模型优先级沿用上游。具体范围与限制见 `resume.md`。

## 额度

SSE 的额度解析、选择和顺序全部沿用上游，不再优先筛选 `codex`。自定义状态栏直接读取收到的 rate-limit snapshot，沿用现有窗口、reset 倒计时和过期判断，不发起额外请求。

## 上下文与遥测

已移除 API 模式启用 TokenBudget、history/notes 的 fork 补丁及 `cpa-set context exp` 设置。实验上下文的支持条件沿用上游，CPA 不再主动注入实验模式。

普通 `cpa-set context <SIZE>` 仍可设置 `model_context_window`；它不保证服务端支持对应窗口。

遥测和日志通过上游配置处理，CPA 不再硬编码关闭参数，见 `telemetry.md`。
