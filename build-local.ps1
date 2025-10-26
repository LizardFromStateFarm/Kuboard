# Kuboard Local Build Script
# Builds executables for all platforms locally

Write-Host "🚀 Building Kuboard locally..." -ForegroundColor Green

# Install dependencies
Write-Host "📦 Installing dependencies..." -ForegroundColor Yellow
npm install

# Build frontend
Write-Host "🔨 Building frontend..." -ForegroundColor Yellow
npm run build

# Build Tauri app for current platform
Write-Host "🏗️ Building Tauri app..." -ForegroundColor Yellow
npm run tauri build

Write-Host "✅ Build complete!" -ForegroundColor Green
Write-Host "📁 Check src-tauri/target/release/bundle/ for installers" -ForegroundColor Cyan
