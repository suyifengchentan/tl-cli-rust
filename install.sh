#!/bin/sh
set -e

REPO="suyifengchentan/tl-cli-rust"
VERSION="0.1.0"
BIN="tl"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

case "$(uname -s)" in
    Darwin)  OS="darwin";  ARCH="aarch64" ;;
    Linux)   OS="linux";   ARCH="x64" ;;
    * )      echo "Unsupported OS"; exit 1 ;;
esac

ARCHIVE="${BIN}-${OS}-${ARCH}"
URL="https://github.com/${REPO}/releases/download/v${VERSION}/${ARCHIVE}"

echo "Installing tl ${VERSION} for ${OS}/${ARCH}..."
echo "  Downloading ${URL}"

if command -v curl > /dev/null 2>&1; then
    curl -fsSL "${URL}" -o "${TMPDIR:-/tmp}/${ARCHIVE}"
elif command -v wget > /dev/null 2>&1; then
    wget -q "${URL}" -O "${TMPDIR:-/tmp}/${ARCHIVE}"
else
    echo "Neither curl nor wget found"
    exit 1
fi

chmod +x "${TMPDIR:-/tmp}/${ARCHIVE}"
sudo mv "${TMPDIR:-/tmp}/${ARCHIVE}" "${INSTALL_DIR}/${BIN}"
echo "Installed to ${INSTALL_DIR}/${BIN}"

# Verify
if command -v ${BIN} > /dev/null 2>&1; then
    ${BIN} --version
else
    echo "Installation complete. Run '${BIN} --help' to get started."
fi
