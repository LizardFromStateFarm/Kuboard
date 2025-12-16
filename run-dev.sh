#!/bin/bash
# Kuboard Development Launch Script
# This script sets environment variables to fix WebKitGTK rendering issues on Linux

# Disable DMA-BUF renderer (common fix for blank screens)
export WEBKIT_DISABLE_DMABUF_RENDERER=1

# Additional fixes for NVIDIA + Wayland
export WEBKIT_DISABLE_COMPOSITING_MODE=1

# Force software rendering if GPU acceleration causes issues (uncomment if needed)
# export WEBKIT_FORCE_SANDBOX=0
# export LIBGL_ALWAYS_SOFTWARE=1

# For Wayland-specific issues, you can try forcing X11 (uncomment if needed)
# export GDK_BACKEND=x11

echo "🚀 Starting Kuboard with WebKitGTK fixes..."
echo "   WEBKIT_DISABLE_DMABUF_RENDERER=1"
echo "   WEBKIT_DISABLE_COMPOSITING_MODE=1"
echo ""

npm run tauri dev

