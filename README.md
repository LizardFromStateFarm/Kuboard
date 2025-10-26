# 🚢 Kuboard

A modern Kubernetes dashboard built with SvelteKit and Tauri.

![Kuboard Screenshot](https://via.placeholder.com/800x400/1a1a1a/10b981?text=Kuboard+Kubernetes+Dashboard)

## ✨ Features

- **🖥️ Desktop Application** - Native desktop app for Windows, macOS, and Linux
- **📊 Real-time Metrics** - Live resource usage graphs and monitoring
- **🔄 Context Management** - Easy switching between Kubernetes contexts
- **📈 Node Management** - Comprehensive node information and metrics
- **🎨 Modern UI** - Clean, responsive interface with dark theme
- **⚡ Fast Performance** - Built with Rust backend and SvelteKit frontend

## 🚀 Quick Start

### Download & Install

**Windows:**
- Download `Kuboard_0.1.0_x64-setup.exe` from [Releases](https://github.com/your-username/kuboard/releases)
- Run the installer and follow the setup wizard

**macOS:**
- Download `Kuboard_0.1.0_x64.dmg` from [Releases](https://github.com/your-username/kuboard/releases)
- Open the DMG and drag Kuboard to Applications

**Linux:**
- Download `Kuboard_0.1.0_amd64.deb` from [Releases](https://github.com/your-username/kuboard/releases)
- Install with `sudo dpkg -i Kuboard_0.1.0_amd64.deb`

### First Run

1. **Launch Kuboard** from your applications
2. **Select Context** - Choose your Kubernetes context
3. **View Dashboard** - Explore your cluster resources and metrics

## 🛠️ Development

### Prerequisites

- Node.js 18+ and npm
- Rust 1.70+ and Cargo
- Tauri CLI (`cargo install tauri-cli`)

### Setup

```bash
# Clone the repository
git clone https://github.com/your-username/kuboard.git
cd kuboard

# Install dependencies
npm install

# Start development server
npm run tauri dev
```

### Building

```bash
# Build for production
npm run tauri build
```

See [BUILD_GUIDE.md](BUILD_GUIDE.md) for detailed build instructions.

## 📚 Documentation

- **[Architecture Overview](ARCHITECTURE_OVERVIEW.md)** - Complete system architecture
- **[UI Organization](UI_ORGANIZATION_GUIDE.md)** - Component structure and usage
- **[Function Documentation](FUNCTION_DOCUMENTATION.md)** - API and function reference
- **[Build Guide](BUILD_GUIDE.md)** - Development and build instructions

## 🏗️ Architecture

### Frontend (SvelteKit)
- **Components:** Modular UI components with TypeScript
- **Styling:** CSS custom properties and responsive design
- **State Management:** Event-driven component communication
- **Charts:** Chart.js integration for real-time metrics

### Backend (Rust + Tauri)
- **Kubernetes Integration:** kube-rs client for cluster communication
- **Metrics Server:** Real-time metrics fetching and processing
- **Context Management:** kubeconfig handling and context switching
- **Resource APIs:** Comprehensive Kubernetes resource management

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

### Development Setup

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

### Code Style

- **Frontend:** TypeScript with Svelte best practices
- **Backend:** Rust with `kuboard_` function prefixes
- **Documentation:** Markdown with clear structure

## 📄 License

This project is licensed under a custom open source license. See [LICENSE](LICENSE) for details.

### Third-Party Licenses

This project uses several open source libraries. See [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md) for complete license information.

## 🙏 Acknowledgments

- [Tauri](https://tauri.app/) - Desktop application framework
- [SvelteKit](https://kit.svelte.dev/) - Frontend framework
- [kube-rs](https://kube.rs/) - Kubernetes client library
- [Chart.js](https://www.chartjs.org/) - Charting library

## 📞 Support

- **Issues:** [GitHub Issues](https://github.com/your-username/kuboard/issues)
- **Discussions:** [GitHub Discussions](https://github.com/your-username/kuboard/discussions)
- **Documentation:** [Wiki](https://github.com/your-username/kuboard/wiki)

---

**Kuboard** - Modern Kubernetes Dashboard 🚢