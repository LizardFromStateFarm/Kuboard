<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  
  // Import our modular components
  import Header from '$lib/components/Header.svelte';
  import ClusterOverviewComponent from '$lib/components/ClusterOverview.svelte';
  import ResourceOverview from '$lib/components/ResourceOverview.svelte';
  import MetricsGraph from '$lib/components/MetricsGraph.svelte';
  
  // Import types
  import type { 
    KubeContext, 
    ClusterOverview, 
    NodeDetails, 
    ResourceTab, 
    ExpandedPanel,
    MetricsDataPoint 
  } from '$lib/types/index.js';
  
  // Import utilities
  import { formatMemory, formatCPU, formatPercentage } from '$lib/utils/formatters.js';


  // State variables
  let contexts: KubeContext[] = [];
  let currentContext: KubeContext | null = null;
  let clusterOverview: ClusterOverview | null = null;
  let loading = false;
  let error = "";
  let success = "";
  let isTauriAvailable = false;

  // Resource data and expanded states
  let nodes: any[] = [];
  let namespaces: any[] = [];
  let pods: any[] = [];
  let deployments: any[] = [];
  let expandedPanel: ExpandedPanel = null;
  let resourceLoading = false;

  // Node selection state
  let selectedNode: NodeDetails | null = null;

  // Resource usage graph state
  let activeResourceTab: ResourceTab = 'cpu';
  let resourceHistory: MetricsDataPoint[] = [];
  let metricsLoading = false;
  let metricsError: string | null = null;
  let metricsServerAvailable = false;
  let isUsingMockData = false;
  let debugInfo = '';
  let lastUpdateTime = '';
  let debugConsole = '';
  let showDebugConsole = false;
  let autoRefreshInterval: number | null = null;
  let refreshIntervalSeconds = 10;
  let maxDataPoints = 30;
  let historyDurationMinutes = 30;
  let maxHistoryDurationMinutes = 720;
  let selectedContextName = '';


  // Check if we're running in Tauri environment
  onMount(async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      isTauriAvailable = true;
      console.log('✅ Tauri environment detected');
      await loadContexts();
    } catch (error) {
      isTauriAvailable = false;
      console.log('🌐 Web development mode - using demo data');
      showDemoData();
    }
  });

  onDestroy(() => {
    if (autoRefreshInterval) {
      clearInterval(autoRefreshInterval);
    }
  });

  // Tauri API functions
  async function loadContexts() {
    if (!isTauriAvailable) return;
    
    try {
      loading = true;
      error = "";
      const { invoke } = await import('@tauri-apps/api/core');
      const response = await invoke('kuboard_list_contexts');
      contexts = response.contexts || [];
      
      if (response.current_context) {
        currentContext = contexts.find(ctx => ctx.name === response.current_context) || null;
        selectedContextName = response.current_context;
      }
      
      success = `Loaded ${contexts.length} contexts`;
      console.log('✅ Contexts loaded:', contexts);
    } catch (err) {
      error = `Failed to load contexts: ${err}`;
      console.error('❌ Error loading contexts:', err);
    } finally {
      loading = false;
    }
  }

  async function setContext(contextName: string) {
    if (!isTauriAvailable) return;
    
    try {
      loading = true;
      error = "";
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('kuboard_set_context', { contextName });
      
      currentContext = contexts.find(ctx => ctx.name === contextName) || null;
      selectedContextName = contextName;
      
      // Load cluster overview after setting context
      await loadClusterOverview();
      
      success = `Switched to context: ${contextName}`;
      console.log('✅ Context switched to:', contextName);
    } catch (err) {
      error = `Failed to set context: ${err}`;
      console.error('❌ Error setting context:', err);
    } finally {
      loading = false;
    }
  }

  async function loadClusterOverview() {
    if (!isTauriAvailable) return;
    
    try {
      loading = true;
      error = "";
      const { invoke } = await import('@tauri-apps/api/core');
      clusterOverview = await invoke('kuboard_get_cluster_overview');
      
      // Load resource details
      await loadResourceDetails();
      
      success = "Cluster overview loaded successfully";
      console.log('✅ Cluster overview loaded:', clusterOverview);
    } catch (err) {
      error = `Failed to load cluster overview: ${err}`;
      console.error('❌ Error loading cluster overview:', err);
    } finally {
      loading = false;
    }
  }

  async function loadResourceDetails() {
    if (!isTauriAvailable) return;
    
    try {
      resourceLoading = true;
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Load all resource types in parallel
      const [nodesData, namespacesData, podsData, deploymentsData] = await Promise.all([
        invoke('kuboard_get_nodes'),
        invoke('kuboard_get_namespaces'),
        invoke('kuboard_get_pods'),
        invoke('kuboard_get_deployments')
      ]);
      
      nodes = nodesData || [];
      namespaces = namespacesData || [];
      pods = podsData || [];
      deployments = deploymentsData || [];
      
      console.log('✅ Resource details loaded');
    } catch (err) {
      console.error('❌ Error loading resource details:', err);
    } finally {
      resourceLoading = false;
    }
  }

  // Demo data for web development
  function showDemoData() {
    console.log('🎭 Loading demo data for web development');
    
    // Demo contexts
    contexts = [
      {
        name: "minikube",
        cluster: "minikube",
        user: "minikube",
        namespace: "default",
        is_current: true
      }
    ];
    
    currentContext = contexts[0];
    selectedContextName = "minikube";
    
    // Demo cluster overview
    clusterOverview = {
      cluster_info: {
        name: "minikube",
        server: "https://127.0.0.1:8443",
        version: "v1.28.0"
      },
      node_count: 1,
      namespace_count: 4,
      pod_count: 12,
      deployment_count: 6,
      kubernetes_version: "v1.28.0",
      cluster_metrics: {
        max_nodes: 1,
        active_nodes: 1,
        nodes: [{
          name: "minikube",
          status: "Ready",
          max_cpu_cores: 2.0,
          max_memory_bytes: 8053063680,
          allocatable_cpu_cores: 1.8,
          allocatable_memory_bytes: 6442450944,
          cpu_usage_percent: 15.5,
          memory_usage_percent: 23.8,
          conditions: ["Ready: True", "MemoryPressure: False", "DiskPressure: False"],
          os: "linux amd64",
          kernel_version: "5.15.0-91-generic",
          kubelet_version: "v1.28.0",
          container_runtime: "containerd://1.7.0",
          disk_capacity: 100000000000,
          disk_allocatable: 90000000000,
          disk_usage_percent: 12.5,
          labels: {
            "kubernetes.io/hostname": "minikube",
            "kubernetes.io/os": "linux",
            "node-role.kubernetes.io/control-plane": ""
          },
          annotations: {
            "kubeadm.alpha.kubernetes.io/cri-socket": "/var/run/containerd/containerd.sock"
          },
          taints: [],
          metrics_available: true
        }]
      }
    };
    
    // Set first node as selected
    selectedNode = clusterOverview.cluster_metrics?.nodes[0] || null;
    
    // Demo resource data
    nodes = [
      {
        metadata: { name: "minikube" },
        status: {
          conditions: [{ type: "Ready", status: "True" }],
          nodeInfo: {
            kubeletVersion: "v1.28.0",
            operatingSystem: "linux"
          }
        }
      }
    ];
    
    namespaces = [
      { metadata: { name: "default", creationTimestamp: "2024-01-01T00:00:00Z" } },
      { metadata: { name: "kube-system", creationTimestamp: "2024-01-01T00:00:00Z" } },
      { metadata: { name: "kube-public", creationTimestamp: "2024-01-01T00:00:00Z" } },
      { metadata: { name: "kube-node-lease", creationTimestamp: "2024-01-01T00:00:00Z" } }
    ];
    
    pods = [
      {
        metadata: { name: "nginx-pod", namespace: "default" },
        status: { phase: "Running" },
        spec: { nodeName: "minikube" }
      }
    ];
    
    deployments = [
      {
        metadata: { name: "nginx-deployment", namespace: "default" },
        status: { 
          conditions: [{ type: "Available", status: "True" }],
          readyReplicas: 1
        },
        spec: { replicas: 1 }
      }
    ];
    
    success = "Demo data loaded successfully";
  }

  // Event handlers for components
  function handleContextChange(event: CustomEvent<string>) {
    const contextName = event.detail;
    setContext(contextName);
  }

  function handleRefresh() {
    if (isTauriAvailable) {
      loadContexts();
    } else {
      showDemoData();
    }
  }

  function handleNodeSelect(event: CustomEvent<NodeDetails>) {
    selectedNode = event.detail;
    console.log('🖥️ Node selected:', selectedNode.name);
    
    // Start auto-refresh for metrics
    startAutoRefresh();
  }

  function handleTabChange(event: CustomEvent<ResourceTab>) {
    activeResourceTab = event.detail;
    console.log('📊 Tab changed to:', activeResourceTab);
  }

  function handleRefreshIntervalChange(event: CustomEvent<number>) {
    refreshIntervalSeconds = event.detail;
    console.log('⏰ Refresh interval changed to:', refreshIntervalSeconds);
    startAutoRefresh();
  }

  function handleHistoryDurationChange(event: CustomEvent<number>) {
    historyDurationMinutes = event.detail;
    console.log('📈 History duration changed to:', historyDurationMinutes);
  }

  function handlePanelToggle(event: CustomEvent<ExpandedPanel>) {
    expandedPanel = event.detail;
    console.log('📋 Panel toggled:', expandedPanel);
  }

  function handleResourceSelect(event: CustomEvent<{type: string, resource: any}>) {
    console.log('🔍 Resource selected:', event.detail);
  }

  function handleDebugConsoleToggle() {
    showDebugConsole = !showDebugConsole;
  }

  function handleDebugConsoleClear() {
    debugConsole = '';
  }

  function handleMetricsRetry(event: CustomEvent<string>) {
    const nodeName = event.detail;
    console.log('🔄 Retrying metrics for node:', nodeName);
    // Implement retry logic
  }

  // Auto-refresh functionality
  function startAutoRefresh() {
    if (autoRefreshInterval) {
      clearInterval(autoRefreshInterval);
    }
    
    if (selectedNode && isTauriAvailable) {
      autoRefreshInterval = setInterval(async () => {
        try {
          await fetchNodeMetrics(selectedNode.name);
        } catch (error) {
          console.error('❌ Auto-refresh error:', error);
        }
      }, refreshIntervalSeconds * 1000);
    }
  }

  async function fetchNodeMetrics(nodeName: string) {
    if (!isTauriAvailable) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const metrics = await invoke('kuboard_get_node_metrics', { nodeName });
      
      // Update selected node with new metrics
      if (selectedNode && selectedNode.name === nodeName) {
        selectedNode.cpu_usage_percent = metrics.cpu.usage_percent;
        selectedNode.memory_usage_percent = metrics.memory.usage_percent;
        selectedNode.disk_usage_percent = metrics.disk.usage_percent;
        selectedNode = selectedNode; // Trigger reactivity
      }
    } catch (error) {
      console.error('❌ Error fetching metrics:', error);
    }
  }

  // Initialize on mount
  onMount(async () => {
    if (isTauriAvailable) {
      await loadContexts();
    }
  });
</script>

<!-- Import CSS variables -->
<link rel="stylesheet" href="/src/lib/styles/variables.css">

<main>
  <!-- Header Component -->
  <Header 
    {contexts} 
    {currentContext} 
    {loading}
    {isTauriAvailable}
    on:contextChange={handleContextChange}
    on:refresh={handleRefresh}
  />

  <!-- Error and Success Messages -->
  {#if error}
    <div class="error-message">
      <strong>Error:</strong> {error}
    </div>
  {/if}

  {#if success}
    <div class="success-message">
      <strong>Success:</strong> {success}
    </div>
  {/if}

  <!-- Cluster Overview Component -->
  {#if clusterOverview}
    <ClusterOverviewComponent 
      {clusterOverview}
      {selectedNode}
      {metricsLoading}
      {metricsError}
      {resourceHistory}
      {activeResourceTab}
      {refreshIntervalSeconds}
      {historyDurationMinutes}
      {debugInfo}
      {lastUpdateTime}
      {debugConsole}
      {showDebugConsole}
      on:nodeSelect={handleNodeSelect}
      on:tabChange={handleTabChange}
      on:refreshIntervalChange={handleRefreshIntervalChange}
      on:historyDurationChange={handleHistoryDurationChange}
      on:debugConsoleToggle={handleDebugConsoleToggle}
      on:debugConsoleClear={handleDebugConsoleClear}
      on:metricsRetry={handleMetricsRetry}
    />
  {/if}

  <!-- Resource Overview Component -->
  {#if clusterOverview}
    <ResourceOverview 
      {nodes}
      {namespaces}
      {pods}
      {deployments}
      {expandedPanel}
      {resourceLoading}
      on:panelToggle={handlePanelToggle}
      on:resourceSelect={handleResourceSelect}
    />
  {/if}
</main>

<style>
  /* Import CSS variables */
  @import '../lib/styles/variables.css';

  main {
    width: 100vw;
    height: 100vh;
    margin: 0;
    padding: 20px;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background-color: var(--background-color);
    color: var(--text-color);
    overflow-x: auto;
    box-sizing: border-box;
  }

  .error-message {
    color: var(--error-color);
    margin: 10px 0;
    padding: 10px;
    border: 1px solid var(--error-color);
    border-radius: var(--radius-sm);
    background: rgba(239, 68, 68, 0.1);
  }

  .success-message {
    color: var(--success-color);
    margin: 10px 0;
    padding: 10px;
    border: 1px solid var(--success-color);
    border-radius: var(--radius-sm);
    background: rgba(16, 185, 129, 0.1);
  }

  /* Responsive Design */
  @media (max-width: 768px) {
    main {
      padding: 10px;
    }
  }
</style>