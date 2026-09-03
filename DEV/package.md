# CPA npm 包

`DEV/scripts/npm` 保存 CPA npm 启动器模板和打包脚本。发布结构拆分为通用启动器和平台二进制包：

```text
@pchuan98/cpa
@pchuan98/cpa-win32-x64
@pchuan98/cpa-linux-x64
@pchuan98/cpa-linux-arm64
```

主包包含 Windows x64 和 Linux x64 的可选依赖。用户只安装 `@pchuan98/cpa`，npm 根据当前系统选择平台包，安装后的命令仍为 `cpa` 和 `cpa-set`，不会下载其他平台的二进制。Linux ARM64 平台包可以通过显式 `-Target linux-arm64` 单独生成，暂未加入主包依赖。

日常使用仓库级入口：

```powershell
.\DEV\scripts\pack.ps1
```

它默认执行 release 编译，自动识别当前实机平台，并使用同一个版本生成当前平台二进制包和 `@pchuan98/cpa` 主调用包。结束时会合并 `pack-manifest.json` 并显示对应的 `push.ps1` 命令。需要快速生成 debug 包时使用 `DEV/scripts/pack.ps1 -DebugBuild`。

`DEV/scripts/npm/pack.ps1` 是底层单包工具。不传 `-Target` 时只编译当前实机平台，不会自动编译其他平台，也不会生成主调用包：

```powershell
.\DEV\scripts\npm\pack.ps1 -Release
```

显式指定当前平台时仍在实机编译；指定其他 Linux 平台时通过 Docker 编译。Windows 目标必须在 Windows 实机编译：

```powershell
.\DEV\scripts\npm\pack.ps1 -Target windows-x64 -Release
.\DEV\scripts\npm\pack.ps1 -Target linux-x64 -Release
.\DEV\scripts\npm\pack.ps1 -Target linux-arm64 -Release
```

只生成主调用包时使用 `-OnlyPack`，不会编译 Rust：

```powershell
.\DEV\scripts\npm\pack.ps1 -OnlyPack -PackageVersion 0.0.14-dev-20260412
```

平台包和主调用包必须使用相同的 `-PackageVersion`。仓库级入口会先让底层平台打包器选择版本，再自动将该版本传给主调用包。Cargo 输出默认位于 `codex-rs/target/cpa-pack/<target>`；显式传入 `-TargetDirectory` 时，它表示该平台的 Cargo 输出目录。

第一次使用某个 Linux 目标时，`DEV/scripts/docker.ps1` 自动拉取 Debian 镜像并创建带 PowerShell、Rust 1.95、Node/npm 和原生依赖的缓存镜像；Dockerfile 变化时自动生成新的缓存镜像。APT 使用清华镜像，Rust toolchain 和 crates.io 使用 RsProxy。npm 上游版本查询和发布仍指定官方 registry。Linux ARM64 在 Windows x64 上通过 Docker 平台模拟，缺少模拟支持时脚本会通过 `tonistiigi/binfmt` 自动安装，因此编译时间会更长。

打包脚本会检查 Git 的 `upstream` remote。不存在时自动添加 `https://github.com/openai/codex`；已存在时必须指向该官方仓库，URL 末尾是否包含 `.git` 或 `/` 不影响判断。

脚本从 npm 官方 registry 读取 `@openai/codex` 当前正式版本，使用打包日期生成版本。例如，上游版本为 `0.0.14`，打包日期为 2026 年 4 月 12 日，首选版本为 `0.0.14-dev-20260412`。版本选择只检查本次生成的平台包：如果 Windows 已经上传该版本但 Linux 尚未上传，Linux 实机会继续使用同一版本补齐，而不是跳到 `.1`。当前平台包已有该版本时，才选择 `.1`、`.2` 等下一版本。所有平台包、启动器和 `codex --version` 使用同一版本。

编译前，打包脚本临时将 `codex-rs/Cargo.toml` 的 workspace 版本改为包版本，再调用 `DEV/scripts/build.ps1`。构建脚本在 Windows 上使用 CRLF migration，在 Linux 上使用 LF migration。它只转换格式不符合当前平台的文件，不在编译后恢复，因此后续构建可以复用 Cargo 缓存。

上游 release profile 保留 Linux ELF 调试符号。npm 打包 release 版本时，脚本只对复制到临时打包目录的 Linux 可执行文件执行 `strip --strip-all`，避免把超过 1 GB 的 DWARF 符号上传到 npm；Cargo target 中的原始二进制和编译缓存保持不变。debug 包不执行 strip。

Git 仓库中，构建脚本在 `.git/info/attributes` 写入仅限本机的换行规则，使 migration 的平台换行差异不出现在 Git 状态中。该文件不会提交，也不会修改全局 Git 配置。打包脚本仍负责恢复 `Cargo.toml` 和 `Cargo.lock`。Codex 运行时代码不修改 migration checksum。

包默认生成到 `DEV/scripts/npm/dist`。底层脚本每次生成一个 `.tgz` 和对应的 `pack-manifest.json`；仓库级入口生成当前平台包和主调用包，并把两条记录按平台在前、主包在后的顺序写入同一个 manifest。安装后使用 `cpa` 命令启动。CPA Provider 默认开启，启动器依次读取 `CPA_API_KEY` 环境变量和 `<CODEX_HOME>/cpa.toml` 中的 `api_key`。未设置 `CODEX_HOME` 时使用 `~/.codex`，两处都不存在密钥时立即退出，并提示运行 `cpa-set api-key <API_KEY>`。

`cpa` 默认向 Codex 传递 `--yolo`。使用 `cpa-set yolo false` 可以持久化关闭自动添加，使用 `cpa-set yolo true` 可以重新开启。`cpa` 启动器不提供 `--safe` 等临时模式参数，其他参数全部原样传递给 Codex。

无论 CPA Provider 是否启用，启动器始终通过上游配置参数关闭 Analytics、Feedback 和 OTEL Statsig metrics。SQLite process tracing 由 fork 的 Rust 初始化补丁关闭。

执行以下命令持久化密钥：

```text
cpa-set api-key sk-xxxxx
```

省略密钥时，`cpa-set api-key` 会在终端中隐藏输入内容。使用 `cpa-set context <SIZE>` 设置上下文窗口，参数必须使用 `k` 为单位，例如：

```text
cpa-set context 256k
```

设置后，`cpa` 在每次启动 Codex 时自动传递：

```text
-c model_context_window=256000
```

不带参数执行 `cpa-set context` 会删除自定义窗口并恢复 Codex 默认值，此时启动器不传递 `model_context_window`。当前上限与 Codex 内置 GPT-5.6 模型的 `max_context_window` 一致，为 `872k`；自动压缩阈值由 Codex 根据窗口自动计算。

使用 `cpa-set provider true` 启用 CPA Provider，这是默认行为。使用 `cpa-set provider false` 后，启动器不再注入 `model_provider` 和 `model_providers.cpa.*` 参数，不再要求配置 CPA API Key，也不会把 `cpa.toml` 中持久化的密钥注入子进程。上下文窗口是独立设置，仍会按配置传递。

仓库开发时可以使用 `DEV/test.bat <CPA_API_KEY>`。该脚本在 Debug 模式执行 `cargo run --bin codex --`，并注入与启用 CPA Provider 相同的 Provider 和遥测参数；它不读取或修改 `cpa.toml`，也不自动添加 `--yolo` 或自定义 context window。

`cpa-set show` 可以显示隐藏密钥后的当前配置，`cpa-set path` 可以输出配置文件位置。所有设置统一保存在 `<CODEX_HOME>/cpa.toml`，不会修改 Codex 的 `config.toml`：

```toml
api_key = "sk-xxxxx"
context_window = 256000
provider_enabled = true
yolo_enabled = true
```

`cpa.toml` 是 CPA 唯一的持久化配置文件。旧的 `~/.cpa/api-key` 和 `~/.cpa/settings.json` 不再读取，也不会被自动迁移或删除。自定义窗口要求当前模型和服务端支持对应的上下文大小。

CPA 启动器不会修改已有 Codex home 或配置文件的权限。保存配置时先写入同目录临时文件，再原子替换 `cpa.toml`；为避免写入意外目标，配置文件是符号链接时会拒绝保存。

底层 npm 打包脚本默认执行 debug 编译，从目标目录的 `debug` 子目录获取两个程序，不传递 `--release`。仓库级 `DEV/scripts/pack.ps1` 默认执行 release 编译。

需要 release 编译时显式添加 `-Release`：

```powershell
.\DEV\scripts\npm\pack.ps1 -TargetDirectory 'D:\.codex\target' -Release
```

## 发布到 npm

使用 `push.ps1` 读取 `dist/pack-manifest.json` 并发布到 npm 官方 registry。脚本先通过官方 registry 检查登录状态；未登录时显示 `npm login` 命令并退出。默认按照平台包在前、通用启动器在后的顺序发布，并全部使用 `latest` tag。脚本发布 manifest 中已有的包，不要求其他平台包已经存在；例如 Windows 打包产生的 manifest 会直接发布 Windows 平台包和通用启动器。以后补充 Linux 包时必须使用相同的 `PackageVersion`，再正常执行 `push.ps1`。已经存在的相同版本会跳过上传并重新设置 `latest`。

```powershell
.\DEV\scripts\npm\push.ps1
```

也可以显式指定 manifest 或单个包：

```powershell
.\DEV\scripts\npm\push.ps1 -ManifestPath '.\DEV\scripts\npm\dist\pack-manifest.json'
.\DEV\scripts\npm\push.ps1 -PackagePath '.\DEV\scripts\npm\dist\pchuan98-cpa-linux-x64-0.0.14-dev-20260412.tgz'
```
