.PHONY: build build-all release clean install

# Single-file output names
BIN_NAME = tl

# Build targets
TARGET_MACOS = aarch64-apple-darwin
TARGET_WIN = x86_64-pc-windows-msvc
TARGET_LINUX = x86_64-unknown-linux-gnu

build:
	cargo build --release

build-all: build-macos build-linux build-windows

build-macos:
	cargo build --release --target $(TARGET_MACOS)
	@echo "  → target/$(TARGET_MACOS)/release/$(BIN_NAME)"

build-linux:
	cargo build --release --target $(TARGET_LINUX)
	@echo "  → target/$(TARGET_LINUX)/release/$(BIN_NAME)"

build-windows:
	cargo build --release --target $(TARGET_WIN)
	@echo "  → target/$(TARGET_WIN)/release/$(BIN_NAME).exe"

release: build-all
	@mkdir -p dist
	cp target/$(TARGET_MACOS)/release/$(BIN_NAME) dist/$(BIN_NAME)-darwin-arm64
	cp target/$(TARGET_LINUX)/release/$(BIN_NAME) dist/$(BIN_NAME)-linux-amd64
	cp target/$(TARGET_WIN)/release/$(BIN_NAME).exe dist/$(BIN_NAME)-windows-amd64.exe
	@echo "Release artifacts in dist/"

clean:
	cargo clean
	rm -rf dist

install:
	cp target/release/$(BIN_NAME) /usr/local/bin/$(BIN_NAME)
	@echo "Installed to /usr/local/bin/$(BIN_NAME)"
