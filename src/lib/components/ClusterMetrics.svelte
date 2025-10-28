<!-- Kuboard Cluster Metrics Component -->
<script lang="ts">
  import DonutChart from './DonutChart.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  export let refreshInterval: number = 10000; // 10 seconds default
  export let autoRefresh: boolean = true;

  // Cluster metrics data
  let clusterMetrics: any = null;
  let loading: boolean = false;
  let error: string | null = null;
  let lastUpdate: string = '';

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
    <h3>📊 Cluster Resource Usage</h3>
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
    <div class="metrics-grid">
      <!-- CPU Usage -->
      <div class="metric-card">
        <div class="metric-header">
          <h4>CPU Usage</h4>
          <div class="metric-status">
            {#if clusterMetrics.metrics_available}
              <span class="status-badge real">Real-time</span>
            {:else}
              <span class="status-badge estimated">Estimated</span>
            {/if}
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
            {#if clusterMetrics.metrics_available}
              <span class="status-badge real">Real-time</span>
            {:else}
              <span class="status-badge estimated">Estimated</span>
            {/if}
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
            {#if clusterMetrics.metrics_available}
              <span class="status-badge real">Real-time</span>
            {:else}
              <span class="status-badge estimated">Estimated</span>
            {/if}
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

    <!-- Cluster Summary -->
    <div class="cluster-summary">
      <div class="summary-item">
        <span class="summary-label">Nodes:</span>
        <span class="summary-value">{clusterMetrics.nodes_count}</span>
      </div>
      <div class="summary-item">
        <span class="summary-label">Data Source:</span>
        <span class="summary-value">
          {clusterMetrics.metrics_available ? 'Metrics Server' : 'Pod Requests'}
        </span>
      </div>
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
</style>
