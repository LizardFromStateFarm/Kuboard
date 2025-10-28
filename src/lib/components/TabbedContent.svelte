<!-- Kuboard Tabbed Content Component -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import ResourceTabs from './ResourceTabs.svelte';
  import WorkloadsTab from './WorkloadsTab.svelte';
  import ConfigTab from './ConfigTab.svelte';
  import NetworkTab from './NetworkTab.svelte';
  import CustomResourcesTab from './CustomResourcesTab.svelte';

  // Props
  export let currentContext: any = null;

  // State
  let activeTab: string = 'workloads';
  let tabCounts: Record<string, number> = {};

  // Tab definitions
  const tabs = [
    { id: 'workloads', label: 'Workloads', icon: '⚙️', count: 0 },
    { id: 'config', label: 'Config', icon: '⚙️', count: 0 },
    { id: 'network', label: 'Network', icon: '🌐', count: 0 },
    { id: 'storage', label: 'Storage', icon: '💾', count: 0, disabled: true },
    { id: 'custom', label: 'Custom Resources', icon: '🔧', count: 0 },
    { id: 'security', label: 'Security', icon: '🔒', count: 0, disabled: true }
  ];

  // Tab change handler
  function handleTabChange(event: CustomEvent<{ tabId: string }>) {
    activeTab = event.detail.tabId;
  }

  // Update tab counts
  function updateTabCount(tabId: string, count: number) {
    tabCounts[tabId] = count;
    // Update the tabs array to trigger reactivity
    const updatedTabs = tabs.map(tab => ({
      ...tab,
      count: tabCounts[tab.id] || 0
    }));
    // This will trigger reactivity in ResourceTabs
  }

  // Load initial counts
  async function loadInitialCounts() {
    if (!currentContext) return;

    try {
      // Load counts for available tabs
      const [pods, deployments, services, configmaps, secrets] = await Promise.all([
        invoke('kuboard_get_pods').catch(() => []),
        invoke('kuboard_get_deployments').catch(() => []),
        invoke('kuboard_get_services').catch(() => []),
        invoke('kuboard_get_configmaps').catch(() => []),
        invoke('kuboard_get_secrets').catch(() => [])
      ]);

      updateTabCount('workloads', (pods as any[]).length + (deployments as any[]).length + (services as any[]).length);
      updateTabCount('config', (configmaps as any[]).length + (secrets as any[]).length);
      updateTabCount('network', (services as any[]).length);
    } catch (error) {
      console.error('Failed to load initial counts:', error);
    }
  }

  // Reactive updates
  $: if (currentContext) {
    loadInitialCounts();
  }
</script>

<div class="tabbed-content">
  <ResourceTabs 
    {tabs}
    {activeTab}
    on:tabChange={handleTabChange}
  />

  <div class="tab-content">
    {#if activeTab === 'workloads'}
      <WorkloadsTab {currentContext} />
    {:else if activeTab === 'config'}
      <ConfigTab {currentContext} />
    {:else if activeTab === 'network'}
      <NetworkTab {currentContext} />
    {:else if activeTab === 'storage'}
      <div class="coming-soon-tab">
        <div class="coming-soon-content">
          <div class="coming-soon-icon">💾</div>
          <h4>Storage Management</h4>
          <p>Storage resources including PersistentVolumes, PersistentVolumeClaims, and StorageClasses will be available in a future update.</p>
        </div>
      </div>
    {:else if activeTab === 'custom'}
      <CustomResourcesTab {currentContext} />
    {:else if activeTab === 'security'}
      <div class="coming-soon-tab">
        <div class="coming-soon-content">
          <div class="coming-soon-icon">🔒</div>
          <h4>Security Management</h4>
          <p>Security resources including RBAC, SecurityContexts, and PodSecurityPolicies will be available in a future update.</p>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  /* Import CSS variables */
  @import '../styles/variables.css';

  .tabbed-content {
    background: rgba(255, 255, 255, 0.05);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
    border: 1px solid rgba(255, 255, 255, 0.1);
    margin-bottom: var(--spacing-lg);
  }

  .tab-content {
    margin-top: var(--spacing-lg);
  }

  .coming-soon-tab {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 300px;
  }

  .coming-soon-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-xl);
    text-align: center;
    color: rgba(255, 255, 255, 0.8);
  }

  .coming-soon-icon {
    font-size: 4rem;
    opacity: 0.7;
  }

  .coming-soon-content h4 {
    margin: 0;
    color: white;
    font-size: 1.5rem;
    font-weight: 600;
  }

  .coming-soon-content p {
    margin: 0;
    font-size: 1rem;
    line-height: 1.5;
    max-width: 500px;
  }

  /* Responsive Design */
  @media (max-width: 768px) {
    .tabbed-content {
      padding: var(--spacing-md);
    }
    
    .coming-soon-content {
      padding: var(--spacing-lg);
    }
    
    .coming-soon-icon {
      font-size: 3rem;
    }
    
    .coming-soon-content h4 {
      font-size: 1.3rem;
    }
    
    .coming-soon-content p {
      font-size: 0.9rem;
    }
  }
</style>
