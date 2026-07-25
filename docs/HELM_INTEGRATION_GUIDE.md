# Helm Integration Guide

## Overview
Kuboard provides a native, binary-free Helm integration. It does not require the `helm` binary to be installed on the user's system, as it interacts directly with the Kubernetes API to discover and decode Helm 3 release data stored in Secrets.

## Technical Implementation

### Storage Backend
Helm 3 stores release information as Kubernetes Secrets in the same namespace as the release.
- **Labels**: `owner=helm`, `name=<release-name>`, `status=<status>`, `version=<version>`
- **Data Key**: `release`
- **Format**: The data is triple-encoded:
    1.  JSON representation of the release object.
    2.  Gzip compressed.
    3.  Base64 encoded.
    4.  Base64 encoded (standard Kubernetes Secret encoding).

### Backend Commands (Rust)
- `kuboard_list_helm_releases`: Discovers all Helm releases across the cluster.
- `kuboard_get_helm_release_details`: Decodes a specific release version to retrieve values, manifest, and notes.

### Frontend Components
- `HelmTab.svelte`: Displays a grid of discovered releases.
- `TabbedContent.svelte`: Integrates the Helm management interface into the main dashboard.

## Discovery Logic
The backend scans all namespaces for Secrets with the label `owner=helm`. It then groups these secrets by release name and namespace, keeping only the latest revision for the primary display.

## Future Enhancements
- [ ] **Rollback Support**: Ability to switch to a previous revision.
- [ ] **Uninstall**: Remove a release and its associated resources.
- [ ] **Values Editor**: Integrate with the Live YAML Editor to modify release values and upgrade.
