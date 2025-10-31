<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  
  // Import our modular components
  import Header from '$lib/components/Header.svelte';
  import ClusterOverviewComponent from '$lib/components/ClusterOverview.svelte';
  import MetricsGraph from '$lib/components/MetricsGraph.svelte';
  
  // Import types
  import type { 
    KubeContext, 
    ClusterOverview, 
    NodeDetails, 
    ResourceTab, 
    MetricsDataPoint 
  } from '$lib/types/index.js';
  
  // Import utilities
  import { formatMemory, formatCPU, formatPercentage } from '$lib/utils/formatters.js';
  
  // Import theme switcher for dev mode
  import ThemeSwitcher from '$lib/components/ThemeSwitcher.svelte';


  // State variables
  let contexts: KubeContext[] = [];
  let currentContext: KubeContext | null = null;
  let clusterOverview: ClusterOverview | null = null;
  let loading = false;
  let error = "";
  let success = "";
  let isTauriAvailable = false;

  // Resource data
  let nodes: any[] = [];
  let namespaces: any[] = [];
  let pods: any[] = [];
  let deployments: any[] = [];
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
  let metricsRefreshInterval: number | null = null;
  let refreshIntervalSeconds = 10;
  let metricsRefreshIntervalSeconds = 5;
  let autoRefreshEnabled = true;
  let metricsAutoRefreshEnabled = true;
  let lastRefreshTime = '';
  let lastMetricsRefreshTime = '';
  let maxDataPoints = 30;
  let historyDurationMinutes = 30;
  let maxHistoryDurationMinutes = 720;
  let selectedContextName = '';


  // Check if we're running in Tauri environment
  onMount(async () => {
    console.log('🚀 Kuboard app starting...');
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      isTauriAvailable = true;
      console.log('✅ Tauri environment detected');
      await loadContexts();
      console.log('✅ Contexts loaded:', contexts.length);
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
      const response = await invoke('kuboard_list_contexts') as any;
      contexts = response.contexts || [];
      
      // Don't auto-select context - let user choose
      // if (response.current_context) {
      //   currentContext = contexts.find(ctx => ctx.name === response.current_context) || null;
      //   selectedContextName = response.current_context;
      // }
      
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
    console.log('🔄 Setting context to:', contextName);
    if (!isTauriAvailable) {
      console.log('⚠️ Tauri not available, skipping context set');
      return;
    }

    try {
      loading = true;
      error = "";
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('kuboard_set_context', { contextName });
      
      currentContext = contexts.find(ctx => ctx.name === contextName) || null;
      selectedContextName = contextName;
      
      console.log('✅ Context set, loading cluster overview...');
      // Load cluster overview after setting context
      await loadClusterOverview();
      
      console.log('✅ Cluster overview loaded, checking metrics...');
      // Check metrics server availability
      await checkMetricsAvailability();
      
      // Start auto-refresh for cluster data
      startAutoRefresh();
      
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
      
      console.log('✅ Cluster overview loaded:', clusterOverview);
      
      // Load resource details
      console.log('🔄 Loading resource details...');
      await loadResourceDetails();
      
      success = "Cluster overview loaded successfully";
    } catch (err) {
      error = `Failed to load cluster overview: ${err}`;
      console.error('❌ Error loading cluster overview:', err);
    } finally {
      loading = false;
    }
  }

  async function loadResourceDetails() {
    if (!isTauriAvailable) {
      console.log('⚠️ Tauri not available, skipping resource details');
      return;
    }
    
    try {
      console.log('🔄 Starting resource details loading...');
      resourceLoading = true;
      const { invoke } = await import('@tauri-apps/api/core');
      
      console.log('🔄 Calling kuboard_get_nodes...');
      // Load all resource types in parallel
      const [nodesData, namespacesData, podsData, deploymentsData] = await Promise.all([
        invoke('kuboard_get_nodes'),
        invoke('kuboard_get_namespaces'),
        invoke('kuboard_get_pods'),
        invoke('kuboard_get_deployments')
      ]);
      
      console.log('📊 Raw nodes data:', nodesData);
      console.log('📊 Raw namespaces data:', namespacesData);
      console.log('📊 Raw pods data:', podsData);
      console.log('📊 Raw deployments data:', deploymentsData);
      
      nodes = (nodesData as any[]) || [];
      namespaces = (namespacesData as any[]) || [];
      pods = (podsData as any[]) || [];
      deployments = (deploymentsData as any[]) || [];
      
      console.log('✅ Resource details loaded - Nodes:', nodes.length, 'Namespaces:', namespaces.length, 'Pods:', pods.length, 'Deployments:', deployments.length);
      console.log('📊 Nodes data sample:', nodes[0]);
      console.log('🔄 Main page: Nodes array updated, length:', nodes.length);
    } catch (err) {
      console.error('❌ Error loading resource details:', err);
      error = `Failed to load resource details: ${err}`;
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

  async function handleNodeSelect(event: CustomEvent<NodeDetails>) {
    selectedNode = event.detail;
    console.log('🖥️ Node selected:', selectedNode.name);
    
    // Load metrics for the selected node
    if (selectedNode && isTauriAvailable) {
      try {
        metricsLoading = true;
        metricsError = null;
        
        const { invoke } = await import('@tauri-apps/api/core');
        
        // Load both current metrics and historical data
        const [metrics, historyData] = await Promise.all([
          invoke('kuboard_get_node_metrics', { nodeName: selectedNode.name }),
          invoke('kuboard_get_node_metrics_history', { nodeName: selectedNode.name, durationMinutes: historyDurationMinutes })
        ]);
        
        console.log('📊 Raw metrics response:', metrics);
        console.log('📊 Raw history response:', historyData);
        
        const metricsData = metrics as any;
        const historyArray = historyData as any[];
        
        // Update the selected node with metrics data
        selectedNode.cpu_usage_percent = metricsData.cpu?.usage_percent || 0;
        selectedNode.memory_usage_percent = metricsData.memory?.usage_percent || 0;
        selectedNode.disk_usage_percent = metricsData.disk?.usage_percent || 0;
        
        // Transform history data to MetricsDataPoint format
        resourceHistory = (historyArray || []).map((item: any) => ({
          timestamp: item.timestamp,
          cpu_usage_cores: parseFloat(item.cpu?.usage?.replace('m', '')) / 1000 || 0,
          memory_usage_bytes: parseFloat(item.memory?.usage?.replace('Gi', '')) * 1024 * 1024 * 1024 || 0,
          disk_usage_bytes: parseFloat(item.disk?.usage?.replace('Gi', '')) * 1024 * 1024 * 1024 || 0,
          cpu_usage_percent: item.cpu?.usage_percent || 0,
          memory_usage_percent: item.memory?.usage_percent || 0,
          disk_usage_percent: item.disk?.usage_percent || 0,
          is_mock_data: item.is_mock_data || false
        }));
        
        console.log('✅ Metrics and history loaded for node:', selectedNode.name);
      } catch (error) {
        console.error('❌ Error loading metrics for node:', error);
        metricsError = `Failed to load metrics for node ${selectedNode.name}`;
      } finally {
        metricsLoading = false;
      }
    }
    
    // Start metrics auto-refresh (cluster data auto-refresh is already running)
    startMetricsAutoRefresh();
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

  function toggleAutoRefresh() {
    autoRefreshEnabled = !autoRefreshEnabled;
    if (autoRefreshEnabled) {
      startAutoRefresh();
    } else {
      if (autoRefreshInterval) {
        clearInterval(autoRefreshInterval);
        autoRefreshInterval = null;
      }
    }
  }

  function toggleMetricsAutoRefresh() {
    metricsAutoRefreshEnabled = !metricsAutoRefreshEnabled;
    if (metricsAutoRefreshEnabled) {
      startMetricsAutoRefresh();
    } else {
      if (metricsRefreshInterval) {
        clearInterval(metricsRefreshInterval);
        metricsRefreshInterval = null;
      }
    }
  }

  function handleHistoryDurationChange(event: CustomEvent<number>) {
    historyDurationMinutes = event.detail;
    console.log('📈 History duration changed to:', historyDurationMinutes);
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

  // Auto-refresh functionality for cluster data
  function startAutoRefresh() {
    if (autoRefreshInterval) {
      clearInterval(autoRefreshInterval);
    }
    
    if (autoRefreshEnabled && currentContext && isTauriAvailable) {
      autoRefreshInterval = setInterval(async () => {
        try {
          console.log('🔄 Auto-refreshing cluster data...');
          lastRefreshTime = new Date().toLocaleTimeString();
          await loadClusterOverview();
        } catch (error) {
          console.error('❌ Auto-refresh error:', error);
        }
      }, refreshIntervalSeconds * 1000);
    }
  }

  // Auto-refresh functionality for metrics
  function startMetricsAutoRefresh() {
    if (metricsRefreshInterval) {
      clearInterval(metricsRefreshInterval);
    }
    
    if (metricsAutoRefreshEnabled && selectedNode && isTauriAvailable) {
      metricsRefreshInterval = setInterval(async () => {
        try {
          console.log('🔄 Auto-refreshing metrics...');
          lastMetricsRefreshTime = new Date().toLocaleTimeString();
          await fetchNodeMetrics(selectedNode!.name);
        } catch (error) {
          console.error('❌ Metrics auto-refresh error:', error);
        }
      }, metricsRefreshIntervalSeconds * 1000);
    }
  }

  async function checkMetricsAvailability() {
    if (!isTauriAvailable) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const availability = await invoke('kuboard_check_metrics_availability') as any;
      
      metricsServerAvailable = availability.available;
      isUsingMockData = availability.using_mock_data || false;
      
      if (metricsServerAvailable) {
        console.log('✅ Metrics server is available');
      } else {
        console.log('⚠️ Metrics server not available, using mock data');
        metricsError = 'Metrics server is not available or no data has been collected yet';
      }
    } catch (error) {
      console.error('❌ Error checking metrics availability:', error);
      metricsServerAvailable = false;
      metricsError = 'Failed to check metrics server availability';
    }
  }

  async function fetchNodeMetrics(nodeName: string) {
    if (!isTauriAvailable) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Load both current metrics and historical data
      const [metrics, historyData] = await Promise.all([
        invoke('kuboard_get_node_metrics', { nodeName }),
        invoke('kuboard_get_node_metrics_history', { nodeName, durationMinutes: historyDurationMinutes })
      ]);
      
      const metricsData = metrics as any;
      const historyArray = historyData as any[];
      
      // Update selected node with new metrics
      if (selectedNode && selectedNode.name === nodeName) {
        selectedNode.cpu_usage_percent = metricsData.cpu?.usage_percent || 0;
        selectedNode.memory_usage_percent = metricsData.memory?.usage_percent || 0;
        selectedNode.disk_usage_percent = metricsData.disk?.usage_percent || 0;
        selectedNode = selectedNode; // Trigger reactivity
        
        // Transform history data to MetricsDataPoint format
        resourceHistory = (historyArray || []).map((item: any) => ({
          timestamp: item.timestamp,
          cpu_usage_cores: parseFloat(item.cpu?.usage?.replace('m', '')) / 1000 || 0,
          memory_usage_bytes: parseFloat(item.memory?.usage?.replace('Gi', '')) * 1024 * 1024 * 1024 || 0,
          disk_usage_bytes: parseFloat(item.disk?.usage?.replace('Gi', '')) * 1024 * 1024 * 1024 || 0,
          cpu_usage_percent: item.cpu?.usage_percent || 0,
          memory_usage_percent: item.memory?.usage_percent || 0,
          disk_usage_percent: item.disk?.usage_percent || 0,
          is_mock_data: item.is_mock_data || false
        }));
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
      {currentContext}
      {selectedNode}
      {nodes}
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
      {autoRefreshEnabled}
      {lastRefreshTime}
      on:nodeSelect={handleNodeSelect}
      on:tabChange={handleTabChange}
      on:refreshIntervalChange={handleRefreshIntervalChange}
      on:historyDurationChange={handleHistoryDurationChange}
      on:debugConsoleToggle={handleDebugConsoleToggle}
      on:debugConsoleClear={handleDebugConsoleClear}
      on:metricsRetry={handleMetricsRetry}
      on:toggleAutoRefresh={toggleAutoRefresh}
    />
  {:else if currentContext}
    <div class="loading-cluster">
      <div class="loading-content">
        <h2>🔄 Loading Cluster Data</h2>
        <p>Fetching cluster overview...</p>
        <div class="loading-spinner">⏳</div>
      </div>
        </div>
      {/if}

  
  <!-- Theme Switcher (Dev Mode Only) -->
  <ThemeSwitcher />
</main>

<style>
  /* Import CSS variables */
  @import '../lib/styles/variables.css';

  main {
    width: 100vw;
    min-height: 100vh;
    margin: 0;
    padding: 5px;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background-color: var(--background-color);
    color: var(--text-primary);
    overflow-x: auto;
    box-sizing: border-box;
    position: relative;
  }

  /* Ensure no white edges */
  main::before {
    content: '';
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: var(--background-color);
    z-index: -1;
    pointer-events: none;
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

  .loading-cluster {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 400px;
    padding: var(--spacing-xl);
  }

  .loading-content {
    text-align: center;
    max-width: 400px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: var(--radius-lg);
    padding: var(--spacing-xl);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .loading-content h2 {
    color: var(--text-primary);
    margin: 0 0 var(--spacing-md) 0;
    font-size: 1.5rem;
  }

  .loading-content p {
    color: var(--text-secondary);
    margin: 0 0 var(--spacing-lg) 0;
    line-height: 1.6;
  }

  .loading-spinner {
    font-size: 2rem;
    animation: spin 2s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  /* Responsive Design */
  @media (max-width: 768px) {
    main {
      padding: 10px;
    }
  }
</style>