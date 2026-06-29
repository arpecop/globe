#!/bin/bash
set -e

VERSION="latest"
REPO="globy-chat/globy"

# Detect OS and architecture
OS=$(uname -s)
ARCH=$(uname -m)

# Map architecture
case "$ARCH" in
  x86_64)   ARCH="x86_64" ;;
  aarch64)  ARCH="aarch64" ;;
  arm64)    ARCH="aarch64" ;;  # macOS M1/M2
  armv7l)   ARCH="armv7" ;;
  *)        echo "❌ Unsupported architecture: $ARCH"; exit 1 ;;
esac

# Map OS
case "$OS" in
  Linux)    OS_NAME="linux" ;;
  Darwin)   OS_NAME="macos" ;;
  MINGW*|MSYS*|CYGWIN*)  OS_NAME="windows" ;;
  *)        echo "❌ Unsupported OS: $OS"; exit 1 ;;
esac

BINARY_NAME="globy-${OS_NAME}-${ARCH}"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${BINARY_NAME}"

echo "📥 Installing Globy..."
echo "🖥️  OS: $OS_NAME"
echo "⚙️  Architecture: $ARCH"

# Download binary
echo "⏳ Downloading $BINARY_NAME..."
if ! curl -sSL -f "$DOWNLOAD_URL" -o globy; then
  echo "❌ Failed to download binary"
  echo "ℹ️  Check if release exists: https://github.com/${REPO}/releases"
  exit 1
fi

chmod +x globy

# Install to PATH
if [[ ":$PATH:" == *":/usr/local/bin:"* ]]; then
  echo "📂 Installing to /usr/local/bin..."
  sudo mv globy /usr/local/bin/
  echo "✅ Installed: /usr/local/bin/globy"
elif [[ ":$PATH:" == *":$HOME/.local/bin:"* ]]; then
  echo "📂 Installing to ~/.local/bin..."
  mkdir -p ~/.local/bin
  mv globy ~/.local/bin/
  echo "✅ Installed: ~/.local/bin/globy"
else
  echo "📂 Installing to current directory..."
  echo "⚠️  Add current directory to PATH to use globally"
fi

echo ""
echo "✨ Installation complete!"
echo ""
echo "🚀 Quick start:"
echo "   globy serve --salt mydevice --port 3000"
echo "   globy cli --connect localhost:3000"
echo ""
echo "📚 More info:"
echo "   globy --help"
echo "   globy info"
