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
  import { Box, Boxes, Layers, Cpu, Clock, Copy, Globe, RefreshCw, AlertTriangle, Inbox } from 'lucide-svelte';
  import { navigationStore } from '../stores/nav';

  // Props
  export let currentContext: any = null;
  export let selectedNamespace: string = 'all';
  export let tabSessionId: string = 'tab-default';

  // Multi-namespace selection state
  let selectedNamespaces: string[] = ['all'];

  // Keep single namespace prop in sync if passed
  $: if (selectedNamespace && selectedNamespaces.includes('all')) {
    if (selectedNamespace !== 'all') {
      selectedNamespaces = [selectedNamespace];
    }
  }

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

  let sessionWorkloadTypeMap: Record<string, string> = {};
  $: selectedWorkloadType = sessionWorkloadTypeMap[tabSessionId] || 'pods';

  function setWorkloadType(typeId: string) {
    sessionWorkloadTypeMap[tabSessionId] = typeId;
    sessionWorkloadTypeMap = { ...sessionWorkloadTypeMap };
  }

  let loadedTypes: Set<string> = new Set();
  
  const workloadTypes = [
    { id: 'pods', label: 'Pods', icon: Box },
    { id: 'deployments', label: 'Deployments', icon: Boxes },
    { id: 'statefulsets', label: 'StatefulSets', icon: Layers },
    { id: 'daemonsets', label: 'DaemonSets', icon: Cpu },
    { id: 'cronjobs', label: 'CronJobs', icon: Clock },
    { id: 'replicasets', label: 'ReplicaSets', icon: Copy },
    { id: 'services', label: 'Services', icon: Globe }
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

  // Multi-namespace filter helper
  function filterByNs(list: any[], nsList: string[]) {
    if (!nsList || nsList.length === 0 || nsList.includes('all')) {
      return list;
    }
    return list.filter(item => {
      const ns = item.metadata?.namespace || item.namespace;
      return nsList.includes(ns);
    });
  }

  $: filteredPods = filterByNs(pods, selectedNamespaces);
  $: filteredDeployments = filterByNs(deployments, selectedNamespaces);
  $: filteredStatefulsets = filterByNs(statefulsets, selectedNamespaces);
  $: filteredDaemonsets = filterByNs(daemonsets, selectedNamespaces);
  $: filteredCronjobs = filterByNs(cronjobs, selectedNamespaces);
  $: filteredReplicasets = filterByNs(replicasets, selectedNamespaces);
  $: filteredServices = filterByNs(services, selectedNamespaces);

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
    setWorkloadType(type);
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

  let lastProcessedWorkloadNav: any = null;

  $: if ($navigationStore && $navigationStore.resourceName && $navigationStore !== lastProcessedWorkloadNav) {
    lastProcessedWorkloadNav = $navigationStore;
    const nav = $navigationStore;
    const rawKind = (nav.tab || 'pods').toLowerCase();
    let targetType = rawKind;
    if (targetType === 'deployment') targetType = 'deployments';
    if (targetType === 'statefulset') targetType = 'statefulsets';
    if (targetType === 'daemonset') targetType = 'daemonsets';
    if (targetType === 'replicaset') targetType = 'replicasets';
    if (targetType === 'cronjob') targetType = 'cronjobs';
    if (targetType === 'pod') targetType = 'pods';
    if (targetType === 'service') targetType = 'services';

    if (workloadTypes.some(t => t.id === targetType)) {
      selectedTargetResourceName[targetType] = nav.resourceName;
      setWorkloadType(targetType);
      loadWorkloadType(targetType, true);
    }
  }

  $: if (currentContext) {
    loadNamespaces();
    if (selectedWorkloadType && !loadedTypes.has(selectedWorkloadType)) {
      loadWorkloadType(selectedWorkloadType);
    }
  }
</script>

<div class="workloads-tab">
  <!-- Sleek Top Sub-nav Tabs Bar -->
  <div class="sub-nav-tabs">
    {#each workloadTypes as type}
      <button 
        class="sub-nav-item"
        class:active={selectedWorkloadType === type.id}
        onclick={() => switchWorkloadType(type.id)}
      >
        <span class="tab-icon">
          <svelte:component this={type.icon} size={15} />
        </span>
        <span class="tab-label">{type.label}</span>
        {#if loadedTypes.has(type.id)}
          <span class="tab-badge">{getWorkloadCount(type.id)}</span>
        {/if}
      </button>
    {/each}
  </div>

  {#if error}
    <div class="error-message">
      <div class="error-icon"><AlertTriangle size={24} /></div>
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
      <div class="loading-spinner"><RefreshCw size={24} class="spin" /></div>
      <p>Loading {selectedWorkloadType}...</p>
    </div>
  {:else if loadedTypes.has(selectedWorkloadType)}
    <div class="workload-content">
      {#if selectedWorkloadType === 'pods'}
        <PodsPanel 
          currentContext={currentContext} 
          pods={filteredPods}
          bind:selectedNamespaces={selectedNamespaces}
          namespacesList={namespacesList}
          loading={loading}
          onRefresh={() => loadWorkloadType('pods', true)}
          initialSelectedName={selectedTargetResourceName['pods']}
          on:podSelect={(e) => console.log('Pod selected:', e.detail)}
          on:navigateToWorkload={handleNavigateToWorkload}
        />

      {:else if selectedWorkloadType === 'deployments'}
        <DeploymentsPanel 
          currentContext={currentContext} 
          deployments={filteredDeployments}
          bind:selectedNamespaces={selectedNamespaces}
          namespacesList={namespacesList}
          loading={loading}
          onRefresh={() => loadWorkloadType('deployments', true)}
          initialSelectedName={selectedTargetResourceName['deployments']}
          on:reload={() => loadWorkloadType('deployments', true)}
          on:navigateToWorkload={handleNavigateToWorkload}
        />

      {:else if selectedWorkloadType === 'statefulsets'}
        <StatefulSetsPanel 
          currentContext={currentContext} 
          statefulsets={filteredStatefulsets}
          bind:selectedNamespaces={selectedNamespaces}
          namespacesList={namespacesList}
          loading={loading}
          onRefresh={() => loadWorkloadType('statefulsets', true)}
          initialSelectedName={selectedTargetResourceName['statefulsets']}
          on:reload={() => loadWorkloadType('statefulsets', true)}
          on:navigateToWorkload={handleNavigateToWorkload}
        />

      {:else if selectedWorkloadType === 'daemonsets'}
        <DaemonSetsPanel 
          currentContext={currentContext} 
          daemonsets={filteredDaemonsets}
          bind:selectedNamespaces={selectedNamespaces}
          namespacesList={namespacesList}
          loading={loading}
          onRefresh={() => loadWorkloadType('daemonsets', true)}
          initialSelectedName={selectedTargetResourceName['daemonsets']}
          on:reload={() => loadWorkloadType('daemonsets', true)}
          on:navigateToWorkload={handleNavigateToWorkload}
        />

      {:else if selectedWorkloadType === 'cronjobs'}
        <CronJobsPanel 
          currentContext={currentContext} 
          cronjobs={filteredCronjobs}
          bind:selectedNamespaces={selectedNamespaces}
          namespacesList={namespacesList}
          loading={loading}
          onRefresh={() => loadWorkloadType('cronjobs', true)}
          initialSelectedName={selectedTargetResourceName['cronjobs']}
          on:navigateToWorkload={handleNavigateToWorkload}
        />

      {:else if selectedWorkloadType === 'replicasets'}
        <ReplicaSetsPanel 
          currentContext={currentContext} 
          replicasets={filteredReplicasets}
          bind:selectedNamespaces={selectedNamespaces}
          namespacesList={namespacesList}
          loading={loading}
          onRefresh={() => loadWorkloadType('replicasets', true)}
          initialSelectedName={selectedTargetResourceName['replicasets']}
          on:navigateToWorkload={handleNavigateToWorkload}
        />

      {:else if selectedWorkloadType === 'services'}
        <ServicesPanel 
          {currentContext} 
          bind:selectedNamespaces={selectedNamespaces}
          namespacesList={namespacesList}
          loading={loading}
          onRefresh={() => loadWorkloadType('services', true)}
          namespace={selectedNamespace} 
          initialSelectedName={selectedTargetResourceName['services']}
          on:navigateToWorkload={handleNavigateToWorkload}
        />
      {/if}
    </div>
  {:else}
    <div class="loading-message">
      <div class="loading-spinner"><RefreshCw size={24} class="spin" /></div>
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

  .sub-nav-bar-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-md);
    padding: 4px 6px;
    background: rgba(255, 255, 255, 0.03);
    border-bottom: 1px solid var(--border-primary);
    margin-bottom: 12px;
  }

  .sub-nav-tabs {
    display: flex;
    flex-wrap: nowrap;
    gap: 6px;
    overflow-x: auto;
    flex: 1;
    min-width: 0;
  }

  .sub-nav-controls-right {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    flex-shrink: 0;
  }

  .sub-nav-item {
    flex: 1 1 0px;
    min-width: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    padding: var(--spacing-xs) var(--spacing-sm);
    border-radius: var(--radius-sm);
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    white-space: nowrap;
  }

  .sub-nav-item:hover {
    color: var(--text-primary);
    background: rgba(255, 255, 255, 0.05);
  }

  .sub-nav-item.active {
    color: var(--primary-color);
    background: rgba(59, 130, 246, 0.1);
    font-weight: 600;
  }

  .tab-badge {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-secondary);
    font-size: 0.75rem;
    padding: 1px 6px;
    border-radius: 10px;
    font-weight: 600;
  }

  .sub-nav-item.active .tab-badge {
    background: rgba(59, 130, 246, 0.2);
    color: var(--primary-color);
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