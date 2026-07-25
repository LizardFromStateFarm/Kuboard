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
  import ServicesPanel from './ServicesPanel.svelte';

  // Props
  export let currentContext: any = null;
  export let selectedNamespace: string = 'all';

  // State
  let pods: any[] = [];
  let deployments: any[] = [];
  let services: any[] = [];
  let replicasets: any[] = [];
  let statefulsets: any[] = [];
  let daemonsets: any[] = [];
  let cronjobs: any[] = [];
  let namespacesList: string[] = [];
  
  let loading: boolean = false;
  let error: string | null = null;
  let lastUpdate: string = '';
  let selectedWorkloadType: string = 'pods';
  let loadedTypes: Set<string> = new Set();
  
  const workloadTypes = [
    { id: 'pods', label: 'Pods', icon: '🟢' },
    { id: 'deployments', label: 'Deployments', icon: '📦' },
    { id: 'statefulsets', label: 'StatefulSets', icon: '📋' },
    { id: 'daemonsets', label: 'DaemonSets', icon: '⚙️' },
    { id: 'cronjobs', label: 'CronJobs', icon: '⏰' },
    { id: 'replicasets', label: 'ReplicaSets', icon: '🔄' },
    { id: 'services', label: 'Services', icon: '🌐' }
  ];

  async function loadNamespaces() {
    if (!currentContext) return;
    try {
      const data: any = await invoke('kuboard_get_namespaces');
      if (Array.isArray(data)) {
        namespacesList = data.map((ns: any) => ns.metadata?.name || ns.name).filter(Boolean);
      }
    } catch (err) {
      console.error('Failed to load namespaces in WorkloadsTab:', err);
    }
  }

  // Filter helper for selected namespace
  function filterByNs(list: any[]) {
    if (!selectedNamespace || selectedNamespace === 'all') return list;
    return list.filter(item => (item.metadata?.namespace || item.namespace) === selectedNamespace);
  }

  $: filteredPods = filterByNs(pods);
  $: filteredDeployments = filterByNs(deployments);
  $: filteredStatefulsets = filterByNs(statefulsets);
  $: filteredDaemonsets = filterByNs(daemonsets);
  $: filteredCronjobs = filterByNs(cronjobs);
  $: filteredReplicasets = filterByNs(replicasets);
  $: filteredServices = filterByNs(services);

  // Load specific workload type
  async function loadWorkloadType(type: string, forceReload: boolean = false) {
    if (!currentContext) return;
    if (loading && !forceReload) return;
    if (loadedTypes.has(type) && !forceReload) return;
    
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
          deployments = Array.isArray(data) ? data : [];
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
          return;
      }
      
      loadedTypes.add(type);
      lastUpdate = new Date().toLocaleTimeString();
    } catch (err) {
      error = err as string;
      console.error(`Failed to load ${type}:`, err);
      loadedTypes.delete(type);
    } finally {
      loading = false;
    }
  }

  // Switch workload type
  async function switchWorkloadType(type: string) {
    selectedWorkloadType = type;
    await loadWorkloadType(type, true);
  }

  let selectedTargetResourceName: Record<string, string | null> = {
    pods: null,
    deployments: null,
    statefulsets: null,
    daemonsets: null,
    cronjobs: null,
    replicasets: null,
    services: null
  };

  function handleNavigateToWorkload(event: CustomEvent<{ type: string; name: string }>) {
    const { type, name } = event.detail;
    let targetType = type.toLowerCase();
    if (targetType === 'deployment') targetType = 'deployments';
    if (targetType === 'statefulset') targetType = 'statefulsets';
    if (targetType === 'daemonset') targetType = 'daemonsets';
    if (targetType === 'replicaset') targetType = 'replicasets';
    if (targetType === 'cronjob') targetType = 'cronjobs';
    if (targetType === 'pod') targetType = 'pods';
    if (targetType === 'service') targetType = 'services';

    selectedTargetResourceName[targetType] = name;
    selectedWorkloadType = targetType;
    loadWorkloadType(targetType, true);
  }

  // Get workload count for badge
  function getWorkloadCount(type: string) {
    switch (type) {
      case 'pods': return filteredPods.length;
      case 'deployments': return filteredDeployments.length;
      case 'statefulsets': return filteredStatefulsets.length;
      case 'daemonsets': return filteredDaemonsets.length;
      case 'cronjobs': return filteredCronjobs.length;
      case 'replicasets': return filteredReplicasets.length;
      case 'services': return filteredServices.length;
      default: return 0;
    }
  }

  onMount(() => {
    if (currentContext) {
      loadNamespaces();
      loadWorkloadType(selectedWorkloadType);
    }
  });

  $: if (currentContext) {
    loadNamespaces();
    if (selectedWorkloadType && !loadedTypes.has(selectedWorkloadType)) {
      loadWorkloadType(selectedWorkloadType);
    }
  }
</script>

<div class="workloads-tab">
  <div class="tab-header">
    <h4>⚙️ Workloads</h4>
    <div class="tab-controls">
      <div class="namespace-selector">
        <label for="workloads-ns-select">Namespace:</label>
        <select 
          id="workloads-ns-select" 
          bind:value={selectedNamespace}
          class="namespace-dropdown"
        >
          <option value="all">All Namespaces</option>
          {#each namespacesList as ns}
            <option value={ns}>{ns}</option>
          {/each}
        </select>
      </div>

      <button 
        class="refresh-button" 
        onclick={() => loadWorkloadType(selectedWorkloadType, true)}
        disabled={loading}
        title="Refresh current workload type"
      >
        {#if loading}🔄{:else}↻{/if}
      </button>
      {#if lastUpdate}
        <span class="last-update">Last: {lastUpdate}</span>
      {/if}
    </div>
  </div>

  <!-- Sleek Sub-nav Tabs -->
  <div class="sub-nav-tabs">
    {#each workloadTypes as type}
      <button 
        class="sub-nav-item"
        class:active={selectedWorkloadType === type.id}
        onclick={() => switchWorkloadType(type.id)}
      >
        <span class="tab-icon">{type.icon}</span>
        <span class="tab-label">{type.label}</span>
        {#if loadedTypes.has(type.id)}
          <span class="tab-badge">{getWorkloadCount(type.id)}</span>
        {/if}
      </button>
    {/each}
  </div>

  {#if error}
    <div class="error-message">
      <div class="error-icon">⚠️</div>
      <div class="error-content">
        <h5>Failed to load {selectedWorkloadType}</h5>
        <p>{error}</p>
        <button class="retry-button" onclick={() => loadWorkloadType(selectedWorkloadType, true)}>
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
    <div class="workload-content">
      {#if selectedWorkloadType === 'pods'}
        {#if filteredPods.length === 0}
          <div class="no-resources">
            <div class="no-resources-icon">📭</div>
            <h5>No Pods Detected</h5>
            <p>No pods found in namespace "{selectedNamespace}"</p>
            <button class="retry-button" onclick={() => loadWorkloadType('pods', true)}>
              Refresh
            </button>
          </div>
        {:else}
          <PodsPanel 
            currentContext={currentContext} 
            pods={filteredPods}
            initialSelectedName={selectedTargetResourceName['pods']}
            on:podSelect={(e) => console.log('Pod selected:', e.detail)}
            on:navigateToWorkload={handleNavigateToWorkload}
          />
        {/if}

      {:else if selectedWorkloadType === 'deployments'}
        {#if filteredDeployments.length === 0}
          <div class="no-resources">
            <div class="no-resources-icon">📦</div>
            <h5>No Deployments Detected</h5>
            <p>No deployments found in namespace "{selectedNamespace}"</p>
            <button class="retry-button" onclick={() => loadWorkloadType('deployments', true)}>
              Refresh
            </button>
          </div>
        {:else}
          <DeploymentsPanel 
            currentContext={currentContext} 
            deployments={filteredDeployments}
            initialSelectedName={selectedTargetResourceName['deployments']}
            on:reload={() => loadWorkloadType('deployments', true)}
            on:navigateToWorkload={handleNavigateToWorkload}
          />
        {/if}

      {:else if selectedWorkloadType === 'statefulsets'}
        {#if filteredStatefulsets.length === 0}
          <div class="no-resources">
            <div class="no-resources-icon">📋</div>
            <h5>No StatefulSets Detected</h5>
            <p>No statefulsets found in namespace "{selectedNamespace}"</p>
            <button class="retry-button" onclick={() => loadWorkloadType('statefulsets', true)}>
              Refresh
            </button>
          </div>
        {:else}
          <StatefulSetsPanel 
            currentContext={currentContext} 
            statefulsets={filteredStatefulsets}
            initialSelectedName={selectedTargetResourceName['statefulsets']}
            on:reload={() => loadWorkloadType('statefulsets', true)}
            on:navigateToWorkload={handleNavigateToWorkload}
          />
        {/if}

      {:else if selectedWorkloadType === 'daemonsets'}
        {#if filteredDaemonsets.length === 0}
          <div class="no-resources">
            <div class="no-resources-icon">⚙️</div>
            <h5>No DaemonSets Detected</h5>
            <p>No daemonsets found in namespace "{selectedNamespace}"</p>
            <button class="retry-button" onclick={() => loadWorkloadType('daemonsets', true)}>
              Refresh
            </button>
          </div>
        {:else}
          <DaemonSetsPanel 
            currentContext={currentContext} 
            daemonsets={filteredDaemonsets}
            initialSelectedName={selectedTargetResourceName['daemonsets']}
            on:reload={() => loadWorkloadType('daemonsets', true)}
            on:navigateToWorkload={handleNavigateToWorkload}
          />
        {/if}

      {:else if selectedWorkloadType === 'cronjobs'}
        {#if filteredCronjobs.length === 0}
          <div class="no-resources">
            <div class="no-resources-icon">⏰</div>
            <h5>No CronJobs Detected</h5>
            <p>No cronjobs found in namespace "{selectedNamespace}"</p>
            <button class="retry-button" onclick={() => loadWorkloadType('cronjobs', true)}>
              Refresh
            </button>
          </div>
        {:else}
          <CronJobsPanel 
            currentContext={currentContext} 
            cronjobs={filteredCronjobs}
            initialSelectedName={selectedTargetResourceName['cronjobs']}
            on:navigateToWorkload={handleNavigateToWorkload}
          />
        {/if}

      {:else if selectedWorkloadType === 'replicasets'}
        {#if filteredReplicasets.length === 0}
          <div class="no-resources">
            <div class="no-resources-icon">🔄</div>
            <h5>No ReplicaSets Detected</h5>
            <p>No replicasets found in namespace "{selectedNamespace}"</p>
            <button class="retry-button" onclick={() => loadWorkloadType('replicasets', true)}>
              Refresh
            </button>
          </div>
        {:else}
          <ReplicaSetsPanel 
            currentContext={currentContext} 
            replicasets={filteredReplicasets}
            initialSelectedName={selectedTargetResourceName['replicasets']}
            on:navigateToWorkload={handleNavigateToWorkload}
          />
        {/if}

      {:else if selectedWorkloadType === 'services'}
        <ServicesPanel 
          {currentContext} 
          namespace={selectedNamespace} 
          initialSelectedName={selectedTargetResourceName['services']}
          on:navigateToWorkload={handleNavigateToWorkload}
        />
      {/if}
    </div>
  {:else}
    <div class="loading-message">
      <div class="loading-spinner">🔄</div>
      <p>Loading {selectedWorkloadType}...</p>
    </div>
  {/if}
</div>

<style>
  @import '../styles/variables.css';

  .workloads-tab {
    padding: 0;
    position: relative;
    min-height: 400px;
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

  .namespace-selector {
    display: flex;
    align-items: center;
    gap: 8px;
    color: rgba(255, 255, 255, 0.8);
    font-size: 0.88rem;
  }

  .namespace-dropdown {
    background: rgba(0, 0, 0, 0.4);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-sm);
    color: white;
    padding: 4px 8px;
    font-size: 0.85rem;
    cursor: pointer;
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

  .sub-nav-tabs {
    display: flex;
    gap: 6px;
    padding: 6px;
    background: rgba(255, 255, 255, 0.03);
    border-bottom: 1px solid var(--border-primary);
    overflow-x: auto;
    margin-bottom: 12px;
  }

  .sub-nav-item {
    display: flex;
    align-items: center;
    gap: 6px;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    font-size: 0.88rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    white-space: nowrap;
  }

  .sub-nav-item:hover {
    color: var(--text-primary);
    background: rgba(255, 255, 255, 0.06);
  }

  .sub-nav-item.active {
    color: white;
    background: var(--primary-color);
    font-weight: 600;
  }

  .tab-badge {
    background: rgba(255, 255, 255, 0.15);
    color: var(--text-primary);
    font-size: 0.75rem;
    padding: 1px 6px;
    border-radius: 10px;
    font-weight: 600;
  }

  .sub-nav-item.active .tab-badge {
    background: rgba(255, 255, 255, 0.25);
    color: white;
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
    background: var(--primary-color);
    border: none;
    border-radius: var(--radius-sm);
    color: white;
    cursor: pointer;
    font-size: 0.8rem;
    padding: 4px 10px;
    transition: var(--transition-normal);
  }

  .retry-button:hover {
    opacity: 0.9;
  }

  .loading-message {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-xl);
    color: rgba(255, 255, 255, 0.8);
    min-height: 300px;
    justify-content: center;
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
    min-height: 200px;
    transition: opacity 0.2s ease-in-out;
  }

  .no-resources {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-xl);
    color: rgba(255, 255, 255, 0.6);
    text-align: center;
  }

  .no-resources-icon {
    font-size: 3rem;
    opacity: 0.7;
  }

  .no-resources h5 {
    margin: 0;
    color: white;
    font-size: 1.2rem;
    font-weight: 600;
  }

  .no-resources p {
    margin: 0;
    font-size: 0.9rem;
  }
</style>