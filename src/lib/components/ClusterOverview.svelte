<!-- Kuboard Cluster Overview Component -->
<script lang="ts">
  import type { ClusterOverview, NodeDetails, ResourceTab } from '../types/index.js';
  import { createEventDispatcher } from 'svelte';
  import MetricsGraph from './MetricsGraph.svelte';

  // Props
  export let clusterOverview: ClusterOverview | null = null;
  export let currentContext: any = null;
  export let selectedNode: NodeDetails | null = null;
  export let nodes: any[] = [];
  export let metricsLoading: boolean = false;
  
  // Debug logging - only show when nodes actually change and are not empty
  $: if (nodes && nodes.length > 0) {
    console.log('🔍 ClusterOverview nodes prop changed:', nodes.length, nodes);
  }
  
  // Force reactivity for nodes display
  $: nodesLength = nodes?.length || 0;
  $: nodesData = nodes || [];
  $: isNodesLoading = nodesLength === 0 && currentContext;
  export let metricsError: string | null = null;
  export let resourceHistory: any[] = [];
  export let activeResourceTab: ResourceTab = 'cpu';
  export let refreshIntervalSeconds: number = 10;
  export let historyDurationMinutes: number = 30;
  export let debugInfo: string = '';
  export let lastUpdateTime: string = '';
  export let debugConsole: string = '';
  export let showDebugConsole: boolean = false;
  export let autoRefreshEnabled: boolean = true;
  export let lastRefreshTime: string = '';
  export let resourceLoading: boolean = false;

  // Events
  const dispatch = createEventDispatcher();

  // Event handlers
  function selectNode(node: any) {
    // Convert the raw node data to NodeDetails format while preserving the original data
    const nodeDetails: NodeDetails = {
      name: node.metadata?.name || 'Unknown',
      status: node.status?.conditions?.find(c => c.type === 'Ready')?.status || 'Unknown',
      max_cpu_cores: 0, // Will be populated by metrics
      max_memory_bytes: 0, // Will be populated by metrics
      allocatable_cpu_cores: 0,
      allocatable_memory_bytes: 0,
      cpu_usage_percent: 0,
      memory_usage_percent: 0,
      disk_usage_percent: 0,
      conditions: node.status?.conditions?.map((c: any) => `${c.type}: ${c.status}`) || [],
      // Extract additional node information from the original data
      os: node.status?.nodeInfo?.operatingSystem || 'Unknown',
      kernel_version: node.status?.nodeInfo?.kernelVersion || 'Unknown',
      kubelet_version: node.status?.nodeInfo?.kubeletVersion || 'Unknown',
      container_runtime: node.status?.nodeInfo?.containerRuntimeVersion || 'Unknown',
      disk_capacity: 0, // Will be populated if available
      disk_allocatable: 0, // Will be populated if available
      labels: node.metadata?.labels || {},
      annotations: node.metadata?.annotations || {},
      taints: node.spec?.taints?.map((t: any) => `${t.key}=${t.value}:${t.effect}`) || [],
      metrics_available: false, // Will be updated by metrics loading
      metrics_error: undefined,
      // Preserve the original node data for the details panel
      originalData: node
    };
    dispatch('nodeSelect', nodeDetails);
  }

  function switchResourceTab(tab: ResourceTab) {
    dispatch('tabChange', tab);
  }

  function updateRefreshInterval(seconds: number) {
    dispatch('refreshIntervalChange', seconds);
  }

  function updateHistoryDuration(minutes: number) {
    dispatch('historyDurationChange', minutes);
  }

  function toggleDebugConsole() {
    dispatch('debugConsoleToggle');
  }

  function retryMetrics() {
    if (selectedNode) {
      dispatch('metricsRetry', selectedNode.name);
    }
  }

  function toggleAutoRefresh() {
    dispatch('toggleAutoRefresh');
  }

  // Helper functions
  function formatMemory(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  function formatCPU(cores: number): string {
    return cores.toFixed(1) + ' cores';
  }

  function getStatusClass(status: string): string {
    return status.toLowerCase().replace(/\s+/g, '-');
  }

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text);
  }
</script>

{#if !currentContext}
  <div class="no-context-message">
    <div class="message-content">
      <h2>🔍 Select a Kubernetes Context</h2>
      <p>Choose a context from the dropdown above to view your cluster information.</p>
      <div class="context-hint">
        <p>💡 <strong>Tip:</strong> Make sure your kubeconfig is properly configured and accessible.</p>
      </div>
    </div>
  </div>
{:else if clusterOverview}
  <div class="cluster-overview">
    <div class="cluster-header">
      <h2>🏗️ Cluster Overview</h2>
      <div class="cluster-info-card">
        <div class="cluster-basic-info">
          <h3>{clusterOverview.cluster_info.name}</h3>
          <p class="cluster-server">{clusterOverview.cluster_info.server}</p>
          {#if clusterOverview.kubernetes_version}
            <span class="k8s-version">Kubernetes {clusterOverview.kubernetes_version}</span>
          {/if}
        </div>
      </div>
    </div>

    <div class="nodes-layout">
      <!-- Left Panel: Node List -->
      <div class="nodes-panel">
        <div class="panel-header">
          <h4>🖥️ Nodes ({nodesLength})</h4>
        <div class="node-summary">
          <span class="summary-item">
            <strong>{nodesLength}</strong> Total
          </span>
        </div>
        </div>
        
        <div class="nodes-list">
          {#if isNodesLoading}
            <div class="loading-nodes">
              <p>🔄 Loading nodes...</p>
            </div>
          {:else if nodesData.length > 0}
            {#each nodesData as node}
              <div 
                class="node-list-item"
                class:selected={selectedNode?.name === node.name}
                onclick={() => selectNode(node)}
                role="button"
                tabindex="0"
                onkeydown={(e) => e.key === 'Enter' || e.key === ' ' ? selectNode(node) : null}
              >
                <div class="node-item-header">
                  <h5 class="node-item-name">{node.metadata?.name || 'Unknown'}</h5>
                  <span class="node-item-status status-{getStatusClass(node.status?.conditions?.find(c => c.type === 'Ready')?.status || 'Unknown')}">
                    {node.status?.conditions?.find(c => c.type === 'Ready')?.status || 'Unknown'}
                  </span>
                </div>
                <div class="node-item-resources">
                  <span class="node-item-resource">Kubelet: {node.status?.nodeInfo?.kubeletVersion || 'Unknown'}</span>
                  <span class="node-item-resource">OS: {node.status?.nodeInfo?.operatingSystem || 'Unknown'}</span>
                </div>
                <div class="node-item-usage">
                  <span class="usage-text">Architecture: {node.status?.nodeInfo?.architecture || 'Unknown'}</span>
                  <span class="usage-text">Container Runtime: {node.status?.nodeInfo?.containerRuntimeVersion || 'Unknown'}</span>
                </div>
              </div>
            {/each}
          {:else}
            <div class="no-node-selected">
              <p>No nodes available</p>
            </div>
          {/if}
        </div>
      </div>

      <!-- Right Panel: Selected Node Details -->
      <div class="node-details-panel">
        {#if selectedNode}
          <div class="node-details">
            <div class="node-details-header">
              <h4>🖥️ {selectedNode.name}</h4>
              <span class="node-details-status status-{selectedNode.status.toLowerCase()}">{selectedNode.status}</span>
            </div>
            
            <div class="node-details-content">
              <!-- Metrics Server Warning -->
              {#if !selectedNode.metrics_available && selectedNode.metrics_error}
                <div class="metrics-warning">
                  <div class="warning-icon">⚠️</div>
                  <div class="warning-content">
                    <h6>Metrics Server Not Available</h6>
                    <p>{selectedNode.metrics_error}</p>
                  </div>
                </div>
              {/if}

              <!-- System Information -->
              <div class="details-section">
                <h5>🖥️ System Information</h5>
                <div class="info-grid">
                  {#if selectedNode.os}
                    <div class="info-item">
                      <span class="info-label">Operating System:</span>
                      <div class="info-value-container">
                        <span class="info-value" title={selectedNode.os}>{selectedNode.os}</span>
                        <button 
                          class="copy-button" 
                          onclick={() => copyToClipboard(selectedNode.os)}
                          title="Copy to clipboard"
                        >
                          📋
                        </button>
                      </div>
                    </div>
                  {/if}
                  {#if selectedNode.kernel_version}
                    <div class="info-item">
                      <span class="info-label">Kernel Version:</span>
                      <div class="info-value-container">
                        <span class="info-value" title={selectedNode.kernel_version}>{selectedNode.kernel_version}</span>
                        <button 
                          class="copy-button" 
                          onclick={() => copyToClipboard(selectedNode.kernel_version)}
                          title="Copy to clipboard"
                        >
                          📋
                        </button>
                      </div>
                    </div>
                  {/if}
                  {#if selectedNode.kubelet_version}
                    <div class="info-item">
                      <span class="info-label">Kubelet Version:</span>
                      <div class="info-value-container">
                        <span class="info-value" title={selectedNode.kubelet_version}>{selectedNode.kubelet_version}</span>
                        <button 
                          class="copy-button" 
                          onclick={() => copyToClipboard(selectedNode.kubelet_version)}
                          title="Copy to clipboard"
                        >
                          📋
                        </button>
                      </div>
                    </div>
                  {/if}
                  {#if selectedNode.container_runtime}
                    <div class="info-item">
                      <span class="info-label">Container Runtime:</span>
                      <div class="info-value-container">
                        <span class="info-value" title={selectedNode.container_runtime}>{selectedNode.container_runtime}</span>
                        <button 
                          class="copy-button" 
                          onclick={() => copyToClipboard(selectedNode.container_runtime)}
                          title="Copy to clipboard"
                        >
                          📋
                        </button>
                      </div>
                    </div>
                  {/if}
                </div>
              </div>

              <!-- Resource Specifications -->
              <div class="details-section">
                <h5>💻 Resource Specifications</h5>
                <div class="resource-grid">
                  <div class="resource-item">
                    <span class="resource-label">Max CPU:</span>
                    <div class="resource-value-container">
                      <span class="resource-value" title={formatCPU(selectedNode.max_cpu_cores)}>{formatCPU(selectedNode.max_cpu_cores)}</span>
                      <button 
                        class="copy-button" 
                        onclick={() => copyToClipboard(formatCPU(selectedNode.max_cpu_cores))}
                        title="Copy to clipboard"
                      >
                        📋
                      </button>
                    </div>
                  </div>
                  <div class="resource-item">
                    <span class="resource-label">Max Memory:</span>
                    <div class="resource-value-container">
                      <span class="resource-value" title={formatMemory(selectedNode.max_memory_bytes)}>{formatMemory(selectedNode.max_memory_bytes)}</span>
                      <button 
                        class="copy-button" 
                        onclick={() => copyToClipboard(formatMemory(selectedNode.max_memory_bytes))}
                        title="Copy to clipboard"
                      >
                        📋
                      </button>
                    </div>
                  </div>
                  <div class="resource-item">
                    <span class="resource-label">Allocatable CPU:</span>
                    <div class="resource-value-container">
                      <span class="resource-value" title={formatCPU(selectedNode.allocatable_cpu_cores)}>{formatCPU(selectedNode.allocatable_cpu_cores)}</span>
                      <button 
                        class="copy-button" 
                        onclick={() => copyToClipboard(formatCPU(selectedNode.allocatable_cpu_cores))}
                        title="Copy to clipboard"
                      >
                        📋
                      </button>
                    </div>
                  </div>
                  <div class="resource-item">
                    <span class="resource-label">Allocatable Memory:</span>
                    <div class="resource-value-container">
                      <span class="resource-value" title={formatMemory(selectedNode.allocatable_memory_bytes)}>{formatMemory(selectedNode.allocatable_memory_bytes)}</span>
                      <button 
                        class="copy-button" 
                        onclick={() => copyToClipboard(formatMemory(selectedNode.allocatable_memory_bytes))}
                        title="Copy to clipboard"
                      >
                        📋
                      </button>
                    </div>
                  </div>
                  {#if selectedNode.disk_capacity}
                    <div class="resource-item">
                      <span class="resource-label">Disk Capacity:</span>
                      <div class="resource-value-container">
                        <span class="resource-value" title={formatMemory(selectedNode.disk_capacity)}>{formatMemory(selectedNode.disk_capacity)}</span>
                        <button 
                          class="copy-button" 
                          onclick={() => copyToClipboard(formatMemory(selectedNode.disk_capacity))}
                          title="Copy to clipboard"
                        >
                          📋
                        </button>
                      </div>
                    </div>
                  {/if}
                  {#if selectedNode.disk_allocatable}
                    <div class="resource-item">
                      <span class="resource-label">Allocatable Disk:</span>
                      <div class="resource-value-container">
                        <span class="resource-value" title={formatMemory(selectedNode.disk_allocatable)}>{formatMemory(selectedNode.disk_allocatable)}</span>
                        <button 
                          class="copy-button" 
                          onclick={() => copyToClipboard(formatMemory(selectedNode.disk_allocatable))}
                          title="Copy to clipboard"
                        >
                          📋
                        </button>
                      </div>
                    </div>
                  {/if}
                </div>
              </div>

              <!-- Resource Usage Graph -->
              <div class="details-section">
                <h5>📈 Resource Usage (Last {historyDurationMinutes} Minutes)</h5>
                
                <!-- Resource Tabs -->
                <div class="resource-tabs">
                  <button 
                    class="resource-tab"
                    class:active={activeResourceTab === 'cpu'}
                    onclick={() => switchResourceTab('cpu')}
                  >
                    CPU
                  </button>
                  <button 
                    class="resource-tab"
                    class:active={activeResourceTab === 'memory'}
                    onclick={() => switchResourceTab('memory')}
                  >
                    Memory
                  </button>
                  <button 
                    class="resource-tab"
                    class:active={activeResourceTab === 'disk'}
                    onclick={() => switchResourceTab('disk')}
                  >
                    Disk
                  </button>
                </div>

                <!-- Resource Graph -->
                <div class="resource-graph">
                  <div class="graph-header">
                    <span class="graph-title">
                      {activeResourceTab === 'cpu' ? 'CPU Usage' : 
                       activeResourceTab === 'memory' ? 'Memory Usage' : 'Disk Usage'}
                    </span>
                    <div class="graph-controls">
                      <div class="refresh-control">
                        <label for="refresh-interval">Refresh:</label>
                        <select 
                          id="refresh-interval" 
                          bind:value={refreshIntervalSeconds}
                          onchange={() => updateRefreshInterval(refreshIntervalSeconds)}
                          class="refresh-select"
                        >
                          <option value={5}>5s</option>
                          <option value={10}>10s</option>
                          <option value={15}>15s</option>
                          <option value={30}>30s</option>
                          <option value={60}>1m</option>
                        </select>
                      </div>
                      <div class="auto-refresh-control">
                        <button 
                          class="auto-refresh-toggle"
                          class:enabled={autoRefreshEnabled}
                          onclick={toggleAutoRefresh}
                          title={autoRefreshEnabled ? 'Disable auto-refresh' : 'Enable auto-refresh'}
                        >
                          {autoRefreshEnabled ? '⏸️ Pause' : '▶️ Resume'}
                        </button>
                        {#if lastRefreshTime}
                          <span class="last-refresh">Last: {lastRefreshTime}</span>
                        {/if}
                      </div>
                      <div class="history-control">
                        <label for="history-duration">History:</label>
                        <select 
                          id="history-duration" 
                          bind:value={historyDurationMinutes}
                          onchange={() => updateHistoryDuration(historyDurationMinutes)}
                          class="history-select"
                        >
                          <option value={30}>30 min</option>
                          <option value={60}>1 hour</option>
                          <option value={120}>2 hours</option>
                          <option value={240}>4 hours</option>
                          <option value={480}>8 hours</option>
                          <option value={720}>12 hours</option>
                        </select>
                      </div>
                      <div class="current-value-container">
                        <span class="current-value">
                          {activeResourceTab === 'cpu' ? selectedNode.cpu_usage_percent.toFixed(1) :
                           activeResourceTab === 'memory' ? selectedNode.memory_usage_percent.toFixed(1) :
                           selectedNode.disk_usage_percent.toFixed(1)}%
                        </span>
                        {#if metricsLoading}
                          <span class="loading-indicator">🔄</span>
                        {/if}
                      </div>
                    </div>
                    
                    <!-- Debug Info Panel -->
                    {#if debugInfo}
                      <div class="debug-panel">
                        <div class="debug-header">
                          <span class="debug-title">🔍 Debug Info</span>
                          <span class="debug-time">{lastUpdateTime}</span>
                        </div>
                        <div class="debug-content">
                          {debugInfo}
                        </div>
                      </div>
                    {/if}
                    
                    <!-- Debug Console Button -->
                    <div class="debug-console-controls">
                      <button 
                        class="debug-console-button"
                        onclick={toggleDebugConsole}
                      >
                        {showDebugConsole ? '🔽 Hide Debug Console' : '🔼 Show Debug Console'}
                      </button>
                    </div>
                    
                    <!-- Debug Console Panel -->
                    {#if showDebugConsole}
                      <div class="debug-console-panel">
                        <div class="debug-console-header">
                          <span class="debug-console-title">🐛 Debug Console</span>
                          <button 
                            class="debug-console-clear"
                            onclick={() => dispatch('debugConsoleClear')}
                          >
                            Clear
                          </button>
                        </div>
                        <div class="debug-console-content">
                          {debugConsole || 'No debug messages yet...'}
                        </div>
                      </div>
                    {/if}
                  </div>
                    
                  {#if metricsError}
                    <div class="metrics-error">
                      <div class="error-icon">⚠️</div>
                      <div class="error-content">
                        <h6>Metrics Server Error</h6>
                        <p>{metricsError}</p>
                        <button 
                          class="retry-button" 
                          onclick={retryMetrics}
                        >
                          Retry
                        </button>
                      </div>
                    </div>
                  {:else if resourceHistory.length === 0}
                    <div class="no-data">
                      <div class="no-data-content">
                        <h6>No Metrics Data</h6>
                        <p>Metrics server is not available or no data has been collected yet</p>
                        {#if metricsError}
                          <p class="error-details">Error: {metricsError}</p>
                        {/if}
                      </div>
                    </div>
                  {:else}
                    <MetricsGraph 
                      data={resourceHistory}
                      type={activeResourceTab}
                      duration={historyDurationMinutes}
                      autoRefresh={true}
                      loading={metricsLoading}
                      error={metricsError}
                    />
                  {/if}
                </div>
              </div>

              <!-- Labels -->
              {#if Object.keys(selectedNode.labels).length > 0}
                <div class="details-section">
                  <h5>🏷️ Labels</h5>
                  <div class="labels-list">
                    {#each Object.entries(selectedNode.labels) as [key, value]}
                      <div class="label-item">
                        <span class="label-key">{key}:</span>
                        <div class="label-value-container">
                          <span class="label-value" title={`${key}: ${value}`}>{value}</span>
                          <button 
                            class="copy-button" 
                            onclick={() => copyToClipboard(`${key}: ${value}`)}
                            title="Copy to clipboard"
                          >
                            📋
                          </button>
                        </div>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}

              <!-- Annotations -->
              {#if Object.keys(selectedNode.annotations).length > 0}
                <div class="details-section">
                  <h5>📝 Annotations</h5>
                  <div class="annotations-list">
                    {#each Object.entries(selectedNode.annotations) as [key, value]}
                      <div class="annotation-item">
                        <span class="annotation-key">{key}:</span>
                        <div class="annotation-value-container">
                          <span class="annotation-value" title={`${key}: ${value}`}>{value}</span>
                          <button 
                            class="copy-button" 
                            onclick={() => copyToClipboard(`${key}: ${value}`)}
                            title="Copy to clipboard"
                          >
                            📋
                          </button>
                        </div>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}

              <!-- Taints -->
              {#if selectedNode.taints.length > 0}
                <div class="details-section">
                  <h5>🚫 Taints</h5>
                  <div class="taints-list">
                    {#each selectedNode.taints as taint}
                      <span class="taint-badge">{taint}</span>
                    {/each}
                  </div>
                </div>
              {/if}
            </div>
          </div>
        {:else}
          <div class="no-selection">
            <div class="no-selection-content">
              <h4>Select a Node</h4>
              <p>Choose a node from the list to view its details</p>
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  /* Import CSS variables */
  @import '../styles/variables.css';

  /* Modern Cluster Overview Styles */
  .no-context-message {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 400px;
    padding: var(--spacing-xl);
  }

  .no-node-selected, .loading-nodes {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 200px;
    padding: var(--spacing-lg);
    color: var(--text-secondary);
    font-style: italic;
  }

  .loading-nodes {
    color: var(--accent-color);
    font-weight: 500;
  }

  .message-content {
    text-align: center;
    max-width: 500px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: var(--radius-lg);
    padding: var(--spacing-xl);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .message-content h2 {
    color: var(--text-primary);
    margin: 0 0 var(--spacing-md) 0;
    font-size: 1.5rem;
  }

  .message-content p {
    color: var(--text-secondary);
    margin: 0 0 var(--spacing-md) 0;
    line-height: 1.6;
  }

  .context-hint {
    background: rgba(59, 130, 246, 0.1);
    border: 1px solid rgba(59, 130, 246, 0.3);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    margin-top: var(--spacing-lg);
  }

  .context-hint p {
    margin: 0;
    color: var(--text-primary);
    font-size: 0.9rem;
  }

  .cluster-overview {
    background: linear-gradient(135deg, var(--primary-color) 0%, var(--accent-color) 100%);
    border-radius: var(--radius-xl);
    padding: var(--spacing-lg);
    margin: 20px 0;
    color: white;
    box-shadow: var(--shadow-xl);
    border: 1px solid var(--primary-color);
  }

  .cluster-header {
    text-align: center;
    margin-bottom: var(--spacing-xl);
  }

  .cluster-header h2 {
    font-size: 2em;
    margin-bottom: var(--spacing-md);
    text-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
  }

  .cluster-info-card {
    background: rgba(255, 255, 255, 0.1);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.2);
  }

  .cluster-basic-info h3 {
    font-size: 1.5em;
    margin-bottom: var(--spacing-sm);
    color: white;
  }

  .cluster-server {
    color: rgba(255, 255, 255, 0.8);
    font-size: 0.9em;
    margin-bottom: var(--spacing-sm);
  }

  .k8s-version {
    background: rgba(255, 255, 255, 0.2);
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    font-size: 0.8em;
    font-weight: 600;
  }

  /* Nodes Layout */
  .nodes-layout {
    display: grid;
    grid-template-columns: 1fr 2fr;
    gap: var(--spacing-lg);
    margin-top: var(--spacing-lg);
  }

  /* Left Panel: Node List */
  .nodes-panel {
    background: rgba(255, 255, 255, 0.05);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .panel-header {
    margin-bottom: var(--spacing-lg);
  }

  .panel-header h4 {
    margin: 0 0 var(--spacing-sm) 0;
    color: white;
    font-size: 1.2em;
  }

  .node-summary {
    display: flex;
    gap: var(--spacing-md);
  }

  .summary-item {
    color: rgba(255, 255, 255, 0.8);
    font-size: 0.9em;
  }

  .nodes-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .node-list-item {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    cursor: pointer;
    transition: var(--transition-normal);
  }

  .node-list-item:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: var(--primary-color);
    transform: translateY(-1px);
  }

  .node-list-item.selected {
    background: rgba(16, 185, 129, 0.2);
    border-color: var(--primary-color);
    box-shadow: 0 4px 12px rgba(16, 185, 129, 0.3);
  }

  .node-item-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-sm);
  }

  .node-item-name {
    margin: 0;
    color: white;
    font-size: 1.1em;
    font-weight: 600;
  }

  .node-item-status {
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    font-size: 0.8em;
    font-weight: 600;
  }

  .status-ready {
    background: rgba(16, 185, 129, 0.2);
    color: var(--primary-color);
  }

  .status-not.ready {
    background: rgba(239, 68, 68, 0.2);
    color: var(--error-color);
  }

  .node-item-resources {
    display: flex;
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-sm);
  }

  .node-item-resource {
    color: rgba(255, 255, 255, 0.7);
    font-size: 0.85em;
  }

  .node-item-usage {
    display: flex;
    gap: var(--spacing-md);
  }

  .usage-text {
    color: rgba(255, 255, 255, 0.6);
    font-size: 0.8em;
  }

  /* Right Panel: Node Details */
  .node-details-panel {
    background: rgba(255, 255, 255, 0.05);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .node-details-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-lg);
    padding-bottom: var(--spacing-md);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .node-details-header h4 {
    margin: 0;
    color: white;
    font-size: 1.3em;
  }

  .node-details-status {
    padding: 4px 12px;
    border-radius: var(--radius-sm);
    font-size: 0.9em;
    font-weight: 600;
  }

  .node-details-content {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  .details-section {
    background: rgba(255, 255, 255, 0.03);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .details-section h5 {
    margin: 0 0 var(--spacing-md) 0;
    color: white;
    font-size: 1.1em;
    font-weight: 600;
  }

  /* System Information Grid */
  .info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: var(--spacing-md);
  }

  .info-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-sm);
    background: rgba(255, 255, 255, 0.05);
    border-radius: var(--radius-sm);
  }

  .info-label {
    color: rgba(255, 255, 255, 0.8);
    font-weight: 500;
    font-size: 0.9em;
  }

  .info-value-container, .resource-value-container, .label-value-container, .annotation-value-container {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    max-width: 250px;
  }

  .info-value, .resource-value, .label-value, .annotation-value {
    color: white;
    font-weight: 600;
    font-size: 0.95em;
    text-align: right;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  .copy-button {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-sm);
    color: white;
    cursor: pointer;
    font-size: 0.8em;
    padding: 2px 6px;
    transition: var(--transition-normal);
    opacity: 0.7;
    flex-shrink: 0;
  }

  .copy-button:hover {
    background: rgba(255, 255, 255, 0.2);
    border-color: rgba(255, 255, 255, 0.3);
    opacity: 1;
    transform: scale(1.1);
  }

  .copy-button:active {
    transform: scale(0.95);
  }

  /* Resource Grid */
  .resource-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: var(--spacing-md);
  }

  .resource-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-sm);
    background: rgba(255, 255, 255, 0.05);
    border-radius: var(--radius-sm);
  }

  .resource-label {
    color: rgba(255, 255, 255, 0.8);
    font-weight: 500;
    font-size: 0.9em;
  }

  /* Resource Graph Styles */
  .resource-tabs {
    display: flex;
    gap: 4px;
    margin-bottom: var(--spacing-md);
    background: rgba(255, 255, 255, 0.05);
    border-radius: var(--radius-md);
    padding: 4px;
  }

  .resource-tab {
    flex: 1;
    padding: 8px 16px;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    color: rgba(255, 255, 255, 0.7);
    font-size: 0.9em;
    font-weight: 500;
    cursor: pointer;
    transition: var(--transition-normal);
  }

  .resource-tab:hover {
    background: rgba(255, 255, 255, 0.1);
    color: white;
  }

  .resource-tab.active {
    background: var(--primary-color);
    color: white;
    font-weight: 600;
  }

  .resource-graph {
    background: rgba(255, 255, 255, 0.05);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .graph-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-md);
  }

  .graph-title {
    color: white;
    font-size: 1.1em;
    font-weight: 600;
  }

  .graph-controls {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .refresh-control, .history-control, .auto-refresh-control {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .refresh-control label, .history-control label {
    color: rgba(255, 255, 255, 0.8);
    font-size: 0.85em;
    font-weight: 500;
    white-space: nowrap;
  }

  .refresh-select, .history-select {
    background: rgba(0, 0, 0, 0.6);
    border: 1px solid rgba(255, 255, 255, 0.4);
    border-radius: var(--radius-sm);
    color: white;
    font-size: 0.85em;
    font-weight: 600;
    padding: 6px 12px;
    cursor: pointer;
    transition: var(--transition-normal);
    min-width: 60px;
    text-align: center;
  }

  .auto-refresh-toggle {
    background: rgba(0, 0, 0, 0.6);
    border: 1px solid rgba(255, 255, 255, 0.4);
    border-radius: var(--radius-sm);
    color: white;
    font-size: 0.85em;
    font-weight: 600;
    padding: 6px 12px;
    cursor: pointer;
    transition: var(--transition-normal);
    min-width: 60px;
    text-align: center;
  }

  .auto-refresh-toggle:hover {
    background: rgba(0, 0, 0, 0.8);
    border-color: var(--accent-color);
  }

  .auto-refresh-toggle.enabled {
    background: var(--accent-color);
    color: white;
  }

  .last-refresh {
    font-size: 0.7rem;
    color: var(--text-secondary);
    font-style: italic;
  }

  .history-select {
    min-width: 80px;
  }

  .refresh-select:hover, .history-select:hover {
    background: rgba(0, 0, 0, 0.8);
    border-color: rgba(255, 255, 255, 0.6);
    transform: scale(1.05);
  }

  .refresh-select:focus, .history-select:focus {
    outline: none;
    border-color: var(--primary-color);
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.3);
    background: rgba(0, 0, 0, 0.8);
  }

  .current-value-container {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .current-value {
    color: var(--primary-color);
    font-size: 1.2em;
    font-weight: 700;
  }

  .loading-indicator {
    animation: spin 1s linear infinite;
    font-size: 0.9em;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  /* Chart Container */
  .chartjs-container {
    position: relative;
    height: 200px;
    background: rgba(0, 0, 0, 0.2);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .chartjs-canvas {
    width: 100%;
    height: 100%;
  }

  .scrolling-indicator {
    position: absolute;
    top: 10px;
    right: 10px;
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--primary-color);
    font-size: 0.8em;
    font-weight: 600;
  }

  .dot {
    width: 8px;
    height: 8px;
    background: var(--primary-color);
    border-radius: 50%;
    animation: pulse 2s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  /* Error and No Data States */
  .metrics-error, .no-data {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: rgba(255, 193, 7, 0.1);
    border: 1px solid rgba(255, 193, 7, 0.3);
    border-radius: var(--radius-md);
    margin-top: var(--spacing-md);
  }

  .no-data {
    background: rgba(108, 117, 125, 0.1);
    border-color: rgba(108, 117, 125, 0.3);
  }

  .error-icon {
    font-size: 1.2em;
    flex-shrink: 0;
  }

  .error-content, .no-data-content {
    flex: 1;
  }

  .error-content h6, .no-data-content h6 {
    margin: 0 0 4px 0;
    color: white;
    font-size: 0.9em;
    font-weight: 600;
  }

  .error-content p, .no-data-content p {
    margin: 0 0 8px 0;
    color: rgba(255, 255, 255, 0.8);
    font-size: 0.85em;
  }

  .retry-button {
    background: var(--primary-color);
    border: none;
    border-radius: var(--radius-sm);
    color: white;
    cursor: pointer;
    font-size: 0.8em;
    padding: 4px 8px;
    transition: var(--transition-normal);
  }

  .retry-button:hover {
    background: var(--accent-color);
  }

  /* Debug Console */
  .debug-panel {
    background: rgba(0, 0, 0, 0.3);
    border-radius: var(--radius-sm);
    padding: var(--spacing-sm);
    margin-top: var(--spacing-sm);
    font-family: 'Courier New', monospace;
    font-size: 0.8em;
  }

  .debug-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-xs);
  }

  .debug-title {
    color: var(--primary-color);
    font-weight: 600;
  }

  .debug-time {
    color: rgba(255, 255, 255, 0.6);
    font-size: 0.8em;
  }

  .debug-content {
    color: rgba(255, 255, 255, 0.8);
    white-space: pre-wrap;
    max-height: 100px;
    overflow-y: auto;
  }

  .debug-console-controls {
    margin-top: var(--spacing-sm);
  }

  .debug-console-button {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-sm);
    color: white;
    cursor: pointer;
    font-size: 0.8em;
    padding: 4px 8px;
    transition: var(--transition-normal);
  }

  .debug-console-button:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .debug-console-panel {
    background: rgba(0, 0, 0, 0.5);
    border-radius: var(--radius-sm);
    margin-top: var(--spacing-sm);
    max-height: 200px;
    overflow: hidden;
  }

  .debug-console-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-sm);
    background: rgba(0, 0, 0, 0.3);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .debug-console-title {
    color: var(--primary-color);
    font-weight: 600;
    font-size: 0.9em;
  }

  .debug-console-clear {
    background: var(--error-color);
    border: none;
    border-radius: var(--radius-sm);
    color: white;
    cursor: pointer;
    font-size: 0.7em;
    padding: 2px 6px;
    transition: var(--transition-normal);
  }

  .debug-console-clear:hover {
    background: #dc2626;
  }

  .debug-console-content {
    padding: var(--spacing-sm);
    color: rgba(255, 255, 255, 0.8);
    font-family: 'Courier New', monospace;
    font-size: 0.8em;
    white-space: pre-wrap;
    max-height: 150px;
    overflow-y: auto;
  }

  /* Labels and Annotations */
  .labels-list, .annotations-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .label-item, .annotation-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-sm);
    background: rgba(255, 255, 255, 0.05);
    border-radius: var(--radius-sm);
  }

  .label-key, .annotation-key {
    color: rgba(255, 255, 255, 0.8);
    font-weight: 500;
    font-size: 0.9em;
    flex-shrink: 0;
    margin-right: var(--spacing-sm);
  }

  /* Taints */
  .taints-list {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-sm);
  }

  .taint-badge {
    background: rgba(239, 68, 68, 0.2);
    color: var(--error-color);
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    font-size: 0.8em;
    font-weight: 500;
  }

  /* No Selection State */
  .no-selection {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 200px;
    background: rgba(255, 255, 255, 0.03);
    border-radius: var(--radius-md);
    border: 2px dashed rgba(255, 255, 255, 0.2);
  }

  .no-selection-content {
    text-align: center;
    color: rgba(255, 255, 255, 0.6);
  }

  .no-selection-content h4 {
    margin: 0 0 var(--spacing-sm) 0;
    color: white;
  }

  /* Metrics Warning */
  .metrics-warning {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: rgba(245, 158, 11, 0.1);
    border: 1px solid rgba(245, 158, 11, 0.3);
    border-radius: var(--radius-md);
    margin-bottom: var(--spacing-md);
  }

  .warning-icon {
    font-size: 1.2em;
    flex-shrink: 0;
  }

  .warning-content h6 {
    margin: 0 0 4px 0;
    color: white;
    font-size: 0.9em;
    font-weight: 600;
  }

  .warning-content p {
    margin: 0;
    color: rgba(255, 255, 255, 0.8);
    font-size: 0.85em;
  }

  /* Responsive Design */
  @media (max-width: 768px) {
    .nodes-layout {
      grid-template-columns: 1fr;
    }
    
    .graph-controls {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--spacing-sm);
    }
    
    .info-grid, .resource-grid {
      grid-template-columns: 1fr;
    }
  }
</style>

