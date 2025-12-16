# 🚀 Kuboard Build Guide

A modern Kubernetes dashboard built with SvelteKit and Tauri.

## 📋 Prerequisites

- **Node.js** 18+ and npm
- **Rust** 1.70+ and Cargo
- **Tauri CLI** (`cargo install tauri-cli`)
- **Git** for cloning the repository

## 🛠️ Quick Start

### 1. Clone and Install
```bash
git clone https://github.com/your-username/kuboard.git
cd kuboard
npm install
```

### 2. Development
```bash
# Start development server
npm run tauri dev
```

**New Features in Development:**
- **Resource Management Tabs**: Comprehensive tabbed interface for managing K8s resources
- **Centralized Theming**: Real-time color editing with ThemeSwitcher component
- **Cluster Metrics**: Real-time donut charts for cluster resource usage
- **Enhanced Node Management**: Dedicated nodes tab with detailed information

### 3. Build for Production
```bash
# Build desktop application
npm run tauri build
```

## 📦 Build Outputs

The build process generates:
- **Windows:** MSI installer and NSIS setup
- **macOS:** DMG and PKG installers  
- **Linux:** AppImage and DEB packages

Outputs are located in `src-tauri/target/release/bundle/`

## 🔧 Development Commands

```bash
# Frontend development
npm run dev          # SvelteKit dev server
npm run build        # Frontend build
npm run preview      # Preview production build

# Tauri development  
npm run tauri dev    # Full app development
npm run tauri build  # Production build
```

## 🐛 Troubleshooting

### Common Issues

**Rust/Cargo not found:**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

**Tauri CLI not found:**
```bash
cargo install tauri-cli
```

**Node modules issues:**
```bash
rm -rf node_modules package-lock.json
npm install
```

**Build fails on Windows:**
- Install Visual Studio Build Tools
- Ensure Windows SDK is installed

### Linux: Blank/White Screen in Tauri Window

If the Tauri window shows a blank or white screen while the localhost server works fine, this is a known issue with WebKitGTK on Linux, especially with:
- **Wayland** display servers
- **NVIDIA GPUs**
- Certain GPU drivers

**Solution 1: Use the included launch scripts**
```bash
# For development
./run-dev.sh

# For running the built app
./run-kuboard.sh
```

**Solution 2: Set environment variables manually**
```bash
# Set these before running the app
export WEBKIT_DISABLE_DMABUF_RENDERER=1
export WEBKIT_DISABLE_COMPOSITING_MODE=1

# Then run
npm run tauri dev
# or
./src-tauri/target/release/kuboard
```

**Solution 3: Force X11 (if using Wayland)**
```bash
export GDK_BACKEND=x11
npm run tauri dev
```

**Solution 4: Create a custom .desktop file**

After installing the .deb package, create a custom launcher:
```bash
# Create a wrapper script
sudo tee /usr/local/bin/kuboard-wrapper << 'EOF'
#!/bin/bash
export WEBKIT_DISABLE_DMABUF_RENDERER=1
export WEBKIT_DISABLE_COMPOSITING_MODE=1
exec /usr/bin/kuboard "$@"
EOF

sudo chmod +x /usr/local/bin/kuboard-wrapper

# Update your .desktop file to use the wrapper
# Edit ~/.local/share/applications/Kuboard.desktop
# Change: Exec=kuboard
# To: Exec=kuboard-wrapper
```

**Why this happens:**
WebKitGTK (used by Tauri on Linux) can have GPU acceleration issues on certain configurations. The `WEBKIT_DISABLE_DMABUF_RENDERER=1` flag disables DMA-BUF rendering which often causes blank screens.

## 📁 Project Structure

```
kuboard/
├── src/                    # SvelteKit frontend
│   ├── lib/
│   │   ├── components/     # UI components
│   │   │   ├── Header.svelte
│   │   │   ├── ClusterOverview.svelte
│   │   │   ├── ClusterMetrics.svelte
│   │   │   ├── TabbedContent.svelte
│   │   │   ├── WorkloadsTab.svelte
│   │   │   ├── NodesTab.svelte
│   │   │   ├── ConfigTab.svelte
│   │   │   ├── NetworkTab.svelte
│   │   │   ├── CustomResourcesTab.svelte
│   │   │   ├── DonutChart.svelte
│   │   │   └── ThemeSwitcher.svelte
│   │   ├── styles/         # CSS and styling
│   │   │   ├── color-palette.css  # Centralized colors
│   │   │   ├── variables.css      # CSS variables
│   │   │   └── README.md          # Color guide
│   │   ├── types/          # TypeScript types
│   │   └── utils/          # Utility functions
│   └── routes/             # SvelteKit routes
├── src-tauri/              # Tauri backend
│   ├── src/                # Rust source code
│   │   ├── commands/       # Tauri command handlers
│   │   ├── kubernetes/     # K8s API integration
│   │   ├── metrics/        # Metrics collection
│   │   └── types.rs        # Rust type definitions
│   └── tauri.conf.json     # Tauri configuration
├── AI_SUMMARY.md           # AI assistant documentation (gitignored)
└── package.json            # Node.js dependencies
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## 📄 License

This project is licensed under a custom open source license. See [LICENSE](LICENSE) for details.