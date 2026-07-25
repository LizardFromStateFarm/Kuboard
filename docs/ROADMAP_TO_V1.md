# 🚀 Kuboard v1.0 Finish-Line Roadmap

This document outlines the structured, prioritized path to completing Kuboard v1.0—a high-performance Rust + Svelte Kubernetes desktop console and AI-native cluster companion.

---

## 📊 **Feature Maturity & Bug Audit Scorecard**

| Feature Area | Maturity Score (1-5) | Status | Key Gaps & Explicit Bugs |
|--------------|----------------------|--------|--------------------------|
| **Cluster Metrics** | **4 / 5** | ✅ High | Live metrics server integration works. Missing container memory limit % gauge and disk I/O metrics. |
| **Workloads & ReplicaSets** | **3 / 5** | ⚠️ Needs Fixes | **Bug**: ReplicaSet details page calls non-existent commands `kuboard_get_replicaset_details` and `kuboard_get_pods_by_selector`. Saving YAML calls unregistered `kuboard_update_replicaset`. |
| **Pods & Container Logs** | **4 / 5** | ✅ High | Multi-container logs, smart follow, 5000 line capping. Exec terminal PTY needs full async channel streaming. |
| **Services & Networking** | **3 / 5** | 🔄 Partial | Services, Ingress, NetworkPolicies panels implemented. Port-forward background process management needs stream stability. |
| **Helm Integration** | **2 / 5** | ⚠️ Needs Overhaul | **Bug**: Helm release secret decoder fails on non-standard base64/protobuf storage formats. Missing rollback/uninstall actions. |
| **Grafana & Observability** | **1 / 5** | 💡 Planned | Integration planned for Phase 4 (Local Grafana + Grafana Cloud PromQL query bridge). |

---

## 🎯 **Finish-Line Execution Roadmap**

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ PHASE 1: Critical Bug Fixes & Navigation Repairs                            │
│ ├─ Fix ReplicaSetDetails command invocations & pod lookup                   │
│ ├─ Standardize YAML editor saving via kuboard_apply_resource_yaml           │
│ ├─ Fix Helm 3 secret release decoder fallback in Rust                       │
│ └─ Fix cross-component back button props & event handlers                   │
└─────────────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ PHASE 2: Core Feature Polish                                                │
│ ├─ Enhance Metrics visualizers (Memory limit % & CPU throttle gauges)       │
│ ├─ Complete Pod Exec PTY websocket streaming & Port-Forward lifecycle       │
│ └─ Add Helm release actions (rollback, uninstall, manifest diff)            │
└─────────────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ PHASE 3: AI Agent CLI & Ecosystem Integration (AI-Native Focus)             │
│ ├─ Local MCP (Model Context Protocol) Server endpoint in Rust backend       │
│ │   └─ Gated Security Model: Read-only by default + Svelte write pop-up    │
│ ├─ Diagnostic Event Streaming (auto-formatting CrashLoopBackOff context)     │
│ └─ Visual Diff Guardrail UI for agent-proposed YAML edits                   │
└─────────────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ PHASE 4: Enterprise Observability & Cluster Intelligence                    │
│ ├─ Grafana & Grafana Cloud PromQL Client Bridge in Rust                     │
│ ├─ Historical Time-Series Visualizer with Time-Range Selector               │
│ ├─ AI Diagnostic Context Enrichment via Grafana PromQL                       │
│ ├─ Live interactive visual cluster topology map (X-Ray force graph)         │
│ └─ Log anomaly grouping & pattern clustering engine                         │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### 🔨 **Phase 1: Critical Bug Fixes & Navigation Repairs**

- [x] **Fix ReplicaSet Details Invocation Mismatch**:
  - Update `src/lib/components/ReplicaSetDetails.svelte` line 91 to invoke `kuboard_get_replicaset` (instead of `kuboard_get_replicaset_details`).
  - Update line 114 to invoke `kuboard_get_replicaset_pods` (instead of `kuboard_get_pods_by_selector`).
  - **Definition of Done**: Verify clicking a ReplicaSet opens details, loads managed pods list without console errors, and clicking "← Back to ReplicaSets" navigates back to the ReplicaSets list without a full page reload or unhandled promise rejection.

- [x] **Fix ReplicaSet & Resource YAML Editing**:
  - Update `ReplicaSetsPanel.svelte` line 99 to call `kuboard_apply_resource_yaml` instead of unregistered `kuboard_update_replicaset`.
  - **Definition of Done**: Verify editing and saving a ReplicaSet YAML in the modal successfully applies the manifest via `kuboard_apply_resource_yaml` and returns clean status with no unhandled errors.

- [x] **Fix Helm Release Secret Decoder in Rust**:
  - Update `src-tauri/src/commands/helm.rs` (`decode_helm_release`) to handle both single-base64 gzipped JSON and dual-base64 gzipped release blobs gracefully.
  - **Definition of Done**: Verify `kuboard_list_helm_releases` correctly lists releases from Helm 3 secret storages across various cluster environments without throwing decoding errors.

- [x] **Repair Cross-Component Navigation Events**:
  - Verify and standardize back button event handlers (`on:back` vs `onBack` props) across `PodDetails`, `DeploymentDetails`, `ReplicaSetDetails`, `StatefulSetDetails`, `DaemonSetDetails`, `CronJobDetails`, `ServiceDetails`.
  - **Definition of Done**: Verify every detail view back button cleanly returns to its parent list view without breaking active tab state or context.

---

### 🎨 **Phase 2: Core Feature Polish**

- [x] **Container Metrics Visualizer Enhancements**:
  - Add container memory limit percentage indicator and CPU throttling metrics display to `PodsPanel` and `MetricsGraph`.
- [x] **Per-Container Log Selection in Log Panel**:
  - Add container dropdown selector in `LogsWindow.svelte` header allowing users to dynamically switch between main containers, sidecars, and init-containers (`spec.initContainers`).
- [x] **Aggregated Workload Multi-Pod Log Streaming**:
  - Added dedicated "📋 Logs" button to action bars & right-click menus across `DeploymentsPanel`, `ReplicaSetsPanel`, `StatefulSetsPanel`, `DaemonSetsPanel`, and their respective detail view headers (`DeploymentDetails`, `ReplicaSetDetails`, `StatefulSetDetails`, `DaemonSetDetails`).
  - Implemented Rust backend multi-pod log multiplexer in `src-tauri/src/commands/metrics.rs` (`kuboard_get_workload_logs`) to aggregate and stream merged, color-coded logs across all constituent pods matching workload label selectors.
- [x] **Interactive Pod Exec Shell Terminal (PTY Channel)**:
  - Complete full interactive PTY terminal streaming (`TerminalWindow.svelte` & `src-tauri/src/kubernetes/exec.rs`) over Tauri async channels.
  - Support container selection (`/bin/sh`, `/bin/bash`, `zsh`), PTY terminal resize events (`SIGWINCH`), full xterm.js ANSI color emulation, and copy/paste shortcuts.
- [x] **Helm Operations Stability**:
  - Add release rollback to revision, release uninstall, and values inspection in `HelmTab.svelte`.
- [x] **UI Vector Icon Modernization (`@lucide/svelte`)**:
  - Installed `@lucide/svelte` package for SVG vector icon modernization across header tabs, navigation bars, resource tables, and action menus.

---

### 🛠️ **Phase 2 Part 2: Senior Developer Power Features & Safety Guardrails**

- [x] **Pod Details Per-Container Metrics Expansion**:
  - Add container selector dropdown in `PodDetails.svelte` Resource Usage section allowing users to inspect metrics graphs for individual containers.
- [x] **Dotted Limits & Requests Threshold Lines on Metrics Graphs**:
  - Add horizontal dotted threshold reference lines in `MetricsGraph.svelte` representing resource limits (`spec.containers[].resources.limits`) and resource requests (`spec.containers[].resources.requests`).
- [x] **Workload Metrics Graphs (Deployments & ReplicaSets)**:
  - Integrate aggregated time-series metrics graphs into `DeploymentDetails.svelte` and `ReplicaSetDetails.svelte` showing workload-wide CPU/Memory utilization trends over time.
- [x] **Global Resource Command Palette (`Cmd/Ctrl + K`)**:
  - Implement an instant modal launcher allowing senior devs to search and switch to any Pod, Service, Deployment, ConfigMap, or Namespace in <100ms.
- [x] **Production Cluster Environment Safety Badges**:
  - Add customizable color-coded environment tags (🔴 `production`, 🟡 `staging`, 🟢 `development`) to context tabs and header bars to prevent accidental deletion or restart on production clusters.
- [x] **Sanitized Manifest Clipboard Export & Secret Redaction**:
  - One-click copy/download of live YAML manifests with auto-redacted Secret data (`[REDACTED]`) for sharing diagnostics safely.

---

### 🔬 **Phase 2 Part 3: Advanced Diagnostic Workflows & Topology Mini-Visualizer**

- [x] **Interactive Secret Value Editor & Base64 Encoder/Decoder**:
  - In-line base64 reveal/conceal toggle, plain-text editing of Secret keys/values (`stringData` / `data`), and instant base64 encoding on save.
- [x] **Universal Dedicated Details Views for All Resource Types**:
  - Dedicated detail view panels (`SecretDetails.svelte`, `RoleDetails.svelte`, `ServiceAccountDetails.svelte`, `ConfigMapDetails.svelte`, `IngressDetails.svelte`) providing deep inspection for RBAC rules, bound tokens, secret keys, ingress routing rules, and config payloads.
- [x] **Resource Dependency Mini-Topology Graph**:
  - Embedded resource hierarchy mini-DAG in `PodDetails`, `DeploymentDetails`, and `ServiceDetails` showing live links to parent Controllers, Pods, Services, PVCs, ConfigMaps, and Secrets.
- [x] **Smart Log Stack Trace & Error Fingerprinting**:
  - Auto-detection and color highlighting for stack traces (`panic:`, `NullPointerException`, `FATAL`, `ERROR`) in `LogsWindow.svelte` with quick jump-to-error navigation controls.
- [x] **Persistent Volume Storage Capacity & Disk Utilization Visualizer**:
  - Add PVC vs PV capacity gauges, storage class breakdown, and volume mount paths visualization in `PersistentVolumeClaimsPanel.svelte`.
- [x] **Interactive Multi-Resource Bulk Action Toolbar**:
  - Multi-select checkbox support in `ResourceTable.svelte` enabling bulk pod restart, bulk resource deletion, and label filtering.

---

### 🧰 **Phase 2 Part 4: Config Tab Overhaul, Helm Integration & Event Stream Alert Hub**

- [x] **Config Tab Overhaul & Resource Unification**:
  - Move Helm releases under Config tab (`HelmTab.svelte` integration).
  - Remove Secrets from Config tab (Secrets managed in Security tab).
  - Overhaul ConfigMaps with a searchable menu/table (`ResourceTable.svelte`), dedicated `ConfigMapDetails.svelte` view panel with live value editing, payload diffing, and revision rollbacks.
- [x] **Live Cluster Events Stream & Anomaly Alert Hub**:
  - Upgrade existing cluster events widget (`EventsPanel.svelte` / `ClusterOverview.svelte`) with real-time event timeline, warning/failed filter tabs, severity badges, and desktop toast notifications.
- [x] **Helm Release & Revision Visualizer**:
  - List installed Helm releases, inspect values.yaml, compare revision diffs, and perform 1-click rollbacks.
- [x] **Multi-Pod Container Log Aggregator & Side-by-Side Split View**:
  - Combined log streaming from multiple selected pods with colored pod-prefix tags and side-by-side split log windows.
- [x] **Cluster Architecture & Diagnostic Report Generator**:
  - 1-click export of structured cluster health markdown/PDF report summarizing nodes, crashing pods, resource pressure, and PVC capacity.

---

### 📈 **Phase 3: Icon Modernization, Observability & Cluster Intelligence**

- [x] **UI Icon Modernization & Vector Symbol System**:
  - Replace unicode emoji icons across all components (`Header`, `TabbedContent`, `ResourceTable`, `WorkloadsTab`, `ConfigTab`, `SecurityTab`, `NetworkTab`, `StorageTab`, `LogsWindow`, `PodDetails`, `ReportGeneratorModal`) with an open-source Svelte icon system (`lucide-svelte` or custom SVG icons) for a polished, professional enterprise aesthetic.
- [x] **Svelte Historical Time-Series Visualizer**:
  - Add configurable time-range selectors (15m, 1h, 6h, 24h, 7d) to `PodDetails`, `NodesTab`, and `ClusterMetrics` views.
  - Render native, lightweight Chart.js time-series graphs replacing generic instantaneous gauges.
- [ ] **Rust Grafana Client Bridge (`src-tauri/src/grafana/`)**:
  - Support connection configurations for both Local Grafana instances (e.g., `http://localhost:3000` or `http://grafana.monitoring.svc:3000`) and Grafana Cloud (`https://<instance>.grafana.net`) using Bearer Token or Basic Auth.
  - Implement `/api/ds/query` PromQL range query wrapper for historical CPU, RAM, Disk I/O, and Network traffic data.
  - Add auto-discovery for Prometheus/Thanos datasources via Grafana `/api/datasources` API.
- [ ] **Live Interactive Visual Cluster Topology Map**:
  - Upgrade `XRayViewer.svelte` to render an interactive force-directed topology graph showing relationships between Ingress → Service → Deployment → ReplicaSet → Pods.
- [ ] **Log Anomaly Grouping & Error Pattern Recognition**:
  - Implement client-side log line fingerprinting in `LogsWindow.svelte` to collapse repeated stack traces and highlight rare anomaly logs automatically.

---

### 🤖 **Phase 4: AI Agent CLI & Ecosystem Integration**

- [ ] **Local MCP (Model Context Protocol) Server Endpoint in Rust**:
  - Implement a local MCP JSON-RPC server (`src-tauri/src/mcp/`) exposing cluster state to external AI agents (Claude, Cursor, Antigravity CLI):
    * `mcp_get_cluster_health`: Returns node statuses, resource pressure, and warning counts.
    * `mcp_get_pod_diagnostics`: Returns full pod spec, conditions, status, events, and recent error log tail.
    * `mcp_list_resources`: Exposes resource inventory safely.
  - **Security & Mutation Gating Model**:
    * **Read-Only Default**: All MCP query tools execute read-only.
    * **Interactive Write Gating**: Any mutation command (e.g. `kubectl apply`, `kubectl delete`, `kubectl scale`) initiated by an AI agent over MCP blocks execution and triggers an explicit modal confirmation pop-up in the Svelte UI. The mutation is executed ONLY after human user approval.

- [ ] **Diagnostic Event Streaming**:
  - Implement automatic formatting of `CrashLoopBackOff`, `OOMKilled`, and `ImagePullBackOff` events.
  - Generate one-click "Copy AI Context" markdown prompts combining container exit codes, termination reasons, and log tail for instant LLM troubleshooting.

- [ ] **Visual Diff Guardrail UI for Agent Edits**:
  - Add a dedicated side-by-side Monaco diff modal in `YamlEditor.svelte`.
  - When an AI agent proposes YAML resource modifications, Kuboard presents a visual diff highlighting additions/deletions, requiring explicit developer click-to-apply approval.

- [ ] **AI Diagnostic Context Enrichment via Grafana**:
  - Expose `mcp_query_historical_metrics` tool on the Rust MCP server so AI agents can inspect metric trends (e.g. 1h memory slope, CPU throttle rate) preceding cluster anomalies or `OOMKilled` events.

---

### 🎨 **Phase 5: Enterprise Polish, Performance & Desktop FX**

- [ ] **Unified Lucide Icon Suite**:
  - Audit all remaining legacy emoji icons across modals, notifications, context menus, and tooltips, replacing them with consistent `lucide-svelte` icons.
- [ ] **Virtualized Table Rendering**:
  - Integrate virtual list scrolling for large cluster tables (e.g., >1000 pods or events) to maintain 60fps UI responsiveness.
- [ ] **Keyboard Shortcut Quick Palette**:
  - Interactive command palette (`Ctrl+K` / `Cmd+K`) for switching contexts, jumping to resources, opening logs/terminals, and triggering health reports.
- [ ] **Multi-Window Desktop Support**:
  - Enable detaching terminal sessions and log streams into standalone Tauri OS native windows.

- [ ] **AI Diagnostic Context Enrichment via Grafana**:
  - Expose `mcp_query_historical_metrics` tool on the Rust MCP server so AI agents can inspect metric trends (e.g. 1h memory slope, CPU throttle rate) preceding cluster anomalies or `OOMKilled` events.

