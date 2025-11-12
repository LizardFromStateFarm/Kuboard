# 🏗️ Kuboard Architecture Overview

This document provides a comprehensive overview of the Kuboard application architecture, including both UI and backend structure, function organization, and documentation status.

### **Key Architecture Points:**
- **Backend (Rust/Tauri):** Modular structure with `kuboard_` prefixed functions organized into:
  - `commands/mod.rs` - Tauri commands (context, cluster, resources, metrics, pod actions, watch)
  - `kubernetes/mod.rs` - Kubernetes client and API integration
  - `metrics/mod.rs` - Metrics server integration with real-time/historical data
  - `utils.rs` - CPU/memory parsing and formatting utilities
  - `types.rs` - Shared type definitions
  - `app_state.rs` - Application state management with async locks

- **Frontend (SvelteKit):** Component-based architecture with 25+ specialized components:
  - Resource management: `PodsPanel`, `DeploymentsPanel`, `StatefulSetsPanel`, `DaemonSetsPanel`, `ReplicaSetsPanel`, `CronJobsPanel`, `NodesTab`, `WorkloadsTab`, `ConfigTab`, `NetworkTab`, `CustomResourcesTab`
  - Detail views: `PodDetails`, `DeploymentDetails`, `StatefulSetDetails`, `DaemonSetDetails`, `ReplicaSetDetails`, `CronJobDetails`, `ServiceDetails`
  - Visualization: `MetricsGraph`, `DonutChart`, `ClusterMetrics`, `LogsWindow`
  - Layout: `Header`, `ClusterOverview`, `TabbedContent`, `ResourceTabs`, `ResourceOverview`
  - Utilities: `ThemeSwitcher`, `QuickActionsMenu`

- **Communication:** Tauri commands bridge frontend TypeScript and backend Rust
- **State Management:** AppState with async RwLock for concurrent access
- **Metrics:** Real-time metrics from Kubernetes metrics server API (`/apis/metrics.k8s.io/v1beta1`)
- **Styling:** Centralized CSS variables with theme support (Dark/Light/High Contrast)

### **Function Naming Convention:**
- Backend: `kuboard_<action>_<resource>` (e.g., `kuboard_get_pods`, `kuboard_fetch_node_metrics_real`)
- Frontend: `<action><Resource>` (e.g., `loadContexts`, `fetchNodeMetrics`)

### **Key Backend Functions:**
- **Context:** `kuboard_list_contexts`, `kuboard_set_context`, `kuboard_get_current_context`
- **Cluster:** `kuboard_get_cluster_overview`, `kuboard_get_cluster_metrics`
- **Resources:** `kuboard_get_nodes`, `kuboard_get_pods`, `kuboard_get_deployments`, etc.
- **Metrics:** `kuboard_get_node_metrics`, `kuboard_get_pod_metrics`, `kuboard_get_node_metrics_history`
- **Pod Actions:** `kuboard_delete_pod`, `kuboard_restart_pod`, `kuboard_get_pod_yaml`, `kuboard_get_pod_logs`, `kuboard_get_pod_events`
- **Watch:** `kuboard_start_pod_watch`, `kuboard_stop_pod_watch`

## 📋 **Table of Contents**

1. [AI Summary](#-ai-summary)
2. [Application Overview](#application-overview)
3. [UI Architecture](#ui-architecture)
4. [UI Organization Guide](#ui-organization-guide)
5. [Styling Architecture](#styling-architecture)
6. [Backend Architecture](#backend-architecture)
7. [Function Organization](#function-organization)
8. [Documentation Status](#documentation-status)
9. [Build System](#build-system)
10. [Development Guidelines](#development-guidelines)

## 🎯 **Application Overview**

Kuboard is a modern Kubernetes dashboard built with:
- **Frontend:** SvelteKit with TypeScript
- **Backend:** Rust with Tauri
- **Architecture:** Modular component-based design
- **Build System:** Vite + Cargo
- **Deployment:** Cross-platform desktop application

## 🎨 **UI Architecture**

### **Component Structure**
```
src/lib/components/
├── Header.svelte              # Main header with context selection
├── ClusterOverview.svelte     # Cluster info and node management
├── ResourceOverview.svelte    # Resource panels (nodes, pods, etc.)
├── MetricsGraph.svelte        # Real-time resource usage graphs
├── TabbedContent.svelte       # Main tabbed interface container
├── ResourceTabs.svelte        # Tab navigation component
├── WorkloadsTab.svelte        # Workloads management (pods, deployments, services, etc.)
├── PodsPanel.svelte           # Advanced pod management with events and container metrics
├── PodDetails.svelte          # Detailed pod information display
├── DeploymentsPanel.svelte    # Deployment management panel
├── DeploymentDetails.svelte   # Detailed deployment information display
├── StatefulSetsPanel.svelte   # StatefulSet management panel
├── StatefulSetDetails.svelte  # Detailed StatefulSet information display
├── DaemonSetsPanel.svelte     # DaemonSet management panel
├── DaemonSetDetails.svelte    # Detailed DaemonSet information display
├── ReplicaSetsPanel.svelte    # ReplicaSet management panel
├── ReplicaSetDetails.svelte   # Detailed ReplicaSet information display
├── CronJobsPanel.svelte      # CronJob management panel
├── CronJobDetails.svelte      # Detailed CronJob information display
├── ServiceDetails.svelte      # Detailed service information display
├── NodesTab.svelte            # Advanced node management and details
├── ConfigTab.svelte           # Configuration management (ConfigMaps, Secrets)
├── NetworkTab.svelte          # Network resources and services
├── CustomResourcesTab.svelte  # Custom Resource Definitions (CRDs)
├── DonutChart.svelte          # Resource usage donut charts
├── ClusterMetrics.svelte      # Cluster-wide metrics display
├── LogsWindow.svelte          # Advanced logs panel with structured entries and smart follow mode
├── QuickActionsMenu.svelte    # Quick action menu for resources
└── ThemeSwitcher.svelte       # Development theme switching tool
```

### **Key Features**
- **Modular Design:** Each component is self-contained
- **Event-Driven:** Clean communication between components
- **Type Safety:** Full TypeScript integration
- **Responsive:** Works on different screen sizes
- **Accessible:** Proper ARIA roles and keyboard navigation
- **Tabbed Interface:** Organized resource management with dedicated tabs
- **Advanced Theming:** Centralized color palette with theme switching
- **Real-time Updates:** Live metrics and resource monitoring
- **Development Tools:** Built-in theme switcher for rapid prototyping

### **Component Responsibilities**

#### **Header Component**
- Context selection and switching
- Application branding
- Mode indication (Desktop vs Demo)
- Refresh functionality

#### **ClusterOverview Component**
- Two-panel layout (node list + node details)
- Node selection and detailed information
- System information display
- Resource specifications and usage
- Labels, annotations, and taints
- Real-time metrics integration
- Debug console for troubleshooting

#### **ResourceOverview Component**
- Expandable resource panels
- Resource counts and status
- Interactive resource selection
- Loading states and error handling

#### **MetricsGraph Component**
- Chart.js integration with capacity-based y-axis scaling
- Real-time data updates with proper data format handling
- Multiple resource types (CPU, Memory, Disk) with dynamic switching
- Capacity-based visualization showing actual utilization percentages
- Utilization percentage display in current value section
- Dynamic y-axis scaling based on resource capacity (cores, GB)
- Robust error handling for various backend data formats
- Auto-refresh functionality with proper chart updates

#### **TabbedContent Component**
- Main container for tabbed resource management
- Coordinates between different resource tabs
- Manages tab state and data loading
- Handles tab switching and updates

#### **ResourceTabs Component**
- Tab navigation interface
- Dynamic tab counts and status
- Disabled state management
- Tab selection and events

#### **WorkloadsTab Component**
- Pods, Deployments, StatefulSets, DaemonSets, ReplicaSets, CronJobs, and Services management
- Lazy loading of resource types
- Detailed resource information display
- Status filtering and sorting
- Integration with all workload panel components

#### **PodsPanel Component**
- Advanced pod management with interactive selection
- Comprehensive pod information display (basic info, controller, labels, annotations)
- Pod events section with color-coded event types and troubleshooting info
- Container-specific metrics with click functionality
- Pod conditions, tolerations, and affinity rules display
- Pod volumes and container details
- Real-time metrics integration with higher precision for pod CPU usage
- Accessibility features with keyboard navigation and proper ARIA roles
- Integrated logs viewing with "View Logs" button for pod and container logs
- Proper pod name extraction for unique tab identification

#### **DeploymentsPanel Component**
- Deployment management with search, filter, and sort functionality
- Deployment list with replica status indicators
- Click to view detailed deployment information
- Quick actions menu integration (context menu)
- Integration with DeploymentDetails component

#### **DeploymentDetails Component**
- Comprehensive deployment information display
- Replica status (desired/current/ready/available)
- ReplicaSet history with rollouts
- Pod template viewer
- Rolling update strategy display
- Scale controls (input field + buttons)
- Rollback functionality
- List of managed pods
- Restart deployment action
- Quick actions menu integration
- Collapsible sections for organization

#### **StatefulSetsPanel Component**
- StatefulSet management with search, filter, and sort functionality
- StatefulSet list with replica status indicators
- Click to view detailed StatefulSet information
- Quick actions menu integration (context menu)
- Integration with StatefulSetDetails component

#### **StatefulSetDetails Component**
- Comprehensive StatefulSet information display
- Replica status (desired/current/ready/available)
- Pod management policy (ordered/parallel)
- Volume claim templates
- Update strategy (rolling update, on delete)
- Pod template viewer
- Scale controls (input field + buttons)
- List of managed pods (ordered by ordinal)
- Pod naming pattern display
- StatefulSet-specific actions (restart, scale, update)
- Quick actions menu integration
- Collapsible sections for organization

#### **DaemonSetsPanel Component**
- DaemonSet management with search, filter, and sort functionality
- DaemonSet list with status indicators
- Click to view detailed DaemonSet information
- Quick actions menu integration (context menu)
- Integration with DaemonSetDetails component
- Empty state handling

#### **DaemonSetDetails Component**
- Comprehensive DaemonSet information display
- Node status (desired/current/ready/available)
- Update strategy (rolling update, on delete)
- Pod template viewer
- List of managed pods (one per node, sorted by node name)
- Node selector and tolerations display
- DaemonSet-specific actions (restart)
- Quick actions menu integration
- Collapsible sections for organization

#### **ReplicaSetsPanel Component**
- ReplicaSet management with search, filter, and sort functionality
- ReplicaSet list with replica status indicators
- Click to view detailed ReplicaSet information
- Quick actions menu integration (context menu)
- Integration with ReplicaSetDetails component

#### **ReplicaSetDetails Component**
- Comprehensive ReplicaSet information display
- Replica status (desired/current/ready/available)
- Pod template viewer
- Pod selector and labels
- List of managed pods
- Owner reference (if owned by Deployment)
- Scale controls (input field + buttons)
- Pod creation/deletion history visualization
- Quick actions menu integration
- Collapsible sections for organization

#### **CronJobsPanel Component**
- CronJob management with search, filter, and sort functionality
- CronJob list with schedule and status
- Click to view detailed CronJob information
- Quick actions menu integration (context menu)
- Integration with CronJobDetails component

#### **CronJobDetails Component**
- Comprehensive CronJob information display
- Schedule display (cron expression)
- Concurrency policy
- Suspend state
- Active jobs list
- Job history (last successful, last failed)
- Next scheduled run time
- Starting deadline seconds
- CronJob-specific actions (suspend/resume, trigger, delete)
- Quick actions menu integration
- Collapsible sections for organization

#### **ServiceDetails Component**
- Comprehensive service information display
- Service endpoints and associated pods
- Port mappings (ports, protocols, target ports)
- Selectors and labels
- External IPs and load balancer status
- Service type information
- Session affinity settings
- Port forward quick action (planned)
- Quick actions menu integration
- Collapsible sections for organization

#### **NodesTab Component**
- Advanced node management interface with simplified card-based layout
- Detailed node information and metrics with capacity-based visualization
- Node selection and detailed panel expansion on click
- Real-time metrics integration with resource type switching (CPU/Memory/Disk)
- Capacity-based y-axis scaling for meaningful utilization visualization
- Utilization percentage display showing actual resource usage vs capacity
- Resource type selector for switching between CPU, Memory, and Disk metrics
- Historical metrics graphs with proper data format handling

#### **ConfigTab Component**
- ConfigMaps and Secrets management
- Configuration data display
- Secret value masking
- Resource creation and editing

#### **NetworkTab Component**
- Network services and resources
- Service discovery and endpoints
- Network policy management
- Load balancer configuration

#### **CustomResourcesTab Component**
- Custom Resource Definitions (CRDs)
- Dynamic resource type discovery
- Custom resource management
- CRD schema validation

#### **DonutChart Component**
- Resource usage visualization
- Interactive donut charts
- Real-time data updates
- Multiple chart types

#### **ClusterMetrics Component**
- Cluster-wide metrics display
- Resource utilization overview
- Performance monitoring
- Alert integration

#### **LogsWindow Component**
- Advanced logs panel with structured LogEntry model and append-only updates
- Timestamp-first display using raw log lines (no custom parsing required)
- Expandable/collapsible log entries with click-to-expand functionality
- Smart follow mode that auto-re-enables when scrolling back to bottom
- Visual follow mode indicator (📡 Following / ⏸️ Paused) in footer
- Log line capping (5000 lines per tab) with automatic trimming of oldest entries
- Keyed list rendering for optimal performance and no UI flickering
- Monospace font styling for better log readability
- JSON log detection and special formatting
- Tab management with unique tab IDs based on namespace/pod/container names
- Container-specific log viewing with proper namespace/pod name handling
- Follow mode with 2-second refresh interval and smart pause/resume
- Close tab functionality with proper state cleanup
- Keyboard navigation and accessibility features

#### **ThemeSwitcher Component**
- Development theme switching
- Color palette management
- Theme persistence
- Live theme preview

#### **PodDetails Component**
- Detailed pod information display
- Container-specific details
- Resource specifications
- Status and conditions

#### **QuickActionsMenu Component**
- Context menu for resource actions
- Pod management actions (delete, restart, view logs)
- Resource-specific operations

## 🎨 **UI Organization Guide**

### **Component Hierarchy**

```
Main Page (+page.svelte)
├── Header
│   ├── Context selector
│   ├── Refresh button
│   └── Mode indicator
├── ClusterOverview
│   ├── ClusterMetrics (donut charts)
│   └── TabbedContent
│       ├── ResourceTabs (navigation)
│       ├── WorkloadsTab
│       │   └── PodsPanel
│       │       ├── Pod list with search/sort
│       │       ├── PodDetails (on selection)
│       │       ├── MetricsGraph (pod metrics)
│       │       └── LogsWindow (pod/container logs)
│       ├── NodesTab
│       │   ├── Node list (card-based)
│       │   ├── Node details panel
│       │   └── MetricsGraph (node metrics)
│       ├── ConfigTab (ConfigMaps, Secrets)
│       ├── NetworkTab (Services, networking)
│       └── CustomResourcesTab (CRDs)
└── ResourceOverview (legacy)
```

### **Component Communication Patterns**

1. **Props Down, Events Up:** Parent components pass data via props, children emit events
2. **Tauri Invoke:** Frontend calls backend via `invoke('kuboard_<command>', { ... })`
3. **State Management:** Main page orchestrates state, components remain stateless where possible
4. **Event Dispatchers:** Svelte `createEventDispatcher()` for component-to-component communication

### **Component Responsibilities**

#### **Layout Components**
- **Header.svelte:** Top-level navigation and context management
- **TabbedContent.svelte:** Main container for resource tabs
- **ResourceTabs.svelte:** Tab navigation with counts and badges

#### **Resource Management Components**
- **PodsPanel.svelte:** Complete pod lifecycle management
  - Search, filter, sort functionality
  - Pod selection and details
  - Container metrics and logs
  - Pod actions (delete, restart, view YAML)
- **NodesTab.svelte:** Node management with capacity-based metrics
- **WorkloadsTab.svelte:** Deployments and Services management
- **ConfigTab.svelte:** ConfigMaps and Secrets viewer/editor
- **NetworkTab.svelte:** Services, endpoints, and networking resources
- **CustomResourcesTab.svelte:** CRD discovery and management

#### **Visualization Components**
- **MetricsGraph.svelte:** Chart.js-based time-series graphs
  - CPU, Memory, Disk metrics
  - Capacity-based y-axis scaling
  - Real-time and historical data
- **DonutChart.svelte:** CSS-based donut charts for resource usage
- **ClusterMetrics.svelte:** Cluster-wide metrics overview
- **LogsWindow.svelte:** Advanced log viewer with follow mode

#### **Utility Components**
- **PodDetails.svelte:** Reusable pod information display
- **QuickActionsMenu.svelte:** Context menu for resource actions
- **ThemeSwitcher.svelte:** Development theme switching tool

### **File Organization**

```
src/
├── lib/
│   ├── components/          # All UI components (18 files)
│   ├── styles/
│   │   ├── color-palette.css
│   │   ├── variables.css
│   │   └── README.md
│   ├── types/
│   │   └── index.ts         # TypeScript interfaces
│   └── utils/
│       ├── formatters.ts    # Data formatting utilities
│       └── performance.ts   # Performance utilities
└── routes/
    ├── +layout.ts           # Layout configuration
    └── +page.svelte         # Main dashboard (orchestrates components)
```

### **Component Development Guidelines**

1. **Single Responsibility:** Each component handles one specific UI concern
2. **Type Safety:** All props use TypeScript interfaces from `lib/types/index.ts`
3. **Event-Driven:** Use Svelte dispatchers for parent-child communication
4. **Accessibility:** Include ARIA roles, keyboard navigation, and semantic HTML
5. **Responsive Design:** Use CSS variables and responsive breakpoints
6. **Styling:** Use CSS custom properties from `variables.css` and `color-palette.css`
7. **Error Handling:** Display loading states and error messages gracefully
8. **Performance:** Use keyed lists, lazy loading, and memoization where appropriate

## 🎨 **Styling Architecture**

### **Color Palette System**
```
src/lib/styles/
├── color-palette.css    # Centralized color definitions
├── variables.css        # Spacing, sizing, and non-color variables
└── README.md           # Styling guide and documentation
```

### **Theme Management**
- **Centralized Colors:** All colors defined in `color-palette.css`
- **Theme Variations:** Dark, Light, and High Contrast themes
- **CSS Variables:** Consistent color usage across components
- **Development Tools:** Live theme switching in dev mode

### **Color Categories**
- **Primary Brand Colors:** Main application branding
- **Status Colors:** Success, warning, error, and info states
- **Background Colors:** Primary, secondary, and tertiary backgrounds
- **Text Colors:** Primary, secondary, muted, and disabled text
- **Border Colors:** Various border and divider colors
- **Interactive Colors:** Hover, focus, and active states

### **Component Styling**
- **Consistent Patterns:** Standardized styling across all components
- **Responsive Design:** Mobile-first approach with breakpoints
- **Accessibility:** High contrast ratios and proper color usage
- **Dark Mode First:** Optimized for dark theme with light theme support

## 🔧 **Backend Architecture**

### **Module Structure**
```
src-tauri/src/
├── lib.rs                 # Main entry point
├── main.rs               # Application entry point
├── commands/             # Tauri commands
│   └── mod.rs            # All Tauri command functions
├── kubernetes/           # Kubernetes integration
│   └── mod.rs            # Kubeconfig and client management
├── metrics/              # Metrics server integration
│   └── mod.rs            # Real-time metrics fetching
├── types.rs              # Type definitions
├── app_state.rs          # Application state management
└── utils.rs              # Utility functions
```

### **Function Organization**

#### **Tauri Commands** (`commands/mod.rs`)

**Context Management:**
- `kuboard_list_contexts` - List available Kubernetes contexts from kubeconfig
- `kuboard_set_context` - Set active Kubernetes context and create client
- `kuboard_get_current_context` - Get currently active context name

**Cluster Operations:**
- `kuboard_get_cluster_overview` - Get cluster information, resource counts, and Kubernetes version
- `kuboard_get_cluster_metrics` - Calculate cluster-wide CPU, memory, and disk metrics

**Resource Management:**
- `kuboard_get_nodes` - Fetch all nodes in the cluster
- `kuboard_get_namespaces` - Fetch all namespaces in the cluster
- `kuboard_get_pods` - Fetch all pods in the cluster
- `kuboard_get_deployments` - Fetch all deployments in the cluster
- `kuboard_get_deployment` - Fetch single deployment by name and namespace
- `kuboard_get_replicasets` - Fetch all ReplicaSets in the cluster
- `kuboard_get_replicaset` - Fetch single ReplicaSet by name and namespace
- `kuboard_get_statefulsets` - Fetch all StatefulSets in the cluster
- `kuboard_get_statefulset` - Fetch single StatefulSet by name and namespace
- `kuboard_get_daemonsets` - Fetch all DaemonSets in the cluster
- `kuboard_get_daemonset` - Fetch single DaemonSet by name and namespace
- `kuboard_get_cronjobs` - Fetch all CronJobs in the cluster
- `kuboard_get_cronjob` - Fetch single CronJob by name and namespace
- `kuboard_get_services` - Fetch all services in the cluster
- `kuboard_get_service` - Fetch single service by name and namespace
- `kuboard_get_service_endpoints` - Fetch service endpoints
- `kuboard_get_configmaps` - Fetch all ConfigMaps in the cluster
- `kuboard_get_secrets` - Fetch all Secrets in the cluster
- `kuboard_get_custom_resources` - Fetch custom resources (CRDs) in the cluster

**Metrics Operations:**
- `kuboard_get_node_metrics` - Get current node metrics from metrics server (real-time)
- `kuboard_get_node_metrics_history` - Get historical node metrics data (time-series)
- `kuboard_get_pod_metrics` - Get current pod metrics from metrics server (real-time)
- `kuboard_get_pod_metrics_history` - Get historical pod metrics data (time-series)
- `kuboard_check_metrics_availability` - Check if metrics server is available

**Pod Operations:**
- `kuboard_get_pod_events` - Fetch pod events for troubleshooting (from Kubernetes API)
- `kuboard_get_pod_logs` - Fetch pod logs with container support and follow mode
- `kuboard_delete_pod` - Delete a pod by name and namespace
- `kuboard_restart_pod` - Restart a pod (delete for recreation by controller)
- `kuboard_get_pod_yaml` - Get pod YAML/JSON representation
- `kuboard_update_pod_from_yaml` - Update pod from YAML/JSON content
- `kuboard_describe_pod` - Get pod describe output

**Deployment Operations:**
- `kuboard_scale_deployment` - Scale deployment to specified replica count
- `kuboard_rollback_deployment` - Rollback deployment to previous revision
- `kuboard_restart_deployment` - Restart deployment (rolling restart)
- `kuboard_get_deployment_replicasets` - Get ReplicaSets managed by deployment
- `kuboard_get_deployment_pods` - Get pods managed by deployment
- `kuboard_delete_deployment` - Delete a deployment
- `kuboard_get_deployment_yaml` - Get deployment YAML/JSON representation

**StatefulSet Operations:**
- `kuboard_scale_statefulset` - Scale StatefulSet to specified replica count
- `kuboard_restart_statefulset` - Restart StatefulSet (rolling restart)
- `kuboard_get_statefulset_pods` - Get pods managed by StatefulSet
- `kuboard_delete_statefulset` - Delete a StatefulSet
- `kuboard_get_statefulset_yaml` - Get StatefulSet YAML/JSON representation

**DaemonSet Operations:**
- `kuboard_restart_daemonset` - Restart DaemonSet (rolling restart)
- `kuboard_get_daemonset_pods` - Get pods managed by DaemonSet
- `kuboard_delete_daemonset` - Delete a DaemonSet
- `kuboard_get_daemonset_yaml` - Get DaemonSet YAML/JSON representation

**ReplicaSet Operations:**
- `kuboard_scale_replicaset` - Scale ReplicaSet to specified replica count
- `kuboard_get_replicaset_pods` - Get pods managed by ReplicaSet
- `kuboard_delete_replicaset` - Delete a ReplicaSet
- `kuboard_get_replicaset_yaml` - Get ReplicaSet YAML/JSON representation

**CronJob Operations:**
- `kuboard_trigger_cronjob` - Trigger CronJob immediately (create Job)
- `kuboard_suspend_cronjob` - Suspend CronJob
- `kuboard_resume_cronjob` - Resume CronJob
- `kuboard_get_cronjob_jobs` - Get Jobs created by CronJob
- `kuboard_delete_cronjob` - Delete a CronJob
- `kuboard_get_cronjob_yaml` - Get CronJob YAML/JSON representation

**Service Operations:**
- `kuboard_delete_service` - Delete a service
- `kuboard_get_service_yaml` - Get service YAML/JSON representation

**Watch Operations:**
- `kuboard_start_pod_watch` - Start watching pods for real-time updates
- `kuboard_stop_pod_watch` - Stop pod watch
- `kuboard_start_deployment_watch` - Start watching deployments for real-time updates
- `kuboard_stop_deployment_watch` - Stop deployment watch
- `kuboard_start_statefulset_watch` - Start watching StatefulSets for real-time updates
- `kuboard_stop_statefulset_watch` - Stop StatefulSet watch
- `kuboard_start_daemonset_watch` - Start watching DaemonSets for real-time updates
- `kuboard_stop_daemonset_watch` - Stop DaemonSet watch
- `kuboard_start_replicaset_watch` - Start watching ReplicaSets for real-time updates
- `kuboard_stop_replicaset_watch` - Stop ReplicaSet watch
- `kuboard_start_service_watch` - Start watching services for real-time updates
- `kuboard_stop_service_watch` - Stop service watch
- `kuboard_start_cronjob_watch` - Start watching CronJobs for real-time updates
- `kuboard_stop_cronjob_watch` - Stop CronJob watch

#### **Kubernetes Integration** (`kubernetes/mod.rs`)

**Client Management:**
- `kuboard_load_kubeconfig` - Load kubeconfig from environment or default location (`~/.kube/config`)
- `kuboard_create_client_from_context` - Create Kubernetes client from kubeconfig context

**Cluster Metrics:**
- `kuboard_calculate_cluster_metrics` - Calculate cluster-wide metrics from node data
  - Aggregates CPU, memory, disk capacity and usage
  - Calculates node readiness and status
  - Generates node details with resource specifications

**Pod Operations:**
- `kuboard_fetch_pod_events` - Fetch pod events from Kubernetes API
  - Returns events sorted by timestamp
  - Includes event type, reason, message, and count
- `kuboard_fetch_pod_logs` - Fetch pod logs from Kubernetes API
  - Supports container-specific log fetching
  - Supports tail lines and follow mode
  - Returns log stream as string

**Watch Module** (`kubernetes/watch.rs`):
- `PodWatcher::new()` - Create new pod watcher instance
- `PodWatcher::start()` - Start watching pods and emit events to frontend
- `PodWatcher::stop()` - Stop pod watch
- `PodWatcher::is_active()` - Check if watch is currently active

#### **Metrics Server Integration** (`metrics/mod.rs`)

**Metrics API Functions:**
- `metrics_api_available` - Check if metrics API (`/apis/metrics.k8s.io/v1beta1`) is available
- `get_node_metrics` - Get all node metrics from metrics API
- `get_node_metrics_by_name` - Get metrics for specific node
- `get_pod_metrics` - Get all pod metrics from metrics API
- `get_pod_metrics_by_name` - Get metrics for specific pod in namespace

**Public Metrics Functions:**
- `kuboard_fetch_node_metrics_real` - Fetch real-time node metrics and parse CPU/memory
  - Returns `MetricsDataPoint` with usage in cores/bytes and percentages
  - Handles metrics server availability checking
  - Parses CPU (m, n, u suffixes) and memory (Ki, Mi, Gi, Ti suffixes)
- `kuboard_fetch_node_metrics_history` - Generate historical metrics timeline
  - Fetches current metrics and generates time-series with variations
  - Returns vector of `MetricsDataPoint` for specified duration
- `kuboard_fetch_pod_metrics_real` - Fetch real-time pod metrics (sums all containers)
  - Aggregates CPU and memory usage across all pod containers
  - Returns `MetricsDataPoint` with total pod usage
- `kuboard_fetch_pod_metrics_history` - Generate historical pod metrics timeline
  - Similar to node history but for pod-level metrics
- `kuboard_check_metrics_server_availability` - Check if metrics server is available

**Parsing Functions:**
- `parse_cpu_quantity` - Parse CPU quantity strings (150m, 1.5, 500000000n, 500000u)
  - Handles millicores (m), nanocores (n), microcores (u), and cores
- `parse_memory_quantity` - Parse memory quantity strings (1Gi, 1024Mi, 123Ki, etc.)
  - Handles binary (Ki, Mi, Gi, Ti) and decimal (K, M, G) suffixes
  - Returns bytes as u64

#### **Utility Functions** (`utils.rs`)

**Parsing:**
- `kuboard_parse_cpu_string` - Parse CPU string (e.g., "1000m", "1") into CPU cores (f64)
- `kuboard_parse_memory_string` - Parse memory string (e.g., "1Gi", "1024Mi") into bytes (u64)

**Formatting:**
- `kuboard_format_memory` - Format bytes into human-readable memory string (Ki, Mi, Gi, Ti)
- `kuboard_format_cpu` - Format CPU cores into human-readable string (m for millicores, cores otherwise)

#### **Application State** (`app_state.rs`)

**AppState Structure:**
- `kubeconfig: Arc<RwLock<Option<Kubeconfig>>>` - Cached kubeconfig
- `current_context: Arc<RwLock<Option<String>>>` - Current context name
- `current_client: Arc<RwLock<Option<Client>>>` - Active Kubernetes client
- `pod_watcher: Arc<RwLock<PodWatcher>>` - Pod watch instance

**Methods:**
- `AppState::new()` - Create new application state instance

## 📊 **Function Organization**

### **Naming Conventions**
- **Backend Functions:** `kuboard_<action>_<resource>` (e.g., `kuboard_get_nodes`, `kuboard_fetch_node_metrics_real`)
- **Frontend Functions:** `<action><Resource>` (e.g., `loadContexts`, `fetchNodeMetrics`)
- **Utility Functions:** `kuboard_<action>_<type>` (e.g., `kuboard_format_memory`, `kuboard_parse_cpu_string`)
- **Internal Functions:** No prefix for module-internal helpers (e.g., `parse_cpu_quantity`, `get_node_metrics_by_name`)

### **Function Categories Summary**

**Total Functions:** ~80+ backend functions organized across 5 modules

1. **Context Management (3 functions):** List, set, get contexts
2. **Cluster Operations (2 functions):** Overview and metrics calculation
3. **Resource Management (18 functions):** Nodes, namespaces, pods, deployments, StatefulSets, DaemonSets, ReplicaSets, CronJobs, services, ConfigMaps, Secrets, CRDs
4. **Metrics Operations (5 functions):** Node/pod metrics (real-time and historical), availability checking
5. **Pod Operations (7 functions):** Events, logs, delete, restart, YAML get/update, describe
6. **Deployment Operations (7 functions):** Scale, rollback, restart, get ReplicaSets/pods, delete, get YAML
7. **StatefulSet Operations (5 functions):** Scale, restart, get pods, delete, get YAML
8. **DaemonSet Operations (4 functions):** Restart, get pods, delete, get YAML
9. **ReplicaSet Operations (4 functions):** Scale, get pods, delete, get YAML
10. **CronJob Operations (6 functions):** Trigger, suspend, resume, get jobs, delete, get YAML
11. **Service Operations (3 functions):** Delete, get YAML, get endpoints
12. **Watch Operations (12 functions):** Start/stop watch for pods, deployments, StatefulSets, DaemonSets, ReplicaSets, services, CronJobs
13. **Kubernetes Integration (4 functions):** Kubeconfig loading, client creation, cluster metrics, pod events/logs
14. **Metrics Parsing (4 functions):** CPU/memory quantity parsing (public and internal)
15. **Utility Functions (4 functions):** CPU/memory parsing and formatting

### **Function Categories**

#### **Context Management**
- List, set, and get Kubernetes contexts
- Handle context switching and validation

#### **Cluster Operations**
- Fetch cluster overview and statistics
- Calculate cluster-wide metrics
- Handle cluster information display

#### **Resource Management**
- Fetch nodes, namespaces, pods, deployments, services
- Handle ConfigMaps, Secrets, and custom resources
- Manage resource selection and display
- Handle resource loading states
- Support for CRDs and dynamic resource types

#### **Metrics Integration**
- Real-time metrics from metrics server
- Historical metrics data
- Metrics server availability checking
- Robust parsing of CPU/memory quantities

#### **Data Formatting**
- Memory formatting (bytes to human-readable)
- CPU formatting (cores to human-readable)
- Status formatting and color classes
- Date/time formatting
- Resource quantity parsing

## 📚 **Documentation Status**

### **✅ Completed Documentation**

#### **UI Documentation**
- **UI_ORGANIZATION_GUIDE.md** - Complete component breakdown
- **Component Props & Events** - Detailed interface documentation
- **Styling Organization** - CSS variables and design system
- **Color Palette Guide** - Comprehensive color management documentation
- **Theme System** - Theme switching and customization guide
- **Usage Examples** - Code examples for component usage

#### **Function Documentation**
- **FUNCTION_DOCUMENTATION.md** - Complete function inventory
- **Backend Functions** - All Rust functions documented
- **Frontend Functions** - All Svelte/TypeScript functions documented
- **Utility Functions** - All formatting and helper functions documented

#### **Architecture Documentation**
- **ARCHITECTURE_OVERVIEW.md** - This comprehensive overview
- **Build System** - Documentation for build process
- **Development Guidelines** - Best practices and conventions

### **📋 Documentation Features**
- **Function Status Tracking** - Working, Mock, Planned status
- **Module Organization** - Clear module responsibilities
- **Type Definitions** - Complete TypeScript interfaces
- **Event Handling** - Component communication patterns
- **Error Handling** - Comprehensive error management
- **Performance Considerations** - Optimization strategies
- **Color System Documentation** - Complete color palette reference
- **Theme Customization** - Step-by-step theming guide
- **Component Styling** - Consistent styling patterns
- **Development Tools** - Theme switcher and debugging tools

## 🔨 **Build System**

### **Frontend Build (Vite + SvelteKit)**
- **Development:** `npm run dev`
- **Production:** `npm run build`
- **Preview:** `npm run preview`

### **Backend Build (Cargo + Tauri)**
- **Development:** `npm run tauri dev`
- **Production:** `npm run tauri build`
- **Desktop App:** Generates MSI and NSIS installers

### **Build Output**
- **Frontend:** Optimized SvelteKit bundle
- **Backend:** Rust binary with Tauri integration
- **Installers:** Cross-platform desktop application
- **Size:** Optimized with gzip compression

## 🚀 **Development Guidelines**

### **Component Development**
1. **Single Responsibility** - Each component has one clear purpose
2. **Type Safety** - Use TypeScript interfaces for all props
3. **Event Handling** - Use Svelte dispatchers for communication
4. **Accessibility** - Include proper ARIA roles and keyboard navigation
5. **Responsive Design** - Test on different screen sizes

### **Function Development**
1. **Naming Conventions** - Follow established patterns
2. **Error Handling** - Comprehensive error management
3. **Documentation** - JSDoc comments for all functions
4. **Testing** - Unit tests for critical functions
5. **Performance** - Optimize for large datasets

### **Code Organization**
1. **Modular Structure** - Keep related functions together
2. **Clear Separation** - Frontend vs Backend responsibilities
3. **Consistent Patterns** - Follow established conventions
4. **Documentation** - Keep docs up to date with code changes

## 🎯 **Benefits Achieved**

### **Maintainability**
- Easy to find and modify specific functionality
- Clear separation of concerns
- Modular component architecture

### **Reusability**
- Components can be used in multiple places
- Utility functions are shared across modules
- Consistent patterns throughout codebase

### **Scalability**
- Easy to add new features without conflicts
- Modular structure supports growth
- Clear interfaces for integration

### **Collaboration**
- Clear documentation for team members
- Consistent coding patterns
- Easy onboarding for new developers

### **Quality**
- Type safety throughout the application
- Comprehensive error handling
- Professional build system
- Consistent theming and styling
- Advanced resource management
- Real-time monitoring capabilities

## 🔄 **Next Steps**

### **Potential Improvements**
1. **Testing** - Add comprehensive unit tests
2. **Performance** - Optimize for very large clusters
3. **Features** - Add more Kubernetes resource types (Storage, Security tabs)
4. **UI/UX** - Enhance user experience with animations and transitions
5. **Documentation** - Add video tutorials and examples
6. **Theming** - Add more theme variations and customization options
7. **Resource Management** - Add resource creation and editing capabilities
8. **Monitoring** - Enhanced alerting and notification system

### **Maintenance**
1. **Keep Documentation Updated** - Sync with code changes
2. **Monitor Performance** - Track build times and bundle sizes
3. **User Feedback** - Collect and implement user suggestions
4. **Security Updates** - Keep dependencies up to date

---

This modular architecture has successfully transformed Kuboard into a comprehensive, maintainable, and professional Kubernetes dashboard with advanced resource management, theming capabilities, and real-time monitoring! 🚀

## 🆕 **Recent Major Updates**

### **Advanced Logs Panel System (Latest)**
- **Redesigned**: Complete logs panel with structured LogEntry model and append-only updates
- **Implemented**: Timestamp-first display using raw log lines (no custom parsing)
- **Added**: Expandable/collapsible log entries with click-to-expand functionality
- **Enhanced**: Append-only log ingestion to prevent UI flashing and preserve scroll position
- **Added**: Smart follow mode that auto-re-enables when scrolling back to bottom
- **Implemented**: Visual follow mode indicator (📡 Following / ⏸️ Paused)
- **Added**: Log line capping (5000 lines per tab) with automatic trimming of oldest entries
- **Enhanced**: Keyed list rendering for optimal performance and no UI flickering
- **Improved**: Monospace font styling for better log readability
- **Added**: JSON log detection and special formatting
- **Fixed**: UI reactivity issues that caused logs list to "freak out" during updates
- **Enhanced**: Scroll management with user scroll detection and auto-scroll when following
- **Added**: Comprehensive error handling and loading states for log operations
- **Improved**: Tab management with unique IDs and proper state cleanup
- **Added**: Container-specific log viewing with proper namespace/pod name handling
- **Enhanced**: Follow mode with 2-second refresh interval and smart pause/resume

### **Advanced Nodes Management**
- Complete nodes tab functionality with simplified card-based layout
- Capacity-based metrics visualization with utilization percentages
- Resource type switching (CPU/Memory/Disk) with dynamic graph updates
- Y-axis scaling based on actual resource capacity for meaningful visualization
- Historical metrics integration with proper data format handling
- Node selection with detailed panel expansion on click

### **Enhanced Metrics Visualization**
- Capacity-based y-axis scaling showing actual resource utilization
- Utilization percentage display in current value section
- Dynamic chart updates when switching between resource types
- Robust error handling for various backend data formats
- Better visual context for understanding resource pressure

### **Tabbed Resource Management**
- Complete overhaul of the resource management interface
- Dedicated tabs for Workloads, Nodes, Config, Network, and Custom Resources
- Lazy loading and efficient resource handling
- Enhanced user experience with organized navigation

### **Advanced Theming System**
- Centralized color palette management
- Live theme switching in development mode
- Support for Dark, Light, and High Contrast themes
- Comprehensive styling documentation

### **Enhanced Component Library**
- 14 specialized components for different resource types
- Improved component organization and responsibilities
- Better separation of concerns and modularity
- Enhanced development tools and debugging capabilities

### **Real-time Monitoring**
- Advanced metrics visualization with capacity-based scaling
- Live resource usage tracking with utilization percentages
- Historical data analysis with meaningful context
- Performance monitoring and alerting
