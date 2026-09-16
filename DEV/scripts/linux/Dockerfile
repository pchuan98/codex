ARG RUST_VERSION
FROM rust:${RUST_VERSION}-slim-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
    sudo curl ca-certificates xz-utils git cmake make perl python3 pkg-config \
    && rm -rf /var/lib/apt/lists/*
RUN case "$(uname -m)" in \
      x86_64) zig_arch=x86_64; checksum=473ec26806133cf4d1918caf1a410f8403a13d979726a9045b421b685031a982 ;; \
      aarch64) zig_arch=aarch64; checksum=ab64e3ea277f6fc5f3d723dcd95d9ce1ab282c8ed0f431b4de880d30df891e4f ;; \
      *) exit 1 ;; \
    esac \
    && curl -fL --retry 3 "https://ziglang.org/download/0.14.0/zig-linux-${zig_arch}-0.14.0.tar.xz" -o /tmp/zig.tar.xz \
    && echo "$checksum  /tmp/zig.tar.xz" | sha256sum -c - \
    && mkdir /opt/zig && tar -xJf /tmp/zig.tar.xz -C /opt/zig --strip-components=1 \
    && rm /tmp/zig.tar.xz
ENV PATH="/opt/zig:${PATH}"
COPY .github/scripts/install-musl-build-tools.sh .github/scripts/install-musl-openssl.sh /opt/build-tools/
RUN sed -i 's/\r$//' /opt/build-tools/*.sh \
    && case "$(uname -m)" in \
         x86_64) export TARGET=x86_64-unknown-linux-musl ;; \
         aarch64) export TARGET=aarch64-unknown-linux-musl ;; \
       esac \
    && export GITHUB_ENV=/opt/codex-musl.env RUNNER_TEMP=/opt OPENSSL_BUILD_JOBS=2 \
    && bash /opt/build-tools/install-musl-build-tools.sh \
    && rustup target add "$TARGET" \
    && rustup component add clippy rustfmt rust-src \
    && rm -rf /var/lib/apt/lists/*
