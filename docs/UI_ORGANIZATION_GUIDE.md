# 🎨 Kuboard UI Organization Guide

This guide explains the modular organization of the Kuboard UI components and how to navigate the codebase.

## 📁 Directory Structure

```
src/
├── lib/
│   ├── components/          # Reusable UI components
│   │   ├── Header.svelte    # Main header with context selector
│   │   ├── ClusterOverview.svelte  # Cluster info and metrics
│   │   ├── ClusterMetrics.svelte   # Cluster-wide resource metrics (donut charts)
│   │   ├── DonutChart.svelte       # CSS-based donut chart component
│   │   ├── TabbedContent.svelte    # Main resource management tabs
│   │   ├── ResourceTabs.svelte     # Tab navigation component
│   │   ├── WorkloadsTab.svelte     # Workloads management container
│   │   ├── PodsPanel.svelte        # Pod management panel
│   │   ├── PodDetails.svelte       # Detailed pod information
│   │   ├── DeploymentsPanel.svelte # Deployment management panel
│   │   ├── DeploymentDetails.svelte # Detailed deployment information
│   │   ├── StatefulSetsPanel.svelte # StatefulSet management panel
│   │   ├── StatefulSetDetails.svelte # Detailed StatefulSet information
│   │   ├── DaemonSetsPanel.svelte  # DaemonSet management panel
│   │   ├── DaemonSetDetails.svelte # Detailed DaemonSet information
│   │   ├── ReplicaSetsPanel.svelte # ReplicaSet management panel
│   │   ├── ReplicaSetDetails.svelte # Detailed ReplicaSet information
│   │   ├── CronJobsPanel.svelte    # CronJob management panel
│   │   ├── CronJobDetails.svelte   # Detailed CronJob information
│   │   ├── ServiceDetails.svelte   # Detailed service information
│   │   ├── StorageTab.svelte       # Storage management container
│   │   ├── PersistentVolumesPanel.svelte # PV management panel
│   │   ├── PersistentVolumeClaimsPanel.svelte # PVC management panel
│   │   ├── StorageClassesPanel.svelte # StorageClass management panel
  │   │   ├── SecurityTab.svelte       # Security management container
  │   │   ├── RolesPanel.svelte        # Role management panel
  │   │   ├── ClusterRolesPanel.svelte # ClusterRole management panel
  │   │   ├── RoleBindingsPanel.svelte # RoleBinding management panel
  │   │   ├── ClusterRoleBindingsPanel.svelte # ClusterRoleBinding management panel
  │   │   ├── ServiceAccountsPanel.svelte # ServiceAccount management panel
  │   │   ├── NodesTab.svelte         # Cluster nodes management
│   │   ├── ConfigTab.svelte        # ConfigMaps and Secrets
│   │   ├── NetworkTab.svelte       # Services and networking
│   │   ├── CustomResourcesTab.svelte # CRDs and custom resources
│   │   ├── ResourceOverview.svelte # Legacy resource panels
│   │   ├── MetricsGraph.svelte     # Resource usage graphs
│   │   ├── LogsWindow.svelte       # Advanced logs panel
│   │   ├── QuickActionsMenu.svelte # Context menu for resources
│   │   ├── PortForwardManager.svelte # Port forwarding UI
│   │   ├── TerminalWindow.svelte   # Terminal/exec with xterm.js
│   │   ├── ResourceDescribe.svelte # Resource describe display
│   │   └── ThemeSwitcher.svelte    # Dev mode theme controls
│   ├── styles/              # CSS organization
│   │   ├── color-palette.css # Centralized color definitions
│   │   ├── variables.css    # CSS custom properties and global styles
│   │   └── README.md        # Color customization guide
│   ├── types/               # TypeScript interfaces
│   │   └── index.ts         # All UI type definitions
│   └── utils/               # Utility functions
│       ├── formatters.ts    # Data formatting functions
│       └── performance.ts   # Performance utilities
├── routes/
│   ├── +layout.ts          # Layout configuration
│   └── +page.svelte        # Main page (orchestrates components)
└── app.html                # HTML template
```

## 🎨 Color System & Theming

### Centralized Color Management
The application uses a centralized color system defined in `src/lib/styles/color-palette.css`:

**Key Features:**
- **Single Source of Truth**: All colors defined in one file
- **CSS Custom Properties**: Easy to modify and maintain
- **Theme Variations**: Dark (default), Light, and High Contrast themes
- **Dev Mode Controls**: ThemeSwitcher component for real-time editing
- **Semantic Naming**: Colors named by purpose, not appearance

**Color Categories:**
- **Primary Brand**: Main application colors
- **Status Colors**: Success, warning, error, info states
- **Background Colors**: Various background shades
- **Text Colors**: Primary, secondary, muted text
- **Status Badges**: Ready, pending, error, info states
- **Donut Charts**: Usage visualization colors
- **Network Services**: Service type indicators

**Usage:**
```css
/* Use semantic color names */
color: var(--text-primary);
background: var(--status-ready-bg);
border: 1px solid var(--border-primary);
```

## 🧩 Component Breakdown

### 1. **Header Component** (`src/lib/components/Header.svelte`)
**Purpose:** Main application header with navigation and context selection

**Features:**
- App title and branding
- Context dropdown selector
- Mode indicator (Desktop vs Demo)
- Refresh button
- Version display

**Props:**
- `contexts: KubeContext[]` - Available Kubernetes contexts
- `currentContext: KubeContext | null` - Currently selected context
- `loading: boolean` - Loading state
- `isTauriAvailable: boolean` - Environment detection

**Events:**
- `contextChange` - Emitted when context is changed
- `refresh` - Emitted when refresh button is clicked

### 2. **Resource Management System**
**Purpose:** Tabbed interface for managing different Kubernetes resource types

**Architecture:**
- **TabbedContent.svelte**: Main container that orchestrates all resource tabs
- **ResourceTabs.svelte**: Navigation component for tab switching
- **Individual Tab Components**: Specialized components for each resource type

**Available Tabs:**
1. **Workloads** (`WorkloadsTab.svelte`): 
   - Pods (`PodsPanel.svelte`, `PodDetails.svelte`)
   - Deployments (`DeploymentsPanel.svelte`, `DeploymentDetails.svelte`)
   - StatefulSets (`StatefulSetsPanel.svelte`, `StatefulSetDetails.svelte`)
   - DaemonSets (`DaemonSetsPanel.svelte`, `DaemonSetDetails.svelte`)
   - ReplicaSets (`ReplicaSetsPanel.svelte`, `ReplicaSetDetails.svelte`)
   - CronJobs (`CronJobsPanel.svelte`, `CronJobDetails.svelte`)
   - Services (`ServiceDetails.svelte`)
2. **Nodes** (`NodesTab.svelte`): Cluster nodes with detailed management
3. **Config** (`ConfigTab.svelte`): ConfigMaps and Secrets
4. **Network** (`NetworkTab.svelte`): Services and networking resources
5. **Storage** (`StorageTab.svelte`): 
     - PersistentVolumes (`PersistentVolumesPanel.svelte`)
     - PersistentVolumeClaims (`PersistentVolumeClaimsPanel.svelte`)
     - StorageClasses (`StorageClassesPanel.svelte`)
6. **Custom Resources** (`CustomResourcesTab.svelte`): CRDs and custom resources
7. **Security** (`SecurityTab.svelte`): 
     - Roles (`RolesPanel.svelte`)
     - ClusterRoles (`ClusterRolesPanel.svelte`)
     - RoleBindings (`RoleBindingsPanel.svelte`)
     - ClusterRoleBindings (`ClusterRoleBindingsPanel.svelte`)
     - ServiceAccounts (`ServiceAccountsPanel.svelte`)
     - Secrets (in ConfigTab)

**Key Features:**
- **Lazy Loading**: Resources only loaded when tab is selected
- **Real-time Counts**: Tab badges show resource counts
- **Consistent UI**: All tabs follow the same design patterns
- **Error Handling**: Graceful error states and retry mechanisms
- **Responsive Design**: Works on different screen sizes

### 3. **Cluster Overview Component** (`src/lib/components/ClusterOverview.svelte`)
**Purpose:** Display cluster information and metrics overview

**Features:**
- Cluster basic information and statistics
- **ClusterMetrics**: Real-time cluster-wide resource usage (donut charts)
- **TabbedContent**: Resource management tabs (workloads, nodes, config, etc.)
- **Node Management**: Two-panel layout (node list + node details)
- Node selection and detailed information display

**Sub-components:**
- **ClusterMetrics.svelte**: Donut charts for CPU, memory, disk usage
- **DonutChart.svelte**: Reusable CSS-based donut chart component
- **TabbedContent.svelte**: Resource management interface
- System information (OS, kernel, kubelet, container runtime)
- Resource specifications and usage
- Labels, annotations, and taints display
- Real-time metrics integration with MetricsGraph component
- Debug console for troubleshooting

**Props:**
- `clusterOverview: ClusterOverview` - Complete cluster data
- `selectedNode: NodeDetails | null` - Currently selected node
- `metricsLoading: boolean` - Metrics loading state
- `metricsError: string | null` - Metrics error state
- `resourceHistory: MetricsDataPoint[]` - Historical metrics data
- `activeResourceTab: ResourceTab` - Active metrics tab (cpu/memory/disk)
- `refreshIntervalSeconds: number` - Auto-refresh interval
- `historyDurationMinutes: number` - Metrics history duration
- `debugInfo: string` - Debug information
- `lastUpdateTime: string` - Last update timestamp
- `debugConsole: string` - Debug console content
- `showDebugConsole: boolean` - Debug console visibility

**Events:**
- `nodeSelect` - Emitted when a node is selected
- `tabChange` - Emitted when metrics tab changes
- `refreshIntervalChange` - Emitted when refresh interval changes
- `historyDurationChange` - Emitted when history duration changes
- `debugConsoleToggle` - Emitted when debug console is toggled
- `debugConsoleClear` - Emitted when debug console is cleared
- `metricsRetry` - Emitted when metrics retry is requested

### 3. **Resource Overview Component** (`src/lib/components/ResourceOverview.svelte`)
**Purpose:** Display and manage Kubernetes resources with expandable panels

**Features:**
- Expandable resource panels for nodes, namespaces, pods, and deployments
- Resource counts and status information
- Loading states and error handling
- Interactive resource selection
- Responsive grid layout

**Props:**
- `nodes: K8sNode[]` - Node resource data
- `namespaces: K8sNamespace[]` - Namespace resource data
- `pods: K8sPod[]` - Pod resource data
- `deployments: K8sDeployment[]` - Deployment resource data
- `expandedPanel: ExpandedPanel` - Currently expanded panel
- `resourceLoading: boolean` - Resource loading state

**Events:**
- `panelToggle` - Emitted when a panel is toggled
- `resourceSelect` - Emitted when a resource is selected

### 4. **Metrics Graph Component** (`src/lib/components/MetricsGraph.svelte`)
**Purpose:** Real-time resource usage graphs with Chart.js integration

**Features:**
- Chart.js integration for professional charts
- Real-time data updates with auto-refresh
- Multiple resource types (CPU, Memory, Disk)
- Customizable time ranges and duration
- Interactive tooltips and data points
- Responsive design for different screen sizes
- Loading states and error handling
- Mock data fallback for development

**Props:**
- `data: MetricsDataPoint[]` - Time-series metrics data
- `type: ResourceTab` - Graph type (cpu/memory/disk)
- `duration: number` - Time duration in minutes
- `autoRefresh: boolean` - Auto-refresh enabled
- `loading: boolean` - Loading state
- `error: string | null` - Error state

## 🎨 Styling Organization

### CSS Variables (`src/lib/styles/variables.css`)
Centralized color scheme and design tokens:
- **Colors:** Primary, accent, background, text colors
- **Spacing:** Consistent spacing scale
- **Typography:** Font sizes and weights
- **Shadows:** Elevation system
- **Transitions:** Animation timing

### Component Styles
Each component has its own `<style>` section with:
- Component-specific styles
- Responsive design
- State-based styling
- Animation effects

## 🔧 Type Definitions (`src/lib/types/index.ts`)

### Core Interfaces:
- `KubeContext` - Kubernetes context information
- `ClusterInfo` - Cluster basic information
- `NodeDetails` - Detailed node information
- `ClusterMetrics` - Cluster metrics data
- `ClusterOverview` - Complete cluster overview
- `MetricsDataPoint` - Time-series metrics data

### Utility Types:
- `ResourceTab` - Metrics tab types
- `ExpandedPanel` - Panel expansion states

## 🚀 Usage Examples

### Using Components:

```svelte
<!-- Main page using components -->
<script>
  import Header from '$lib/components/Header.svelte';
  import ClusterOverview from '$lib/components/ClusterOverview.svelte';
  import ResourceOverview from '$lib/components/ResourceOverview.svelte';
  
  let contexts = [];
  let currentContext = null;
  let clusterOverview = null;
</script>

<Header 
  {contexts} 
  {currentContext} 
  on:contextChange={handleContextChange}
  on:refresh={handleRefresh}
/>

{#if clusterOverview}
  <ClusterOverview 
    {clusterOverview}
    on:nodeSelect={handleNodeSelect}
  />
{/if}
```

### Event Handling:

```svelte
<script>
  function handleContextChange(event) {
    const contextName = event.detail;
    // Handle context change
  }
  
  function handleNodeSelect(event) {
    const node = event.detail;
    // Handle node selection
  }
</script>
```

## 📋 Development Guidelines

### 1. **Component Creation:**
- Keep components focused on single responsibility
- Use TypeScript for all props and events
- Include comprehensive JSDoc comments
- Follow Svelte best practices

### 2. **Styling:**
- Use CSS custom properties from `variables.css`
- Follow the design system consistently
- Implement responsive design
- Test in both light and dark themes

### 3. **State Management:**
- Use props for data flow down
- Use events for actions up
- Keep state as local as possible
- Use stores for global state

### 4. **Testing:**
- Test component isolation
- Test event handling
- Test responsive behavior
- Test accessibility

## 🔍 Finding Components

### By Functionality:
- **Header/Navigation:** `Header.svelte`
- **Cluster Info:** `ClusterOverview.svelte`
- **Resource Lists:** `ResourceOverview.svelte`
- **Node Details:** `NodeDetails.svelte`
- **Graphs/Charts:** `MetricsGraph.svelte`

### By File Type:
- **Components:** `src/lib/components/`
- **Styles:** `src/lib/styles/`
- **Types:** `src/lib/types/`
- **Utils:** `src/lib/utils/`

## 🎯 Benefits of This Organization

1. **Maintainability:** Easy to find and modify specific functionality
2. **Reusability:** Components can be used in multiple places
3. **Testing:** Isolated components are easier to test
4. **Collaboration:** Clear separation of concerns
5. **Scalability:** Easy to add new features without conflicts

## 📚 Next Steps

1. **Extract remaining components** from the main page
2. **Create utility functions** for common operations
3. **Add component documentation** with examples
4. **Implement component testing** strategy
5. **Create design system** documentation

This modular approach makes the Kuboard UI much more maintainable and easier to work with!
