# 🎨 Kuboard UI Organization Guide

This guide explains the modular organization of the Kuboard UI components, state stores, and layout structure.

## 📁 Directory Structure

```
src/
├── app.html                      # HTML root template
├── routes/
│   ├── +layout.ts                # Layout configuration (SSR disabled for Tauri desktop)
│   ├── +page.svelte              # Main application dashboard (orchestrates layout, tabs & interactive welcome context selection grid)
│   └── page-optimized.svelte     # Experimental cached dashboard
└── lib/
    ├── components/               # Reusable UI components (58 components)
    │   ├── Header.svelte         # Top bar with context selector, mode indicator, refresh
    │   ├── GlobalSearch.svelte   # Command palette / global resource search modal
    │   ├── KeyboardManager.svelte# Keyboard shortcuts listener & overlay
    │   ├── ClusterOverview.svelte# Cluster summary, nodes overview, debug console
    │   ├── ClusterMetrics.svelte # Donut charts & node pool capacity visualizers
    │   ├── DonutChart.svelte     # SVG/CSS donut chart visualizer
    │   ├── MetricsGraph.svelte   # Chart.js time-series utilization graph with 15m/30m/1h/6h/24h resolution toggles
    │   ├── ResourceTable.svelte  # Generic table with persistent search bar, multi-namespace controls slot, empty-state persistence, sort, and bulk actions
    │   ├── MultiNamespaceSelect.svelte # Multi-select namespace dropdown filter with search popover, batch checkbox selection, and instant event dispatching
    │   ├── TabbedContent.svelte  # Main tab container with isolated per-session navigation (tabSessionId)
    │   ├── ResourceTabs.svelte   # Top-level tab navigation bar with single-line auto-shrinking flex layout
    │   │
    │   ├── WorkloadsTab.svelte   # Container for workload sub-tabs (Pods, Deployments, StatefulSets, DaemonSets, CronJobs, ReplicaSets, Services) with multi-select namespace filter & persistent search controls
    │   │   ├── PodsPanel.svelte           # Pod list, search, multi-namespace auto-refresh filtering, watch stream
    │   │   ├── PodDetails.svelte          # Pod detail view with single-click action bar (Logs, Exec, Port Forward, Edit YAML, Delete), container metrics selector, and click-to-copy fields
    │   │   ├── PodConditions.svelte       # Pod conditions list
    │   │   ├── PodEvents.svelte           # Pod-specific event stream
    │   │   ├── PodVolumes.svelte          # Pod volume mounts view
    │   │   ├── DeploymentsPanel.svelte    # Deployment list & scaling
    │   │   ├── DeploymentDetails.svelte   # Deployment detail view with single-click action bar (Logs, Scale, Restart, Edit YAML, Delete) and owned pod counts
    │   │   ├── StatefulSetsPanel.svelte   # StatefulSet list & management
    │   │   ├── StatefulSetDetails.svelte  # StatefulSet detail view with single-click action bar (Logs, Scale, Edit YAML, Delete)
    │   │   ├── DaemonSetsPanel.svelte     # DaemonSet list
    │   │   ├── DaemonSetDetails.svelte    # DaemonSet detail view with single-click action bar (Logs, Edit YAML, Delete)
    │   │   ├── ReplicaSetsPanel.svelte    # ReplicaSet list
    │   │   ├── ReplicaSetDetails.svelte   # ReplicaSet detail view with single-click action bar (Logs, Scale, Edit YAML, Delete)
    │   │   ├── CronJobsPanel.svelte       # CronJob list & manual trigger
    │   │   ├── CronJobDetails.svelte      # CronJob detail view with single-click action bar (Trigger Job, Edit YAML, Delete)
    │   │   └── ServiceDetails.svelte      # Service detail view with single-click action bar (Port Forward, Edit YAML, Delete)
    │   │
    │   ├── NodesTab.svelte       # Cluster node list & resource capacity cards
    │   ├── ConfigTab.svelte      # ConfigMaps & Secrets management
    │   ├── NetworkTab.svelte     # Network resources container
    │   │   ├── ServicesPanel.svelte           # Kubernetes Services list
    │   │   ├── IngressesPanel.svelte          # Ingress rules list
    │   │   ├── IngressClassesPanel.svelte     # IngressClasses list
    │   │   └── NetworkPoliciesPanel.svelte    # NetworkPolicies list
    │   │
    │   ├── StorageTab.svelte     # Persistent storage container
    │   │   ├── PersistentVolumesPanel.svelte       # PV management
    │   │   ├── PersistentVolumeClaimsPanel.svelte  # PVC management
    │   │   └── StorageClassesPanel.svelte          # StorageClasses management
    │   │
    │   ├── SecurityTab.svelte    # Security & RBAC container
    │   │   ├── RolesPanel.svelte                 # Roles list
    │   │   ├── ClusterRolesPanel.svelte          # ClusterRoles list
    │   │   ├── RoleBindingsPanel.svelte          # RoleBindings list
    │   │   ├── ClusterRoleBindingsPanel.svelte   # ClusterRoleBindings list
    │   │   └── ServiceAccountsPanel.svelte       # ServiceAccounts list
    │   │
    │   ├── CustomResourcesTab.svelte # CRD discovery & custom resource instances
    │   ├── HelmTab.svelte        # Helm releases viewer & chart metadata
    │   ├── LinterTab.svelte      # Best-practice Kubernetes manifest linter
    │   ├── XRayViewer.svelte     # Visual cluster resource dependency graph
    │   │
    │   ├── LogsWindow.svelte          # Unified sticky bottom dock panel with multi-tab pod logs & embedded exec terminals
    │   ├── TerminalWindow.svelte      # Web-based pod container exec terminal (xterm.js) embedded in LogsWindow tabs
    │   ├── PortForwardManager.svelte  # Active local port forward sessions
    │   ├── ResourceDescribe.svelte    # kubectl describe text & YAML representation
    │   ├── YamlEditor.svelte          # Live Monaco editor for resource YAML
    │   ├── QuickActionsMenu.svelte    # Context menu for quick resource operations
    │   ├── ResourceOverview.svelte    # Legacy summary panel
    │   └── ThemeSwitcher.svelte       # Dark/Light theme customization tool
    │
    ├── stores/                   # Svelte state stores
    │   ├── logs.ts               # Multi-tab pod log & exec terminal session store (isolated per profile tab)
    │   ├── terminal.ts           # Exec terminal helper utilities & bindings
    │   ├── nav.ts                # Active tab, section, and navigation state
    │   ├── editor.ts             # Monaco editor open states & draft content
    │   ├── keyboard.ts           # Active keybindings & shortcut modal state
    │   └── xray.ts               # Selected target node for resource topology
    │
    ├── styles/                   # Design system & CSS tokens
    │   ├── color-palette.css     # Centralized dark/light CSS variables
    │   ├── variables.css         # Spacing, typography, and radius design tokens
    │   └── README.md             # Styling documentation
    │
    ├── types/                    # TypeScript interfaces
    │   └── index.ts              # Data contracts, API shapes, resource models
    │
    └── utils/                    # Frontend helper functions
        ├── formatters.ts         # CPU/Memory units, timestamps, status badges
        └── performance.ts        # Debounce, throttle, and DOM batching
```

## 🎨 Color System & Design Tokens

Kuboard uses a centralized color palette system in `src/lib/styles/color-palette.css` and `variables.css`.

- **Dark Mode First**: Tailored for high-density Kubernetes cluster monitoring.
- **Semantic CSS Variables**: Colors represent states (`--status-ready-bg`, `--status-error-bg`, `--status-pending-bg`).
- **Theme Switcher**: Interactively swap between Dark, Light, and High Contrast palettes.

## 🧩 State Management & Navigation Flow

1. **Global Stores (`src/lib/stores/`)**:
   - `logs.ts`: Manages multi-tab pod log & exec terminal session states (`logsSessionStore`, `activeLogsState`) isolated per cluster profile tab session (`tabSessionId`).
   - `nav.ts`: Tracks active top-level tab (`workloads`, `nodes`, `config`, `network`, `storage`, `security`, `custom_resources`, `helm`, `linter`, `xray`).
   - `editor.ts`: Manages Monaco YAML editor instances and diff buffers.
   - `keyboard.ts`: Central registry for global keybindings (`/` for search, `Escape` for closing modals).
   - `xray.ts`: Stores focused resource node and dependency graph links.

2. **Component Hierarchy**:
   - `+page.svelte`: Orchestrates top-level context selection, state polling, and global modals (Logs, Terminal, Port Forward, Search).
   - `ThemeSwitcher.svelte`: Overhauled **Settings & Preferences Modal** with sub-tabs for Appearance & Color Themes and Grafana Integration (Endpoint URL, Bearer Token/Key, Datasource selection, live HTTP connection test).
   - `TabbedContent.svelte`: Renders the active tab container.
   - `WorkloadsTab.svelte`, `ConfigTab.svelte`, `SecurityTab.svelte`, `NetworkTab.svelte`, `StorageTab.svelte`: Unified sub-nav tab bars sharing a cohesive glassmorphic design token system (`rgba(59, 130, 246, 0.1)` active pill highlights, `rgba(255, 255, 255, 0.05)` hover fills, and 6px gap spacing across all main tabs).
   - `SecurityTab.svelte`: Sub-tabs for Secrets, Roles, ClusterRoles, RoleBindings, ClusterRoleBindings, and ServiceAccounts. Integrated `RoleDetails.svelte` (RBAC policy rules table, verb badges) and `RoleBindingDetails.svelte` (Target Role ref card, bound subjects table).
   - `MiniTopologyDAG.svelte` & `XRayViewer.svelte`: Interactive cluster topology maps with clickable controller parent cards (Deployments, StatefulSets, DaemonSets, ReplicaSets), current resource cards, and child dependency cards (ConfigMaps, Secrets, PVCs) triggering direct cross-tab navigation via `navigationStore`.

3. **Event Dispatching & Cross-Navigation**:
   - Resource Detail views emit `navigateToWorkload` events and update `navigationStore` to jump seamlessly between owner controllers (e.g. from ReplicaSet to parent Deployment or Pod).
