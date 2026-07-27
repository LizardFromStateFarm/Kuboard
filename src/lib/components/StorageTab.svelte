<!-- Kuboard Storage Tab Component -->
<script lang="ts">
  import PersistentVolumeClaimsPanel from './PersistentVolumeClaimsPanel.svelte';
  import PersistentVolumesPanel from './PersistentVolumesPanel.svelte';
  import StorageClassesPanel from './StorageClassesPanel.svelte';
  import { HardDrive, Database, Layers } from 'lucide-svelte';

  // Props
  export let currentContext: any = null;
  export let namespace: string = 'all';
  export let tabSessionId: string = 'tab-default';

  // State
  let sessionTabMap: Record<string, 'pvc' | 'pv' | 'sc'> = {};
  $: activeTab = sessionTabMap[tabSessionId] || 'pvc';

  function setActiveTab(tab: 'pvc' | 'pv' | 'sc') {
    sessionTabMap[tabSessionId] = tab;
    sessionTabMap = { ...sessionTabMap };
  }

  import { navigationStore } from '../stores/nav';

  let lastProcessedStorageNav: any = null;

  $: if ($navigationStore && ($navigationStore.tab === 'storage' || $navigationStore.tab === 'pvc') && $navigationStore.resourceName && $navigationStore !== lastProcessedStorageNav) {
    lastProcessedStorageNav = $navigationStore;
    setActiveTab('pvc');
  }
</script>

<div class="storage-tab">
  <div class="tab-navigation">
    <button 
      class="nav-item" 
      class:active={activeTab === 'pvc'} 
      onclick={() => setActiveTab('pvc')}
    >
      <HardDrive size={15} /> Volume Claims
    </button>
    <button 
      class="nav-item" 
      class:active={activeTab === 'pv'} 
      onclick={() => setActiveTab('pv')}
    >
      <Database size={15} /> Volumes
    </button>
    <button 
      class="nav-item" 
      class:active={activeTab === 'sc'} 
      onclick={() => setActiveTab('sc')}
    >
      <Layers size={15} /> Storage Classes
    </button>
  </div>

  <div class="tab-content">
    {#if activeTab === 'pvc'}
      <PersistentVolumeClaimsPanel {currentContext} {namespace} />
    {:else if activeTab === 'pv'}
      <PersistentVolumesPanel {currentContext} />
    {:else if activeTab === 'sc'}
      <StorageClassesPanel {currentContext} />
    {/if}
  </div>
</div>

<style>
  .storage-tab {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .tab-navigation {
    display: flex;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: rgba(255, 255, 255, 0.02);
    border-bottom: 1px solid var(--border-primary);
  }

  .nav-item {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    padding: var(--spacing-xs) var(--spacing-sm);
    cursor: pointer;
    font-size: 0.95rem;
    font-weight: 500;
    transition: all 0.2s ease;
    border-radius: var(--radius-sm);
  }

  .nav-item:hover {
    color: var(--text-primary);
    background: rgba(255, 255, 255, 0.05);
  }

  .nav-item.active {
    color: var(--primary-color);
    background: rgba(59, 130, 246, 0.1);
  }

  .tab-content {
    flex: 1;
    overflow-y: auto;
  }
</style>
