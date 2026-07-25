<!-- Kuboard Header Component -->
<script lang="ts">
  import type { KubeContext } from '../types/index.js';
  import ThemeSwitcher from './ThemeSwitcher.svelte';

  // Props
  export let contexts: KubeContext[] = [];
  export let currentContext: KubeContext | null = null;
  export let loading: boolean = false;
  export let isTauriAvailable: boolean = false;

  // Events
  import { createEventDispatcher } from 'svelte';
  const dispatch = createEventDispatcher();

  interface ClusterTabItem {
    id: string;
    name: string;
    contextName: string;
  }

  let clusterTabs: ClusterTabItem[] = [];
  let activeTabId: string = '';
  let showNewTabModal = false;
  let selectedContextName = '';

  // New tab modal state
  let newTabContextName = '';

  // Auto-sync initial context tab
  $: if (currentContext?.name) {
    selectedContextName = currentContext.name;
    if (clusterTabs.length === 0) {
      const id = 'tab-' + Date.now();
      clusterTabs = [{ 
        id, 
        name: currentContext.name, 
        contextName: currentContext.name
      }];
      activeTabId = id;
    }
  }

  function handleContextChange() {
    if (selectedContextName) {
      const activeTab = clusterTabs.find(t => t.id === activeTabId);
      if (activeTab) {
        activeTab.contextName = selectedContextName;
        activeTab.name = selectedContextName;
        clusterTabs = [...clusterTabs];
      }
      dispatch('contextChange', selectedContextName);
    }
  }

  function handleRefresh() {
    dispatch('refresh');
  }

  function selectTab(tabId: string) {
    const tab = clusterTabs.find(t => t.id === tabId);
    if (tab) {
      activeTabId = tabId;
      selectedContextName = tab.contextName;
      dispatch('contextChange', tab.contextName);
    }
  }

  function openNewTabModal() {
    newTabContextName = currentContext?.name || (contexts[0]?.name || '');
    showNewTabModal = true;
  }

  function createNewTab() {
    if (!newTabContextName) return;

    const id = 'tab-' + Date.now() + '-' + Math.random().toString(36).substr(2, 4);
    const sameCount = clusterTabs.filter(t => t.contextName === newTabContextName).length;
    const tabName = sameCount > 0 ? `${newTabContextName} (${sameCount + 1})` : newTabContextName;

    const newTab: ClusterTabItem = {
      id,
      name: tabName,
      contextName: newTabContextName
    };

    clusterTabs = [...clusterTabs, newTab];
    activeTabId = id;
    selectedContextName = newTabContextName;
    showNewTabModal = false;

    dispatch('contextChange', newTabContextName);
  }

  function closeTab(tabId: string) {
    if (clusterTabs.length <= 1) return;
    const index = clusterTabs.findIndex(t => t.id === tabId);
    clusterTabs = clusterTabs.filter(t => t.id !== tabId);
    if (activeTabId === tabId) {
      const nextTab = clusterTabs[Math.max(0, index - 1)];
      activeTabId = nextTab.id;
      selectedContextName = nextTab.contextName;
      dispatch('contextChange', nextTab.contextName);
    }
  }
</script>

<header class="flat-header">
  <div class="header-main-row">
    <!-- Brand Title -->
    <div class="brand">
      <span class="brand-logo">🚢</span>
      <span class="brand-name">Kuboard</span>
    </div>

    <!-- Cluster Tabs Bar (Kubeconfig contexts only) -->
    <div class="cluster-tabs-list">
      {#each clusterTabs as tab}
        <div 
          class="cluster-tab" 
          class:active={tab.id === activeTabId}
          onclick={() => selectTab(tab.id)}
          role="button"
          tabindex="0"
          onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && selectTab(tab.id)}
        >
          <span class="tab-icon">☸️</span>
          <span class="tab-name">{tab.name}</span>
          {#if clusterTabs.length > 1}
            <button class="tab-close" onclick={(e) => { e.stopPropagation(); closeTab(tab.id); }} title="Close tab">×</button>
          {/if}
        </div>
      {/each}

      <button class="add-tab-trigger" onclick={openNewTabModal} title="Open new cluster tab">+</button>
    </div>

    <!-- Header Actions -->
    <div class="header-actions">
      <!-- Context Selector -->
      <div class="context-picker">
        <select 
          id="context-select" 
          bind:value={selectedContextName}
          onchange={handleContextChange}
          class="context-select-input"
          disabled={contexts.length === 0}
        >
          {#if contexts.length === 0}
            <option value="">Loading contexts...</option>
          {:else}
            <option value="">Select Context</option>
            {#each contexts as context}
              <option value={context.name}>{context.name}</option>
            {/each}
          {/if}
        </select>
      </div>

      <button onclick={handleRefresh} disabled={loading} class="refresh-btn" title="Refresh cluster data">
        {loading ? "⏳" : "↻"}
      </button>
      <ThemeSwitcher />
    </div>
  </div>
</header>

<!-- Modal for Opening New Tab -->
{#if showNewTabModal}
  <div class="tab-modal-overlay" onclick={() => showNewTabModal = false} role="button" tabindex="-1">
    <div class="tab-modal-content" onclick={(e) => e.stopPropagation()} role="dialog">
      <div class="modal-header">
        <h4>➕ Open New Tab</h4>
        <button class="close-modal-btn" onclick={() => showNewTabModal = false}>×</button>
      </div>

      <div class="modal-body">
        <div class="form-group">
          <label for="new-tab-context">Select Kubeconfig Context:</label>
          <select id="new-tab-context" bind:value={newTabContextName} class="modal-select">
            {#each contexts as ctx}
              <option value={ctx.name}>{ctx.name}</option>
            {/each}
          </select>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn-cancel" onclick={() => showNewTabModal = false}>Cancel</button>
        <button class="btn-primary" onclick={createNewTab}>Create Tab</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .flat-header {
    background: var(--background-secondary);
    border-bottom: 1px solid var(--border-primary);
    padding: 6px 16px;
    margin-bottom: 16px;
  }

  .header-main-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .brand-logo {
    font-size: 1.3rem;
  }

  .brand-name {
    font-size: 1.1rem;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.3px;
  }

  .cluster-tabs-list {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 6px;
    overflow-x: auto;
    scrollbar-width: none;
    padding: 2px 0;
  }

  .cluster-tabs-list::-webkit-scrollbar {
    display: none;
  }

  .cluster-tab {
    display: flex;
    align-items: center;
    gap: 6px;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    padding: 5px 10px;
    color: var(--text-secondary);
    font-size: 0.85rem;
    cursor: pointer;
    transition: all 0.15s ease;
    white-space: nowrap;
    user-select: none;
  }

  .cluster-tab:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-primary);
  }

  .cluster-tab.active {
    background: var(--primary-color);
    border-color: var(--primary-color);
    color: white;
    font-weight: 600;
  }

  .tab-icon {
    font-size: 0.85rem;
  }

  .tab-close {
    background: transparent;
    border: none;
    color: inherit;
    opacity: 0.7;
    font-size: 0.95rem;
    line-height: 1;
    cursor: pointer;
    padding: 0 2px;
    border-radius: 2px;
  }

  .tab-close:hover {
    opacity: 1;
    background: rgba(0, 0, 0, 0.2);
  }

  .add-tab-trigger {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--border-primary);
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    width: 28px;
    height: 28px;
    font-size: 1rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
    flex-shrink: 0;
  }

  .add-tab-trigger:hover {
    background: rgba(255, 255, 255, 0.12);
    color: var(--text-primary);
  }

  /* Modal Overlay and Window */
  .tab-modal-overlay {
    position: fixed;
    top: 0; left: 0; right: 0; bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(4px);
    z-index: 2000;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 20px;
  }

  .tab-modal-content {
    background: #181824;
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-lg);
    width: 100%;
    max-width: 400px;
    box-shadow: 0 12px 32px rgba(0,0,0,0.5);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-primary);
  }

  .modal-header h4 {
    margin: 0;
    color: white;
    font-size: 1rem;
  }

  .close-modal-btn {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 1.2rem;
    cursor: pointer;
  }

  .modal-body {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .form-group label {
    font-size: 0.85rem;
    color: var(--text-secondary);
    font-weight: 500;
  }

  .modal-select {
    background: rgba(0, 0, 0, 0.4);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    color: white;
    padding: 8px 12px;
    font-size: 0.9rem;
    outline: none;
  }

  .modal-select:focus {
    border-color: var(--primary-color);
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    padding: 12px 16px;
    border-top: 1px solid var(--border-primary);
    background: rgba(0, 0, 0, 0.2);
  }

  .btn-cancel {
    background: transparent;
    border: 1px solid var(--border-primary);
    color: var(--text-secondary);
    padding: 6px 14px;
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .btn-primary {
    background: var(--primary-color);
    border: none;
    color: white;
    padding: 6px 16px;
    border-radius: var(--radius-sm);
    font-weight: 600;
    cursor: pointer;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-shrink: 0;
  }

  .context-select-input {
    background: var(--background-tertiary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 0.85rem;
    font-weight: 500;
    padding: 5px 10px;
    cursor: pointer;
    max-width: 200px;
  }

  .context-select-input:hover {
    border-color: var(--border-secondary);
  }

  .refresh-btn {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 0.9rem;
    padding: 4px 10px;
    cursor: pointer;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .refresh-btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.12);
  }

  .refresh-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
