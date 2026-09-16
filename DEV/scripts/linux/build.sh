#!/usr/bin/env bash
set -euo pipefail

repository=$1
target_directory=$2
profile=$3
shift 3
case "$(uname -m)" in
  x86_64) target=x86_64-unknown-linux-musl; platform=linux/amd64 ;;
  aarch64) target=aarch64-unknown-linux-musl; platform=linux/arm64 ;;
  *) echo 'Unsupported Linux architecture.' >&2; exit 1 ;;
esac
for tool in docker tar; do
  command -v "$tool" >/dev/null || { echo "Required command '$tool' was not found." >&2; exit 1; }
done
docker info >/dev/null
rust_version=$(sed -n 's/^channel = "\([^"]*\)"/\1/p' "$repository/codex-rs/rust-toolchain.toml" | tr -d '\r')
[[ "$rust_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo 'Expected a pinned Rust version.' >&2; exit 1; }
image="codex-cpa-musl:${rust_version}-$(uname -m)"
tar -C "$repository" -cf - DEV/scripts/linux/Dockerfile \
  .github/scripts/install-musl-build-tools.sh .github/scripts/install-musl-openssl.sh |
  docker build --platform "$platform" --build-arg "RUST_VERSION=$rust_version" \
    -f DEV/scripts/linux/Dockerfile -t "$image" -

mkdir -p "$target_directory/musl-cache/cargo" "$target_directory/musl-cache/home"
docker run --rm --init --platform "$platform" --user "$(id -u):$(id -g)" \
  --mount "type=bind,source=$repository,target=/workspace" \
  --mount "type=bind,source=$target_directory,target=/codex-target" \
  --workdir /workspace/codex-rs \
  -e HOME=/codex-target/musl-cache/home \
  -e CARGO_HOME=/codex-target/musl-cache/cargo \
  -e CARGO_TARGET_DIR=/codex-target \
  -e "CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-2}" \
  -e "CARGO_NET_OFFLINE=${CARGO_NET_OFFLINE:-false}" \
  "$image" bash /workspace/DEV/scripts/linux/compile.sh "$target" "$profile" "$@"
