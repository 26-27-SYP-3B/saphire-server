BIN := "saphire-server"
BUILD_DIR := "build"

# Target triples covering the old go matrix: darwin, linux, windows x amd64, arm64
# (order: darwin/arm64, darwin/amd64, linux/arm64, linux/amd64, windows/arm64, windows/amd64).
# Requires the targets and linkers to be installed, e.g. via rustup:
#   rustup target add aarch64-apple-darwin x86_64-apple-darwin \
#       aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu \
#       aarch64-pc-windows-msvc x86_64-pc-windows-msvc
CROSS_TARGETS := "aarch64-apple-darwin \
    x86_64-apple-darwin \
    aarch64-unknown-linux-gnu \
    x86_64-unknown-linux-gnu \
    aarch64-pc-windows-msvc \
    x86_64-pc-windows-msvc"

default: help

# Show available recipes
help:
    @just --list

# Run tests first, then build a release binary for the host architecture into ./build
build: test
    set -euo pipefail; \
    rm -rf {{BUILD_DIR}}; \
    mkdir -p {{BUILD_DIR}}; \
    cargo build --release; \
    os=$(uname -s | tr '[:upper:]' '[:lower:]'); \
    arch=$(uname -m); [ "$arch" = "x86_64" ] && arch="amd64"; \
    cp target/release/{{BIN}} {{BUILD_DIR}}/{{BIN}}-$os-$arch; \
    ls -lh {{BUILD_DIR}}

# Run tests first, then cross-compile all targets into ./build
cross: test
    set -euo pipefail; \
    rm -rf {{BUILD_DIR}}; \
    mkdir -p {{BUILD_DIR}}; \
    for target in {{CROSS_TARGETS}}; do \
    IFS=- read -r arch _ os _ <<< "$target"; \
    [ "$arch" = "aarch64" ] && arch="arm64"; \
    [ "$arch" = "x86_64" ] && arch="amd64"; \
    ext=""; [ "$os" = "windows" ] && ext=".exe"; \
    cargo build --release --target "$target"; \
    cp "target/$target/release/{{BIN}}" "{{BUILD_DIR}}/{{BIN}}-$os-$arch$ext"; \
    done; \
    ls -lh {{BUILD_DIR}}

# Run from source, compiles on the fly
run:
    cargo run

# Run unit tests
test:
    cargo test

clean:
    rm -rf {{BUILD_DIR}}; \
    cargo clean
