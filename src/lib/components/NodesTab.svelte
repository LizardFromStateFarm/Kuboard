<!-- Kuboard Nodes Tab Component -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { createEventDispatcher } from 'svelte';
  import { formatMemory, formatCPU } from '$lib/utils/formatters';
  import MetricsGraph from './MetricsGraph.svelte';

  // Props
  export let currentContext: any = null;
  export let nodes: any[] = [];

  // State
  let selectedNode: any = null;
  let showFullDetails: boolean = false;
  let refreshTimer: any;
  let isLoading: boolean = false;
  
  // Metrics state
  let nodeMetrics: any = null;
  let metricsLoading: boolean = false;
  let metricsError: string | null = null;
  let selectedResourceType: 'cpu' | 'memory' | 'disk' = 'cpu';
  let selectedTimeRange: number = 30; // Default to 30 minutes

  // Event dispatcher
  const dispatch = createEventDispatcher();

  // Debug logging
  console.log('🚀 NodesTab component script loaded');
  
  // Set loading state when context changes but nodes haven't loaded yet
  $: if (currentContext && (!nodes || nodes.length === 0)) {
    isLoading = true;
  } else {
    isLoading = false;
  }

  // Select node (for dispatching to other components if needed)
  function selectNode(node: any) {
    selectedNode = {
      name: node.metadata?.name || 'Unknown',
      status: node.status?.conditions?.find((c: any) => c.type === 'Ready')?.status || 'Unknown',
      os: node.status?.nodeInfo?.operatingSystem || 'Unknown',
      kernelVersion: node.status?.nodeInfo?.kernelVersion || 'Unknown',
      kubeletVersion: node.status?.nodeInfo?.kubeletVersion || 'Unknown',
      containerRuntime: node.status?.nodeInfo?.containerRuntimeVersion || 'Unknown',
      diskCapacity: parseInt(node.status?.capacity?.['ephemeral-storage']?.replace('Ki', '') || '0') * 1024,
      diskAllocatable: parseInt(node.status?.allocatable?.['ephemeral-storage']?.replace('Ki', '') || '0') * 1024,
      cpuCapacity: node.status?.capacity?.cpu || '0',
      memoryCapacity: parseInt(node.status?.capacity?.memory?.replace('Ki', '') || '0') * 1024,
      cpuAllocatable: node.status?.allocatable?.cpu || '0',
      memoryAllocatable: parseInt(node.status?.allocatable?.memory?.replace('Ki', '') || '0') * 1024,
      architecture: node.status?.nodeInfo?.architecture || 'Unknown',
      creationTimestamp: node.metadata?.creationTimestamp || 'Unknown'
    };
    dispatch('nodeSelect', selectedNode);
  }

  // Load metrics for a specific node
  async function loadNodeMetrics(node: any) {
    if (!node?.metadata?.name) return;
    
    metricsLoading = true;
    metricsError = null;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
        const metrics = await invoke('kuboard_get_node_metrics_history', {
          nodeName: node.metadata.name,
          durationMinutes: selectedTimeRange
        });
      nodeMetrics = metrics;
    } catch (err) {
      metricsError = err as string;
      console.error('Failed to load node metrics:', err);
    } finally {
      metricsLoading = false;
    }
  }

  // Show full details view
  function showFullNodeDetails(node: any) {
    selectedNode = {
      name: node.metadata?.name || 'Unknown',
      status: node.status?.conditions?.find((c: any) => c.type === 'Ready')?.status || 'Unknown',
      os: node.status?.nodeInfo?.operatingSystem || 'Unknown',
      kernelVersion: node.status?.nodeInfo?.kernelVersion || 'Unknown',
      kubeletVersion: node.status?.nodeInfo?.kubeletVersion || 'Unknown',
      containerRuntime: node.status?.nodeInfo?.containerRuntimeVersion || 'Unknown',
      diskCapacity: parseInt(node.status?.capacity?.['ephemeral-storage']?.replace('Ki', '') || '0') * 1024,
      diskAllocatable: parseInt(node.status?.allocatable?.['ephemeral-storage']?.replace('Ki', '') || '0') * 1024,
      cpuCapacity: node.status?.capacity?.cpu || '0',
      memoryCapacity: parseInt(node.status?.capacity?.memory?.replace('Ki', '') || '0') * 1024,
      cpuAllocatable: node.status?.allocatable?.cpu || '0',
      memoryAllocatable: parseInt(node.status?.allocatable?.memory?.replace('Ki', '') || '0') * 1024,
      architecture: node.status?.nodeInfo?.architecture || 'Unknown',
      creationTimestamp: node.metadata?.creationTimestamp || 'Unknown'
    };
    showFullDetails = true;
    loadNodeMetrics(node);
  }

  // Back to nodes list
  function backToNodesList() {
    showFullDetails = false;
    selectedNode = null;
    nodeMetrics = null;
    selectedResourceType = 'cpu'; // Reset to CPU when going back
  }

  // Change resource type for metrics graph
  function changeResourceType(type: 'cpu' | 'memory' | 'disk') {
    selectedResourceType = type;
  }

  // Change time range for metrics graph
  function changeTimeRange(minutes: number) {
    selectedTimeRange = minutes;
    // Reload metrics with new time range if a node is selected
    if (selectedNode) {
      loadNodeMetrics(selectedNode);
    }
  }

  // Get status class
  function getStatusClass(status: string): string {
    switch (status?.toLowerCase()) {
      case 'ready': return 'ready';
      case 'notready': return 'not-ready';
      case 'unknown': return 'unknown';
      default: return 'unknown';
    }
  }

  // Copy to clipboard function
  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
    } catch (err) {
      console.error('Failed to copy to clipboard:', err);
    }
  }

  // Lifecycle
  onMount(() => {
    console.log('🚀 NodesTab onMount called - currentContext:', currentContext, 'nodes:', nodes.length);
  });

  onDestroy(() => {
    if (refreshTimer) {
      clearInterval(refreshTimer);
    }
  });
</script>

    <div class="nodes-tab">
  
  <div class="tab-header">
    <h4>🖥️ Nodes</h4>
    <div class="tab-controls">
      <button 
        class="refresh-button" 
        onclick={() => console.log('Refresh clicked')}
        title="Refresh nodes"
      >
        ↻
      </button>
    </div>
  </div>

  <!-- Always show the basic UI structure -->
  <div class="nodes-content">
    {#if showFullDetails && selectedNode}
      <!-- Full Details View -->
      <div class="full-details-view">
        <div class="details-header">
          <button class="back-button" onclick={backToNodesList}>
            ← Back to Nodes
          </button>
          <h3>{selectedNode.name}</h3>
        </div>
        
        <div class="node-details-content">
          <!-- System Information -->
          <div class="details-section">
            <h6>System Information</h6>
            <div class="info-grid">
              <div class="info-item">
                <span class="info-label">Status:</span>
                <div class="info-value-container">
                  <span class="info-value status-badge status-{getStatusClass(selectedNode.status === 'True' ? 'Ready' : 'NotReady')}">
                    {selectedNode.status === 'True' ? 'Ready' : 'Not Ready'}
                  </span>
                </div>
              </div>
              <div class="info-item">
                <span class="info-label">Operating System:</span>
                <div class="info-value-container">
                  <span class="info-value" title={selectedNode.os}>{selectedNode.os}</span>
                  <button 
                    class="copy-button" 
                    onclick={() => copyToClipboard(selectedNode.os || '')}
                    title="Copy to clipboard"
                  >
                    📋
                  </button>
                </div>
              </div>
              <div class="info-item">
                <span class="info-label">Kernel Version:</span>
                <div class="info-value-container">
                  <span class="info-value" title={selectedNode.kernelVersion}>{selectedNode.kernelVersion}</span>
                  <button 
                    class="copy-button" 
                    onclick={() => copyToClipboard(selectedNode.kernelVersion || '')}
                    title="Copy to clipboard"
                  >
                    📋
                  </button>
                </div>
              </div>
              <div class="info-item">
                <span class="info-label">Kubelet Version:</span>
                <div class="info-value-container">
                  <span class="info-value" title={selectedNode.kubeletVersion}>{selectedNode.kubeletVersion}</span>
                  <button 
                    class="copy-button" 
                    onclick={() => copyToClipboard(selectedNode.kubeletVersion || '')}
                    title="Copy to clipboard"
                  >
                    📋
                  </button>
                </div>
              </div>
              <div class="info-item">
                <span class="info-label">Container Runtime:</span>
                <div class="info-value-container">
                  <span class="info-value" title={selectedNode.containerRuntime}>{selectedNode.containerRuntime}</span>
                  <button 
                    class="copy-button" 
                    onclick={() => copyToClipboard(selectedNode.containerRuntime || '')}
                    title="Copy to clipboard"
                  >
                    📋
                  </button>
                </div>
              </div>
              <div class="info-item">
                <span class="info-label">Architecture:</span>
                <div class="info-value-container">
                  <span class="info-value" title={selectedNode.architecture}>{selectedNode.architecture}</span>
                  <button 
                    class="copy-button" 
                    onclick={() => copyToClipboard(selectedNode.architecture || '')}
                    title="Copy to clipboard"
                  >
                    📋
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- Resource Information -->
          <div class="details-section">
            <h6>Resource Information</h6>
            <div class="resource-grid">
              <div class="resource-item">
                <span class="resource-label">CPU Capacity:</span>
                <div class="resource-value-container">
                  <span class="resource-value" title={selectedNode.cpuCapacity}>{formatCPU(selectedNode.cpuCapacity || '0')}</span>
                  <button 
                    class="copy-button" 
                    onclick={() => copyToClipboard(selectedNode.cpuCapacity || '')}
                    title="Copy to clipboard"
                  >
                    📋
                  </button>
                </div>
              </div>
              <div class="resource-item">
                <span class="resource-label">CPU Allocatable:</span>
                <div class="resource-value-container">
                  <span class="resource-value" title={selectedNode.cpuAllocatable}>{formatCPU(selectedNode.cpuAllocatable || '0')}</span>
                  <button 
                    class="copy-button" 
                    onclick={() => copyToClipboard(selectedNode.cpuAllocatable || '')}
                    title="Copy to clipboard"
                  >
                    📋
                  </button>
                </div>
              </div>
              <div class="resource-item">
                <span class="resource-label">Memory Capacity:</span>
                <div class="resource-value-container">
                  <span class="resource-value" title={formatMemory(selectedNode.memoryCapacity || 0)}>{formatMemory(selectedNode.memoryCapacity || 0)}</span>
                  <button 
                    class="copy-button" 
                    onclick={() => copyToClipboard(formatMemory(selectedNode.memoryCapacity || 0))}
                    title="Copy to clipboard"
                  >
                    📋
                  </button>
                </div>
              </div>
              <div class="resource-item">
                <span class="resource-label">Memory Allocatable:</span>
                <div class="resource-value-container">
                  <span class="resource-value" title={formatMemory(selectedNode.memoryAllocatable || 0)}>{formatMemory(selectedNode.memoryAllocatable || 0)}</span>
                  <button 
                    class="copy-button" 
                    onclick={() => copyToClipboard(formatMemory(selectedNode.memoryAllocatable || 0))}
                    title="Copy to clipboard"
                  >
                    📋
                  </button>
                </div>
              </div>
              <div class="resource-item">
                <span class="resource-label">Disk Capacity:</span>
                <div class="resource-value-container">
                  <span class="resource-value" title={formatMemory(selectedNode.diskCapacity || 0)}>{formatMemory(selectedNode.diskCapacity || 0)}</span>
                  <button 
                    class="copy-button" 
                    onclick={() => copyToClipboard(formatMemory(selectedNode.diskCapacity || 0))}
                    title="Copy to clipboard"
                  >
                    📋
                  </button>
                </div>
              </div>
              <div class="resource-item">
                <span class="resource-label">Disk Allocatable:</span>
                <div class="resource-value-container">
                  <span class="resource-value" title={formatMemory(selectedNode.diskAllocatable || 0)}>{formatMemory(selectedNode.diskAllocatable || 0)}</span>
                  <button 
                    class="copy-button" 
                    onclick={() => copyToClipboard(formatMemory(selectedNode.diskAllocatable || 0))}
                    title="Copy to clipboard"
                  >
                    📋
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- Metrics Section -->
          <div class="details-section">
            <h6>Resource Usage Metrics</h6>
            {#if metricsLoading}
              <div class="metrics-loading">
                <div class="loading-spinner">⏳</div>
                <p>Loading metrics...</p>
              </div>
            {:else if metricsError}
              <div class="metrics-error">
                <div class="error-icon">⚠️</div>
                <p>Failed to load metrics: {metricsError}</p>
                <button class="retry-button" onclick={() => loadNodeMetrics(selectedNode)}>
                  Retry
                </button>
              </div>
            {:else if nodeMetrics}
              <!-- Resource Type Selector -->
              <div class="resource-type-selector">
                <div class="metrics-header">
                  <h6>📊 Resource Usage</h6>
                  <div class="time-range-selector">
                    <label for="timeRange">Time Range:</label>
                    <select 
                      id="timeRange"
                      bind:value={selectedTimeRange}
                      onchange={() => changeTimeRange(selectedTimeRange)}
                      class="time-range-select"
                    >
                      <option value="30">30 minutes</option>
                      <option value="60">1 hour</option>
                      <option value="120">2 hours</option>
                      <option value="240">4 hours</option>
                      <option value="480">8 hours</option>
                      <option value="720">12 hours</option>
                    </select>
                  </div>
                </div>
                <div class="resource-tabs">
                  <button 
                    class="resource-tab {selectedResourceType === 'cpu' ? 'active' : ''}"
                    onclick={() => changeResourceType('cpu')}
                  >
                    🖥️ CPU
                  </button>
                  <button 
                    class="resource-tab {selectedResourceType === 'memory' ? 'active' : ''}"
                    onclick={() => changeResourceType('memory')}
                  >
                    💾 Memory
                  </button>
                  <button 
                    class="resource-tab {selectedResourceType === 'disk' ? 'active' : ''}"
                    onclick={() => changeResourceType('disk')}
                  >
                    💿 Disk
                  </button>
                </div>
              </div>
              
              <div class="metrics-graph-container">
                <MetricsGraph
                  data={nodeMetrics}
                  type={selectedResourceType}
                  duration={selectedTimeRange}
                  loading={metricsLoading}
                  error={metricsError}
                  maxCpuCores={parseFloat(selectedNode.cpuCapacity || '0')}
                  maxMemoryBytes={selectedNode.memoryCapacity || 0}
                  maxDiskBytes={selectedNode.diskCapacity || 0}
                />
              </div>
            {:else}
              <div class="metrics-placeholder">
                <p>No metrics data available</p>
              </div>
            {/if}
          </div>
        </div>
      </div>
    {:else}
      <!-- Nodes List View (always show this) -->
      <div class="nodes-list-view">
        <div class="nodes-header">
          <h5>Cluster Nodes {nodes && nodes.length > 0 ? `(${nodes.length})` : ''}</h5>
          <p>Click on a node to view detailed information and metrics</p>
        </div>
        
        {#if isLoading}
          <!-- Loading State -->
          <div class="loading-nodes">
            <div class="loading-spinner">⏳</div>
            <h5>Loading Nodes...</h5>
            <p>Please wait while we fetch the cluster nodes information.</p>
          </div>
        {:else if nodes && nodes.length > 0}
          <!-- Nodes Grid -->
          <div class="nodes-grid">
            {#each nodes as node}
              <div class="node-card" onclick={() => showFullNodeDetails(node)} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' || e.key === ' ' ? showFullNodeDetails(node) : null}>
                <div class="node-card-content">
                  <div class="node-header">
                    <h6 class="node-name">{node.metadata?.name || 'Unknown'}</h6>
                    <div class="node-status">
                      <span class="status-badge status-{getStatusClass(node.status?.conditions?.find((c: any) => c.type === 'Ready')?.status || 'Unknown')}">
                        {node.status?.conditions?.find((c: any) => c.type === 'Ready')?.status || 'Unknown'}
                      </span>
                    </div>
                  </div>
                  
                  <div class="node-summary">
                    <div class="summary-item">
                      <span class="summary-label">OS:</span>
                      <span class="summary-value">{node.status?.nodeInfo?.operatingSystem || 'Unknown'}</span>
                    </div>
                    <div class="summary-item">
                      <span class="summary-label">Architecture:</span>
                      <span class="summary-value">{node.status?.nodeInfo?.architecture || 'Unknown'}</span>
                    </div>
                    <div class="summary-item">
                      <span class="summary-label">Kubelet:</span>
                      <span class="summary-value">{node.status?.nodeInfo?.kubeletVersion || 'Unknown'}</span>
                    </div>
                  </div>
                  
                  <div class="node-resources">
                    <div class="resource-summary">
                      <div class="resource-item">
                        <span class="resource-label">CPU:</span>
                        <span class="resource-value">{formatCPU(node.status?.capacity?.cpu || '0')}</span>
                      </div>
                      <div class="resource-item">
                        <span class="resource-label">Memory:</span>
                        <span class="resource-value">{formatMemory(parseInt(node.status?.capacity?.memory?.replace('Ki', '') || '0') * 1024)}</span>
                      </div>
                      <div class="resource-item">
                        <span class="resource-label">Disk:</span>
                        <span class="resource-value">{formatMemory(parseInt(node.status?.capacity?.['ephemeral-storage']?.replace('Ki', '') || '0') * 1024)}</span>
                      </div>
                    </div>
                  </div>
                  
                  <div class="node-actions">
                    <span class="click-hint">Click to view details →</span>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <!-- No Nodes Available -->
          <div class="no-nodes-message">
            <div class="no-nodes-icon">🖥️</div>
            <h5>No Nodes Available</h5>
            <p>No nodes are currently available in this cluster context.</p>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  /* Import CSS variables */
  @import '../styles/variables.css';
  @import '../styles/color-palette.css';

  .nodes-tab {
    padding: 0;
  }

  .tab-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 5px;
    padding-bottom: var(--spacing-sm);
    border-bottom: 1px solid var(--border-primary);
  }

  .tab-header h4 {
    margin: 0;
    color: var(--text-primary);
    font-size: 1.2rem;
  }

  .tab-controls {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .refresh-button {
    background: var(--background-card);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    cursor: pointer;
    font-size: 1.2rem;
    padding: var(--spacing-sm);
    transition: var(--transition-normal);
  }

  .refresh-button:hover {
    background: var(--accent-color);
    border-color: var(--accent-color);
    color: white;
  }

  .nodes-content {
    min-height: 200px;
  }

  /* Loading State */
  .loading-nodes {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-xl);
    color: var(--text-secondary);
    text-align: center;
  }

  .loading-spinner {
    font-size: 2rem;
    animation: spin 2s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .loading-nodes h5 {
    margin: 0;
    color: var(--text-primary);
    font-size: 1.2rem;
  }

  .loading-nodes p {
    margin: 0;
    font-size: 0.9rem;
  }

  /* No Nodes Message */
  .no-nodes-message {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-xl);
    color: var(--text-secondary);
    text-align: center;
  }

  .no-nodes-icon {
    font-size: 3rem;
    opacity: 0.7;
  }

  .no-nodes-message h5 {
    margin: 0;
    color: var(--text-primary);
    font-size: 1.2rem;
  }

  .no-nodes-message p {
    margin: 0;
    font-size: 0.9rem;
  }

  /* Nodes List View */
  .nodes-list-view {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  .nodes-header {
    text-align: center;
    margin-bottom: 5px;
  }

  .nodes-header h5 {
    margin: 0 0 var(--spacing-sm) 0;
    color: var(--text-primary);
    font-size: 1.3rem;
  }

  .nodes-header p {
    margin: 0;
    color: var(--text-secondary);
    font-size: 0.9rem;
  }

  .nodes-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: var(--spacing-md);
  }

  .node-card {
    background: var(--background-card);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: var(--transition-normal);
    overflow: hidden;
  }

  .node-card:hover {
    border-color: var(--accent-color);
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .node-card-content {
    padding: var(--spacing-md);
  }

  .node-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-md);
  }

  .node-name {
    margin: 0;
    color: var(--text-primary);
    font-size: 1.1rem;
    font-weight: 600;
  }

  .node-status {
    display: flex;
    align-items: center;
  }

  .status-badge {
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    font-size: 0.8rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  .status-ready {
    background: var(--status-ready-bg);
    color: var(--status-ready-text);
    border: 1px solid var(--status-ready-border);
  }

  .status-not-ready {
    background: var(--status-error-bg);
    color: var(--status-error-text);
    border: 1px solid var(--status-error-border);
  }

  .status-unknown {
    background: var(--status-pending-bg);
    color: var(--status-pending-text);
    border: 1px solid var(--status-pending-border);
  }

  .node-summary {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-md);
  }

  .summary-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .summary-label {
    color: var(--text-secondary);
    font-size: 0.9rem;
    font-weight: 500;
  }

  .summary-value {
    color: var(--text-primary);
    font-size: 0.9rem;
    font-weight: 600;
  }

  .node-resources {
    margin-bottom: var(--spacing-md);
  }

  .resource-summary {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .resource-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .resource-label {
    color: var(--text-secondary);
    font-size: 0.85rem;
    font-weight: 500;
  }

  .resource-value {
    color: var(--text-primary);
    font-size: 0.85rem;
    font-weight: 600;
  }

  .node-actions {
    text-align: center;
    padding-top: var(--spacing-sm);
    border-top: 1px solid var(--border-primary);
  }

  .click-hint {
    color: var(--accent-color);
    font-size: 0.8rem;
    font-weight: 500;
  }

  /* Full Details View */
  .full-details-view {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  .details-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding-bottom: var(--spacing-md);
    border-bottom: 1px solid var(--border-primary);
  }

  .back-button {
    background: var(--background-card);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    cursor: pointer;
    font-size: 0.9rem;
    padding: var(--spacing-sm) var(--spacing-md);
    transition: var(--transition-normal);
  }

  .back-button:hover {
    background: var(--accent-color);
    border-color: var(--accent-color);
    color: white;
  }

  .details-header h3 {
    margin: 0;
    color: var(--text-primary);
    font-size: 1.5rem;
  }

  .node-details-content {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  .details-section {
    background: var(--background-card);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
  }

  .details-section h6 {
    margin: 0 0 var(--spacing-md) 0;
    color: var(--text-primary);
    font-size: 1.1rem;
    font-weight: 600;
  }

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
    background: rgba(255, 255, 255, 0.02);
    border-radius: var(--radius-sm);
  }

  .info-label {
    color: var(--text-secondary);
    font-weight: 500;
    font-size: 0.9rem;
  }

  .info-value-container {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .info-value {
    color: var(--text-primary);
    font-weight: 600;
    font-size: 0.95rem;
  }

  .copy-button {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    cursor: pointer;
    font-size: 0.8rem;
    padding: 2px 6px;
    transition: var(--transition-normal);
    opacity: 0.7;
  }

  .copy-button:hover {
    background: rgba(255, 255, 255, 0.2);
    border-color: rgba(255, 255, 255, 0.3);
    opacity: 1;
  }

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
    background: rgba(255, 255, 255, 0.02);
    border-radius: var(--radius-sm);
  }

  .resource-label {
    color: var(--text-secondary);
    font-weight: 500;
    font-size: 0.9rem;
  }

  .resource-value-container {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .resource-value {
    color: var(--text-primary);
    font-weight: 600;
    font-size: 0.95rem;
  }

  /* Metrics Section */
  .metrics-loading, .metrics-error, .metrics-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-lg);
    text-align: center;
  }

  .metrics-loading {
    color: var(--text-secondary);
  }

  .metrics-error {
    color: var(--error-color);
  }

  .metrics-placeholder {
    color: var(--text-muted);
  }

  .error-icon {
    font-size: 1.5rem;
  }

  .retry-button {
    background: var(--accent-color);
    border: none;
    border-radius: var(--radius-sm);
    color: white;
    cursor: pointer;
    font-size: 0.9rem;
    padding: var(--spacing-sm) var(--spacing-md);
    transition: var(--transition-normal);
  }

  .retry-button:hover {
    background: var(--primary-color);
  }

      .metrics-graph-container {
        background: var(--background-card);
        border-radius: var(--radius-sm);
        padding: var(--spacing-sm);
        border: 1px solid var(--border-primary);
      }

      /* Resource Type Selector */
      .resource-type-selector {
        margin-bottom: var(--spacing-md);
      }

      .metrics-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: var(--spacing-sm);
        flex-wrap: wrap;
        gap: var(--spacing-sm);
      }

      .resource-type-selector h6 {
        margin: 0;
        color: var(--text-primary);
        font-size: 1.1rem;
        font-weight: 600;
      }

      .time-range-selector {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
      }

      .time-range-selector label {
        color: var(--text-secondary);
        font-size: 0.9rem;
        font-weight: 500;
        white-space: nowrap;
      }

      .time-range-select {
        background: var(--background-card);
        border: 1px solid var(--border-primary);
        border-radius: var(--radius-sm);
        color: var(--text-primary);
        font-size: 0.9rem;
        font-weight: 500;
        padding: var(--spacing-xs) var(--spacing-sm);
        cursor: pointer;
        transition: var(--transition-normal);
        min-width: 120px;
      }

      .time-range-select:hover {
        border-color: var(--accent-color);
        background: var(--background-tertiary);
      }

      .time-range-select:focus {
        outline: none;
        border-color: var(--primary-color);
        box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.3);
      }

      .resource-tabs {
        display: flex;
        gap: var(--spacing-xs);
        background: var(--background-secondary);
        border-radius: var(--radius-sm);
        padding: var(--spacing-xs);
        border: 1px solid var(--border-secondary);
      }

      .resource-tab {
        background: transparent;
        border: 1px solid transparent;
        border-radius: var(--radius-sm);
        color: var(--text-secondary);
        cursor: pointer;
        font-size: 0.9rem;
        font-weight: 500;
        padding: var(--spacing-xs) var(--spacing-sm);
        transition: var(--transition-normal);
        flex: 1;
        text-align: center;
      }

      .resource-tab:hover {
        background: var(--background-tertiary);
        border-color: var(--border-primary);
        color: var(--text-primary);
      }

      .resource-tab.active {
        background: var(--primary-color);
        border-color: var(--primary-color);
        color: white;
        font-weight: 600;
      }

      .resource-tab.active:hover {
        background: var(--accent-color);
        border-color: var(--accent-color);
      }

  /* Responsive Design */
  @media (max-width: 768px) {
    .nodes-grid {
      grid-template-columns: 1fr;
    }
    
    .info-grid, .resource-grid {
      grid-template-columns: 1fr;
    }
    
    .details-header {
      flex-direction: column;
      align-items: flex-start;
    }
  }
</style>