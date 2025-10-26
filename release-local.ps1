# Kuboard Local Release Script
# Builds and prepares release files locally

Write-Host "🚀 Building Kuboard for release..." -ForegroundColor Green

# Install dependencies
Write-Host "📦 Installing dependencies..." -ForegroundColor Yellow
npm install

# Build frontend
Write-Host "🔨 Building frontend..." -ForegroundColor Yellow
npm run build

# Build Tauri app
Write-Host "🏗️ Building Tauri app..." -ForegroundColor Yellow
npm run tauri build

Write-Host "✅ Build complete!" -ForegroundColor Green
Write-Host "📁 Release files are in src-tauri/target/release/bundle/" -ForegroundColor Cyan
Write-Host "📋 Next steps:" -ForegroundColor Yellow
Write-Host "1. Go to https://github.com/LizardFromStateFarm/Kuboard/releases" -ForegroundColor White
Write-Host "2. Click 'Create a new release'" -ForegroundColor White
Write-Host "3. Upload the files from the bundle directory" -ForegroundColor White
Write-Host "4. Publish the release" -ForegroundColor White
