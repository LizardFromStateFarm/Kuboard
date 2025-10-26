#!/bin/bash
# Kuboard Local Build Script
# Builds executables for all platforms locally

echo "🚀 Building Kuboard locally..."

# Install dependencies
echo "📦 Installing dependencies..."
npm install

# Build frontend
echo "🔨 Building frontend..."
npm run build

# Build Tauri app for current platform
echo "🏗️ Building Tauri app..."
npm run tauri build

echo "✅ Build complete!"
echo "📁 Check src-tauri/target/release/bundle/ for installers"
