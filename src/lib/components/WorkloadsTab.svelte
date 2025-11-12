<!-- Kuboard Workloads Tab Component -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import PodsPanel from './PodsPanel.svelte';
  import ReplicaSetsPanel from './ReplicaSetsPanel.svelte';
  import DeploymentsPanel from './DeploymentsPanel.svelte';
  import StatefulSetsPanel from './StatefulSetsPanel.svelte';
  import DaemonSetsPanel from './DaemonSetsPanel.svelte';
  import CronJobsPanel from './CronJobsPanel.svelte';

  // Props
  export let currentContext: any = null;

  // State
  let pods: any[] = [];
  let deployments: any[] = [];
  let services: any[] = [];
  let replicasets: any[] = [];
  let statefulsets: any[] = [];
  let daemonsets: any[] = [];
  let cronjobs: any[] = [];
  
  let loading: boolean = false;
  let error: string | null = null;
  let lastUpdate: string = '';
  let selectedWorkloadType: string = 'pods'; // Default to pods
  let loadedTypes: Set<string> = new Set(); // Track which types have been loaded

  // Workload type definitions
  const workloadTypes = [
    { id: 'pods', label: 'Pods', icon: '🟢', description: 'Running containers' },
    { id: 'deployments', label: 'Deployments', icon: '📦', description: 'Deployment controllers' },
    { id: 'statefulsets', label: 'StatefulSets', icon: '📋', description: 'StatefulSet controllers' },
    { id: 'daemonsets', label: 'DaemonSets', icon: '⚙️', description: 'DaemonSet controllers' },
    { id: 'cronjobs', label: 'CronJobs', icon: '⏰', description: 'Scheduled jobs' },
    { id: 'replicasets', label: 'ReplicaSets', icon: '🔄', description: 'ReplicaSet controllers' },
    { id: 'services', label: 'Services', icon: '🌐', description: 'Network services' }
  ];

  // Load specific workload type
  async function loadWorkloadType(type: string, forceReload: boolean = false) {
    console.log(`loadWorkloadType called: type=${type}, forceReload=${forceReload}`);
    if (!currentContext) {
      console.log('No current context, returning');
      return;
    }
    if (loading && !forceReload) {
      console.log('Already loading and not force reload, returning');
      return;
    }
    if (loadedTypes.has(type) && !forceReload) {
      console.log(`Type ${type} already loaded and not force reload, returning`);
      return;
    }
    
    console.log(`Loading ${type}...`);
    loading = true;
    error = null;
    
    try {
      let data: unknown;
      
      switch (type) {
        case 'pods':
          data = await invoke('kuboard_get_pods');
          pods = Array.isArray(data) ? data : [];
          break;
        case 'deployments':
          data = await invoke('kuboard_get_deployments');
          const newDeployments = Array.isArray(data) ? [...data] : [];
          console.log(`Updating deployments: ${deployments.length} -> ${newDeployments.length}`);
          if (newDeployments.length > 0) {
            console.log('Sample deployment status:', {
              name: newDeployments[0].metadata?.name,
              ready: newDeployments[0].status?.readyReplicas,
              desired: newDeployments[0].spec?.replicas,
              available: newDeployments[0].status?.availableReplicas
            });
          }
          // Force new array reference to trigger reactivity
          deployments = newDeployments;
          deployments = [...deployments]; // Double assignment to ensure reactivity
          console.log('Deployments array updated, triggering reactivity');
          break;
        case 'replicasets':
          data = await invoke('kuboard_get_replicasets');
          replicasets = Array.isArray(data) ? data : [];
          break;
        case 'statefulsets':
          data = await invoke('kuboard_get_statefulsets');
          statefulsets = Array.isArray(data) ? data : [];
          break;
        case 'daemonsets':
          data = await invoke('kuboard_get_daemonsets');
          daemonsets = Array.isArray(data) ? data : [];
          break;
        case 'cronjobs':
          data = await invoke('kuboard_get_cronjobs');
          cronjobs = Array.isArray(data) ? data : [];
          break;
        case 'services':
          data = await invoke('kuboard_get_services');
          services = Array.isArray(data) ? data : [];
          break;
        default:
          console.warn(`Unknown workload type: ${type}`);
          return;
      }
      
      loadedTypes.add(type);
      lastUpdate = new Date().toLocaleTimeString();
      console.log(`Successfully loaded ${type}, count:`, getWorkloadCount(type));
    } catch (err) {
      error = err as string;
      console.error(`Failed to load ${type}:`, err);
      // Remove from loadedTypes if it was previously loaded, so we can retry
      loadedTypes.delete(type);
    } finally {
      loading = false;
      console.log(`Finished loading ${type}`);
    }
  }

  // Load workloads data (legacy function for compatibility)
  async function loadWorkloads() {
    // This function is kept for compatibility but now just loads the selected type
    await loadWorkloadType(selectedWorkloadType);
  }

  // Switch workload type
  async function switchWorkloadType(type: string) {
    selectedWorkloadType = type;
    // Force reload when switching to ensure we get fresh data
    await loadWorkloadType(type, true);
  }

  // Get current workload data
  function getCurrentWorkloads() {
    switch (selectedWorkloadType) {
      case 'pods': return pods;
      case 'deployments': return deployments;
      case 'statefulsets': return statefulsets;
      case 'daemonsets': return daemonsets;
      case 'cronjobs': return cronjobs;
      case 'replicasets': return replicasets;
      case 'services': return services;
      default: return [];
    }
  }

  // Get workload count
  function getWorkloadCount(type: string) {
    switch (type) {
      case 'pods': return pods.length;
      case 'deployments': return deployments.length;
      case 'statefulsets': return statefulsets.length;
      case 'daemonsets': return daemonsets.length;
      case 'cronjobs': return cronjobs.length;
      case 'replicasets': return replicasets.length;
      case 'services': return services.length;
      default: return 0;
    }
  }


  // Get status class for pods
  function getPodStatusClass(status: string): string {
    switch (status?.toLowerCase()) {
      case 'running': return 'running';
      case 'pending': return 'pending';
      case 'succeeded': return 'ready';
      case 'failed': return 'failed';
      case 'unknown': return 'unknown';
      default: return 'unknown';
    }
  }

  // Get status class for deployments
  function getDeploymentStatusClass(deployment: any): string {
    const available = deployment.status?.conditions?.find((c: any) => c.type === 'Available');
    if (available?.status === 'True') return 'available';
    return 'not-available';
  }

  // Get status class for services
  function getServiceStatusClass(service: any): string {
    const type = service.spec?.type || 'ClusterIP';
    return type.toLowerCase();
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

  // Format memory
  function formatMemory(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  // Format CPU
  function formatCPU(millicores: number): string {
    if (millicores === 0) return '0m';
    if (millicores >= 1000) {
      return (millicores / 1000).toFixed(1) + ' cores';
    }
    return millicores + 'm';
  }

  // Lifecycle
  onMount(() => {
    // Don't auto-load anything - wait for user selection
  });

  // Reactive updates
  $: if (currentContext) {
    // Only load if a type is selected and not already loaded
    if (selectedWorkloadType && !loadedTypes.has(selectedWorkloadType)) {
      loadWorkloadType(selectedWorkloadType);
    }
  }
</script>

<div class="workloads-tab">
  <div class="tab-header">
    <h4>⚙️ Workloads</h4>
    <div class="tab-controls">
      <button 
        class="refresh-button" 
        onclick={() => loadWorkloadType(selectedWorkloadType)}
        disabled={loading}
        title="Refresh current workload type"
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

  <!-- Workload Type Selector -->
  <div class="workload-type-selector">
    <div class="selector-header">
      <h5>Select Workload Type</h5>
      <p>Choose a workload type to view and manage</p>
    </div>
    <div class="workload-type-grid">
      {#each workloadTypes as type}
        <button 
          class="workload-type-card"
          class:active={selectedWorkloadType === type.id}
          class:loaded={loadedTypes.has(type.id)}
          onclick={() => switchWorkloadType(type.id)}
          disabled={loading}
        >
          <div class="type-icon">{type.icon}</div>
          <div class="type-info">
            <div class="type-label">{type.label}</div>
            <div class="type-description">{type.description}</div>
            {#if loadedTypes.has(type.id)}
              <div class="type-count">{getWorkloadCount(type.id)} items</div>
            {/if}
          </div>
          {#if loadedTypes.has(type.id)}
            <div class="loaded-indicator">✓</div>
          {/if}
        </button>
      {/each}
    </div>
  </div>

  {#if error}
    <div class="error-message">
      <div class="error-icon">⚠️</div>
      <div class="error-content">
        <h5>Failed to load {selectedWorkloadType}</h5>
        <p>{error}</p>
        <button class="retry-button" onclick={() => loadWorkloadType(selectedWorkloadType)}>
          Retry
        </button>
      </div>
    </div>
  {:else if loading}
    <div class="loading-message">
      <div class="loading-spinner">🔄</div>
      <p>Loading {selectedWorkloadType}...</p>
    </div>
  {:else if loadedTypes.has(selectedWorkloadType)}
    <!-- Workload Content -->
    <div class="workload-content">
      {#if selectedWorkloadType !== 'pods'}
        <div class="content-header">
          <h5>
            {workloadTypes.find(t => t.id === selectedWorkloadType)?.icon} 
            {workloadTypes.find(t => t.id === selectedWorkloadType)?.label}
            <span class="item-count">
              {#if selectedWorkloadType === 'deployments'}
                ({deployments.length})
              {:else if selectedWorkloadType === 'statefulsets'}
                ({statefulsets.length})
              {:else if selectedWorkloadType === 'daemonsets'}
                ({daemonsets.length})
              {:else if selectedWorkloadType === 'cronjobs'}
                ({cronjobs.length})
              {:else if selectedWorkloadType === 'replicasets'}
                ({replicasets.length})
              {:else if selectedWorkloadType === 'services'}
                ({services.length})
              {/if}
            </span>
          </h5>
        </div>
      {/if}

      {#if selectedWorkloadType === 'pods'}
        <!-- Pods Panel -->
        <PodsPanel 
          currentContext={currentContext} 
          pods={pods}
          on:podSelect={(e) => console.log('Pod selected:', e.detail)}
        />

      {:else if selectedWorkloadType === 'deployments'}
        <!-- Deployments Panel -->
        <DeploymentsPanel 
          currentContext={currentContext} 
          deployments={deployments}
          on:reload={() => {
            console.log('WorkloadsTab: reload event received for deployments');
            loadWorkloadType('deployments', true);
          }}
        />

      {:else if selectedWorkloadType === 'replicasets'}
        <!-- ReplicaSets Panel -->
        <ReplicaSetsPanel 
          currentContext={currentContext} 
          replicasets={replicasets}
        />

      {:else if selectedWorkloadType === 'statefulsets'}
        <!-- StatefulSets Panel -->
        <StatefulSetsPanel 
          currentContext={currentContext} 
          statefulsets={statefulsets}
          on:reload={() => loadWorkloadType('statefulsets', true)}
        />

      {:else if selectedWorkloadType === 'daemonsets'}
        <!-- DaemonSets Panel -->
        <DaemonSetsPanel 
          currentContext={currentContext} 
          daemonsets={daemonsets}
          on:reload={() => loadWorkloadType('daemonsets', true)}
        />

      {:else if selectedWorkloadType === 'cronjobs'}
        <!-- CronJobs Panel -->
        <CronJobsPanel 
          currentContext={currentContext} 
          cronjobs={cronjobs}
        />

      {:else if selectedWorkloadType === 'services'}
        <!-- Services List -->
        <div class="workload-list">
          {#each services as service}
            <div class="workload-item">
              <div class="workload-info">
                <span class="workload-name">{service.metadata?.name || 'Unknown'}</span>
                <span class="workload-namespace">{service.metadata?.namespace || 'default'}</span>
              </div>
              <div class="workload-details">
                <div class="workload-status">
                  <span class="status-badge status-{getServiceStatusClass(service)}">
                    {service.spec?.type || 'ClusterIP'}
                  </span>
                </div>
                <div class="workload-ports">
                  <span class="port-info">
                    Ports: {service.spec?.ports?.length || 0}
                  </span>
                </div>
                <div class="workload-age">
                  <span class="age-info">{formatAge(service.metadata?.creationTimestamp)}</span>
                </div>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {:else if selectedWorkloadType === 'pods' && pods.length === 0}
    <!-- No pods found -->
    <div class="no-resources">
      <div class="no-resources-icon">📭</div>
      <h5>No Pods Found</h5>
      <p>No pods are currently running in this cluster</p>
      <button class="retry-button" onclick={() => loadWorkloadType('pods')}>
        Refresh
      </button>
    </div>
  {:else}
    <!-- No workload type selected -->
    <div class="no-selection">
      <div class="no-selection-icon">👆</div>
      <h5>Select a Workload Type</h5>
      <p>Choose a workload type from above to view and manage resources</p>
    </div>
  {/if}
</div>

<style>
  /* Import CSS variables */
  @import '../styles/variables.css';

  .workloads-tab {
    padding: 0;
  }

  .tab-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 5px;
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

  .workload-type-selector {
    margin-bottom: 5px;
  }

  .selector-header {
    margin-bottom: var(--spacing-md);
  }

  .selector-header h5 {
    margin: 0 0 4px 0;
    color: white;
    font-size: 1rem;
    font-weight: 600;
  }

  .selector-header p {
    margin: 0;
    color: rgba(255, 255, 255, 0.7);
    font-size: 0.9rem;
  }

  .workload-type-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: var(--spacing-md);
  }

  .workload-type-card {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    cursor: pointer;
    transition: var(--transition-normal);
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    position: relative;
  }

  .workload-type-card:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.05);
    border-color: rgba(255, 255, 255, 0.2);
    transform: translateY(-1px);
  }

  .workload-type-card.active {
    background: var(--primary-color);
    border-color: var(--primary-color);
    color: white;
  }

  .workload-type-card.loaded {
    border-color: var(--success-color);
  }

  .workload-type-card:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .type-icon {
    font-size: 1.5rem;
    flex-shrink: 0;
  }

  .type-info {
    flex: 1;
  }

  .type-label {
    color: white;
    font-size: 0.9rem;
    font-weight: 600;
    margin-bottom: 2px;
  }

  .type-description {
    color: rgba(255, 255, 255, 0.7);
    font-size: 0.8rem;
    margin-bottom: 4px;
  }

  .type-count {
    color: var(--success-color);
    font-size: 0.75rem;
    font-weight: 600;
  }

  .loaded-indicator {
    position: absolute;
    top: 8px;
    right: 8px;
    color: var(--success-color);
    font-size: 0.8rem;
    font-weight: bold;
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

  .workload-content {
    background: rgba(255, 255, 255, 0.02);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .content-header {
    margin-bottom: var(--spacing-md);
    padding-bottom: var(--spacing-sm);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .content-header h5 {
    margin: 0;
    color: white;
    font-size: 1rem;
    font-weight: 600;
  }

  .item-count {
    color: rgba(255, 255, 255, 0.6);
    font-weight: 400;
  }

  .workload-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .workload-item {
    padding: var(--spacing-sm);
    background: rgba(255, 255, 255, 0.03);
    border-radius: var(--radius-sm);
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .workload-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-xs);
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

  .workload-details {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .workload-status {
    margin-bottom: var(--spacing-xs);
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
    background: var(--status-ready-bg);
    color: var(--status-ready-text);
    border: 1px solid var(--status-ready-border);
    font-weight: 600;
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

  .workload-replicas, .workload-ports, .workload-restarts, .workload-resources {
    margin-bottom: 2px;
  }

  .replica-info, .port-info, .restart-info, .resource-info {
    color: rgba(255, 255, 255, 0.8);
    font-size: 0.8rem;
    font-weight: 500;
  }

  .workload-resources {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .workload-age {
    margin-top: 2px;
  }

  .age-info {
    color: rgba(255, 255, 255, 0.6);
    font-size: 0.7rem;
  }

  .no-selection, .no-resources {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-xl);
    color: rgba(255, 255, 255, 0.6);
    text-align: center;
  }

  .no-selection-icon, .no-resources-icon {
    font-size: 3rem;
    opacity: 0.7;
  }

  .no-selection h5, .no-resources h5 {
    margin: 0;
    color: white;
    font-size: 1.2rem;
    font-weight: 600;
  }

  .no-selection p, .no-resources p {
    margin: 0;
    font-size: 0.9rem;
  }

  .no-resources .retry-button {
    background: var(--primary-color);
    color: white;
    border: none;
    border-radius: var(--radius-sm);
    padding: var(--spacing-xs) var(--spacing-sm);
    cursor: pointer;
    transition: var(--transition-normal);
    font-size: 0.9rem;
  }

  .no-resources .retry-button:hover {
    background: var(--primary-dark);
  }

  /* Responsive Design */
  @media (max-width: 768px) {
    .workload-type-grid {
      grid-template-columns: 1fr;
    }
    
    .workload-info {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--spacing-xs);
    }
  }
</style>