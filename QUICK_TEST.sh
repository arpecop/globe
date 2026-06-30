#!/bin/bash
# Quick test script for Globy SSH Terminal Service
# Usage: ./QUICK_TEST.sh

set -e

cd "$(dirname "$0")"

echo "🧪 Globy SSH Terminal Service - Quick Test"
echo "==========================================="
echo ""

# Step 1: Build
echo "📦 Building release binary..."
cargo build --release 2>/dev/null || {
    echo "❌ Build failed"
    exit 1
}
echo "✅ Build complete"
echo ""

# Step 2: Check SSH keys
echo "🔐 Checking SSH keys..."
if [ ! -f ~/.ssh/id_ed25519 ]; then
    echo "❌ SSH key not found at ~/.ssh/id_ed25519"
    echo "   Generate with: ssh-keygen -t ed25519"
    exit 1
fi
echo "✅ SSH key found"
echo ""

# Step 3: Show instructions
echo "🚀 Starting tests in 3 terminals..."
echo ""
echo "Terminal 1 (Relay Server):"
echo "  cd /home/rudix/Desktop/globy"
echo "  ./target/release/globy --host --port 3000"
echo ""
echo "Terminal 2 (Alice):"
echo "  ssh -p 2222 alice@localhost"
echo "  (when prompted, enter: Alice)"
echo "  (type messages in TUI)"
echo ""
echo "Terminal 3 (Bob):"
echo "  ssh -p 2222 bob@localhost"
echo "  (when prompted, enter: Bob)"
echo "  (see Alice's messages appear)"
echo ""
echo "==========================================="
echo ""

# Step 4: Offer to start relay
read -p "Start relay server now? (y/n) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Starting relay on port 2222..."
    echo ""
    ./target/release/globy --host --port 3000
else
    echo "Manual start commands:"
    echo ""
    echo "  ./target/release/globy --host --port 3000"
    echo ""
    echo "Then in other terminals:"
    echo "  ssh -p 2222 alice@localhost"
    echo "  ssh -p 2222 bob@localhost"
fi
