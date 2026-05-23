#!/usr/bin/env bash
set -euo pipefail

REPO="Lucent-sh/vale.sh"
INSTALL_DIR="${VALE_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${VALE_VERSION:-latest}"

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
  x86_64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

case "$OS" in
  linux) TARGET="${ARCH}-unknown-linux-gnu" ;;
  darwin) TARGET="${ARCH}-apple-darwin" ;;
  *) echo "Unsupported OS: $OS" >&2; exit 1 ;;
esac

if [ "$VERSION" = "latest" ]; then
  VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
    | grep '"tag_name"' | cut -d'"' -f4)
fi

URL="https://github.com/$REPO/releases/download/$VERSION/vale-$TARGET.tar.gz"

echo "Installing vale $VERSION for $TARGET..."
mkdir -p "$INSTALL_DIR"
curl -fsSL "$URL" | tar xz -C "$INSTALL_DIR" vale
chmod +x "$INSTALL_DIR/vale"

echo "vale installed to $INSTALL_DIR/vale"
echo "Run 'vale doctor' to verify your installation."
