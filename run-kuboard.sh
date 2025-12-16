#!/bin/bash
# Kuboard Launch Script
# This script sets environment variables to fix WebKitGTK rendering issues on Linux
# Use this to run the built application

# Disable DMA-BUF renderer (common fix for blank screens)
export WEBKIT_DISABLE_DMABUF_RENDERER=1

# Additional fixes for NVIDIA + Wayland
export WEBKIT_DISABLE_COMPOSITING_MODE=1

# Determine the app location
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_PATH="$SCRIPT_DIR/src-tauri/target/release/kuboard"
DEBUG_APP_PATH="$SCRIPT_DIR/src-tauri/target/debug/kuboard"

echo "🚀 Starting Kuboard with WebKitGTK fixes..."
echo "   WEBKIT_DISABLE_DMABUF_RENDERER=1"
echo "   WEBKIT_DISABLE_COMPOSITING_MODE=1"
echo ""

if [ -f "$APP_PATH" ]; then
    echo "Running release build..."
    "$APP_PATH"
elif [ -f "$DEBUG_APP_PATH" ]; then
    echo "Running debug build..."
    "$DEBUG_APP_PATH"
else
    echo "Error: Kuboard executable not found. Please build the app first:"
    echo "  npm run tauri build"
    exit 1
fi

