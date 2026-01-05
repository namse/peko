FROM mcr.microsoft.com/devcontainers/rust:1-bullseye

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    pkg-config \
    && apt-get clean -y && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-wasip2
RUN rustup toolchain install nightly
