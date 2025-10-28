# 🚢 Kuboard

A modern Kubernetes dashboard built with SvelteKit and Tauri.

## ✨ Features

- **🖥️ Desktop Application** - Native desktop app for Windows, macOS, and Linux
- **📊 Real-time Metrics** - Live resource usage graphs and cluster-wide donut charts
- **🔄 Context Management** - Easy switching between Kubernetes contexts
- **📋 Resource Management** - Tabbed interface for workloads, nodes, config, and networking
- **🖥️ Node Management** - Comprehensive node information and detailed metrics
- **🎨 Modern UI** - Clean, responsive interface with centralized theming system
- **⚡ Fast Performance** - Built with Rust backend and SvelteKit frontend
- **🎨 Customizable Themes** - Dark, light, and high-contrast themes with dev mode controls

## 🚀 Quick Start

### Download & Install

**Windows:**
- Download `Kuboard_v1.0.0_x64-setup.exe` from [Releases](https://github.com/LizardFromStateFarm/Kuboard/releases)
- Run the installer and follow the setup wizard
- Alternative: Download `Kuboard_v1.0.0_x64_en-US.msi` for enterprise deployment

### First Run

1. **Launch Kuboard** from your applications
2. **Select Context** - Choose your Kubernetes context
3. **View Dashboard** - Explore your cluster resources and metrics

## 📋 Resource Management

Kuboard features a comprehensive tabbed interface for managing Kubernetes resources:

### Available Tabs
- **🖥️ Nodes** - Cluster nodes with detailed system information and resource specs
- **⚙️ Workloads** - Pods, Deployments, and Services
- **⚙️ Config** - ConfigMaps and Secrets
- **🌐 Network** - Services and networking resources
- **🔧 Custom Resources** - CRDs and custom resource definitions
- **💾 Storage** - PersistentVolumes and StorageClasses (coming soon)
- **🔒 Security** - RBAC and SecurityContexts (coming soon)

### Key Features
- **Lazy Loading** - Resources only load when tab is selected
- **Real-time Counts** - Tab badges show current resource counts
- **Detailed Views** - Comprehensive information for each resource type
- **Copy to Clipboard** - Easy copying of resource details
- **Responsive Design** - Works on different screen sizes

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

### Build Outputs
- **Windows**: `src-tauri/target/release/bundle/nsis/` and `msi/`

See [docs/BUILD_GUIDE.md](docs/BUILD_GUIDE.md) for detailed build instructions.

## 📚 Documentation

- **[Architecture Overview](docs/ARCHITECTURE_OVERVIEW.md)** - Complete system architecture
- **[UI Organization](docs/UI_ORGANIZATION_GUIDE.md)** - Component structure and usage
- **[Function Documentation](docs/FUNCTION_DOCUMENTATION.md)** - API and function reference
- **[Build Guide](docs/BUILD_GUIDE.md)** - Development and build instructions

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

We welcome contributions! Please see our [Contributing Guidelines](docs/CONTRIBUTING.md) for details.

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

This project is licensed under PolyForm Noncommercial License 1.0.0. See [LICENSE](LICENSE) for details.

### Third-Party Licenses

This project uses several open source libraries. See [docs/THIRD_PARTY_LICENSES.md](docs/THIRD_PARTY_LICENSES.md) for complete license information.

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