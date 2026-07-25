<!-- Kuboard Global Search Component -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { navigateTo } from '../stores/nav';

  // State
  let visible = false;
  let query = '';
  let results: any[] = [];
  let allResources: any[] = []; // Cache all resources
  let loading = false;
  let selectedIndex = 0;
  let inputElement: HTMLInputElement;

  const kindIcons: Record<string, string> = {
    'Pod': '📦',
    'Deployment': '🚀',
    'Service': '🌐',
    'Node': '🖥️',
    'Namespace': '📂',
    'StatefulSet': '💾',
    'DaemonSet': '👾',
    'ReplicaSet': '🔄',
    'Job': '🏃',
    'CronJob': '⏰',
    'ConfigMap': '⚙️',
    'Secret': '🔑',
    'Ingress': '🌐',
    'PersistentVolume': '💾',
    'PersistentVolumeClaim': '📥',
    'StorageClass': '🏗️'
  };

  const kindToTab: Record<string, string> = {
    'Pod': 'workloads',
    'Deployment': 'workloads',
    'StatefulSet': 'workloads',
    'DaemonSet': 'workloads',
    'ReplicaSet': 'workloads',
    'Job': 'workloads',
    'CronJob': 'workloads',
    'Service': 'network',
    'Ingress': 'network',
    'IngressClass': 'network',
    'NetworkPolicy': 'network',
    'ConfigMap': 'config',
    'Secret': 'config',
    'PersistentVolume': 'storage',
    'PersistentVolumeClaim': 'storage',
    'StorageClass': 'storage',
    'Node': 'nodes',
    'Namespace': 'overview',
    'Role': 'security',
    'ClusterRole': 'security',
    'RoleBinding': 'security',
    'ClusterRoleBinding': 'security',
    'ServiceAccount': 'security'
  };

  async function openSearch() {
    visible = true;
    query = '';
    results = [];
    selectedIndex = 0;
    loading = true;
    
    // Wait for next tick to focus input
    setTimeout(() => inputElement?.focus(), 10);

    try {
      allResources = await invoke('kuboard_search_resources') as any[];
    } catch (e) {
      console.error('Failed to fetch resources for search:', e);
    } finally {
      loading = false;
    }
  }

  async function handleKeyDown(event: KeyboardEvent) {
    // Open with Ctrl+K or Cmd+K
    if ((event.ctrlKey || event.metaKey) && event.key === 'k') {
      event.preventDefault();
      if (!visible) {
        await openSearch();
      } else {
        visible = false;
      }
    }

    // Close with Escape
    if (event.key === 'Escape' && visible) {
      visible = false;
    }

    if (!visible) return;

    // Navigation in results
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      selectedIndex = (selectedIndex + 1) % (results.length || 1);
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      selectedIndex = (selectedIndex - 1 + (results.length || 1)) % (results.length || 1);
    } else if (event.key === 'Enter') {
      event.preventDefault();
      if (results[selectedIndex]) {
        selectResult(results[selectedIndex]);
      }
    }
  }

  function filterResults() {
    if (query.length < 2) {
      results = [];
      return;
    }

    const q = query.toLowerCase();
    results = allResources.filter(r => 
      r.name.toLowerCase().includes(q) || 
      (r.namespace && r.namespace.toLowerCase().includes(q)) ||
      r.kind.toLowerCase().includes(q)
    ).slice(0, 10);
    
    selectedIndex = 0;
  }

  function selectResult(result: any) {
    const tab = kindToTab[result.kind] || 'workloads';
    navigateTo({
      tab,
      resourceType: result.kind.toLowerCase(),
      resourceName: result.name,
      namespace: result.namespace
    });
    visible = false;
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeyDown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeyDown);
  });

  $: if (visible && query) {
    filterResults();
  }
</script>

{#if visible}
  <div class="search-overlay" onclick={() => visible = false}>
    <div class="search-modal" onclick={(e) => e.stopPropagation()}>
      <div class="search-input-wrapper">
        <span class="search-icon">🔍</span>
        <input
          bind:this={inputElement}
          bind:value={query}
          placeholder="Search resources (name, kind, namespace)..."
          spellcheck="false"
          autocomplete="off"
        />
        <div class="search-shortcut">ESC to close</div>
      </div>

      <div class="search-results">
        {#if loading && results.length === 0}
          <div class="search-status">Searching...</div>
        {:else if query.length > 0 && query.length < 2}
          <div class="search-status">Type at least 2 characters...</div>
        {:else if query.length >= 2 && results.length === 0 && !loading}
          <div class="search-status">No resources found matching "{query}"</div>
        {:else}
          {#each results as result, i}
            <div 
              class="result-item" 
              class:selected={i === selectedIndex}
              onclick={() => selectResult(result)}
              onmouseenter={() => selectedIndex = i}
            >
              <span class="result-icon">{kindIcons[result.kind] || '📄'}</span>
              <div class="result-info">
                <div class="result-name">{result.name}</div>
                <div class="result-meta">
                  <span class="result-kind">{result.kind}</span>
                  {#if result.namespace}
                    <span class="result-namespace">in {result.namespace}</span>
                  {/if}
                </div>
              </div>
              {#if i === selectedIndex}
                <div class="result-enter">↵</div>
              {/if}
            </div>
          {/each}
        {/if}
      </div>

      <div class="search-footer">
        <div class="footer-item"><span>↑↓</span> to navigate</div>
        <div class="footer-item"><span>↵</span> to select</div>
        <div class="footer-item"><span>⌘K</span> to toggle</div>
      </div>
    </div>
  </div>
{/if}

<style>
  .search-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(4px);
    z-index: 1000;
    display: flex;
    justify-content: center;
    padding-top: 15vh;
  }

  .search-modal {
    width: 100%;
    max-width: 600px;
    background: var(--bg-secondary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    height: fit-content;
    max-height: 70vh;
  }

  .search-input-wrapper {
    display: flex;
    align-items: center;
    padding: var(--spacing-lg);
    border-bottom: 1px solid var(--border-primary);
    gap: var(--spacing-md);
  }

  .search-icon {
    font-size: 1.2rem;
    color: var(--text-secondary);
  }

  input {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--text-primary);
    font-size: 1.1rem;
    outline: none;
  }

  .search-shortcut {
    font-size: 0.75rem;
    color: var(--text-muted);
    background: rgba(255, 255, 255, 0.05);
    padding: 2px 6px;
    border-radius: 4px;
    text-transform: uppercase;
  }

  .search-results {
    overflow-y: auto;
  }

  .search-status {
    padding: var(--spacing-xl);
    text-align: center;
    color: var(--text-secondary);
    font-style: italic;
  }

  .result-item {
    display: flex;
    align-items: center;
    padding: var(--spacing-md) var(--spacing-lg);
    gap: var(--spacing-md);
    cursor: pointer;
    transition: background 0.1s;
  }

  .result-item.selected {
    background: rgba(59, 130, 246, 0.1);
  }

  .result-icon {
    font-size: 1.2rem;
    width: 24px;
    text-align: center;
  }

  .result-info {
    flex: 1;
  }

  .result-name {
    font-weight: 600;
    color: var(--text-primary);
  }

  .result-meta {
    font-size: 0.8rem;
    color: var(--text-secondary);
    display: flex;
    gap: var(--spacing-sm);
  }

  .result-kind {
    color: var(--primary-color);
    font-weight: 500;
  }

  .result-enter {
    color: var(--text-muted);
    font-size: 1.2rem;
  }

  .search-footer {
    display: flex;
    padding: var(--spacing-sm) var(--spacing-lg);
    background: rgba(0, 0, 0, 0.1);
    border-top: 1px solid var(--border-primary);
    gap: var(--spacing-lg);
  }

  .footer-item {
    font-size: 0.75rem;
    color: var(--text-muted);
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .footer-item span {
    background: rgba(255, 255, 255, 0.1);
    padding: 1px 4px;
    border-radius: 3px;
    color: var(--text-secondary);
    font-weight: 600;
  }
</style>
