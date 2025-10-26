<!-- Optimized Kuboard Main Page -->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  
  // Import our modular components
  import Header from '$lib/components/Header.svelte';
  import ClusterOverviewComponent from '$lib/components/ClusterOverview.svelte';
  import ResourceOverview from '$lib/components/ResourceOverview.svelte';
  import MetricsGraph from '$lib/components/MetricsGraph.svelte';
  
  // Import performance utilities
  import { 
    optimizedContextSwitch, 
    loadAllResourcesOptimized, 
    preloadCriticalData,
    createDebouncedRefresh,
    startPerformanceMonitoring,
    clusterCache,
    invalidateCache
  } from '$lib/utils/performance.js';
  
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

  // Performance monitoring
  let performanceMonitor = startPerformanceMonitoring();

  // Check if we're running in Tauri environment
  onMount(async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      isTauriAvailable = true;
      console.log('✅ Tauri environment detected');
      
      // Use optimized preloading
      await loadContextsOptimized();
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

  // Optimized context loading with preloading
  async function loadContextsOptimized() {
    if (!isTauriAvailable) return;
    
    try {
      loading = true;
      error = "";
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Use optimized preloading
      const response = await preloadCriticalData(invoke);
      contexts = response.contexts || [];
      
      if (response.current_context) {
        currentContext = contexts.find(ctx => ctx.name === response.current_context) || null;
        selectedContextName = response.current_context;
        
        // Get preloaded overview from cache
        clusterCache.subscribe(cache => {
          if (cache.overview) {
            clusterOverview = cache.overview;
          }
        })();
      }
      
      success = `Loaded ${contexts.length} contexts with preloading`;
      console.log('✅ Contexts loaded with optimization:', contexts);
    } catch (err) {
      error = `Failed to load contexts: ${err}`;
      console.error('❌ Error loading contexts:', err);
    } finally {
      loading = false;
    }
  }

  // Optimized context switching
  async function setContextOptimized(contextName: string) {
    if (!isTauriAvailable) return;
    
    try {
      loading = true;
      error = "";
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Use optimized context switching
      const overview = await optimizedContextSwitch(contextName, invoke);
      
      currentContext = contexts.find(ctx => ctx.name === contextName) || null;
      selectedContextName = contextName;
      clusterOverview = overview;
      
      // Load resource details in background
      loadResourceDetailsOptimized();
      
      success = `Switched to context: ${contextName} (optimized)`;
      console.log('✅ Context switched with optimization:', contextName);
    } catch (err) {
      error = `Failed to set context: ${err}`;
      console.error('❌ Error setting context:', err);
    } finally {
      loading = false;
    }
  }

  // Optimized resource loading
  async function loadResourceDetailsOptimized() {
    if (!isTauriAvailable) return;
    
    try {
      resourceLoading = true;
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Use optimized batch loading
      const resources = await loadAllResourcesOptimized(invoke);
      
      nodes = resources.nodes || [];
      namespaces = resources.namespaces || [];
      pods = resources.pods || [];
      deployments = resources.deployments || [];
      
      console.log('✅ Resources loaded with optimization');
    } catch (err) {
      console.error('❌ Error loading resources:', err);
    } finally {
      resourceLoading = false;
    }
  }

  // Debounced refresh to prevent excessive API calls
  const debouncedRefresh = createDebouncedRefresh(() => {
    if (currentContext) {
      loadResourceDetailsOptimized();
    }
  }, 1000);

  // Event handlers
  function handleContextChange(event: CustomEvent<string>) {
    setContextOptimized(event.detail);
  }

  function handleRefresh() {
    debouncedRefresh();
  }

  function handleNodeSelect(event: CustomEvent<NodeDetails>) {
    selectedNode = event.detail;
  }

  function handleTabChange(event: CustomEvent<ResourceTab>) {
    activeResourceTab = event.detail;
  }

  function handleRefreshIntervalChange(event: CustomEvent<number>) {
    refreshIntervalSeconds = event.detail;
  }

  function handleHistoryDurationChange(event: CustomEvent<number>) {
    historyDurationMinutes = event.detail;
  }

  function handlePanelToggle(event: CustomEvent<ExpandedPanel>) {
    expandedPanel = event.detail;
  }

  function handleResourceSelect(event: CustomEvent<{type: string, resource: any}>) {
    console.log('Resource selected:', event.detail);
  }

  function handleDebugConsoleToggle() {
    showDebugConsole = !showDebugConsole;
  }

  function handleDebugConsoleClear() {
    debugConsole = '';
  }

  function handleMetricsRetry() {
    // Retry metrics loading
    console.log('Retrying metrics...');
  }

  // Demo data for development
  function showDemoData() {
    contexts = [
      { name: 'minikube', cluster: 'minikube', user: 'minikube', namespace: 'default', is_current: true },
      { name: 'docker-desktop', cluster: 'docker-desktop', user: 'docker-desktop', namespace: 'default', is_current: false }
    ];
    currentContext = contexts[0];
    selectedContextName = 'minikube';
    
    clusterOverview = {
      cluster_info: {
        name: 'minikube',
        server: 'https://kubernetes.docker.internal:6443',
        version: '1.28.0'
      },
      node_count: 1,
      namespace_count: 4,
      pod_count: 8,
      deployment_count: 3,
      kubernetes_version: '1.28.0',
      cluster_metrics: null
    };
  }

  // Performance monitoring
  $: if (clusterOverview) {
    const duration = performanceMonitor.end();
    console.log(`⏱️ Page loaded in ${duration.toFixed(2)}ms`);
  }
</script>

<!-- Optimized Main Page Layout -->
<main class="main-container">
  <!-- Header with optimized context switching -->
  <Header 
    {contexts} 
    {currentContext} 
    {loading} 
    {isTauriAvailable}
    on:contextChange={handleContextChange}
    on:refresh={handleRefresh}
  />

  <!-- Cluster Overview with optimized loading -->
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
    {isUsingMockData}
    on:nodeSelect={handleNodeSelect}
    on:switchResourceTab={handleTabChange}
    on:updateRefreshInterval={handleRefreshIntervalChange}
    on:updateHistoryDuration={handleHistoryDurationChange}
    on:toggleDebugConsole={handleDebugConsoleToggle}
    on:clearDebugConsole={handleDebugConsoleClear}
    on:retryMetrics={handleMetricsRetry}
  />

  <!-- Resource Overview with optimized loading -->
  <ResourceOverview 
    {clusterOverview}
    {nodes}
    {namespaces}
    {pods}
    {deployments}
    {resourceLoading}
    {expandedPanel}
    on:panelToggle={handlePanelToggle}
    on:resourceSelect={handleResourceSelect}
  />
</main>

<style>
  .main-container {
    min-height: 100vh;
    background: var(--bg-primary);
    color: var(--text-primary);
  }
</style>
