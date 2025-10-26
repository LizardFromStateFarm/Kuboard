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

## 📁 Project Structure

```
kuboard/
├── src/                    # SvelteKit frontend
│   ├── lib/components/     # UI components
│   ├── lib/utils/         # Utility functions
│   └── routes/            # SvelteKit routes
├── src-tauri/             # Tauri backend
│   ├── src/               # Rust source code
│   └── tauri.conf.json    # Tauri configuration
└── package.json           # Node.js dependencies
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## 📄 License

This project is licensed under a custom open source license. See [LICENSE](LICENSE) for details.