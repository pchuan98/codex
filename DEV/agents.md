# 原生子代理的独立 Provider

角色文件支持 `model_provider`、`model_providers`，以及可选的
`model_catalog_json`、`model_context_window`、`model_auto_compact_token_limit`。
例如在用户级 `.codex/agents/xiaohongshu.toml` 中配置：

```toml
name = "xiaohongshu"
description = "小红书搜索执行 agent"
model = "deepseek-flash"
model_provider = "deepseek"
model_reasoning_effort = "high"
developer_instructions = "读取主会话提供的 plan.md，执行分配给 xiaohongshu 的任务，并返回有来源的结果。"

[model_providers.deepseek]
name = "DeepSeek"
base_url = "https://api.deepseek.com/"
wire_api = "responses"
env_key = "DEEPSEEK_API_KEY"
```

启动 CPA 前，在同一进程环境中提供 `DEEPSEEK_API_KEY`。上例只是客户端配置，
服务端必须实际支持所配置的模型和 Responses 协议；客户端不转换 Chat Completions。
主会话调用 `spawn_agent(agent_type="xiaohongshu", message="读取 D:/work/plan.md 并执行任务")`。
共享文件仍受继承的文件系统权限约束，MCP 沿用现有父会话继承机制。

## 优先级与边界

- 未指定 `model_provider` 时保留父会话的实际连接。仅定义 provider 不切换连接。
  如果定义会改变当前继承的同名 provider，必须显式指定 `model_provider`，否则报错。
- 显式选择优先于父会话（包括 CPA 的 CLI 参数）；可以引用父配置已有的 provider。
- 角色中的自定义 provider 定义整体替换子代理配置中的同名定义，不混合父连接的认证和 headers。
  内置 provider 的保留名称和 Bedrock 定制规则沿用上游校验。
- 切换连接时清除父模型目录（受管目录除外）、context window、自动压缩阈值及模型生成的基础指令；
  角色自己的模型目录和上下文配置随后生效。目录路径相对于角色文件解析。
  完整历史 fork 也重新判断目标模型的自动启用状态，保留历史任务内容和自定义基础指令。
- 只有 provider 连接和显式目录都相同才共享模型管理器，否则创建目标 provider 的管理器。
  自定义模型没有目录元数据时沿用上游通用能力元数据，但请求模型名不替换。
- 角色发现、项目可信状态、敏感文件读取、受管配置、沙箱、审批、MCP 和 capability
  的边界不放宽。provider 配置不能覆盖更高优先级的 legacy managed 配置。
- 未知 provider、缺失的指定环境变量和无效角色配置均报错；不会回退到 CPA。

## 生命周期

首次启动和嵌套启动使用角色覆盖后的配置。已加载子代理的后续消息继续由其原生会话处理。
V1/V2 恢复保留已记录的模型和 provider ID，重新从当前可信角色文件、父配置解析连接与认证；
不在历史中额外保存密钥。恢复期间角色文件和 provider 定义必须仍然可用。
改变配置文件的端点或认证会在重新加载时生效；保存的 provider ID 不被角色的新选择替换。
任务历史、等待、追问、恢复和结果回传继续使用原生机制。

## 明确选择 V1 或 V2

在用户 `config.toml` 的已有 `[features]` 段中配置：

```toml
[features]
multi_agent = true
multi_agent_v2 = false
```

本 fork 将显式 `multi_agent_v2 = false` 解释为选择 V1，优先于 Astra 等模型目录
中的 V2 默认值；同样支持表形式 `multi_agent_v2 = { enabled = false }`。
未设置此选项（或表中未指定 enabled）时保留模型默认选择，`true` 仍然选择 V2。
`agents.enabled = false` 仍阻止关闭 V2 后启用代理；同时关闭 `multi_agent` 时，
显式关闭 V2 会禁用多代理。受管配置约束继续按原有优先级生效。

不需要 `model_catalog_json` 或复制模型目录。配置值从合并后的 ConfigToml 读取，
会随配置重载重新计算。本改动不增加 TOML 字段，不改变模型工具 schema 或历史格式。
建议切换后新建会话；旧 V2 子会话不会自动转换成 V1 生命周期。

独立 Provider 的角色配置继续使用前文的方式。V1 创建指定角色时使用
`fork_context=false`（默认值），让该角色的 Provider 覆盖生效。

## 验证

按项目约束不新增、修改或运行测试，不运行 formatter 或 snapshot 命令。
本功能允许的最小编译验证为在 `codex-rs` 下运行 `cargo check -p codex-core --lib`。
版本选择修改已通过该编译检查与 `git diff --check`。

2026-09-11 已核对用户运行的 `0.154.0-dev-20260911` 实际会话：主会话使用
CPA/Astra，V1 的 xiaohongshu 子代理使用 `deepseek-flash` 和 `deepseek` Provider，
完整收到初始任务。子代理成功调用一次首页列表和一次帖子详情，完成后回传结果并关闭。
此记录确认该配置下的跨 Provider 创建、任务传递、MCP 执行和结果回传链路。
