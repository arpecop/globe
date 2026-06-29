#!/bin/bash
set -e

# Build script for cross-platform compilation

RELEASE_DIR="releases"
VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)

echo "🔨 Building Globy v$VERSION"

mkdir -p "$RELEASE_DIR"

# Build for all platforms
TARGETS=(
  "x86_64-unknown-linux-gnu"
  "aarch64-unknown-linux-gnu"
  "x86_64-apple-darwin"
  "aarch64-apple-darwin"
  "x86_64-pc-windows-gnu"
)

for target in "${TARGETS[@]}"; do
  echo "📦 Building for $target..."

  if ! cargo build --release --target "$target" 2>/dev/null; then
    echo "⚠️  Skipped $target (toolchain not installed)"
    continue
  fi

  # Determine binary name
  if [[ "$target" == *"windows"* ]]; then
    BINARY="target/$target/release/globy.exe"
    OUTPUT="$RELEASE_DIR/globy-${target%-*}.exe"
  else
    BINARY="target/$target/release/globy"
    OUTPUT="$RELEASE_DIR/globy-${target%-*}"
  fi

  if [ -f "$BINARY" ]; then
    cp "$BINARY" "$OUTPUT"
    chmod +x "$OUTPUT"
    SIZE=$(du -h "$OUTPUT" | cut -f1)
    echo "✅ Built: $OUTPUT ($SIZE)"
  fi
done

echo ""
echo "✨ Build complete!"
echo "📂 Binaries in: $RELEASE_DIR/"
ls -lh "$RELEASE_DIR/"
