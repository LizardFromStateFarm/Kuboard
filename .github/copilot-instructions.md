# Copilot Instructions for Kuboard

<!-- Use this file to provide workspace-specific custom instructions to Copilot. For more details, visit https://code.visualstudio.com/docs/copilot/copilot-customization#_use-a-githubcopilotinstructionsmd-file -->

## Project Overview

This is a cross-platform Kubernetes desktop GUI application built with:
- **Backend**: Rust + Tauri for native desktop functionality
- **Frontend**: SvelteKit with TypeScript for the user interface
- **Kubernetes Integration**: Using the `kube` crate for Kubernetes API interactions

## Architecture

### Backend (Rust)
- Located in `src-tauri/src/`
- Uses Tauri commands to expose Kubernetes functionality to the frontend
- Implements kubeconfig loading, context switching, and cluster operations
- Does NOT modify the user's kubeconfig file - operates in memory only

### Frontend (SvelteKit)
- Located in `src/`
- Responsive design with context dropdown and cluster dashboard
- Real-time updates and error handling
- Cross-platform UI that works on macOS, Windows, and Linux

## Key Features

1. **Context Management**: Load and switch between Kubernetes contexts
2. **Cluster Overview**: Display node count, pods, deployments, and cluster info
3. **Non-intrusive**: Never modifies the user's kubeconfig file
4. **Cross-platform**: Runs on Windows, macOS, and Linux

## Development Guidelines

- Use async/await patterns for Kubernetes API calls
- Implement proper error handling with user-friendly messages
- Follow Tauri best practices for secure command invocation
- Use TypeScript interfaces that match Rust structs for type safety
- Test Kubernetes logic with mock kubeconfig files

## Dependencies

### Rust Dependencies
- `kube`: Kubernetes client library
- `k8s-openapi`: Kubernetes API types
- `tauri`: Desktop app framework
- `tokio`: Async runtime
- `serde`: Serialization

### Frontend Dependencies
- `@tauri-apps/api`: Tauri frontend bindings
- SvelteKit for reactive UI components

## Testing

Use `tokio::test` for async tests with mock kubeconfig files to validate context switching and cluster operations without requiring a real Kubernetes cluster.
