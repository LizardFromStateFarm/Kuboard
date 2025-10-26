#!/bin/bash
# Kuboard Local Release Script
# Builds and prepares release files locally

echo "🚀 Building Kuboard for release..."

# Install dependencies
echo "📦 Installing dependencies..."
npm install

# Build frontend
echo "🔨 Building frontend..."
npm run build

# Build Tauri app
echo "🏗️ Building Tauri app..."
npm run tauri build

echo "✅ Build complete!"
echo "📁 Release files are in src-tauri/target/release/bundle/"
echo "📋 Next steps:"
echo "1. Go to https://github.com/LizardFromStateFarm/Kuboard/releases"
echo "2. Click 'Create a new release'"
echo "3. Upload the files from the bundle directory"
echo "4. Publish the release"
