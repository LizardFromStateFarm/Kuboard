# Kuboard Function Documentation

## 📋 **Complete Function Inventory**

### 🔧 **Backend Functions (Rust - Tauri Commands)**

#### **Context Management Commands**
| Function Name | Description | Status | Module |
|---------------|-------------|--------|--------|
| `kuboard_list_contexts` | Lists available Kubernetes contexts from kubeconfig | ✅ Working | `commands` |
| `kuboard_set_context` | Sets the active Kubernetes context | ✅ Working | `commands` |
| `kuboard_get_current_context` | Returns the currently active context | ✅ Working | `commands` |

#### **Cluster Overview Commands**
| Function Name | Description | Status | Module |
|---------------|-------------|--------|--------|
| `kuboard_get_cluster_overview` | Gets cluster information and metrics | ✅ Working | `commands` |

#### **Resource Management Commands**
| Function Name | Description | Status | Module |
|---------------|-------------|--------|--------|
| `kuboard_get_nodes` | Fetches all nodes in the cluster | ✅ Working | `commands` |
| `kuboard_get_namespaces` | Fetches all namespaces in the cluster | ✅ Working | `commands` |
| `kuboard_get_pods` | Fetches all pods in the cluster | ✅ Working | `commands` |
| `kuboard_get_deployments` | Fetches all deployments in the cluster | ✅ Working | `commands` |
| `kuboard_get_services` | Fetches all services in the cluster | ✅ Working | `commands` |
| `kuboard_get_configmaps` | Fetches all ConfigMaps in the cluster | ✅ Working | `commands` |
| `kuboard_get_secrets` | Fetches all Secrets in the cluster | ✅ Working | `commands` |
| `kuboard_get_custom_resources` | Fetches custom resources in the cluster | ✅ Working | `commands` |

#### **Metrics Commands (Real Implementation)**
| Function Name | Description | Status | Module |
|---------------|-------------|--------|--------|
| `kuboard_get_node_metrics` | Fetches current node metrics from metrics server | ✅ Working | `commands` |
| `kuboard_get_node_metrics_history` | Fetches historical node metrics data | ✅ Working | `commands` |
| `kuboard_get_pod_metrics` | Fetches current pod metrics from metrics server | ✅ Working | `commands` |
| `kuboard_get_pod_metrics_history` | Fetches historical pod metrics data | ✅ Working | `commands` |
| `kuboard_get_pod_events` | Fetches pod events for troubleshooting | ✅ Working | `commands` |
| `kuboard_get_cluster_metrics` | Fetches cluster-wide metrics | ✅ Working | `commands` |
| `kuboard_check_metrics_availability` | Checks if metrics server is available | ✅ Working | `commands` |

### 🔧 **Backend Helper Functions (Rust)**

#### **Kubernetes Integration**
| Function Name | Description | Status | Module |
|---------------|-------------|--------|--------|
| `kuboard_load_kubeconfig` | Loads kubeconfig from environment or default location | ✅ Working | `kubernetes` |
| `kuboard_create_client_from_context` | Creates Kubernetes client from context | ✅ Working | `kubernetes` |
| `kuboard_fetch_node_metrics` | Fetches node metrics (currently mock data) | ⚠️ Mock | `kubernetes` |
| `kuboard_calculate_cluster_metrics` | Calculates cluster-wide metrics from nodes | ✅ Working | `kubernetes` |
| `kuboard_fetch_pod_events` | Fetches pod events from Kubernetes API | ✅ Working | `kubernetes` |

#### **Metrics Server Integration**
| Function Name | Description | Status | Module |
|---------------|-------------|--------|--------|
| `kuboard_fetch_node_metrics_real` | Fetches real-time metrics from metrics server | ✅ Working | `metrics` |
| `kuboard_fetch_node_metrics_history` | Fetches historical metrics data | ✅ Working | `metrics` |
| `kuboard_fetch_pod_metrics_real` | Fetches real-time pod metrics from metrics server | ✅ Working | `metrics` |
| `kuboard_fetch_pod_metrics_history` | Fetches historical pod metrics data | ✅ Working | `metrics` |
| `kuboard_check_metrics_server_availability` | Checks if metrics server is available | ✅ Working | `metrics` |
| `metrics_api_available` | Internal function to check metrics API availability | ✅ Working | `metrics` |
| `get_node_metrics_by_name` | Internal function to fetch node metrics by name | ✅ Working | `metrics` |
| `get_pod_metrics_by_name` | Internal function to fetch pod metrics by name | ✅ Working | `metrics` |
| `parse_cpu_quantity` | Parses CPU quantity strings (150m, 1.5, etc.) | ✅ Working | `metrics` |
| `parse_memory_quantity` | Parses memory quantity strings (1Gi, 1024Mi, etc.) | ✅ Working | `metrics` |

#### **Utility Functions**
| Function Name | Description | Status | Module |
|---------------|-------------|--------|--------|
| `kuboard_parse_cpu_string` | Parses CPU string (e.g., "1000m", "1") into CPU cores | ✅ Working | `utils` |
| `kuboard_parse_memory_string` | Parses memory string (e.g., "1Gi", "1024Mi") into bytes | ✅ Working | `utils` |
| `kuboard_format_memory` | Formats bytes into human-readable memory string | ✅ Working | `utils` |
| `kuboard_format_cpu` | Formats CPU cores into human-readable string | ✅ Working | `utils` |

### 🎨 **Frontend Functions (Svelte/TypeScript)**

#### **PodsPanel Component** (`src/lib/components/PodsPanel.svelte`)
| Function Name | Description | Status | Module |
|---------------|-------------|--------|--------|
| `showFullPodDetails` | Shows detailed pod information and loads metrics/events | ✅ Working | `PodsPanel` |
| `loadPodMetrics` | Loads pod metrics from backend | ✅ Working | `PodsPanel` |
| `loadPodEvents` | Loads pod events from backend | ✅ Working | `PodsPanel` |
| `loadContainerMetrics` | Loads metrics for specific container | ✅ Working | `PodsPanel` |
| `selectContainer` | Selects container and loads its metrics | ✅ Working | `PodsPanel` |
| `changeResourceType` | Changes resource type for metrics graph | ✅ Working | `PodsPanel` |
| `changeContainerResourceType` | Changes resource type for container metrics | ✅ Working | `PodsPanel` |
| `backToPodsList` | Returns to pods list view | ✅ Working | `PodsPanel` |
| `getControllerInfo` | Extracts controller information from pod | ✅ Working | `PodsPanel` |
| `getPodError` | Detects pod errors (CrashLoopBackOff, OOMKilled, etc.) | ✅ Working | `PodsPanel` |
| `handleSort` | Handles column sorting with three-state cycle (asc → desc → unsorted) | ✅ Working | `PodsPanel` |
| `getSortedPods` | Returns pods sorted by current sort column and direction | ✅ Working | `PodsPanel` |
| `parseSearchQuery` | Parses search query to detect label, field, IP, or general search | ✅ Working | `PodsPanel` |
| `matchLabels` | Matches pod labels against search query | ✅ Working | `PodsPanel` |
| `searchPodFields` | Searches all pod fields and returns match count and locations | ✅ Working | `PodsPanel` |
| `scorePod` | Scores pod relevance for search results (exact matches prioritized) | ✅ Working | `PodsPanel` |
| `searchPods` | Main search function that filters pods based on query type | ✅ Working | `PodsPanel` |
| `compareName` | Compares pod names for sorting | ✅ Working | `PodsPanel` |
| `compareStatus` | Compares pod status for sorting | ✅ Working | `PodsPanel` |
| `compareErrors` | Compares pod error states for sorting | ✅ Working | `PodsPanel` |
| `compareNamespace` | Compares pod namespaces for sorting | ✅ Working | `PodsPanel` |
| `compareRestarts` | Compares pod restart counts for sorting | ✅ Working | `PodsPanel` |
| `compareAge` | Compares pod creation timestamps for sorting | ✅ Working | `PodsPanel` |
| `compareNode` | Compares pod node names for sorting | ✅ Working | `PodsPanel` |
| `getQoSClassClass` | Gets CSS class for QoS class | ✅ Working | `PodsPanel` |
| `getConditionStatusClass` | Gets CSS class for condition status | ✅ Working | `PodsPanel` |
| `getTolerationEffectClass` | Gets CSS class for toleration effect | ✅ Working | `PodsPanel` |
| `formatObject` | Formats object for display | ✅ Working | `PodsPanel` |
| `formatEventTime` | Formats event timestamp | ✅ Working | `PodsPanel` |
| `getEventTypeClass` | Gets CSS class for event type | ✅ Working | `PodsPanel` |
| `getEventReasonClass` | Gets CSS class for event reason | ✅ Working | `PodsPanel` |
| `generateMockEvents` | Generates mock events for demonstration | ✅ Working | `PodsPanel` |

#### **Main Page Orchestration** (`src/routes/+page.svelte`)
| Function Name | Description | Status | Module |
|---------------|-------------|--------|--------|
| `loadContexts` | Loads available contexts from backend | ✅ Working | `+page.svelte` |
| `setContext` | Switches to selected context | ✅ Working | `+page.svelte` |
| `loadClusterOverview` | Loads cluster overview data | ✅ Working | `+page.svelte` |
| `loadResourceDetails` | Loads detailed resource data (nodes, pods, etc.) | ✅ Working | `+page.svelte` |
| `showDemoData` | Shows demo data when not connected | ✅ Working | `+page.svelte` |
| `fetchNodeMetrics` | Fetches current node metrics | ✅ Working | `+page.svelte` |
| `fetchNodeMetricsHistory` | Fetches historical node metrics | ✅ Working | `+page.svelte` |
| `startAutoRefresh` | Starts auto-refresh for metrics | ✅ Working | `+page.svelte` |

#### **Event Handlers** (`src/routes/+page.svelte`)
| Function Name | Description | Status | Module |
|---------------|-------------|--------|--------|
| `handleContextChange` | Handles context change events | ✅ Working | `+page.svelte` |
| `handleRefresh` | Handles refresh button events | ✅ Working | `+page.svelte` |
| `handleNodeSelect` | Handles node selection events | ✅ Working | `+page.svelte` |
| `handleTabChange` | Handles resource tab changes | ✅ Working | `+page.svelte` |
| `handleRefreshIntervalChange` | Handles refresh interval changes | ✅ Working | `+page.svelte` |
| `handleHistoryDurationChange` | Handles history duration changes | ✅ Working | `+page.svelte` |
| `handlePanelToggle` | Handles panel toggle events | ✅ Working | `+page.svelte` |
| `handleResourceSelect` | Handles resource selection events | ✅ Working | `+page.svelte` |
| `handleDebugConsoleToggle` | Handles debug console toggle | ✅ Working | `+page.svelte` |
| `handleDebugConsoleClear` | Handles debug console clear | ✅ Working | `+page.svelte` |
| `handleMetricsRetry` | Handles metrics retry events | ✅ Working | `+page.svelte` |

#### **Utility Functions** (`src/lib/utils/formatters.ts`)
| Function Name | Description | Status | Module |
|---------------|-------------|--------|--------|
| `formatMemory` | Formats bytes into human-readable memory units | ✅ Working | `formatters.ts` |
| `formatCPU` | Formats CPU cores into readable format | ✅ Working | `formatters.ts` |
| `formatPercentage` | Calculates and formats percentage values | ✅ Working | `formatters.ts` |
| `formatTime` | Formats timestamps into readable time | ✅ Working | `formatters.ts` |
| `formatDate` | Formats date strings into localized format | ✅ Working | `formatters.ts` |
| `formatDuration` | Formats duration in minutes to human-readable string | ✅ Working | `formatters.ts` |
| `formatResourceQuantity` | Formats Kubernetes quantity strings | ✅ Working | `formatters.ts` |
| `truncateText` | Truncates text to specified length with ellipsis | ✅ Working | `formatters.ts` |
| `formatStatus` | Formats status for display | ✅ Working | `formatters.ts` |
| `getStatusClass` | Gets status color class for CSS | ✅ Working | `formatters.ts` |
| `formatNodeConditions` | Formats node conditions for display | ✅ Working | `formatters.ts` |
| `formatLabels` | Formats labels for display | ✅ Working | `formatters.ts` |
| `formatAnnotations` | Formats annotations for display | ✅ Working | `formatters.ts` |
| `formatTaints` | Formats taints for display | ✅ Working | `formatters.ts` |

## 🏗️ **Current File Structure**

### **Backend Structure (Rust)**
```
src-tauri/src/
├── lib.rs                 # Main entry point (orchestrates modules)
├── main.rs               # Application entry point
├── commands/
│   └── mod.rs            # Tauri commands (410 lines)
├── kubernetes/
│   └── mod.rs            # Kubernetes integration
├── metrics/
│   └── mod.rs            # Metrics server integration (402 lines)
├── types.rs              # Type definitions
├── app_state.rs          # Application state management
└── utils.rs              # Utility functions
```

### **Frontend Structure (Svelte)**
```
src/
├── routes/
│   ├── +layout.ts        # Layout configuration
│   └── +page.svelte      # Main dashboard (506 lines)
├── lib/
│   ├── components/       # Reusable UI components
│   │   ├── Header.svelte
│   │   ├── ClusterOverview.svelte
│   │   ├── ResourceOverview.svelte
│   │   ├── MetricsGraph.svelte
│   │   ├── PodsPanel.svelte
│   │   ├── WorkloadsTab.svelte
│   │   ├── NodesTab.svelte
│   │   ├── ConfigTab.svelte
│   │   ├── NetworkTab.svelte
│   │   ├── CustomResourcesTab.svelte
│   │   ├── TabbedContent.svelte
│   │   ├── ResourceTabs.svelte
│   │   ├── DonutChart.svelte
│   │   ├── ClusterMetrics.svelte
│   │   └── ThemeSwitcher.svelte
│   ├── styles/
│   │   └── variables.css # CSS custom properties
│   ├── types/
│   │   └── index.ts       # TypeScript interfaces
│   └── utils/
│       └── formatters.ts # Data formatting utilities
└── app.html              # App template
```

## 🎯 **Naming Convention Benefits**

### **Why `kuboard_` Prefix?**
1. **Namespace Protection**: Prevents conflicts with external crates
2. **Clear Ownership**: Immediately identifies Kuboard-specific functions
3. **IDE Support**: Better autocomplete and search
4. **Documentation**: Easier to generate API docs
5. **Maintenance**: Clear separation of concerns

### **Function Naming Pattern**
- **Backend**: `kuboard_<action>_<resource>` (e.g., `kuboard_get_nodes`)
- **Frontend**: `<action><Resource>` (e.g., `loadContexts`)
- **Utilities**: `kuboard_<action>_<type>` (e.g., `kuboard_format_memory`)

## 📊 **Implementation Status**

### **✅ Phase 1: File Structure - COMPLETED**
1. ✅ Created modular file structure
2. ✅ Moved functions to appropriate modules
3. ✅ Added `kuboard_` prefixes to all backend functions
4. ✅ Updated imports and references

### **✅ Phase 2: Function Optimization - COMPLETED**
1. ✅ Implemented real metrics server integration
2. ✅ Added comprehensive error handling
3. ✅ Optimized performance for large clusters
4. ✅ Added robust parsing for CPU/memory quantities

### **✅ Phase 3: Documentation & Testing - COMPLETED**
1. ✅ Updated comprehensive documentation
2. ✅ Created UI organization guide
3. ✅ Documented all functions and components
4. ✅ Verified build and functionality

## 🎯 **Current Status**

### **✅ Completed Features:**
- **Modular UI Architecture:** 4 main components with clear separation
- **Backend Function Organization:** All functions properly prefixed and organized
- **Real Metrics Integration:** Working metrics server integration
- **Comprehensive Documentation:** Updated guides and function documentation
- **Build System:** Successfully building and generating installers
- **Type Safety:** Full TypeScript integration with proper interfaces

### **🚀 Benefits Achieved:**
1. **Maintainability:** Easy to find and modify specific functionality
2. **Reusability:** Components can be used in multiple places
3. **Testing:** Isolated components are easier to test
4. **Collaboration:** Clear separation of concerns
5. **Scalability:** Easy to add new features without conflicts
6. **Documentation:** Comprehensive guides for developers

This modular approach has successfully transformed the codebase into a maintainable and professional structure! 🚀
