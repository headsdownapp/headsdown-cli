#!/bin/sh
# HeadsDown CLI installer
# Usage: curl -fsSL https://headsdown.app/install.sh | sh
#
# Detects OS/arch, downloads the latest release binary from GitHub,
# and installs it to /usr/local/bin (or ~/bin as fallback).

set -e

REPO="headsdownapp/headsdown-cli"
BINARY="hd"

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)  OS_TAG="linux" ;;
    Darwin) OS_TAG="darwin" ;;
    *)      echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
    x86_64|amd64) ARCH_TAG="x86_64" ;;
    aarch64|arm64) ARCH_TAG="aarch64" ;;
    *)             echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

ARTIFACT="hd-${ARCH_TAG}-${OS_TAG}"

echo ""
echo "  HeadsDown CLI installer"
echo ""
echo "  Platform: ${OS_TAG}/${ARCH_TAG}"

# Get the latest release tag
echo "  Fetching latest release..."
LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST" ]; then
    echo "  Error: Could not determine latest release"
    exit 1
fi

echo "  Version: ${LATEST}"

# Download
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST}/${ARTIFACT}"
echo "  Downloading ${ARTIFACT}..."

TMPDIR="${TMPDIR:-/tmp}"
TMPFILE="${TMPDIR}/hd-download-$$"

curl -fsSL -o "$TMPFILE" "$DOWNLOAD_URL"
chmod +x "$TMPFILE"

# Install
INSTALL_DIR="/usr/local/bin"
if [ ! -w "$INSTALL_DIR" ]; then
    # Try with sudo
    if command -v sudo >/dev/null 2>&1; then
        echo "  Installing to ${INSTALL_DIR} (requires sudo)..."
        sudo mv "$TMPFILE" "${INSTALL_DIR}/${BINARY}"
    else
        # Fallback to ~/bin
        INSTALL_DIR="$HOME/bin"
        mkdir -p "$INSTALL_DIR"
        mv "$TMPFILE" "${INSTALL_DIR}/${BINARY}"
        echo ""
        echo "  Note: Installed to ~/bin. Make sure it's in your PATH:"
        echo "    export PATH=\"\$HOME/bin:\$PATH\""
    fi
else
    mv "$TMPFILE" "${INSTALL_DIR}/${BINARY}"
fi

echo ""
echo "  ✓ Installed ${BINARY} ${LATEST} to ${INSTALL_DIR}/${BINARY}"
echo ""
echo "  Get started:"
echo "    hd auth       # authenticate"
echo "    hd status     # check your status"
echo "    hd busy 2h    # set yourself to busy"
echo ""
