# CPA 启动设置

`cpa` 和 `just run` 共用 `scripts/npm/bin/launch.js`。启用 CPA Provider 时注入以下配置：

```toml
model_provider = "cpa"

[model_providers.cpa]
name = "OpenAI"
base_url = "https://codex.pchuan.top/v1"
env_key = "CPA_API_KEY"
wire_api = "responses"
```

地址固定在启动器中。`name = "OpenAI"` 用于上游 Provider 能力识别，Provider ID 仍为 `cpa`，使用 API Key 认证。Responses、工具调用、SSE 和远程压缩沿用上游，实际支持由服务端决定。

## 配置

配置文件为 `<CODEX_HOME>/cpa.toml`，默认 `~/.codex/cpa.toml`：

```toml
api_key = "你的密钥"
provider_enabled = true
yolo_enabled = true
context_window = 256000
disabled_features = ["content_item_kinds"]
```

未设置时，Provider 和自动 `--yolo` 默认开启，上下文窗口与禁用列表使用 Codex 默认值。`CPA_API_KEY` 环境变量优先于存储的密钥。新配置文件在 POSIX 系统上使用 `0600` 权限。

| 命令 | 用途 |
|---|---|
| `cpa-set api-key` | 隐藏输入并保存密钥 |
| `cpa-set provider true` / `false` | 开关 CPA Provider |
| `cpa-set yolo true` / `false` | 开关自动追加 `--yolo` |
| `cpa-set context 256k` | 设置上下文窗口，最大 872k；不带值恢复默认 |
| `cpa-set show` | 查看配置，密钥脱敏 |
| `cpa-set path` | 查看配置路径 |

启动器默认追加 `--no-daemon`，显式 `--remote` 时保留远程连接行为。遥测和日志通过 Codex 的 `config.toml` 配置，启动器不注入关闭参数。跨 Provider 恢复见 [resume.md](resume.md)。

## 持久化禁用功能

```powershell
cpa-set disable add content_item_kinds
cpa-set disable remove content_item_kinds
cpa-set disable list
cpa-set disable clear
```

禁用列表在启动时转换为 `--disable <FEATURE>`；可一次添加或删除多个名称。显式传入 `--enable <FEATURE>` 可临时覆盖，不修改已保存配置。`--` 后的参数不参与覆盖判断。

若服务端拒绝 `content_item_metadata`，可禁用 `content_item_kinds` 绕过；该开关同时关闭内容分类和来源元数据，保留消息正文。服务端兼容后移除对应禁用项。
