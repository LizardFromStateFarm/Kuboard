# 🏗️ Kuboard Architecture Overview

This document provides a comprehensive overview of the Kuboard application architecture, including both UI and backend structure, function organization, and documentation status.

## 📋 **Table of Contents**

1. [Application Overview](#application-overview)
2. [UI Architecture](#ui-architecture)
3. [Styling Architecture](#styling-architecture)
4. [Backend Architecture](#backend-architecture)
5. [Function Organization](#function-organization)
6. [Documentation Status](#documentation-status)
7. [Build System](#build-system)
8. [Development Guidelines](#development-guidelines)

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
├── Header.svelte           # Main header with context selection
├── ClusterOverview.svelte  # Cluster info and node management
├── ResourceOverview.svelte # Resource panels (nodes, pods, etc.)
├── MetricsGraph.svelte     # Real-time resource usage graphs
├── TabbedContent.svelte    # Main tabbed interface container
├── ResourceTabs.svelte     # Tab navigation component
├── WorkloadsTab.svelte     # Workloads management (pods, deployments, services)
├── PodsPanel.svelte        # Advanced pod management with events and container metrics
├── NodesTab.svelte         # Advanced node management and details
├── ConfigTab.svelte        # Configuration management (ConfigMaps, Secrets)
├── NetworkTab.svelte       # Network resources and services
├── CustomResourcesTab.svelte # Custom Resource Definitions (CRDs)
├── DonutChart.svelte       # Resource usage donut charts
├── ClusterMetrics.svelte   # Cluster-wide metrics display
├── LogsWindow.svelte       # Advanced logs panel with structured entries and smart follow mode
└── ThemeSwitcher.svelte    # Development theme switching tool
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
- Pods, Deployments, and Services management
- Lazy loading of resource types
- Detailed resource information display
- Status filtering and sorting

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
- `kuboard_list_contexts` - List available contexts
- `kuboard_set_context` - Set active context
- `kuboard_get_current_context` - Get current context
- `kuboard_get_cluster_overview` - Get cluster information
- `kuboard_get_nodes` - Fetch all nodes
- `kuboard_get_namespaces` - Fetch all namespaces
- `kuboard_get_pods` - Fetch all pods
- `kuboard_get_deployments` - Fetch all deployments
- `kuboard_get_services` - Fetch all services
- `kuboard_get_configmaps` - Fetch all ConfigMaps
- `kuboard_get_secrets` - Fetch all Secrets
- `kuboard_get_custom_resources` - Fetch custom resources
- `kuboard_get_node_metrics` - Get current node metrics
- `kuboard_get_node_metrics_history` - Get historical metrics
- `kuboard_get_pod_metrics` - Get current pod metrics
- `kuboard_get_pod_metrics_history` - Get historical pod metrics
- `kuboard_get_pod_events` - Get pod events for troubleshooting
- `kuboard_get_pod_logs` - Get pod logs with container support
- `kuboard_check_metrics_availability` - Check metrics server

#### **Kubernetes Integration** (`kubernetes/mod.rs`)
- `kuboard_load_kubeconfig` - Load kubeconfig
- `kuboard_create_client_from_context` - Create Kubernetes client
- `kuboard_calculate_cluster_metrics` - Calculate cluster metrics
- `kuboard_fetch_pod_events` - Fetch pod events from Kubernetes API
- `kuboard_fetch_pod_logs` - Fetch pod logs from Kubernetes API

#### **Metrics Server Integration** (`metrics/mod.rs`)
- `kuboard_fetch_node_metrics_real` - Fetch real-time metrics
- `kuboard_fetch_node_metrics_history` - Fetch historical metrics
- `kuboard_fetch_pod_metrics_real` - Fetch real-time pod metrics
- `kuboard_fetch_pod_metrics_history` - Fetch historical pod metrics
- `kuboard_check_metrics_server_availability` - Check availability
- `parse_cpu_quantity` - Parse CPU quantities
- `parse_memory_quantity` - Parse memory quantities

#### **Utility Functions** (`utils.rs`)
- `kuboard_parse_cpu_string` - Parse CPU strings
- `kuboard_parse_memory_string` - Parse memory strings
- `kuboard_format_memory` - Format memory display
- `kuboard_format_cpu` - Format CPU display

## 📊 **Function Organization**

### **Naming Conventions**
- **Backend Functions:** `kuboard_<action>_<resource>` (e.g., `kuboard_get_nodes`)
- **Frontend Functions:** `<action><Resource>` (e.g., `loadContexts`)
- **Utility Functions:** `kuboard_<action>_<type>` (e.g., `kuboard_format_memory`)

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
