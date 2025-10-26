# 🎨 Kuboard UI Organization Guide

This guide explains the modular organization of the Kuboard UI components and how to navigate the codebase.

## 📁 Directory Structure

```
src/
├── lib/
│   ├── components/          # Reusable UI components
│   │   ├── Header.svelte    # Main header with context selector
│   │   ├── ClusterOverview.svelte  # Cluster info and node management
│   │   ├── ResourceOverview.svelte # Resource panels (nodes, pods, etc.)
│   │   └── MetricsGraph.svelte     # Resource usage graphs
│   ├── styles/              # CSS organization
│   │   └── variables.css    # CSS custom properties and global styles
│   ├── types/               # TypeScript interfaces
│   │   └── index.ts         # All UI type definitions
│   └── utils/               # Utility functions
│       └── formatters.ts    # Data formatting functions
├── routes/
│   ├── +layout.ts          # Layout configuration
│   └── +page.svelte        # Main page (orchestrates components)
└── app.html                # HTML template
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

### 2. **Cluster Overview Component** (`src/lib/components/ClusterOverview.svelte`)
**Purpose:** Display cluster information and comprehensive node management

**Features:**
- Cluster basic information and statistics
- Two-panel layout (node list + node details)
- Node selection and detailed information display
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
