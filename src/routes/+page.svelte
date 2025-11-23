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
  let contextLoadingAbortController: AbortController | null = null;
  let activeLoadingContext: string | null = null; // Track which context we're currently loading
  const CONTEXT_SET_TIMEOUT = 3000; // 3 seconds for context set
  const CLUSTER_LOAD_TIMEOUT = 5000; // 5 seconds for cluster overview

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
      
      // Auto-dismiss success message after 2 seconds
      success = `Loaded ${contexts.length} contexts`;
      setTimeout(() => success = '', 2000);
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

    // Cancel any existing context loading
    if (contextLoadingAbortController) {
      console.log('🛑 Cancelling previous context load');
      contextLoadingAbortController.abort();
    }

    // Set the active loading context FIRST - this ensures we ignore results from old contexts
    // This must happen before any async operations
    activeLoadingContext = contextName;
    
    // Immediately update UI to show we're switching
    currentContext = contexts.find(ctx => ctx.name === contextName) || null;
    selectedContextName = contextName;
    loading = true;
    error = ""; // Clear any previous errors
    
    // Create new abort controller for this context switch
    contextLoadingAbortController = new AbortController();
    const signal = contextLoadingAbortController.signal;

    try {
      loading = true;
      error = "";
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Set context with timeout
      const contextPromise = invoke('kuboard_set_context', { contextName });
      const timeoutPromise = new Promise((_, reject) => {
        setTimeout(() => reject(new Error('Context connection timeout - unable to connect to cluster')), CONTEXT_SET_TIMEOUT);
      });
      
      // Race between context set and timeout
      await Promise.race([contextPromise, timeoutPromise]);
      
      // Check if operation was aborted or if context has changed
      if (signal.aborted || activeLoadingContext !== contextName) {
        console.log('🛑 Context switch aborted or context changed');
        return;
      }
      
      // Context already set above, just verify
      if (currentContext?.name !== contextName) {
        currentContext = contexts.find(ctx => ctx.name === contextName) || null;
      }
      
      // Quick connection test - try to get nodes first (fastest check)
      console.log('🔍 Testing connection...');
      if (signal.aborted || activeLoadingContext !== contextName) return;
      
      const testPromise = invoke('kuboard_get_nodes').catch(() => {
        throw new Error('Connection test failed - unable to reach cluster');
      });
      const testTimeoutPromise = new Promise((_, reject) => {
        setTimeout(() => reject(new Error('Connection test timeout')), 3000);
      });
      
      try {
        await Promise.race([testPromise, testTimeoutPromise]);
        console.log('✅ Connection test passed');
      } catch (testErr: any) {
        // Check if context changed during test
        if (signal.aborted || activeLoadingContext !== contextName) {
          console.log('🛑 Connection test aborted - context changed');
          return;
        }
        throw new Error(`Connection failed: ${testErr?.message || 'Unable to connect to cluster'}`);
      }
      
      // Check again before loading full overview
      if (signal.aborted || activeLoadingContext !== contextName) return;
      
      console.log('✅ Context set, loading cluster overview...');
      
      // Load cluster overview with timeout and abort support
      const overviewPromise = loadClusterOverview(signal);
      const overviewTimeoutPromise = new Promise((_, reject) => {
        setTimeout(() => reject(new Error('Cluster overview timeout')), CLUSTER_LOAD_TIMEOUT);
      });
      
      await Promise.race([overviewPromise, overviewTimeoutPromise]);
      
      if (signal.aborted) return;
      
      // Check if operation was aborted before proceeding
      if (signal.aborted) return;
      
      console.log('✅ Cluster overview loaded, checking metrics...');
      // Check metrics server availability (non-blocking)
      checkMetricsAvailability().catch(err => {
        if (!signal.aborted) {
          console.warn('⚠️ Metrics check failed:', err);
        }
      });
      
      // Check again before starting auto-refresh
      if (signal.aborted) return;
      
      // Start auto-refresh for cluster data
      startAutoRefresh();
      
      // Auto-dismiss success message after 3 seconds
      if (!signal.aborted) {
        success = `Switched to context: ${contextName}`;
        setTimeout(() => success = '', 3000);
        console.log('✅ Context switched to:', contextName);
      }
    } catch (err: any) {
      // Check if operation was aborted
      if (signal.aborted) {
        console.log('🛑 Context switch aborted');
        return;
      }
      
      // Only show error if this is still the active context
      if (activeLoadingContext === contextName) {
        const errorMessage = err?.message || String(err);
        error = `Failed to set context: ${errorMessage}`;
        console.error('❌ Error setting context:', err);
        // Don't auto-dismiss - let user retry manually
      } else {
        console.log('🛑 Error from old context, ignoring');
      }
    } finally {
      if (!signal.aborted) {
        loading = false;
      }
    }
  }

  async function loadClusterOverview(abortSignal?: AbortSignal) {
    if (!isTauriAvailable) return;
    
    // Get the context name we're loading for
    const loadingContext = activeLoadingContext;
    if (!loadingContext) {
      console.log('🛑 No active loading context');
      return;
    }
    
    // Check if operation was aborted or context changed
    if (abortSignal?.aborted || contextLoadingAbortController?.signal.aborted || activeLoadingContext !== loadingContext) {
      console.log('🛑 Cluster overview load aborted');
      return;
    }
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      clusterOverview = await invoke('kuboard_get_cluster_overview');
      
      // Check if operation was aborted or context changed after async call
      if (abortSignal?.aborted || contextLoadingAbortController?.signal.aborted || activeLoadingContext !== loadingContext) {
        console.log('🛑 Cluster overview load aborted after API call - context changed');
        return;
      }
      
      console.log('✅ Cluster overview loaded:', clusterOverview);
      
      // Load resource details (non-blocking, don't wait for it, but pass abort signal)
      console.log('🔄 Loading resource details...');
      loadResourceDetails(abortSignal).catch(err => {
        if (!abortSignal?.aborted && !contextLoadingAbortController?.signal.aborted && activeLoadingContext === loadingContext) {
          console.warn('⚠️ Resource details loading failed:', err);
        }
      });
      
      // Success message only if still the active context
      if (!abortSignal?.aborted && !contextLoadingAbortController?.signal.aborted && activeLoadingContext === loadingContext) {
        success = "Cluster overview loaded successfully";
        setTimeout(() => success = '', 2000);
      }
    } catch (err) {
      // Check if operation was aborted or context changed
      if (abortSignal?.aborted || contextLoadingAbortController?.signal.aborted || activeLoadingContext !== loadingContext) {
        console.log('🛑 Cluster overview load aborted in catch - context changed');
        return;
      }
      // Only show error if this is still the active context
      if (activeLoadingContext === loadingContext) {
        error = `Failed to load cluster overview: ${err}`;
        console.error('❌ Error loading cluster overview:', err);
      }
    }
  }

  async function loadResourceDetails(abortSignal?: AbortSignal) {
    if (!isTauriAvailable) {
      console.log('⚠️ Tauri not available, skipping resource details');
      return;
    }
    
    // Get the context name we're loading for
    const loadingContext = activeLoadingContext;
    if (!loadingContext) {
      console.log('🛑 No active loading context');
      return;
    }
    
    // Check if operation was aborted before starting
    if (abortSignal?.aborted || contextLoadingAbortController?.signal.aborted || activeLoadingContext !== loadingContext) {
      console.log('🛑 Resource details load aborted before start');
      return;
    }
    
    try {
      console.log('🔄 Starting resource details loading...');
      resourceLoading = true;
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Check again after async import
      if (abortSignal?.aborted || contextLoadingAbortController?.signal.aborted || activeLoadingContext !== loadingContext) {
        console.log('🛑 Resource details load aborted after import');
        return;
      }
      
      console.log('🔄 Calling kuboard_get_nodes...');
      // Load all resource types in parallel, but check abort signal after each
      const promises = [
        invoke('kuboard_get_nodes'),
        invoke('kuboard_get_namespaces'),
        invoke('kuboard_get_pods'),
        invoke('kuboard_get_deployments')
      ];
      
      const results = await Promise.allSettled(promises);
      
      // Check if operation was aborted or context changed after API calls
      if (abortSignal?.aborted || contextLoadingAbortController?.signal.aborted || activeLoadingContext !== loadingContext) {
        console.log('🛑 Resource details load aborted after API calls - context changed');
        return;
      }
      
      // Process results only if not aborted and still the active context
      const nodesData = results[0].status === 'fulfilled' ? results[0].value : null;
      const namespacesData = results[1].status === 'fulfilled' ? results[1].value : null;
      const podsData = results[2].status === 'fulfilled' ? results[2].value : null;
      const deploymentsData = results[3].status === 'fulfilled' ? results[3].value : null;
      
      // Check one more time before updating state
      if (abortSignal?.aborted || contextLoadingAbortController?.signal.aborted || activeLoadingContext !== loadingContext) {
        console.log('🛑 Resource details load aborted before state update - context changed');
        return;
      }
      
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
      // Only log error if not aborted and still the active context
      if (!abortSignal?.aborted && !contextLoadingAbortController?.signal.aborted && activeLoadingContext === loadingContext) {
        console.error('❌ Error loading resource details:', err);
        error = `Failed to load resource details: ${err}`;
      } else {
        console.log('🛑 Resource details load aborted in catch - context changed');
      }
    } finally {
      // Only update loading state if not aborted and still the active context
      if (!abortSignal?.aborted && !contextLoadingAbortController?.signal.aborted && activeLoadingContext === loadingContext) {
        resourceLoading = false;
      }
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
    
    // Auto-dismiss success message
    success = "Demo data loaded successfully";
    setTimeout(() => success = '', 2000);
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

  <!-- Error Message (shown as toast) -->
  {#if error}
    <div class="error-message toast">
      <div class="error-content">
        <strong>Error:</strong> {error}
      </div>
      <div class="error-actions">
        {#if error.includes('Failed to set context') || error.includes('Failed to load cluster overview')}
          <button class="retry-button" onclick={() => {
            if (selectedContextName) {
              error = '';
              setContext(selectedContextName);
            }
          }} title="Retry">
            🔄 Retry
          </button>
        {/if}
        <button class="toast-close" onclick={() => error = ''} title="Dismiss">×</button>
      </div>
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
    position: fixed;
    top: 20px;
    right: 20px;
    max-width: 450px;
    color: var(--error-color);
    padding: 12px 16px;
    border: 1px solid var(--error-color);
    border-radius: var(--radius-md);
    background: rgba(239, 68, 68, 0.15);
    backdrop-filter: blur(10px);
    z-index: 1000;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
    animation: slideInRight 0.3s ease-out;
  }

  .error-message.toast {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .error-content {
    flex: 1;
  }

  .error-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    justify-content: flex-end;
  }

  .retry-button {
    background: var(--error-color);
    border: 1px solid var(--error-color);
    border-radius: var(--radius-sm);
    color: white;
    cursor: pointer;
    font-size: 0.85rem;
    padding: 4px 12px;
    transition: all 0.2s;
    font-weight: 500;
  }

  .retry-button:hover {
    background: #dc2626;
    border-color: #dc2626;
    transform: translateY(-1px);
  }

  .toast-close {
    background: transparent;
    border: none;
    color: var(--error-color);
    font-size: 1.5em;
    cursor: pointer;
    padding: 0;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0.7;
    transition: opacity 0.2s;
  }

  .toast-close:hover {
    opacity: 1;
  }

  @keyframes slideInRight {
    from {
      transform: translateX(100%);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
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