<!-- Kuboard Cluster Overview Component -->
<script lang="ts">
  import type { ClusterOverview, NodeDetails, ResourceTab } from '../types/index.js';
  import { createEventDispatcher } from 'svelte';
  import MetricsGraph from './MetricsGraph.svelte';
  import ClusterMetrics from './ClusterMetrics.svelte';
  import TabbedContent from './TabbedContent.svelte';

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
  
  // Debug: Log all nodes prop changes
  $: console.log('🔄 ClusterOverview: nodes prop updated:', nodes?.length || 0, 'currentContext:', !!currentContext);
  $: console.log('🔍 ClusterOverview: Full nodes data:', nodes);
  
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

  // Events
  const dispatch = createEventDispatcher();

  // Event handlers
  function selectNode(node: any) {
    // Convert the raw node data to NodeDetails format while preserving the original data
    const nodeDetails: NodeDetails = {
      name: node.metadata?.name || 'Unknown',
      status: node.status?.conditions?.find((c: any) => c.type === 'Ready')?.status || 'Unknown',
      max_cpu_cores: parseFloat(node.status?.capacity?.cpu?.replace('m', '')) / 1000 || 0,
      max_memory_bytes: parseFloat(node.status?.capacity?.memory?.replace('Ki', '')) * 1024 || 0,
      allocatable_cpu_cores: parseFloat(node.status?.allocatable?.cpu?.replace('m', '')) / 1000 || 0,
      allocatable_memory_bytes: parseFloat(node.status?.allocatable?.memory?.replace('Ki', '')) * 1024 || 0,
      cpu_usage_percent: 0,
      memory_usage_percent: 0,
      disk_usage_percent: 0,
      conditions: node.status?.conditions?.map((c: any) => `${c.type}: ${c.status}`) || [],
      // Extract additional node information from the original data
      os: node.status?.nodeInfo?.operatingSystem || 'Unknown',
      kernel_version: node.status?.nodeInfo?.kernelVersion || 'Unknown',
      kubelet_version: node.status?.nodeInfo?.kubeletVersion || 'Unknown',
      container_runtime: node.status?.nodeInfo?.containerRuntimeVersion || 'Unknown',
      disk_capacity: parseFloat(node.status?.capacity?.['ephemeral-storage']?.replace('Ki', '')) * 1024 || 0,
      disk_allocatable: parseFloat(node.status?.allocatable?.['ephemeral-storage']?.replace('Ki', '')) * 1024 || 0,
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

    <!-- Cluster Metrics Section -->
    <ClusterMetrics 
      refreshInterval={refreshIntervalSeconds * 1000}
      autoRefresh={autoRefreshEnabled}
    />

    <!-- Resource Management Tabs -->
    <TabbedContent {currentContext} {nodes} />
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
    background: var(--gradient-primary);
    border-radius: var(--radius-xl);
    padding: var(--spacing-lg);
    margin: 20px 0;
    color: white;
    box-shadow: 0 20px 25px rgba(59, 130, 246, 0.15);
    border: 1px solid rgba(59, 130, 246, 0.3);
    position: relative;
    overflow: hidden;
  }

  .cluster-overview::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(135deg, rgba(255, 255, 255, 0.1) 0%, rgba(255, 255, 255, 0.05) 100%);
    pointer-events: none;
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
    background: rgba(0, 0, 0, 0.3);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.2);
    position: relative;
    z-index: 1;
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
    background: var(--status-ready-bg);
    color: var(--status-ready-text);
    border: 1px solid var(--status-ready-border);
    font-weight: 600;
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

