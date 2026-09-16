# 编译、运行、打包与发布

统一入口为 `DEV/justfile`，需要 PowerShell 7.4+、Node/npm 和 just。Windows 需要本机 Rust；Linux 需要当前用户可访问的 Docker Engine，Rust 和 musl 工具链由构建镜像提供。配套 PowerShell/JavaScript 脚本负责实现，日常从 `DEV` 执行：

| 命令 | 行为 |
|---|---|
| `just run [Codex 参数...]` | 编译当前源码的 Debug Codex 和 Code Mode host，然后使用新二进制运行 CPA |
| `just pack` | 清理旧打包产物，编译并打包本机 Debug 版本 |
| `just pack --release` | 清理旧打包产物，编译并打包本机 Release 版本 |
| `just publish npm` | 发布当前打包结果到 npm 官方 registry |
| `just publish gitea` | 发布当前打包结果到 Gitea npm registry |

## 运行

`just run` 不读取旧包的 manifest，也不需要先执行 pack。参数原样转发，例如 `just run resume <id>`、`just run --version`。编译版本使用当前 Cargo workspace 版本，运行目录为 `DEV`，可传 `-C <目录>`。

编译同时生成 Code Mode host，避免单独构建 Codex 后缺少 host。编译缓存按 `CARGO_TARGET_DIR` 或默认 `codex-rs/target` 复用；不替换 `codex-copy.exe`。Windows 的 V8 binding 和 archive 配置由 `scripts/build.ps1` 处理。

## 打包

只编译当前机器的 CPU 架构，支持 Windows x64、Linux x64、Linux ARM64。Windows 使用本机 MSVC；Linux 默认在同架构 Docker 环境中编译静态 musl 程序，不依赖安装机器的 GLIBC。每次生成本机平台包和通用启动器包，输出到固定的 `DEV/scripts/npm/dist`。

Linux 的 `run`、Debug/Release `pack` 共用此路径。首次运行会构建工具链镜像，复用上游 musl OpenSSL、libcap 和 C/C++ 配置；Rust 版本读取 `codex-rs/rust-toolchain.toml`，V8 版本读取 Cargo workspace，并使用对应 musl sandbox 制品及上游校验值。镜像、Cargo 下载与编译缓存后续复用，容器以当前用户 UID/GID 写入产物。默认编译并行度为 2，可通过 `CARGO_BUILD_JOBS` 调整。

Linux 输出为 `<CARGO_TARGET_DIR>/<架构>-unknown-linux-musl/debug|release`，与旧 GNU 输出隔离；`run` 和打包只读取 musl 目录。构建完成后检查每个 ELF，发现动态解释器或 `NEEDED` 动态库依赖时停止，不会打包成通用 Linux 包。此检查不代表所有内核和系统工具均兼容。

打包开始前清理该目录中的旧 `.tgz`、manifest 和旧运行目录，保留 Cargo target 缓存；拒绝把其他目录用作清理目标。Debug 是默认值，Release 必须显式使用 `--release`。Linux Release 只对暂存副本 strip，不修改 Cargo 缓存中的二进制。

版本从 npm 官方的 `@openai/codex` 正式版本加打包日期生成，例如 `0.162.1-dev-20261014`；自动选版本时同时检查 npm 和 Gitea，当前平台包或启动器同版本已存在时递增 `.1`、`.2`。查询失败时停止，避免误把网络或认证错误当作版本可用。平台包、启动器和包内 `codex --version` 使用相同版本。打包时临时更新 workspace 版本，结束后恢复 `Cargo.toml` 和 `Cargo.lock`。

SQL migration 换行规则仅写入 `.git/info/attributes`，不改全局配置。`dist/pack-manifest.json` 记录平台包和启动器包，供发布使用。

## 发布

`just publish` 必须指定 `npm` 或 `gitea`，发布 manifest 中的包，先平台包再启动器，使用 `latest` tag 和已有 npm 凭据。

- npm：`https://registry.npmjs.org/`，检查已有登录。
- Gitea：`https://git.pchuan.top/api/packages/chuan/npm/`，由发布请求验证凭据。

同版本已存在时只更新 tag；不同内容必须重新打包成新版本。发布不重新编译或更改 registry 配置。

启动器声明 Windows x64、Linux x64 和 Linux ARM64 的同版本可选平台依赖，由 npm 按安装平台选择；打包仍只生成本机平台程序。

## CPA 启动设置

首次创建 `cpa.toml` 时，在支持 POSIX 权限的系统上使用 `0600`，仅当前用户可读写；更新已有文件时保留原权限。

`cpa` 和 `just run` 复用相同启动逻辑。配置文件为 `<CODEX_HOME>/cpa.toml`，默认 `~/.codex/cpa.toml`：

```toml
api_key = "你的密钥"
provider_enabled = true
yolo_enabled = true
context_window = 256000
disabled_features = ["content_item_kinds"]
```

`CPA_API_KEY` 环境变量优先于存储的密钥。`cpa-set api-key` 支持终端隐藏输入；`provider true|false` 和 `yolo true|false` 分别控制 Provider 与自动 `--yolo`。

`cpa-set context 256k` 设置普通上下文窗口；不带值则恢复默认窗口。旧 `context_mode` 字段仅被忽略，下一次保存配置时移除，不再支持 `exp` 或 `default` 模式命令。

默认添加 `--no-daemon`，显式 `--remote` 时保留远程连接行为。遥测不再由启动器注入，需配置时见 `telemetry.md`。

## 持久化禁用功能

```powershell
cpa-set disable add content_item_kinds
cpa-set disable add apps plugins
cpa-set disable remove content_item_kinds
cpa-set disable list
cpa-set disable clear
```

禁用列表保存在 `cpa.toml` 的 `disabled_features` 单行字符串数组中，缺失或为空时不追加参数。支持任意以英文字母开头、由字母、数字、下划线和连字符组成的功能名；实际功能是否存在由运行的 Codex 校验。添加会去重并保留顺序，删除不存在的名称不报错；`show` 同时显示禁用列表。

`cpa` 和 `just run` 启动时为每项自动追加 `--disable <FEATURE>`。本次显式传入 `--enable <FEATURE>` 或 `--enable=<FEATURE>` 时跳过该项持久化禁用，不修改配置；显式 `--disable` 也不重复追加。`--` 后的内容按普通参数转发，不参与覆盖判断。例如 `cpa --enable content_item_kinds` 可以临时恢复已禁用的功能。

针对上游 `content_item_metadata` 字段被服务端拒绝的问题，可用 `cpa-set disable add content_item_kinds` 持久化临时绕过。这个开关同时停止发送内容分类与来源元数据，消息正文仍保留；上游兼容后使用 `disable remove content_item_kinds` 恢复默认行为。

截至上游 `6ae8b8aa7c`，Responses Lite 仍会携带该字段：旧删除提交 `806d2d1cc1` 之后，`1b6e0d8145` 重新引入了来源元数据。仅升级到当前源码不能证明服务端已兼容；确认实际请求成功前保留绕过设置。
