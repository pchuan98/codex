#!/usr/bin/env bash
set -euo pipefail
target=$1
profile=$2
shift 2
while IFS='=' read -r key value; do
  export "$key=$value"
done < /opt/codex-musl.env
export AWS_LC_SYS_NO_JITTER_ENTROPY=1
target_variable=${target^^}
target_variable=${target_variable//-/_}
# musl-gcc's Debian specs inject an interpreter for PIE links. Use a static
# executable so the output also runs on hosts without a musl loader installed.
export "CARGO_TARGET_${target_variable}_RUSTFLAGS=-C target-feature=+crt-static -C relocation-model=static -C link-arg=-static"

version=$(sed -n 's/^v8 = "=\([^"]*\)"/\1/p' Cargo.toml | tr -d '\r')
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo 'Expected a pinned V8 version.' >&2; exit 1; }
v8_directory="$CARGO_TARGET_DIR/v8/$version/$target"
mkdir -p "$v8_directory"
archive="librusty_v8_ptrcomp_sandbox_release_${target}.a.gz"
binding="src_binding_ptrcomp_sandbox_release_${target}.rs"
manifest="rusty_v8_ptrcomp_sandbox_release_${target}.sha256"
base_url="https://github.com/openai/codex/releases/download/rusty-v8-v${version}"
for artifact in "$manifest" "$archive" "$binding"; do
  if [[ ! -s "$v8_directory/$artifact" ]]; then
    curl -fL --retry 3 "$base_url/$artifact" -o "$v8_directory/$artifact.part"
    mv "$v8_directory/$artifact.part" "$v8_directory/$artifact"
  fi
done
trusted="/workspace/third_party/v8/rusty_v8_${version//./_}_release_manifests.sha256"
expected=$(grep -F "  $manifest" "$trusted" | cut -d ' ' -f 1)
actual=$(sha256sum "$v8_directory/$manifest" | cut -d ' ' -f 1)
[[ -n "$expected" && "$actual" == "$expected" ]] || { echo 'V8 manifest checksum mismatch.' >&2; exit 1; }
(cd "$v8_directory" && tr -d '\r' < "$manifest" | sha256sum -c -)
export RUSTY_V8_ARCHIVE="$v8_directory/$archive"
export RUSTY_V8_SRC_BINDING_PATH="$v8_directory/$binding"

arguments=(build --target "$target")
case "$profile" in
  debug) ;;
  release) arguments+=(--release) ;;
  *) echo "Unsupported profile: $profile" >&2; exit 1 ;;
esac
for binary in "$@"; do arguments+=(--bin "$binary"); done
printf 'Building %s (%s), V8 %s\n' "$target" "$profile" "$version"
cargo "${arguments[@]}"
for binary in "$@"; do
  executable="$CARGO_TARGET_DIR/$target/$profile/$binary"
  program_headers=$(readelf -lW "$executable")
  dynamic_entries=$(readelf -dW "$executable")
  if grep -q 'INTERP' <<< "$program_headers" || grep -q '(NEEDED)' <<< "$dynamic_entries"; then
    echo "Refusing dynamically linked Linux output: $executable" >&2
    exit 1
  fi
  printf 'Verified static ELF: %s\n' "$executable"
done
