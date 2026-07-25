<!-- Kuboard Custom Resources Tab Component -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  // Props
  export let currentContext: any = null;

  // State
  let customResourceDefinitions: any[] = [];
  let filteredCRDs: any[] = [];
  let selectedCRD: any = null;
  let crdInstances: any[] = [];
  let instancesLoading: boolean = false;
  let instancesError: string | null = null;
  let searchQuery: string = '';
  
  let loading: boolean = false;
  let error: string | null = null;
  let lastUpdate: string = '';

  // Load custom resources data
  async function loadCustomResources() {
    if (!currentContext || loading) return;
    
    loading = true;
    error = null;
    
    try {
      customResourceDefinitions = await invoke('kuboard_list_crds');
      filterCRDs();
      lastUpdate = new Date().toLocaleTimeString();
    } catch (err) {
      error = err as string;
      console.error('Failed to load CRDs:', err);
    } finally {
      loading = false;
    }
  }

  function filterCRDs() {
    if (!searchQuery) {
      filteredCRDs = customResourceDefinitions;
    } else {
      const q = searchQuery.toLowerCase();
      filteredCRDs = customResourceDefinitions.filter(crd => 
        crd.metadata.name.toLowerCase().includes(q) ||
        crd.spec.names.kind.toLowerCase().includes(q)
      );
    }
  }

  async function selectCRD(crd: any) {
    selectedCRD = crd;
    instancesLoading = true;
    instancesError = null;
    crdInstances = [];
    
    try {
      const group = crd.spec.group;
      const version = crd.spec.versions.find((v: any) => v.served).name;
      const kind = crd.spec.names.kind;
      
      crdInstances = await invoke('kuboard_list_custom_resource_instances', {
        group,
        version,
        kind
      });
    } catch (err) {
      instancesError = err as string;
      console.error('Failed to load CRD instances:', err);
    } finally {
      instancesLoading = false;
    }
  }

  // Lifecycle
  onMount(() => {
    loadCustomResources();
  });

  $: if (searchQuery !== undefined) {
    filterCRDs();
  }

  // Reactive updates
  $: if (currentContext) {
    loadCustomResources();
  }
</script>

<div class="custom-resources-tab">
  <div class="tab-header">
    <div class="tab-controls">
      <button 
        class="refresh-button" 
        onclick={loadCustomResources}
        disabled={loading}
        title="Refresh custom resources"
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
        <h5>Failed to load custom resources</h5>
        <p>{error}</p>
        <button class="retry-button" onclick={loadCustomResources}>
          Retry
        </button>
      </div>
    </div>
  {:else if loading}
    <div class="loading-message">
      <div class="loading-spinner">🔄</div>
      <p>Loading custom resources...</p>
    </div>
  {:else}
    <div class="custom-resources-content">
      {#if !selectedCRD}
        <div class="crd-list-view">
          <div class="search-bar">
            <input 
              type="text" 
              placeholder="Filter CRDs by name or kind..." 
              bind:value={searchQuery}
            />
          </div>
          
          <div class="crd-grid">
            {#each filteredCRDs as crd}
              <button class="crd-card" onclick={() => selectCRD(crd)}>
                <div class="crd-icon">🔧</div>
                <div class="crd-info">
                  <div class="crd-kind">{crd.spec.names.kind}</div>
                  <div class="crd-name">{crd.metadata.name}</div>
                  <div class="crd-group">{crd.spec.group}</div>
                </div>
              </button>
            {/each}
          </div>
        </div>
      {:else}
        <div class="instance-view">
          <div class="view-header">
            <button class="back-button" onclick={() => selectedCRD = null}>← Back to CRDs</button>
            <div class="header-info">
              <h5>{selectedCRD.spec.names.kind} Instances</h5>
              <span class="crd-full-name">{selectedCRD.metadata.name}</span>
            </div>
          </div>

          {#if instancesLoading}
            <div class="loading-state">
              <div class="spinner"></div>
              <p>Loading instances of {selectedCRD.spec.names.kind}...</p>
            </div>
          {:else if instancesError}
            <div class="error-banner">
              <span class="error-icon">⚠️</span>
              <p>{instancesError}</p>
            </div>
          {:else if crdInstances.length === 0}
            <div class="empty-state">
              <div class="empty-icon">📭</div>
              <h5>No Instances Found</h5>
              <p>There are no instances of <strong>{selectedCRD.spec.names.kind}</strong> in this cluster.</p>
            </div>
          {:else}
            <div class="instance-list">
              {#each crdInstances as instance}
                <div class="instance-item">
                  <div class="instance-info">
                    <span class="instance-name">{instance.metadata.name}</span>
                    <span class="instance-namespace">{instance.metadata.namespace || 'Cluster-wide'}</span>
                  </div>
                  <div class="instance-actions">
                    <button class="btn-action" onclick={() => console.log('View details', instance)}>View</button>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  /* Import CSS variables */
  @import '../styles/variables.css';

  .custom-resources-tab {
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

  .custom-resources-content {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
    min-height: 400px;
  }

  .crd-list-view {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  .search-bar input {
    width: 100%;
    padding: 10px 16px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    color: white;
    outline: none;
  }

  .search-bar input:focus {
    border-color: var(--primary-color);
  }

  .crd-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: var(--spacing-md);
  }

  .crd-card {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    cursor: pointer;
    text-align: left;
    transition: var(--transition-normal);
  }

  .crd-card:hover {
    background: rgba(255, 255, 255, 0.05);
    border-color: var(--primary-color);
    transform: translateY(-2px);
  }

  .crd-icon {
    font-size: 1.5rem;
    opacity: 0.7;
  }

  .crd-kind {
    font-weight: 700;
    color: white;
    font-size: 0.95rem;
  }

  .crd-name {
    font-size: 0.8rem;
    color: var(--text-secondary);
    margin: 2px 0;
  }

  .crd-group {
    font-size: 0.75rem;
    color: var(--primary-color);
    font-family: monospace;
  }

  .instance-view {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  .view-header {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .back-button {
    background: transparent;
    border: none;
    color: var(--primary-color);
    cursor: pointer;
    padding: 0;
    font-size: 0.9rem;
    width: fit-content;
  }

  .header-info h5 {
    margin: 0;
    font-size: 1.2rem;
    color: white;
  }

  .crd-full-name {
    font-size: 0.85rem;
    color: var(--text-muted);
    font-family: monospace;
  }

  .instance-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .instance-item {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .instance-name {
    font-weight: 600;
    color: white;
    display: block;
  }

  .instance-namespace {
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .btn-action {
    background: rgba(255, 255, 255, 0.1);
    border: none;
    color: white;
    padding: 4px 12px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: 0.85rem;
  }

  .btn-action:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .loading-state, .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-xxl);
    color: var(--text-secondary);
    text-align: center;
  }

  .spinner {
    width: 30px;
    height: 30px;
    border: 2px solid rgba(255, 255, 255, 0.1);
    border-top-color: var(--primary-color);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: var(--spacing-md);
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .empty-icon {
    font-size: 2.5rem;
    margin-bottom: var(--spacing-md);
    opacity: 0.5;
  }
</style>
