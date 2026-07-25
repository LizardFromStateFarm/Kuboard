# 🏗️ Kuboard Architecture Overview

This document provides a comprehensive technical overview of the Kuboard application architecture, detailing the Rust backend, Svelte Kit frontend, IPC communication, state management, and future AI agent integration points.

---

## 🎯 **System Architecture & Key Principles**

Kuboard is designed as a high-performance desktop application to monitor, inspect, and manage Kubernetes clusters at scale (aimed as an advanced competitor to Lens and k9s).

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                             SvelteKit Frontend                              │
│ ┌───────────────────┐ ┌───────────────────┐ ┌─────────────────────────────┐ │
│ │  Component Layer  │ │   Svelte Stores   │ │     Monaco & Charting       │ │
│ │ (58 UI Panels)    │ │ (nav, editor, etc)│ │ (xterm.js, Chart.js, Monaco)│ │
│ └─────────┬─────────┘ └─────────┬─────────┘ └──────────────┬──────────────┘ │
└───────────┼─────────────────────┼──────────────────────────┼────────────────┘
            │                     │                          │
            ▼                     ▼                          ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          Tauri 2.0 IPC Bridge                               │
│                   (invoke handlers & event listeners)                       │
└─────────────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                            Rust / Tokio Backend                             │
│ ┌───────────────────┐ ┌───────────────────┐ ┌─────────────────────────────┐ │
│ │  AppState Async   │ │  Kubernetes API   │ │   Metrics & Watch Engine    │ │
│ │  Thread Safety    │ │  (kube-rs Client) │ │ (metrics.k8s.io & channels) │ │
│ └───────────────────┘ └───────────────────┘ └──────────────┬──────────────┘ │
                                                             │
                                                             ▼
                                                ┌──────────────────────────┐
                                                │   Grafana PromQL Bridge  │
                                                │  (Local & Grafana Cloud) │
                                                └──────────────────────────┘
```

### **Core Stack:**
- **Frontend**: SvelteKit 2 + Svelte 5 + TypeScript + Vite + Chart.js + xterm.js + Monaco Editor.
- **Backend**: Rust (Edition 2021) + Tauri 2.0 + Tokio async runtime + `kube-rs` + `k8s-openapi`.
- **Target Desktop**: Cross-Platform Windows, macOS, Linux natively packaged binaries.

---

## 🔧 **Backend Architecture (Rust / Tauri)**

The backend is modularized into clear domain responsibility layers under `src-tauri/src/`:

```
src-tauri/src/
├── main.rs                   # Entry point binary wrapper
├── lib.rs                    # Application builder & Tauri command registry
├── app_state.rs              # Thread-safe global state container (AppState)
├── types.rs                  # Rust data structures & Serde serialization shapes
├── utils.rs                  # Memory & CPU unit parsers (1000m -> 1.0, 1Gi -> bytes)
├── metrics/
│   └── mod.rs                # Metrics server client (/apis/metrics.k8s.io/v1beta1)
├── grafana/
│   └── mod.rs                # Grafana PromQL client (Local & Grafana Cloud)
├── kubernetes/
│   ├── mod.rs                # Kubeconfig loader & kube-rs client factory
│   ├── watch.rs              # Resource watch loop streams (Tokio tasks)
│   ├── exec.rs               # Pod exec session management
│   └── port_forward.rs       # Active port-forward tunnel manager
└── commands/                 # Domain-specific Tauri command modules (25 files)
    ├── cluster.rs            # Cluster overview & aggregated metrics
    ├── contexts.rs           # Kubeconfig context list & selection
    ├── pods.rs               # Pod list, details, logs, events, restart, delete
    ├── deployments.rs        # Deployment scaling, rollback, restart
    ├── replicasets.rs        # ReplicaSet management & pod queries
    ├── statefulsets.rs       # StatefulSet management & pod queries
    ├── daemonsets.rs         # DaemonSet management & pod queries
    ├── cronjobs.rs           # CronJob schedule, trigger, suspend, resume
    ├── networking.rs         # Services, Ingresses, IngressClasses, NetworkPolicies
    ├── storage.rs            # PVs, PVCs, StorageClasses
    ├── security.rs           # Roles, ClusterRoles, RoleBindings, ClusterRoleBindings, ServiceAccounts
    ├── resources.rs          # Nodes, Namespaces, ConfigMaps, Secrets
    ├── custom_resources.rs   # CRD discovery & custom resource instances
    ├── describe.rs           # Pod describe output builder
    ├── exec.rs               # Pod shell execution
    ├── port_forward.rs       # Local port forward controller
    ├── helm.rs               # Helm 3 release secret parser & details
    ├── graph.rs              # X-Ray resource dependency DAG builder
    ├── linter.rs             # K8s manifest security & hygiene linter
    ├── search.rs             # Fuzzy multi-resource search
    ├── yaml.rs               # Generic YAML fetcher & apply
    ├── grafana.rs            # Grafana configuration & PromQL query RPCs
    └── optimized.rs          # Experimental cached resource batching
```

### **Thread Safety & Async Concurrency (`AppState`)**
```rust
pub struct AppState {
    pub kubeconfig: Arc<RwLock<Option<Kubeconfig>>>,
    pub current_context: Arc<RwLock<Option<String>>>,
    pub current_client: Arc<RwLock<Option<Client>>>,
    pub pod_watcher: Arc<RwLock<PodWatcher>>,
    pub grafana_config: Arc<RwLock<Option<GrafanaConfig>>>,
}
```
All Kubernetes API and Grafana PromQL interactions leverage non-blocking Tokio tasks and `Arc<RwLock<...>>` for safe shared state across Tauri IPC threads.

---

## 🎨 **Frontend Architecture (SvelteKit / TypeScript)**

The UI is built with a modular component hierarchy in `src/lib/components/` (58 components) and stores in `src/lib/stores/`:

### **State Stores (`src/lib/stores/`)**:
- `logs.ts`: Manages multi-tab pod log & exec terminal session states (`logsSessionStore`, `activeLogsState`) isolated per cluster profile tab session (`tabSessionId`).
- `terminal.ts`: Helper utilities and store bindings for interactive pod exec terminal sessions.
- `nav.ts`: Maintains active tab selection (`workloads`, `nodes`, `config`, `network`, `storage`, `security`, `custom_resources`, `helm`, `linter`, `xray`).
- `editor.ts`: Controls Monaco YAML editor buffer & modal states.
- `keyboard.ts`: Central keyboard shortcut listener and help dialog manager.
- `xray.ts`: Maintains topology node selection and graph filters.

### **Component Responsibility Categories**:
1. **Layout & Header**: `Header.svelte`, `GlobalSearch.svelte`, `KeyboardManager.svelte`.
2. **Overview & Monitoring**: `ClusterOverview.svelte`, `ClusterMetrics.svelte`, `DonutChart.svelte`, `MetricsGraph.svelte`.
3. **Tabbed Content Navigation**: `TabbedContent.svelte`, `ResourceTabs.svelte`.
4. **Workload Management**: `WorkloadsTab.svelte`, `PodsPanel.svelte`, `DeploymentsPanel.svelte`, `StatefulSetsPanel.svelte`, `DaemonSetsPanel.svelte`, `ReplicaSetsPanel.svelte`, `CronJobsPanel.svelte`.
5. **Resource Details**: `PodDetails.svelte`, `DeploymentDetails.svelte`, `StatefulSetDetails.svelte`, `DaemonSetDetails.svelte`, `ReplicaSetDetails.svelte`, `CronJobDetails.svelte`, `ServiceDetails.svelte`, `ResourceDescribe.svelte`.
6. **Networking, Storage & Security**: `NetworkTab.svelte`, `ServicesPanel.svelte`, `IngressesPanel.svelte`, `StorageTab.svelte`, `PersistentVolumesPanel.svelte`, `SecurityTab.svelte`, `RolesPanel.svelte`.
7. **Developer & Inspection Tools**: `LogsWindow.svelte` (unified sticky bottom dock panel supporting multi-pod log streams and embedded `TerminalWindow.svelte` exec shells), `PortForwardManager.svelte`, `YamlEditor.svelte`, `HelmTab.svelte`, `LinterTab.svelte`, `XRayViewer.svelte`.

---

## 🤖 **AI-Native Extension Architecture Vision (MCP & Agent Protocol)**

To position Kuboard as an advanced, AI-native Kubernetes console, the backend architecture is designed to support the **Model Context Protocol (MCP)**:

1. **Local MCP Server Endpoint**:
   - Exposes a local JSON-RPC / SSE MCP server in the Rust backend (`src-tauri/src/mcp/`).
   - Allows external AI coding agents (Claude, Cursor, Antigravity) to query cluster state, pod logs, and event histories safely.

2. **Diagnostic Context Formatter**:
   - Automatically formats CrashLoopBackOff container logs and failed pod events into structured Markdown context blocks for AI analysis.

3. **Visual Diff Guardrail UI**:
   - Before an AI agent applies a resource modification, Kuboard renders a side-by-side Monaco diff view for developer verification.

4. **Grafana PromQL Metric Enrichment**:
   - Exposes `mcp_query_historical_metrics` tool allowing AI agents to evaluate historical PromQL data preceding `OOMKilled` or bottleneck incidents.
