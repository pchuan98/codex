# 构建、打包与发布

日常命令见 [README.md](README.md)。配套脚本是 `DEV/justfile` 的实现，运行和打包均同时构建 Codex 与 Code Mode host。

## 构建与运行

- Windows x64：本机 MSVC Rust 工具链。
- Linux x64 / ARM64：同架构 Docker 环境，输出静态 musl 程序；首次构建镜像后复用缓存。默认编译并行度为 2，用 `CARGO_BUILD_JOBS` 调整。
- Rust 版本读取 `codex-rs/rust-toolchain.toml`，V8 版本读取 Cargo workspace 的固定依赖，由脚本配置对应制品。
- 编译缓存使用 `CARGO_TARGET_DIR`，未设置时为 `codex-rs/target`；Linux 产物位于其中的 `<架构>-unknown-linux-musl/debug|release`。
- Linux 构建检查 ELF 无动态解释器和动态库依赖；Release 打包只 strip 暂存副本。

`just run` 使用新编译的 Debug 程序，参数原样转发，不需要先打包。CPA 设置见 [cpa.md](cpa.md)。

## 打包

只构建当前机器架构，生成本机平台包和通用启动器包，输出到 `DEV/scripts/npm/dist`。

每次打包清理该目录中的旧包和 manifest，保留 Cargo 编译缓存。默认 Debug，Release 使用 `just pack --release`。

版本由 npm 官方 `@openai/codex` 正式版本和打包日期生成。同版本在 npm 或 Gitea 已存在时递增后缀，查询失败则停止。平台包、启动器和包内 `codex --version` 使用相同版本；临时修改的 `Cargo.toml` 和 `Cargo.lock` 在打包结束后恢复。

`dist/pack-manifest.json` 记录当前包的版本、名称和文件，供发布读取。启动器支持 Windows x64、Linux x64 和 Linux ARM64，由 npm 选择对应平台依赖。

## 发布

使用已有 npm 凭据发布 manifest 中的包，先平台包再启动器，tag 为 `latest`：

- `just publish npm`：`https://registry.npmjs.org/`
- `just publish gitea`：`https://git.pchuan.top/api/packages/chuan/npm/`

发布不重新编译。同版本已存在时只更新 tag，内容变化需要重新打包为新版本。
