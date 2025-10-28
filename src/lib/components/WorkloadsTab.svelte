<!-- Kuboard Workloads Tab Component -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  // Props
  export let currentContext: any = null;

  // State
  let pods: any[] = [];
  let deployments: any[] = [];
  let services: any[] = [];
  let replicasets: any[] = [];
  let daemonsets: any[] = [];
  let statefulsets: any[] = [];
  let jobs: any[] = [];
  let cronjobs: any[] = [];
  
  let loading: boolean = false;
  let error: string | null = null;
  let lastUpdate: string = '';

  // Load workloads data
  async function loadWorkloads() {
    if (!currentContext || loading) return;
    
    loading = true;
    error = null;
    
    try {
      // Load all workload types in parallel
      const [podsData, deploymentsData, servicesData] = await Promise.all([
        invoke('kuboard_get_pods').catch(() => []),
        invoke('kuboard_get_deployments').catch(() => []),
        invoke('kuboard_get_services').catch(() => [])
      ]);

      pods = podsData as any[] || [];
      deployments = deploymentsData as any[] || [];
      services = servicesData as any[] || [];
      
      lastUpdate = new Date().toLocaleTimeString();
    } catch (err) {
      error = err as string;
      console.error('Failed to load workloads:', err);
    } finally {
      loading = false;
    }
  }

  // Get status class for workload
  function getStatusClass(status: string): string {
    return status.toLowerCase().replace(/\s+/g, '-');
  }

  // Format age
  function formatAge(creationTimestamp: string): string {
    if (!creationTimestamp) return 'Unknown';
    const created = new Date(creationTimestamp);
    const now = new Date();
    const diffMs = now.getTime() - created.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMins / 60);
    const diffDays = Math.floor(diffHours / 24);
    
    if (diffDays > 0) return `${diffDays}d`;
    if (diffHours > 0) return `${diffHours}h`;
    return `${diffMins}m`;
  }

  // Get pod status
  function getPodStatus(pod: any): string {
    if (pod.status?.phase) return pod.status.phase;
    return 'Unknown';
  }

  // Get deployment status
  function getDeploymentStatus(deployment: any): string {
    const conditions = deployment.status?.conditions || [];
    const available = conditions.find((c: any) => c.type === 'Available');
    if (available?.status === 'True') return 'Available';
    return 'Not Available';
  }

  // Get service type
  function getServiceType(service: any): string {
    return service.spec?.type || 'ClusterIP';
  }

  // Lifecycle
  onMount(() => {
    loadWorkloads();
  });

  // Reactive updates
  $: if (currentContext) {
    loadWorkloads();
  }
</script>

<div class="workloads-tab">
  <div class="tab-header">
    <h4>⚙️ Workloads</h4>
    <div class="tab-controls">
      <button 
        class="refresh-button" 
        onclick={loadWorkloads}
        disabled={loading}
        title="Refresh workloads"
      >
        {#if loading}
          🔄
        {:else}
          ↻
        {/if}
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
        <h5>Failed to load workloads</h5>
        <p>{error}</p>
        <button class="retry-button" onclick={loadWorkloads}>
          Retry
        </button>
      </div>
    </div>
  {:else if loading}
    <div class="loading-message">
      <div class="loading-spinner">🔄</div>
      <p>Loading workloads...</p>
    </div>
  {:else}
    <div class="workloads-grid">
      <!-- Pods Section -->
      <div class="workload-section">
        <div class="section-header">
          <h5>Pods ({pods.length})</h5>
        </div>
        <div class="workload-list">
          {#each pods.slice(0, 10) as pod}
            <div class="workload-item">
              <div class="workload-info">
                <span class="workload-name">{pod.metadata?.name || 'Unknown'}</span>
                <span class="workload-namespace">{pod.metadata?.namespace || 'default'}</span>
              </div>
              <div class="workload-status">
                <span class="status-badge status-{getStatusClass(getPodStatus(pod))}">
                  {getPodStatus(pod)}
                </span>
                <span class="workload-age">{formatAge(pod.metadata?.creationTimestamp)}</span>
              </div>
            </div>
          {/each}
          {#if pods.length > 10}
            <div class="workload-more">
              <span>... and {pods.length - 10} more</span>
            </div>
          {/if}
        </div>
      </div>

      <!-- Deployments Section -->
      <div class="workload-section">
        <div class="section-header">
          <h5>Deployments ({deployments.length})</h5>
        </div>
        <div class="workload-list">
          {#each deployments.slice(0, 10) as deployment}
            <div class="workload-item">
              <div class="workload-info">
                <span class="workload-name">{deployment.metadata?.name || 'Unknown'}</span>
                <span class="workload-namespace">{deployment.metadata?.namespace || 'default'}</span>
              </div>
              <div class="workload-status">
                <span class="status-badge status-{getStatusClass(getDeploymentStatus(deployment))}">
                  {getDeploymentStatus(deployment)}
                </span>
                <span class="workload-age">{formatAge(deployment.metadata?.creationTimestamp)}</span>
              </div>
            </div>
          {/each}
          {#if deployments.length > 10}
            <div class="workload-more">
              <span>... and {deployments.length - 10} more</span>
            </div>
          {/if}
        </div>
      </div>

      <!-- Services Section -->
      <div class="workload-section">
        <div class="section-header">
          <h5>Services ({services.length})</h5>
        </div>
        <div class="workload-list">
          {#each services.slice(0, 10) as service}
            <div class="workload-item">
              <div class="workload-info">
                <span class="workload-name">{service.metadata?.name || 'Unknown'}</span>
                <span class="workload-namespace">{service.metadata?.namespace || 'default'}</span>
              </div>
              <div class="workload-status">
                <span class="status-badge status-{getStatusClass(getServiceType(service))}">
                  {getServiceType(service)}
                </span>
                <span class="workload-age">{formatAge(service.metadata?.creationTimestamp)}</span>
              </div>
            </div>
          {/each}
          {#if services.length > 10}
            <div class="workload-more">
              <span>... and {services.length - 10} more</span>
            </div>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  /* Import CSS variables */
  @import '../styles/variables.css';

  .workloads-tab {
    background: rgba(255, 255, 255, 0.03);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .tab-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-md);
    padding-bottom: var(--spacing-sm);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .tab-header h4 {
    margin: 0;
    color: white;
    font-size: 1.1rem;
    font-weight: 600;
  }

  .tab-controls {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .refresh-button {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-sm);
    color: white;
    cursor: pointer;
    font-size: 0.9rem;
    padding: 6px 12px;
    transition: var(--transition-normal);
  }

  .refresh-button:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.2);
    border-color: rgba(255, 255, 255, 0.3);
  }

  .refresh-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
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

  .error-content h5 {
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

  .workloads-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: var(--spacing-lg);
  }

  .workload-section {
    background: rgba(255, 255, 255, 0.02);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .section-header {
    margin-bottom: var(--spacing-md);
  }

  .section-header h5 {
    margin: 0;
    color: white;
    font-size: 1rem;
    font-weight: 600;
  }

  .workload-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .workload-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-sm);
    background: rgba(255, 255, 255, 0.03);
    border-radius: var(--radius-sm);
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .workload-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .workload-name {
    color: white;
    font-size: 0.9rem;
    font-weight: 600;
  }

  .workload-namespace {
    color: rgba(255, 255, 255, 0.6);
    font-size: 0.8rem;
  }

  .workload-status {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 2px;
  }

  .status-badge {
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .status-running, .status-available, .status-clusterip {
    background: rgba(16, 185, 129, 0.2);
    color: var(--primary-color);
  }

  .status-pending, .status-not-available {
    background: rgba(245, 158, 11, 0.2);
    color: #f59e0b;
  }

  .status-failed, .status-error {
    background: rgba(239, 68, 68, 0.2);
    color: var(--error-color);
  }

  .status-loadbalancer, .status-nodeport {
    background: rgba(59, 130, 246, 0.2);
    color: #3b82f6;
  }

  .workload-age {
    color: rgba(255, 255, 255, 0.6);
    font-size: 0.7rem;
  }

  .workload-more {
    text-align: center;
    padding: var(--spacing-sm);
    color: rgba(255, 255, 255, 0.6);
    font-size: 0.8rem;
    font-style: italic;
  }

  /* Responsive Design */
  @media (max-width: 768px) {
    .workloads-grid {
      grid-template-columns: 1fr;
    }
    
    .workload-item {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--spacing-sm);
    }
    
    .workload-status {
      align-items: flex-start;
    }
  }
</style>
