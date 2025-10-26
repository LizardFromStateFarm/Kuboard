<!-- Kuboard Metrics Graph Component -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Chart, registerables } from 'chart.js';
  import type { MetricsDataPoint, ResourceTab } from '../types/index.js';
  import { createEventDispatcher } from 'svelte';

  // Register Chart.js components
  Chart.register(...registerables);

  // Props
  export let data: MetricsDataPoint[] = [];
  export let type: ResourceTab = 'cpu';
  export let duration: number = 30; // minutes
  export let autoRefresh: boolean = true;
  export let loading: boolean = false;
  export let error: string | null = null;

  // Events
  const dispatch = createEventDispatcher();

  // Chart instance
  let chartInstance: Chart | null = null;
  let chartCanvas: HTMLCanvasElement;

  // Initialize chart
  onMount(() => {
    if (chartCanvas && data.length > 0) {
      initializeChart();
    }
  });

  // Cleanup
  onDestroy(() => {
    if (chartInstance) {
      chartInstance.destroy();
    }
  });

  // Initialize Chart.js
  function initializeChart() {
    if (chartInstance) {
      chartInstance.destroy();
    }

    const ctx = chartCanvas.getContext('2d');
    if (!ctx) return;

    const chartData = {
      labels: data.map(point => new Date(point.timestamp * 1000).toLocaleTimeString()),
      datasets: [{
        label: getResourceLabel(type),
        data: data.map(point => getResourceValue(point, type)),
        borderColor: getResourceColor(type),
        backgroundColor: getResourceColor(type, 0.1),
        borderWidth: 2,
        fill: true,
        tension: 0.4,
        pointRadius: 3,
        pointHoverRadius: 6,
        pointBackgroundColor: getResourceColor(type),
        pointBorderColor: '#ffffff',
        pointBorderWidth: 2
      }]
    };

    chartInstance = new Chart(ctx, {
      type: 'line',
      data: chartData,
      options: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          legend: {
            display: false
          },
          tooltip: {
            backgroundColor: 'rgba(0, 0, 0, 0.8)',
            titleColor: '#ffffff',
            bodyColor: '#ffffff',
            borderColor: getResourceColor(type),
            borderWidth: 1,
            callbacks: {
              label: function(context) {
                return `${getResourceLabel(type)}: ${context.parsed.y.toFixed(1)}%`;
              }
            }
          }
        },
        scales: {
          x: {
            display: true,
            grid: {
              color: 'rgba(255, 255, 255, 0.1)',
              drawBorder: false
            },
            ticks: {
              color: 'rgba(255, 255, 255, 0.7)',
              maxTicksLimit: 8
            }
          },
          y: {
            display: true,
            min: 0,
            max: 100,
            grid: {
              color: 'rgba(255, 255, 255, 0.1)',
              drawBorder: false
            },
            ticks: {
              color: 'rgba(255, 255, 255, 0.7)',
              callback: function(value) {
                return value + '%';
              }
            }
          }
        },
        interaction: {
          intersect: false,
          mode: 'index'
        },
        animation: {
          duration: 750,
          easing: 'easeInOutQuart'
        }
      }
    });
  }

  // Update chart when data changes
  $: if (chartInstance && data.length > 0) {
    updateChart();
  }

  // Update chart when type changes
  $: if (chartInstance) {
    initializeChart();
  }

  function updateChart() {
    if (!chartInstance) return;

    const newData = {
      labels: data.map(point => new Date(point.timestamp * 1000).toLocaleTimeString()),
      datasets: [{
        ...chartInstance.data.datasets[0],
        label: getResourceLabel(type),
        data: data.map(point => getResourceValue(point, type)),
        borderColor: getResourceColor(type),
        backgroundColor: getResourceColor(type, 0.1)
      }]
    };

    chartInstance.data = newData;
    chartInstance.update('active');
  }

  // Helper functions
  function getResourceLabel(type: ResourceTab): string {
    switch (type) {
      case 'cpu': return 'CPU Usage';
      case 'memory': return 'Memory Usage';
      case 'disk': return 'Disk Usage';
      default: return 'Usage';
    }
  }

  function getResourceValue(point: MetricsDataPoint, type: ResourceTab): number {
    switch (type) {
      case 'cpu': return point.cpu_usage_percent;
      case 'memory': return point.memory_usage_percent;
      case 'disk': return point.disk_usage_percent;
      default: return 0;
    }
  }

  function getResourceColor(type: ResourceTab, alpha: number = 1): string {
    const colors = {
      cpu: `rgba(16, 185, 129, ${alpha})`,
      memory: `rgba(59, 130, 246, ${alpha})`,
      disk: `rgba(245, 158, 11, ${alpha})`
    };
    return colors[type] || `rgba(255, 255, 255, ${alpha})`;
  }

  function formatTime(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleTimeString();
  }

  function getCurrentValue(): number {
    if (data.length === 0) return 0;
    const latest = data[data.length - 1];
    return getResourceValue(latest, type);
  }

  // Helper function for resource icons
  function getResourceIcon(type: ResourceTab): string {
    switch (type) {
      case 'cpu': return '🖥️';
      case 'memory': return '💾';
      case 'disk': return '💿';
      default: return '📊';
    }
  }
</script>

<div class="metrics-graph">
  <div class="graph-header">
    <div class="graph-title">
      <span class="resource-icon">{getResourceIcon(type)}</span>
      {getResourceLabel(type)}
    </div>
    <div class="graph-stats">
      <div class="current-value">
        {getCurrentValue().toFixed(1)}%
      </div>
      {#if loading}
        <div class="loading-indicator">🔄</div>
      {/if}
    </div>
  </div>

  <div class="graph-container">
    {#if error}
      <div class="error-state">
        <div class="error-icon">⚠️</div>
        <div class="error-content">
          <h6>Graph Error</h6>
          <p>{error}</p>
          <button class="retry-button" onclick={() => dispatch('retry')}>
            Retry
          </button>
        </div>
      </div>
    {:else if data.length === 0}
      <div class="no-data-state">
        <div class="no-data-icon">📊</div>
        <div class="no-data-content">
          <h6>No Data Available</h6>
          <p>No metrics data to display</p>
        </div>
      </div>
    {:else}
      <div class="chart-wrapper">
        <canvas bind:this={chartCanvas} class="chart-canvas"></canvas>
        {#if autoRefresh}
          <div class="live-indicator">
            <span class="pulse-dot"></span>
            Live Data
          </div>
        {/if}
      </div>
    {/if}
  </div>

  <div class="graph-footer">
    <div class="time-range">
      Last {duration} minutes
    </div>
    <div class="data-points">
      {data.length} data points
    </div>
  </div>
</div>

<style>
  /* Import CSS variables */
  @import '../styles/variables.css';

  .metrics-graph {
    background: var(--card-background);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
    border: 1px solid var(--border-color);
    box-shadow: var(--shadow-md);
  }

  .graph-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-md);
    padding-bottom: var(--spacing-sm);
    border-bottom: 1px solid var(--border-color);
  }

  .graph-title {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    color: var(--text-color);
    font-size: 1.1em;
    font-weight: 600;
  }

  .resource-icon {
    font-size: 1.2em;
  }

  .graph-stats {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .current-value {
    color: var(--primary-color);
    font-size: 1.5em;
    font-weight: 700;
  }

  .loading-indicator {
    animation: spin 1s linear infinite;
    font-size: 1.2em;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .graph-container {
    position: relative;
    height: 300px;
    margin: var(--spacing-md) 0;
  }

  .chart-wrapper {
    position: relative;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.2);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .chart-canvas {
    width: 100% !important;
    height: 100% !important;
  }

  .live-indicator {
    position: absolute;
    top: var(--spacing-sm);
    right: var(--spacing-sm);
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    background: rgba(16, 185, 129, 0.2);
    color: var(--primary-color);
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    font-size: 0.8em;
    font-weight: 600;
  }

  .pulse-dot {
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

  .graph-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-top: var(--spacing-sm);
    border-top: 1px solid var(--border-color);
    color: var(--text-muted);
    font-size: 0.85em;
  }

  /* Error and No Data States */
  .error-state, .no-data-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: var(--spacing-md);
  }

  .error-icon, .no-data-icon {
    font-size: 2em;
    opacity: 0.6;
  }

  .error-content, .no-data-content {
    text-align: center;
  }

  .error-content h6, .no-data-content h6 {
    margin: 0 0 var(--spacing-xs) 0;
    color: var(--text-color);
    font-size: 1em;
    font-weight: 600;
  }

  .error-content p, .no-data-content p {
    margin: 0 0 var(--spacing-sm) 0;
    color: var(--text-muted);
    font-size: 0.9em;
  }

  .retry-button {
    background: var(--primary-color);
    border: none;
    border-radius: var(--radius-sm);
    color: white;
    cursor: pointer;
    font-size: 0.8em;
    padding: 6px 12px;
    transition: var(--transition-normal);
  }

  .retry-button:hover {
    background: var(--accent-color);
    transform: translateY(-1px);
  }

  /* Responsive Design */
  @media (max-width: 768px) {
    .graph-container {
      height: 250px;
    }
    
    .graph-header {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--spacing-sm);
    }
    
    .graph-stats {
      align-self: flex-end;
    }
    
    .current-value {
      font-size: 1.2em;
    }
  }

  @media (max-width: 480px) {
    .metrics-graph {
      padding: var(--spacing-md);
    }
    
    .graph-container {
      height: 200px;
    }
    
    .graph-footer {
      flex-direction: column;
      gap: var(--spacing-xs);
      text-align: center;
    }
  }
</style>
