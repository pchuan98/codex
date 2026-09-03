# 上游同步任务

## 开始前

先阅读以下文档，确认本 fork 的修改范围：

- `DEV/README.md`：仓库约束和本地二进制说明。
- `DEV/telemetry.md`：遥测和 process tracing logs 修改。
- `DEV/resume.md`：会话恢复修改。
- `DEV/ui.md`：输入区和状态栏修改。

阅读这些文档时，不检查实现代码。遇到冲突或需要确认上游行为时，再查看相关代码。

## Rebase

1. 确认 `upstream` 指向 `https://github.com/openai/codex.git`，`origin` 仍指向 fork 仓库。
2. 执行 `git fetch upstream main` 获取官方 `main` 的最新提交。
3. 检查工作区和当前分支。保留用户尚未提交的修改。
4. 切换到本地 `main`，执行 `git merge --ff-only upstream/main`，再切回 `dev`。
5. 将 fork 的提交 rebase 到本地 `main`。不要使用 `origin/main` 作为上游基线。
6. 解决冲突时保留上游的新结构，再接回 `DEV/` 文档记录的行为。
7. 自定义功能优先放在独立文件中，减少对上游入口和调度文件的修改。
8. 按 `DEV/README.md` 的限制进行验证。禁止运行 `just`、测试、formatter 和 snapshot 命令。
9. 如需更新 `origin/dev`，确认 rebase 结果后使用 `git push --force-with-lease origin dev`。

## 结果

完成后提供以下内容：

- 按功能分类列出本次引入的上游提交，并用表格说明行为变化。
- 列出发生冲突的文件和处理结果。
- 指出哪些 fork 修改因上游实现变化需要重新审核。
- 列出执行过的验证命令和结果。

如果多次 rebase 已改变相关上游实现，询问用户是否重新审核 fork 的实现方案。

## Windows 编译

上游提交 `97576b179` 删除了 Code Mode 的进程内 V8 fallback。当前 Code Mode 由 `codex-code-mode-host.exe` 提供，单独编译 `codex.exe` 无法使用 Code Mode。

`code-mode-runtime` 启用了 V8 sandbox。`denoland/rusty_v8` 的 `v150.4.0` Release 没有 Windows MSVC sandbox 制品，因此需要通过 `RUSTY_V8_ARCHIVE` 和 `RUSTY_V8_SRC_BINDING_PATH` 使用 OpenAI 发布的对应制品。

先下载 V8 binding，再为当前 PowerShell 会话设置编译目录和 V8 制品地址。`RUSTY_V8_ARCHIVE` 支持 URL，`RUSTY_V8_SRC_BINDING_PATH` 必须指向本地文件：

```powershell
$env:CARGO_TARGET_DIR = 'D:\.codex\target'
$rustyV8Binding = Join-Path $env:CARGO_TARGET_DIR 'src_binding_ptrcomp_sandbox_release_x86_64-pc-windows-msvc.rs'
Invoke-WebRequest -Uri 'https://github.com/openai/codex/releases/download/rusty-v8-v150.4.0/src_binding_ptrcomp_sandbox_release_x86_64-pc-windows-msvc.rs' -OutFile $rustyV8Binding
$env:RUSTY_V8_ARCHIVE = 'https://github.com/openai/codex/releases/download/rusty-v8-v150.4.0/rusty_v8_ptrcomp_sandbox_release_x86_64-pc-windows-msvc.lib.gz'
$env:RUSTY_V8_SRC_BINDING_PATH = $rustyV8Binding
```

一次编译两个程序：

```powershell
cargo build --bin codex --bin codex-code-mode-host
```

编译 release 版本：

```powershell
cargo build --release --bin codex --bin codex-code-mode-host
```

也可以使用 `DEV/scripts/build.ps1` 配置 V8 环境并执行 debug 编译：

```powershell
.\DEV\scripts\build.ps1 --codex --host --d 'D:\.codex\target'
```

`--codex` 编译 Codex，`--host` 编译 Code Mode host。两者可以单独使用。`--d` 指定 Cargo 输出目录，默认值为 `codex-rs/target`。不传 `--target` 时按照当前系统和 CPU 架构执行本机编译。

Windows 上指定 `linux-x64` 或 `linux-arm64` 时，脚本通过 Docker 自动拉取和配置 Linux 构建环境：

```powershell
.\DEV\scripts\build.ps1 --codex --host --release --target linux-x64
.\DEV\scripts\build.ps1 --codex --host --release --target linux-arm64
```

Docker 目标默认输出到 `codex-rs/target/docker/<target>`。Linux ARM64 使用 Docker Desktop 的 ARM64 平台模拟；缺少模拟支持时脚本会通过 `tonistiigi/binfmt` 自动安装，在 x64 电脑上会明显慢于 Linux x64。Docker 构建镜像使用清华 Debian APT 镜像和 RsProxy Rust/crates.io 镜像；npm 仍使用官方 registry。

脚本支持 Windows x64 MSVC、Linux x64 和 Linux ARM64 Rust 工具链，并自动下载缺失的 V8 binding、配置当前进程所需的 V8 环境变量。Windows 使用 CRLF migration，Linux 使用 LF migration。构建脚本通过 `.git/info/attributes` 保存本机规则，只在平台格式变化时改写文件。

添加 `--release` 时执行 release 编译。CPA npm 包的构建和版本规则见 `DEV/package.md`。

部署时，将 `codex.exe` 和 `codex-code-mode-host.exe` 放在同一目录。标准 package 构建会自动包含 host。

`cx.bat` 运行 `codex-rs/target/debug/codex-copy.exe`。Cargo 默认更新 `codex.exe`。使用 `cx` 验证前，比较两个文件的修改时间，或者直接运行新生成的 `codex.exe`。设置 `CARGO_TARGET_DIR` 后，从该目录获取编译结果。
