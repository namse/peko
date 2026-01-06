FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    pkg-config \
    clang \
    lld \
    wget \
    tar \
    && apt-get clean -y && rm -rf /var/lib/apt/lists/*

ENV WASI_VERSION=29
ENV WASI_VERSION_FULL=${WASI_VERSION}.0

RUN ARCH=$(uname -m) && \
    if [ "$ARCH" = "x86_64" ]; then \
    WASI_ARCH="x86_64"; \
    elif [ "$ARCH" = "aarch64" ]; then \
    WASI_ARCH="arm64"; \
    else \
    echo "Unsupported architecture: $ARCH"; exit 1; \
    fi && \
    WASI_URL="https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-${WASI_VERSION}/wasi-sdk-${WASI_VERSION_FULL}-${WASI_ARCH}-linux.tar.gz" && \
    echo "Downloading WASI-SDK from: $WASI_URL" && \
    wget -O wasi-sdk.tar.gz "$WASI_URL" && \
    mkdir -p /opt/wasi-sdk && \
    tar xvf wasi-sdk.tar.gz -C /opt/wasi-sdk --strip-components=1 && \
    rm wasi-sdk.tar.gz

ENV WASI_SDK_PATH=/opt/wasi-sdk

ENV CC_wasm32_wasip2=/opt/wasi-sdk/bin/clang
ENV CFLAGS_wasm32_wasip2="--sysroot=/opt/wasi-sdk/share/wasi-sysroot"

RUN rustup toolchain install nightly-2026-01-05
RUN rustup target add wasm32-wasip2
