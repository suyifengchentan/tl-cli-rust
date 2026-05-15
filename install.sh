#!/bin/sh
set -e

REPO="suyifengchentan/tl-cli-rust"
BIN="tl"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Resolve latest version if not specified
if [ -z "${VERSION}" ]; then
    if command -v curl > /dev/null 2>&1; then
        VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"tag_name":' | sed -E 's/.*"v?([0-9.]+)".*/\1/')
    elif command -v wget > /dev/null 2>&1; then
        VERSION=$(wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"tag_name":' | sed -E 's/.*"v?([0-9.]+)".*/\1/')
    fi
    VERSION="${VERSION:-0.1.0}"
fi

case "$(uname -s)" in
    Darwin)  ARCH="darwin-aarch64" ;;
    Linux)   ARCH="linux-x64" ;;
    * )      echo "Unsupported OS: $(uname -s)"; exit 1 ;;
esac

ARTIFACT="${BIN}-${ARCH}"
URL="https://github.com/${REPO}/releases/download/v${VERSION}/${ARTIFACT}"

echo "Installing tl v${VERSION} for ${ARCH}..."
echo "  Downloading ${URL}"

if command -v curl > /dev/null 2>&1; then
    curl -fsSL "${URL}" -o "${TMPDIR:-/tmp}/${ARTIFACT}"
elif command -v wget > /dev/null 2>&1; then
    wget -q "${URL}" -O "${TMPDIR:-/tmp}/${ARTIFACT}"
else
    echo "Neither curl nor wget found"
    exit 1
fi

chmod +x "${TMPDIR:-/tmp}/${ARTIFACT}"
sudo mv "${TMPDIR:-/tmp}/${ARTIFACT}" "${INSTALL_DIR}/${BIN}"
echo "Installed to ${INSTALL_DIR}/${BIN}"

if command -v ${BIN} > /dev/null 2>&1; then
    ${BIN} --version
else
    echo "Installation complete. Run '${BIN} --help' to get started."
fi
