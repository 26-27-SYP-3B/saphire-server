BIN := "saphire-backend"
BUILD_DIR := "build"

# Run tests first, then cross-compile for darwin/linux/windows x amd64/arm64 into ./build
build: test
    set -euo pipefail; \
    rm -rf {{BUILD_DIR}}; \
    mkdir -p {{BUILD_DIR}}; \
    for os in darwin linux windows; do \
    for arch in amd64 arm64; do \
    ext=""; \
    [ "$os" = "windows" ] && ext=".exe"; \
    GOOS=$os GOARCH=$arch go build -o {{BUILD_DIR}}/{{BIN}}-$os-$arch$ext .; \
    done; \
    done; \
    ls -lh {{BUILD_DIR}}

# Run from source, compiles on the fly
run:
    go run .

# Run unit tests
test:
    go test -v ./...

clean:
    rm -rf {{BUILD_DIR}}/*