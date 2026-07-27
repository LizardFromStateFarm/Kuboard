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
│ PHASE 3: Icon Modernization, Observability & Topology                       │
│ ├─ Grafana & Grafana Cloud PromQL Client Bridge in Rust                     │
│ ├─ Historical Time-Series Visualizer with Time-Range Selector               │
│ ├─ Live interactive visual cluster topology map (X-Ray force graph)         │
│ └─ Log anomaly grouping & pattern clustering engine                         │
└─────────────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ PHASE 4: Enterprise Resource Expansion & Lens/K9s Feature Parity            │
│ ├─ 360° Resource Hyperlinking (Node & Namespace links across all details)   │
│ ├─ Complete Lens/K9s Catalog (Jobs, HPA, VPA, PDB, Endpoints, Gateway API)   │
│ └─ Namespaces Management Panel & Quotas                                     │
└─────────────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ PHASE 4 Part 2: Cluster Operations, Multi-Kubeconfig & UX Friction Polish   │
│ ├─ Node Maintenance Lifecycle (Cordon, Uncordon, Drain actions)             │
│ ├─ Universal Resource Creation & YAML Apply Wizard (`kubectl apply -f`)     │
│ ├─ Multi-Kubeconfig Import & Custom Path File Selector                      │
│ ├─ Global Toast Notifications, Network Health Bar & RBAC Permission Badges  │
│ ├─ Persistent Active Port-Forward Header Bar & Browser Quick Launch         │
│ ├─ Tabbed Multi-Session Exec Shell Terminal & Log File Export (.log)        │
│ └─ ConfigMap/Secret Edit Rollout Restart Prompt & Workload Rollout Progress  │
└─────────────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ PHASE 5: High-Performance Mass Cluster Engine (57+ Nodes, 10,000+ Pods)     │
│ ├─ Rust metadata.managedFields payload stripping (65% payload reduction)   │
│ ├─ Tokio K8s Informer / Delta Watch Streams (no 10s full-JSON polling)      │
│ ├─ Virtualized Table Windowing (`svelte-virtual` 60fps rendering)           │
│ └─ Multi-Context Tab Session Isolation                                      │
└─────────────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ PHASE 6: AI Agent CLI, Local MCP Server & Intelligent Companion             │
│ ├─ Local MCP (Model Context Protocol) Server endpoint in Rust               │
│ │   └─ Gated Security Model: Read-only default + Svelte write approval modal│
│ ├─ Diagnostic Event Streaming (auto-formatting CrashLoopBackOff context)     │
│ └─ Visual Diff Guardrail UI for agent-proposed YAML edits                   │
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
- [x] **UI Vector Icon Modernization (`lucide-svelte`)**:
  - Replaced legacy text emojis with clean, responsive vector Lucide icons (`lucide-svelte`) across 100% of the frontend UI, including Cluster Loading screen (`🔄 Loading Cluster Data`), Welcome & Error screens, QuickActionsMenu, GlobalSearch, Header context badges, Logs & Terminal windows, Resource Overview, and all 12+ Resource Details sheets.

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
  - Replaced all unicode emojis across `Header`, `ResourceTabs`, `TabbedContent`, `ResourceTable`, `WorkloadsTab`, `ConfigTab`, `SecurityTab`, `NetworkTab`, `StorageTab`, `EventsPanel`, `PodsPanel`, `LinterTab`, `ClusterMetrics`, `ReportGeneratorModal`, `PodDetails`, and sub-panels using `lucide-svelte` SVG vector icons.
- [x] **Svelte Historical Time-Series Visualizer**:
  - Added duration selector (`15m`, `30m`, `1h`, `6h`, `24h`) in `MetricsGraph.svelte` for Chart.js time-series historical data resolutions.
- [x] **Rust Grafana Client Bridge (`src-tauri/src/commands/grafana.rs`)**:
  - Support connection configurations for both Local Grafana instances (e.g., `http://localhost:3000` or `http://grafana.monitoring.svc:3000`) and Grafana Cloud (`https://<instance>.grafana.net`) using Bearer Token or Basic Auth.
  - Implement `/api/v1/query_range` PromQL range query wrapper for historical CPU, RAM, Disk I/O, and Network traffic data.
  - Add auto-discovery for Prometheus/Thanos datasources via Grafana `/api/datasources` API.
- [x] **Live Interactive Visual Cluster Topology Map**:
  - Upgraded `XRayViewer.svelte` to render interactive topology graph with clickable resource node cards that trigger direct navigation across Ingress → Service → Deployment → ReplicaSet → Pods.
- [x] **Log Anomaly Grouping & Error Pattern Recognition**:
  - Implemented client-side log line fingerprinting in `LogsWindow.svelte` to collapse repeated log patterns with count badges (`[x15]`), highlight rare anomaly logs, and provide expand-in-place inspection.
- [x] **Resource Details Toolbar Overhaul & Click-to-Copy Fields**:
  - Overhaul resource action controls across all resource details views (`PodDetails`, `DeploymentDetails`, `StatefulSetDetails`, `DaemonSetDetails`, `ServiceDetails`, `ConfigMapDetails`, `SecretDetails`, `IngressDetails`, etc.).
  - Replace sub-menu actions dropdown with direct, single-click toolbar action buttons (e.g. direct "Logs", "Exec Shell", "Edit YAML", "Port Forward", "Restart", "Delete" buttons) placed directly on the details tab toolbar for streamlined 1-click execution.
  - Remove redundant "Copy Name" and "Copy IP" action menu items; enable direct click-to-copy on all resource names, Pod IPs, Node IPs, and Cluster IPs across all details headers and metadata cards (matching the existing Pod name banner click-to-copy pattern).
- [x] **Pod Details Per-Container Metrics Selector**:
  - Add container dropdown selector to the Resource Usage section in `PodDetails.svelte` allowing developers to inspect CPU/Memory metrics for specific containers, sidecars, and init-containers.
- [x] **Header & Tab Live Indicator Cleanup**:
  - Remove redundant status pills across panel sub-headers; live stream metrics operate seamlessly whenever data is present.
- [x] **Pod Details Icon Modernization**:
  - Replace remaining legacy emojis in `PodDetails.svelte` (`🌐 Resource Topology`, `💾 Memory Usage`, `⚡ CPU Usage`) with `lucide-svelte` vector icons (`Network`, `HardDrive`, `Cpu`).
- [x] **Interactive Mini-Topology DAG Navigation**:
  - Upgrade resource nodes in the mini-topology panel (Pods, Services, Deployments, PVCs, ConfigMaps, Secrets) to be fully interactive, clickable links navigating directly to target resource details (matching "Controlled By" link behavior).
- [x] **Deployment Details Owned Resources & Pod Count Repair**:
  - Fix pod lookup and label selector matching logic in `DeploymentDetails.svelte` so owned ReplicaSets and Pods accurately display live counts and active pod lists instead of remaining at 0.
- [x] **Cluster Events Time-Frame Selector**:
  - Add duration selector dropdown (`15m`, `30m`, `1h`, `6h`, `24h`, `All`) in `EventsPanel.svelte` allowing developers to dynamically filter event stream by time window.
- [x] **Nodes Tab Complete Overhaul**:
  - Overhaul `NodesTab.svelte` with a 2-tier layout: (1) Node Pools summary table with click-to-filter capability, and (2) Interactive All Nodes table with persistent search bar, status badges, CPU/RAM allocatable capacity gauges, and node action controls (Copy IP, Details).
- [x] **Helm Tab Refresh Button Repair**:
  - Fixed duplicated HTML button attributes inside `HelmTab.svelte` refresh button template rendering raw div class strings.
- [x] **Logs Window Tab Cleanup & Auto-Close**:
  - Integrated `closeGlobalLogTab` and `setGlobalLogsOpen` in `LogsWindow.svelte` so closing all log tabs automatically closes the floating window and prevents closed tabs from restoring upon re-opening.
- [x] **Resource Details Action Bar Overhaul & Direct Action Buttons**:
  - Updated `PodDetails`, `DeploymentDetails`, `StatefulSetDetails`, `DaemonSetDetails`, `ServiceDetails`, `CronJobDetails`, and `ReplicaSetDetails` to replace nested dropdown action menus with direct single-click action buttons (`Logs`, `Port Forward`, `Scale`, `Restart`, `Trigger Job`, `Edit YAML`, `Delete`) and integrated live Monaco `YamlEditor` and click-to-copy fields.
- [x] **Node Pool Details View**:
  - Added dedicated interactive Node Pool Details view in `NodesTab.svelte` displaying aggregate pool capacity (healthy node count, total CPU cores, total RAM capacity, role/instance type) and searchable pool member nodes table with click-to-copy internal IPs and direct node details inspection.
- [x] **Settings & Preferences Overhaul with Grafana Integration**:
  - Overhauled `ThemeSwitcher.svelte` into a multi-tab Settings Modal with sub-tabs for Appearance & Color Themes and Grafana Integration (Grafana Endpoint URL, API Token / Service Account Key, target Prometheus/Thanos datasource selector, live HTTP connection health testing, and localStorage persistence).
- [x] **Interactive Controller Parents & Multi-Tier Topology Maps**:
  - Connected `MiniTopologyDAG.svelte` and `XRayViewer.svelte` to `navigationStore` so clicking any controller parent (Deployments, StatefulSets, DaemonSets, ReplicaSets), current resource card, or child dependency (ConfigMaps, Secrets, PVCs) triggers instant cross-tab navigation. Expanded Rust backend graph resolver (`src-tauri/src/commands/graph.rs`) to include StatefulSet, DaemonSet, and Ingress routing edges.
- [x] **Security Tab RBAC Details Views (`RoleDetails.svelte` & `RoleBindingDetails.svelte`)**:
  - Created dedicated interactive Details views for Roles, ClusterRoles, RoleBindings, and ClusterRoleBindings in `SecurityTab.svelte`. Includes RBAC policy rules breakdown tables with verb badges (`get`, `list`, `create`, `delete`), bound Role ref cards with direct cross-inspection links, and subjects tables (Users, Groups, ServiceAccounts).
- [x] **Sub-Nav Bar Design System Unification**:
  - Standardized all sub-nav tab bars across `WorkloadsTab.svelte`, `ConfigTab.svelte`, `SecurityTab.svelte`, `NetworkTab.svelte`, and `StorageTab.svelte`. Unified active tab highlights (`rgba(59, 130, 246, 0.1)` glassmorphic primary pills), hover states (`rgba(255, 255, 255, 0.05)`), font weights, and icon alignments.

---

---

### 🏛️ **Phase 4: Enterprise Resource Expansion & Lens/K9s Feature Parity**

- [ ] **Universal Deep Cross-Linking & Interconnected Resource Navigation**:
  - **Node Direct Link**: Make Node names clickable across all Pods, Workloads, and Events tables & detail views -> opens target Node Details view.
  - **Namespace Direct Link**: Make Namespace badges & labels clickable everywhere -> opens dedicated `NamespaceDetailsModal.svelte` (displaying status, phase, labels, annotations, creation timestamp, resource quotas, limit ranges, and active workloads summary).
  - **Endpoints & EndpointSlices Interconnection**: Show controlled-by links, target IP addresses, port maps, and serving Pods with 100% clickable links navigating directly to Pod details.
  - **360° Resource Hyperlinking**: Universal cross-navigation across Endpoints <-> Services <-> Pods <-> Nodes <-> Owner Controllers <-> Storage PVCs <-> Secrets <-> ConfigMaps.

- [ ] **Full Lens/K9s-Parity Resource Catalog Expansion**:
  - **Workloads Tab Expansion**:
    - Add `ReplicationControllers` panel & details view.
    - Add `Jobs` panel & details view (execution duration, active/succeeded/failed pod counts).
    - Add `HorizontalPodAutoscalers` (HPA) panel & details view (min/max replicas, current/target CPU & RAM utilization metrics).
    - Add `VerticalPodAutoscalers` (VPA) panel & details view (recommendations, target containers).
    - Add `PodDisruptionBudgets` (PDB) panel & details view (min available, max unavailable, allowed disruptions).
  - **Config Tab Expansion**:
    - Add `ResourceQuotas` panel & details view (hard vs used limits per namespace).
    - Add `LimitRanges` panel & details view (default min/max cpu & memory limits per container/pod).
    - Add `PriorityClasses` panel & details view (global cluster priority values, preemption policy).
    - Add `RuntimeClasses` panel & details view (container runtime handlers, overheads).
    - Add `Leases` panel & details view (leader election leases, renew timestamps).
    - Add `MutatingWebhookConfigurations` & `ValidatingWebhookConfigurations` panels & details views (failure policy, match conditions, webhook endpoints).
    - Add `AdmissionPolicies`, `ValidatingAdmissionPolicies`, and `ValidatingAdmissionPolicyBindings` panels & details views.
  - **Network Tab Expansion**:
    - Add `Endpoints` & `EndpointSlices` panels & details views.
    - Add **Gateway API** sub-category (`GatewayClasses`, `Gateways`, `HTTPRoutes`, `BackendTLSPolicies`, `ReferenceGrants`, `TLSRoutes`) with dedicated route inspection.
  - **Cluster Tab Expansion**:
    - Add `Namespaces` management panel & details view (1-click namespace creation, deletion, status monitoring, and quota usage).

---

### 🔧 **Phase 4 Part 2: Cluster Operations, Multi-Kubeconfig & UX Friction Polish**

- [ ] **Node Lifecycle & Maintenance Controls (`NodesTab.svelte` & Rust Backend)**:
  - Add **Cordon / Uncordon** toggle actions to Node tables and details toolbar (`kubectl cordon` / `uncordon` equivalents via `kube-rs`).
  - Add **Drain Node** action modal with options for grace period, ignoring DaemonSets, and deleting local data.

- [ ] **Universal Resource Creation Wizard & Manifest Applicator (`kubectl apply -f`)**:
  - Add global `➕ Create Resource` button in `Header.svelte`.
  - Upgrade `YamlEditor.svelte` with a creation modal providing standard starter templates (Pod, Deployment, Service, ConfigMap, Ingress, Job) and single-click `kubectl apply` execution.

- [ ] **Multi-Kubeconfig File Importer & Custom Context Switcher**:
  - Add "Import Kubeconfig" modal in `Header.svelte` allowing users to load external `.kube/config` files from disk or paste raw YAML kubeconfig strings without overwriting default user configuration.

- [ ] **Global Toast Notification System & Network Health Status Bar**:
  - Implement top-right toast notification store and UI overlay for async task feedback (success, warnings, API failure tracebacks).
  - Add real-time K8s API connectivity indicator (🟢 Connected / 🟡 Degraded / 🔴 Disconnected) in `Header.svelte`.

- [ ] **Live Operation Task & Resource Status Progress Drawer**:
  - Floating bottom/side status popup or drawer triggered whenever a user issues mutating operations (e.g. Pod deletion, Node cordon/drain, YAML apply, Deployment scale/restart).
  - Tracks live execution state (`Pending` -> `In-Progress` -> `Resource Terminating / Applying` -> `Success / Error`).
  - Automatically updates resource status in real-time and auto-dismisses/clears once the operation finishes and resource reaches terminal state.

- [ ] **Persistent Active Port-Forward Status Bar & Browser Launcher**:
  - Add a persistent status bar drawer showing active port-forward sessions (`🔌 2 Active Forwards`) with 1-click "Open in Browser" and single-click terminate actions across all views.

- [ ] **Multi-Tab Exec Shell Terminal Sessions**:
  - Upgrade `TerminalWindow.svelte` to support tabbed terminal sessions, allowing developers to maintain active interactive PTY shells in multiple containers simultaneously.

- [ ] **Log Export (`.log`), ISO Timestamp Toggle & RegEx Filtering**:
  - Add single-click "📥 Download Logs" (.log text export) in `LogsWindow.svelte`.
  - Add ISO timestamp toggle and real-time RegEx pattern filter input in log search bar.

- [ ] **ConfigMap / Secret Edit Rollout Restart Prompt**:
  - When saving ConfigMap or Secret changes, detect dependent workload Deployments/DaemonSets and display a prompt offering 1-click rolling restart (`kubectl rollout restart`).

- [ ] **Label Selector Filter Bar (`key=value` Tag Filtering)**:
  - Add key-value label filter builder in `ResourceTable.svelte` enabling filtering across workload rows by Kubernetes labels (e.g. `app=nginx`, `environment=prod`).

- [ ] **RBAC SelfSubjectAccessReview Permission Degradation**:
  - Execute permission reviews on context switch to gracefully badge or disable restricted resource actions for non-cluster-admin user roles.

- [ ] **Complete Stubbed Keyboard Navigation**:
  - Wire up missing key bindings in `KeyboardManager.svelte` (`g g` top, `G` bottom, `Enter` details, `e` edit yaml, `x` X-Ray) and add a visual shortcut cheat sheet modal (`?`).

- [ ] **Terminal & Log Theme Synchronization**:
  - Ensure light theme active state in `ThemeSwitcher.svelte` updates log/terminal background contrast and text readability.

---

### ⚡ **Phase 5: High-Performance Mass Cluster Engine (57+ Nodes, 10,000+ Pods) & Desktop UX**

- [ ] **Mass Cluster Performance & Scalability Engine**:
  - **ManagedFields Stripping in Rust Backend**: Strip `metadata.managedFields` from dynamic K8s JSON objects in Rust backend prior to Tauri IPC serialization. Cuts IPC payload size by 65% across 10,000+ pods.
  - **Rust K8s Informer / Delta Watch Streams**: Replace 10-second full JSON polling loops with Tokio-backed `kube::runtime::watcher` Informers. Maintains local memory cache in Rust `AppState` and streams only diffs (`Added`, `Modified`, `Deleted`) over Tauri `emit()` channels.
  - **Virtualized Table Windowing**: Integrate virtual list scrolling (`svelte-virtual`) into `ResourceTable.svelte` to render 10,000+ Pod rows without UI frame drops or memory leaks.
  - **Multi-Context Tab Session Isolation**: Enable independent context bindings per tab session so developers can open `prod-us-east` in Tab 1 and `staging-eu-west` in Tab 2 side-by-side without global context collision.
- [ ] **Enterprise Desktop UX**:
  - **Unified Lucide Vector Icon Suite**: Final audit of all remaining legacy emoji icons across modals, notifications, context menus, and tooltips.
  - **Keyboard Shortcut Command Palette (`Ctrl+K` / `Cmd+K`)**: Interactive command palette for switching contexts, jumping to resources, opening logs/terminals, and triggering health reports.
  - **Multi-Window Desktop Support**: Support detaching terminal sessions and log streams into standalone OS native windows.

---

### 🤖 **Phase 6: AI Agent CLI, Local MCP Server & Intelligent Companion (Final AI Phase)**

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

