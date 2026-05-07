#!/bin/bash
set -euo pipefail

REPO="123hi123/quick-vless"
BIN_NAME="quick-vless"
INSTALL_DIR="/usr/local/bin"

echo "=== Quick-VLESS Installer ==="
echo

# Check root
if [ "$(id -u)" -ne 0 ]; then
    echo "Please run as root: curl -sSL ... | sudo bash"
    exit 1
fi

# Detect arch
ARCH=$(uname -m)
case "$ARCH" in
    x86_64) ASSET="quick-vless-x86_64-linux" ;;
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
echo "  1. quick-vless init --port 443 --sni www.microsoft.com"
echo "  2. quick-vless user add <name> --expires 30d"
echo
