<!-- Kuboard Cluster Metrics Component -->
<script lang="ts">
  import DonutChart from './DonutChart.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import MetricsGraph from './MetricsGraph.svelte';
  import { Activity, Layers, Server, RefreshCw, ArrowLeft } from 'lucide-svelte';
  import { navigateTo } from '../stores/nav';

  export let refreshInterval: number = 10000; // 10 seconds default
  export let autoRefresh: boolean = true;
  export let nodes: any[] = [];

  // Cluster metrics data
  let clusterMetrics: any = null;
  let loading: boolean = false;
  let error: string | null = null;
  let lastUpdate: string = '';
  let selectedPoolName: string | null = null;

  function getNodePoolName(node: any): string {
    const labels = node?.metadata?.labels || {};
    return labels['cloud.google.com/gke-nodepool'] ||
           labels['eks.amazonaws.com/nodegroup'] ||
           labels['agentpool'] ||
           labels['kops.k8s.io/instancegroup'] ||
           labels['nodepool'] ||
           (labels['node-role.kubernetes.io/control-plane'] !== undefined || labels['node-role.kubernetes.io/master'] !== undefined ? 'control-plane' : 'worker-pool');
  }

  function groupNodePools(nodesList: any[]) {
    if (!nodesList || nodesList.length === 0) return [];
    const poolsMap: Record<string, any> = {};

    for (const node of nodesList) {
      const poolName = getNodePoolName(node);
      if (!poolsMap[poolName]) {
        poolsMap[poolName] = {
          name: poolName,
          nodes: [],
          readyCount: 0,
          totalCpuCapacity: 0,
          totalMemCapacity: 0,
          instanceTypes: new Set<string>()
        };
      }
      const pool = poolsMap[poolName];
      pool.nodes.push(node);

      const isReady = node?.status?.conditions?.some((c: any) => c.type === 'Ready' && c.status === 'True');
      if (isReady) pool.readyCount++;

      const cpuCap = parseFloat(node?.status?.capacity?.cpu || '0');
      const memCapKi = parseInt(node?.status?.capacity?.memory?.replace('Ki', '') || '0');
      pool.totalCpuCapacity += cpuCap;
      pool.totalMemCapacity += memCapKi * 1024;

      const instanceType = node?.metadata?.labels?.['node.kubernetes.io/instance-type'] || node?.metadata?.labels?.['beta.kubernetes.io/instance-type'] || 'standard';
      pool.instanceTypes.add(instanceType);
    }

    return Object.values(poolsMap).map(pool => ({
      ...pool,
      instanceTypesString: Array.from(pool.instanceTypes).join(', ')
    }));
  }

  $: nodePools = groupNodePools(nodes);
  $: activePool = selectedPoolName ? nodePools.find(p => p.name === selectedPoolName) : null;

  function jumpToNode(nodeName: string) {
    navigateTo({ tab: 'nodes', resourceName: nodeName });
  }

  // Refresh interval timer
  let refreshTimer: number | null = null;

  // Load cluster metrics
  async function loadClusterMetrics() {
    if (loading) return;
    
    loading = true;
    error = null;
    
    try {
      const metrics = await invoke('kuboard_get_cluster_metrics');
      clusterMetrics = metrics;
      lastUpdate = new Date().toLocaleTimeString();
    } catch (err) {
      error = err as string;
      console.error('Failed to load cluster metrics:', err);
    } finally {
      loading = false;
    }
  }

  // Format bytes to human readable
  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  // Format CPU cores
  function formatCores(cores: number): string {
    return cores.toFixed(1) + ' cores';
  }

  // Get color based on usage percentage
  function getUsageColor(percentage: number): string {
    if (percentage >= 90) return '#ef4444'; // Red
    if (percentage >= 75) return '#f59e0b'; // Orange
    if (percentage >= 50) return '#06b6d4'; // Cyan
    return '#10b981'; // Green
  }

  // Start auto-refresh
  function startAutoRefresh() {
    if (refreshTimer) {
      clearInterval(refreshTimer);
    }
    
    if (autoRefresh && refreshInterval > 0) {
      refreshTimer = setInterval(loadClusterMetrics, refreshInterval);
    }
  }

  // Stop auto-refresh
  function stopAutoRefresh() {
    if (refreshTimer) {
      clearInterval(refreshTimer);
      refreshTimer = null;
    }
  }

  // Toggle auto-refresh
  function toggleAutoRefresh() {
    autoRefresh = !autoRefresh;
    if (autoRefresh) {
      startAutoRefresh();
    } else {
      stopAutoRefresh();
    }
  }

  // Manual refresh
  function manualRefresh() {
    loadClusterMetrics();
  }

  // Lifecycle
  onMount(() => {
    loadClusterMetrics();
    startAutoRefresh();
    
    return () => {
      stopAutoRefresh();
    };
  });

  // Reactive updates
  $: if (autoRefresh && refreshInterval > 0) {
    startAutoRefresh();
  }
</script>

<div class="cluster-metrics">
  <div class="metrics-header">
    <h3><Activity size={18} /> Cluster Resource Usage</h3>
    <div class="metrics-controls">
      <button 
        class="refresh-button" 
        onclick={manualRefresh}
        disabled={loading}
        title="Refresh metrics"
      >
        {#if loading}
          🔄
        {:else}
          ↻
        {/if}
      </button>
      <button 
        class="auto-refresh-toggle"
        class:enabled={autoRefresh}
        onclick={toggleAutoRefresh}
        title={autoRefresh ? 'Disable auto-refresh' : 'Enable auto-refresh'}
      >
        {autoRefresh ? '⏸️' : '▶️'}
      </button>
      {#if lastUpdate}
        <span class="last-update">Last: {lastUpdate}</span>
      {/if}
    </div>
  </div>

  {#if error}
    <div class="error-message">
      <div class="error-icon">⚠️</div>
      <div class="error-content">
        <h4>Failed to load cluster metrics</h4>
        <p>{error}</p>
        <button class="retry-button" onclick={manualRefresh}>
          Retry
        </button>
      </div>
    </div>
  {:else if clusterMetrics}
    {#if !clusterMetrics.metrics_available}
      <div class="no-metrics-notice">
        <div class="notice-icon">⚠️</div>
        <div class="notice-body">
          <h4>No Metrics Available</h4>
          <p>The Metrics Server is not detected on your cluster. Mock metrics are disabled.</p>
          <div class="setup-hint">
            <strong>To enable metrics on Minikube:</strong>
            <code>minikube addons enable metrics-server</code>
          </div>
        </div>
      </div>
    {:else}
      <div class="metrics-grid">
        <!-- CPU Usage -->
        <div class="metric-card">
          <div class="metric-header">
            <h4>CPU Usage</h4>
            <div class="metric-status">
              <span class="status-badge real">Real-time</span>
            </div>
          </div>
          <div class="metric-content">
            <DonutChart
              value={clusterMetrics.cpu.usage_percent}
              label="CPU"
              color={getUsageColor(clusterMetrics.cpu.usage_percent)}
              size={100}
              strokeWidth={6}
            />
            <div class="metric-details">
              <div class="detail-item">
                <span class="detail-label">Used:</span>
                <span class="detail-value">{formatCores(clusterMetrics.cpu.used_cores)}</span>
              </div>
              <div class="detail-item">
                <span class="detail-label">Total:</span>
                <span class="detail-value">{formatCores(clusterMetrics.cpu.total_cores)}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Memory Usage -->
        <div class="metric-card">
          <div class="metric-header">
            <h4>Memory Usage</h4>
            <div class="metric-status">
              <span class="status-badge real">Real-time</span>
            </div>
          </div>
          <div class="metric-content">
            <DonutChart
              value={clusterMetrics.memory.usage_percent}
              label="Memory"
              color={getUsageColor(clusterMetrics.memory.usage_percent)}
              size={100}
              strokeWidth={6}
            />
            <div class="metric-details">
              <div class="detail-item">
                <span class="detail-label">Used:</span>
                <span class="detail-value">{formatBytes(clusterMetrics.memory.used_bytes)}</span>
              </div>
              <div class="detail-item">
                <span class="detail-label">Total:</span>
                <span class="detail-value">{formatBytes(clusterMetrics.memory.total_bytes)}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Disk Usage -->
        <div class="metric-card">
          <div class="metric-header">
            <h4>Disk Usage</h4>
            <div class="metric-status">
              <span class="status-badge real">Real-time</span>
            </div>
          </div>
          <div class="metric-content">
            <DonutChart
              value={clusterMetrics.disk.usage_percent}
              label="Disk"
              color={getUsageColor(clusterMetrics.disk.usage_percent)}
              size={100}
              strokeWidth={6}
            />
            <div class="metric-details">
              <div class="detail-item">
                <span class="detail-label">Used:</span>
                <span class="detail-value">{formatBytes(clusterMetrics.disk.used_bytes)}</span>
              </div>
              <div class="detail-item">
                <span class="detail-label">Total:</span>
                <span class="detail-value">{formatBytes(clusterMetrics.disk.total_bytes)}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    {/if}

    <!-- Cluster Summary -->
    <div class="cluster-summary">
      <div class="summary-item">
        <span class="summary-label">Nodes:</span>
        <span class="summary-value">{clusterMetrics.nodes_count}</span>
      </div>
      <div class="summary-item">
        <span class="summary-label">Node Pools:</span>
        <span class="summary-value">{nodePools.length}</span>
      </div>
      <div class="summary-item">
        <span class="summary-label">Data Source:</span>
        <span class="summary-value">
          {clusterMetrics.metrics_available ? 'Metrics Server' : 'Pod Requests'}
        </span>
      </div>
    </div>

    <!-- Node Pools Section -->
    <div class="node-pools-section">
      {#if activePool}
        <!-- Node Pool Detailed View -->
        <div class="nodepool-detail-view">
          <div class="nodepool-header">
            <button class="back-to-pools-btn" onclick={() => selectedPoolName = null}>
              ← Back to Node Pools
            </button>
            <h4><Layers size={16} /> Node Pool: <span class="highlight-pool-name">{activePool.name}</span></h4>
            <div class="nodepool-badges">
              <span class="pool-badge">{activePool.readyCount} / {activePool.nodes.length} Nodes Ready</span>
              {#if activePool.instanceTypesString}
                <span class="pool-badge instance-type">{activePool.instanceTypesString}</span>
              {/if}
            </div>
          </div>

          <div class="pool-summary-cards">
            <div class="pool-stat-card">
              <span class="stat-label">Total CPU Capacity</span>
              <span class="stat-value">{formatCores(activePool.totalCpuCapacity)}</span>
            </div>
            <div class="pool-stat-card">
              <span class="stat-label">Total Memory Capacity</span>
              <span class="stat-value">{formatBytes(activePool.totalMemCapacity)}</span>
            </div>
          </div>

          <h5>Constituent Nodes in Pool</h5>
          <div class="nodepool-nodes-table-wrapper">
            <table class="nodepool-nodes-table">
              <thead>
                <tr>
                  <th>Node Name</th>
                  <th>Status</th>
                  <th>Internal IP</th>
                  <th>Kubelet Version</th>
                  <th>OS / Arch</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                {#each activePool.nodes as node}
                  {@const isReady = node.status?.conditions?.some((c) => c.type === 'Ready' && c.status === 'True')}
                  {@const internalIp = node.status?.addresses?.find((a) => a.type === 'InternalIP')?.address || 'N/A'}
                  <tr>
                    <td class="node-name-cell">
                      <strong>{node.metadata?.name}</strong>
                    </td>
                    <td>
                      <span class="status-badge {isReady ? 'real' : 'estimated'}">
                        {isReady ? 'Ready' : 'NotReady'}
                      </span>
                    </td>
                    <td><code>{internalIp}</code></td>
                    <td>{node.status?.nodeInfo?.kubeletVersion || 'N/A'}</td>
                    <td>{node.status?.nodeInfo?.operatingSystem} / {node.status?.nodeInfo?.architecture}</td>
                    <td>
                      <button class="btn-view-node" onclick={() => jumpToNode(node.metadata?.name)}>
                        🖥️ View Node Details
                      </button>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </div>
      {:else}
        <!-- Node Pools Summary List -->
        <div class="nodepools-header">
          <h4><Layers size={16} /> Cluster Node Pools ({nodePools.length})</h4>
          <span class="nodepools-subtitle">Click a node pool to inspect constituent nodes & capacity</span>
        </div>

        {#if nodePools.length === 0}
          <div class="nodepools-empty">No node pools discovered or nodes loading...</div>
        {:else}
          <div class="nodepools-grid">
            {#each nodePools as pool}
              <div 
                class="nodepool-card" 
                role="button" 
                tabindex="0"
                onclick={() => selectedPoolName = pool.name}
                onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && (selectedPoolName = pool.name)}
              >
                <div class="nodepool-card-header">
                  <span class="pool-name"><Layers size={14} /> {pool.name}</span>
                  <span class="pool-status-badge {pool.readyCount === pool.nodes.length ? 'status-ok' : 'status-warn'}">
                    {pool.readyCount} / {pool.nodes.length} Ready
                  </span>
                </div>
                <div class="nodepool-card-body">
                  <div class="pool-metric-row">
                    <span class="metric-label">Nodes:</span>
                    <span class="metric-val">{pool.nodes.length}</span>
                  </div>
                  <div class="pool-metric-row">
                    <span class="metric-label">Total CPU:</span>
                    <span class="metric-val">{formatCores(pool.totalCpuCapacity)}</span>
                  </div>
                  <div class="pool-metric-row">
                    <span class="metric-label">Total Memory:</span>
                    <span class="metric-val">{formatBytes(pool.totalMemCapacity)}</span>
                  </div>
                  {#if pool.instanceTypesString}
                    <div class="pool-metric-row">
                      <span class="metric-label">Instance Type:</span>
                      <span class="metric-val code-val">{pool.instanceTypesString}</span>
                    </div>
                  {/if}
                </div>
                <div class="nodepool-card-footer">
                  <span>View Details & Nodes →</span>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      {/if}
    </div>
  {:else if loading}
    <div class="loading-message">
      <div class="loading-spinner">🔄</div>
      <p>Loading cluster metrics...</p>
    </div>
  {/if}
</div>

<style>
  /* Import CSS variables */
  @import '../styles/variables.css';

  .cluster-metrics {
    background: rgba(255, 255, 255, 0.05);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
    border: 1px solid rgba(255, 255, 255, 0.1);
    margin-bottom: var(--spacing-lg);
  }

  .metrics-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-lg);
    padding-bottom: var(--spacing-md);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .metrics-header h3 {
    margin: 0;
    color: white;
    font-size: 1.2rem;
    font-weight: 600;
  }

  .metrics-controls {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .refresh-button, .auto-refresh-toggle {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-sm);
    color: white;
    cursor: pointer;
    font-size: 0.9rem;
    padding: 6px 12px;
    transition: var(--transition-normal);
  }

  .refresh-button:hover, .auto-refresh-toggle:hover {
    background: rgba(255, 255, 255, 0.2);
    border-color: rgba(255, 255, 255, 0.3);
  }

  .refresh-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .auto-refresh-toggle.enabled {
    background: var(--accent-color);
    border-color: var(--accent-color);
  }

  .last-update {
    font-size: 0.8rem;
    color: rgba(255, 255, 255, 0.6);
  }

  .error-message {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: var(--radius-md);
  }

  .error-icon {
    font-size: 1.2rem;
    flex-shrink: 0;
  }

  .error-content h4 {
    margin: 0 0 4px 0;
    color: white;
    font-size: 0.9rem;
    font-weight: 600;
  }

  .error-content p {
    margin: 0 0 8px 0;
    color: rgba(255, 255, 255, 0.8);
    font-size: 0.85rem;
  }

  .retry-button {
    background: var(--error-color);
    border: none;
    border-radius: var(--radius-sm);
    color: white;
    cursor: pointer;
    font-size: 0.8rem;
    padding: 4px 8px;
    transition: var(--transition-normal);
  }

  .retry-button:hover {
    background: #dc2626;
  }

  .loading-message {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-xl);
    color: rgba(255, 255, 255, 0.8);
  }

  .loading-spinner {
    font-size: 1.5rem;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: var(--spacing-lg);
    margin-bottom: var(--spacing-lg);
  }

  .metric-card {
    background: rgba(255, 255, 255, 0.03);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .metric-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-md);
  }

  .metric-header h4 {
    margin: 0;
    color: white;
    font-size: 1rem;
    font-weight: 600;
  }

  .status-badge {
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .status-badge.real {
    background: var(--status-ready-bg);
    color: var(--status-ready-text);
    border: 1px solid var(--status-ready-border);
    font-weight: 600;
  }

  .status-badge.estimated {
    background: rgba(245, 158, 11, 0.2);
    color: #f59e0b;
  }

  .metric-content {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .metric-details {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .detail-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .detail-label {
    color: rgba(255, 255, 255, 0.7);
    font-size: 0.8rem;
    font-weight: 500;
  }

  .detail-value {
    color: white;
    font-size: 0.8rem;
    font-weight: 600;
  }

  .cluster-summary {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-md);
    background: rgba(255, 255, 255, 0.03);
    border-radius: var(--radius-md);
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .summary-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .summary-label {
    color: rgba(255, 255, 255, 0.7);
    font-size: 0.9rem;
    font-weight: 500;
  }

  .summary-value {
    color: white;
    font-size: 0.9rem;
    font-weight: 600;
  }

  .no-metrics-notice {
    display: flex;
    gap: 16px;
    align-items: flex-start;
    background: rgba(245, 158, 11, 0.08);
    border: 1px solid rgba(245, 158, 11, 0.25);
    border-radius: var(--radius-md);
    padding: 16px;
    margin-bottom: 16px;
  }

  .notice-icon {
    font-size: 2rem;
  }

  .notice-body h4 {
    margin: 0 0 6px 0;
    color: #f59e0b;
    font-size: 1.1rem;
  }

  .notice-body p {
    margin: 0 0 10px 0;
    color: var(--text-secondary);
    font-size: 0.9rem;
  }

  .setup-hint {
    font-size: 0.85rem;
    color: var(--text-primary);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .setup-hint code {
    background: rgba(0, 0, 0, 0.4);
    padding: 4px 8px;
    border-radius: 4px;
    font-family: monospace;
    color: var(--success-color);
    width: fit-content;
  }

  /* Responsive Design */
  @media (max-width: 768px) {
    .metrics-grid {
      grid-template-columns: 1fr;
    }
    
    .metric-content {
      flex-direction: column;
      text-align: center;
    }
    
    .cluster-summary {
      flex-direction: column;
      gap: var(--spacing-sm);
    }
  }

  /* Node Pools Section Styles */
  .node-pools-section {
    margin-top: 24px;
    padding-top: 20px;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
  }

  .nodepools-header h4, .nodepool-header h4 {
    margin: 0 0 4px 0;
    color: white;
    font-size: 1.1rem;
  }

  .nodepools-subtitle {
    color: var(--text-secondary);
    font-size: 0.85rem;
  }

  .nodepools-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 16px;
    margin-top: 16px;
  }

  .nodepool-card {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: var(--radius-md);
    padding: 16px;
    cursor: pointer;
    transition: all 0.2s ease;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
  }

  .nodepool-card:hover {
    background: rgba(255, 255, 255, 0.07);
    border-color: var(--primary-color);
    transform: translateY(-2px);
  }

  .nodepool-card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }

  .pool-name {
    font-weight: 600;
    color: white;
    font-size: 1rem;
  }

  .pool-status-badge {
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .pool-status-badge.status-ok {
    background: rgba(16, 185, 129, 0.15);
    color: #10b981;
    border: 1px solid rgba(16, 185, 129, 0.3);
  }

  .pool-status-badge.status-warn {
    background: rgba(245, 158, 11, 0.15);
    color: #f59e0b;
    border: 1px solid rgba(245, 158, 11, 0.3);
  }

  .pool-metric-row {
    display: flex;
    justify-content: space-between;
    font-size: 0.85rem;
    padding: 3px 0;
  }

  .metric-label {
    color: var(--text-secondary);
  }

  .metric-val {
    color: white;
    font-weight: 500;
  }

  .code-val {
    font-family: monospace;
    font-size: 0.8rem;
    background: rgba(0, 0, 0, 0.3);
    padding: 2px 6px;
    border-radius: 4px;
  }

  .nodepool-card-footer {
    margin-top: 14px;
    padding-top: 8px;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
    font-size: 0.8rem;
    color: var(--primary-color);
    text-align: right;
    font-weight: 500;
  }

  .back-to-pools-btn {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.15);
    color: white;
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: 0.85rem;
    margin-bottom: 12px;
    transition: all 0.2s ease;
  }

  .back-to-pools-btn:hover {
    background: rgba(255, 255, 255, 0.12);
  }

  .highlight-pool-name {
    color: var(--primary-color);
  }

  .nodepool-badges {
    display: flex;
    gap: 8px;
    margin-top: 6px;
  }

  .pool-badge {
    background: rgba(46, 145, 190, 0.15);
    border: 1px solid rgba(46, 145, 190, 0.3);
    color: var(--primary-color);
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 0.8rem;
  }

  .pool-summary-cards {
    display: flex;
    gap: 16px;
    margin: 16px 0;
  }

  .pool-stat-card {
    flex: 1;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: var(--radius-md);
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .stat-label {
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  .stat-value {
    font-size: 1.2rem;
    font-weight: 600;
    color: white;
  }

  .nodepool-nodes-table-wrapper {
    overflow-x: auto;
    margin-top: 12px;
  }

  .nodepool-nodes-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.85rem;
  }

  .nodepool-nodes-table th, .nodepool-nodes-table td {
    padding: 10px 12px;
    text-align: left;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  }

  .nodepool-nodes-table th {
    color: var(--text-secondary);
    font-weight: 600;
    background: rgba(255, 255, 255, 0.02);
  }

  .btn-view-node {
    background: rgba(46, 145, 190, 0.15);
    border: 1px solid rgba(46, 145, 190, 0.3);
    color: var(--primary-color);
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: 0.8rem;
    transition: all 0.2s ease;
  }

  .btn-view-node:hover {
    background: var(--primary-color);
    color: white;
  }
</style>
