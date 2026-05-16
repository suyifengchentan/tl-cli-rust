#!/bin/sh
set -e

REPO="suyifengchentan/tl-cli-rust"
BIN="tl"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${VERSION:-0.1.1}"

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
    if ! curl -fsI "${URL}" > /dev/null 2>&1; then
        echo "Release asset not found: ${URL}"
        echo "Check that version v${VERSION} exists and includes ${ARTIFACT}."
        exit 1
    fi
elif command -v wget > /dev/null 2>&1; then
    if ! wget -q --spider "${URL}" > /dev/null 2>&1; then
        echo "Release asset not found: ${URL}"
        echo "Check that version v${VERSION} exists and includes ${ARTIFACT}."
        exit 1
    fi
fi

if command -v curl > /dev/null 2>&1; then
    curl -fsSL "${URL}" -o "${TMPDIR:-/tmp}/${ARTIFACT}"
elif command -v wget > /dev/null 2>&1; then
    wget -q "${URL}" -O "${TMPDIR:-/tmp}/${ARTIFACT}"
else
    echo "Neither curl nor wget found"
    exit 1
fi

chmod +x "${TMPDIR:-/tmp}/${ARTIFACT}"
mkdir -p "${INSTALL_DIR}"
mv "${TMPDIR:-/tmp}/${ARTIFACT}" "${INSTALL_DIR}/${BIN}"
echo "Installed to ${INSTALL_DIR}/${BIN}"

case ":$PATH:" in
    *":${INSTALL_DIR}:"*) ;;
    *)
        echo ""
        echo "Add ${INSTALL_DIR} to your PATH if it is not already available:"
        echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
        echo ""
        echo "You can add that line to your shell profile, such as ~/.zshrc or ~/.bashrc."
        ;;
esac

if command -v ${BIN} > /dev/null 2>&1; then
    ${BIN} --version
else
    echo "Installation complete. Run '${BIN} --help' to get started."
fi
