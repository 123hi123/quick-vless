#!/bin/bash
set -euo pipefail

REPO="123hi123/quick-node"
BIN_NAME="quick-node"
INSTALL_DIR="/usr/local/bin"

echo "=== Quick-Node Installer ==="
echo

# Check root
if [ "$(id -u)" -ne 0 ]; then
    echo "Please run as root: curl -sSL ... | sudo bash"
    exit 1
fi

# Detect arch
ARCH=$(uname -m)
case "$ARCH" in
    x86_64) ASSET="quick-node-x86_64-linux" ;;
    aarch64) ASSET="quick-node-aarch64-linux" ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

# Download latest release
echo "Downloading $BIN_NAME..."
LATEST_URL="https://github.com/$REPO/releases/latest/download/$ASSET"
curl -sL -o "$INSTALL_DIR/$BIN_NAME" "$LATEST_URL"
chmod +x "$INSTALL_DIR/$BIN_NAME"

echo "$BIN_NAME installed to $INSTALL_DIR/$BIN_NAME"
echo
echo "Next steps:"
echo "  1. quick-node init"
echo "  2. quick-node user add <name> --expires 30d"
echo
