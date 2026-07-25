<!-- Kuboard Config Tab Component (Overhauled) -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import ResourceTable from './ResourceTable.svelte';
  import ConfigMapDetails from './ConfigMapDetails.svelte';
  import HelmTab from './HelmTab.svelte';
  import QuickActionsMenu from './QuickActionsMenu.svelte';
  import { FileText, Package } from 'lucide-svelte';

  // Props
  export let currentContext: any = null;
  export let namespace: string = 'all';
  export let tabSessionId: string = 'tab-default';

  // State
  let sessionSubTabMap: Record<string, 'configmaps' | 'helm'> = {};
  $: activeSubTab = sessionSubTabMap[tabSessionId] || 'configmaps';
  let configmaps: any[] = [];
  let loading: boolean = false;
  let error: string | null = null;
  let searchQuery: string = '';
  let sortColumn: string = 'name';
  let sortDirection: 'asc' | 'desc' = 'asc';
  let selectedConfigMap: any = null;

  // Context Menu State
  let contextMenuVisible = false;
  let contextMenuPosition = { x: 0, y: 0 };
  let contextMenuResource: any = null;

  async function fetchConfigMaps() {
    if (!currentContext) return;
    loading = true;
    error = null;
    try {
      const data = await invoke('kuboard_get_configmaps');
      configmaps = (data as any[]) || [];
    } catch (err: any) {
      console.error('Failed to fetch ConfigMaps:', err);
      error = String(err);
    } finally {
      loading = false;
    }
  }

  function handleSort(column: string) {
    if (sortColumn === column) {
      sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
    } else {
      sortColumn = column;
      sortDirection = 'asc';
    }
  }

  function formatAge(timestamp: string): string {
    if (!timestamp) return '-';
    const created = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - created.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMins / 60);
    const diffDays = Math.floor(diffHours / 24);
    if (diffDays > 0) return `${diffDays}d`;
    if (diffHours > 0) return `${diffHours}h`;
    return `${diffMins}m`;
  }

  function handleContextMenu(event: MouseEvent, cm: any) {
    event.preventDefault();
    event.stopPropagation();
    contextMenuResource = cm;
    contextMenuPosition = { x: event.clientX, y: event.clientY };
    contextMenuVisible = true;
  }

  $: filteredConfigMaps = configmaps.filter(cm => {
    const matchesNamespace = namespace === 'all' || cm.metadata?.namespace === namespace;
    const q = searchQuery.toLowerCase();
    const matchesSearch = !q || 
      (cm.metadata?.name || '').toLowerCase().includes(q) ||
      (cm.metadata?.namespace || '').toLowerCase().includes(q);
    return matchesNamespace && matchesSearch;
  });

  $: sortedConfigMaps = [...filteredConfigMaps].sort((a, b) => {
    let comp = 0;
    if (sortColumn === 'name') {
      comp = (a.metadata?.name || '').localeCompare(b.metadata?.name || '');
    } else if (sortColumn === 'namespace') {
      comp = (a.metadata?.namespace || '').localeCompare(b.metadata?.namespace || '');
    } else if (sortColumn === 'keys') {
      const keysA = Object.keys(a.data || {}).length;
      const keysB = Object.keys(b.data || {}).length;
      comp = keysA - keysB;
    }
    return sortDirection === 'asc' ? comp : -comp;
  });

  onMount(() => {
    fetchConfigMaps();
  });

  $: if (currentContext || namespace) {
    fetchConfigMaps();
  }
</script>

<div class="config-tab-container">
  <!-- Config Sub-Tabs Navigation Header -->
  <div class="config-top-nav">
    <div class="sub-tabs">
      <button 
        class="sub-tab-btn {activeSubTab === 'configmaps' ? 'active' : ''}" 
        onclick={() => { sessionSubTabMap[tabSessionId] = 'configmaps'; sessionSubTabMap = { ...sessionSubTabMap }; }}
      >
        <FileText size={15} /> ConfigMaps ({filteredConfigMaps.length})
      </button>
      <button 
        class="sub-tab-btn {activeSubTab === 'helm' ? 'active' : ''}" 
        onclick={() => { sessionSubTabMap[tabSessionId] = 'helm'; sessionSubTabMap = { ...sessionSubTabMap }; }}
      >
        <Package size={15} /> Helm Releases
      </button>
    </div>

    {#if activeSubTab === 'configmaps'}
      <button class="refresh-btn" onclick={fetchConfigMaps} disabled={loading}>
        {loading ? '🔄' : '↻ Refresh'}
      </button>
    {/if}
  </div>

  <!-- Sub-Tab Content View -->
  {#if activeSubTab === 'configmaps'}
    {#if selectedConfigMap}
      <ConfigMapDetails configMap={selectedConfigMap} onBack={() => selectedConfigMap = null} />
    {:else}
      {#if error}
        <div class="alert-error">⚠️ {error}</div>
      {/if}

      <ResourceTable
        items={configmaps}
        filteredItems={sortedConfigMaps}
        bind:searchQuery
        searchPlaceholder="Search ConfigMaps by name or namespace..."
        noItemsMessage="No ConfigMaps found in this context."
        noSearchResultsMessage="No ConfigMaps match your search query:"
      >
        <svelte:fragment slot="header">
          <tr>
            <th class="sortable" onclick={() => handleSort('name')}>
              Name {sortColumn === 'name' ? (sortDirection === 'asc' ? '▲' : '▼') : ''}
            </th>
            <th class="sortable" onclick={() => handleSort('namespace')}>
              Namespace {sortColumn === 'namespace' ? (sortDirection === 'asc' ? '▲' : '▼') : ''}
            </th>
            <th class="sortable" onclick={() => handleSort('keys')}>
              Data Keys {sortColumn === 'keys' ? (sortDirection === 'asc' ? '▲' : '▼') : ''}
            </th>
            <th>Age</th>
          </tr>
        </svelte:fragment>

        <svelte:fragment slot="rows">
          {#each sortedConfigMaps as cm}
            <tr 
              class="clickable-row" 
              onclick={() => selectedConfigMap = cm}
              oncontextmenu={(e) => handleContextMenu(e, cm)}
            >
              <td class="name-cell">📄 {cm.metadata?.name}</td>
              <td>{cm.metadata?.namespace || 'default'}</td>
              <td>{Object.keys(cm.data || {}).length} keys</td>
              <td>{formatAge(cm.metadata?.creationTimestamp)}</td>
            </tr>
          {/each}
        </svelte:fragment>
      </ResourceTable>
    {/if}
  {:else if activeSubTab === 'helm'}
    <div class="helm-subtab-wrapper">
      <HelmTab {currentContext} />
    </div>
  {/if}
</div>

{#if contextMenuResource}
  <QuickActionsMenu
    resource={contextMenuResource}
    resourceType="configmap"
    position={contextMenuPosition}
    bind:visible={contextMenuVisible}
    on:close={() => contextMenuResource = null}
    on:deleted={fetchConfigMaps}
  />
{/if}

<style>
  .config-tab-container {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .config-top-nav {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: var(--background-secondary);
    padding: 8px 12px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-primary);
  }

  .sub-tabs {
    display: flex;
    gap: 8px;
  }

  .sub-tab-btn {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    padding: 6px 14px;
    border-radius: var(--radius-sm);
    font-weight: 600;
    font-size: 0.9rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .sub-tab-btn:hover {
    color: var(--text-primary);
    background: rgba(255, 255, 255, 0.05);
  }

  .sub-tab-btn.active {
    background: var(--primary-color);
    color: white;
  }

  .refresh-btn {
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid var(--border-primary);
    color: white;
    padding: 5px 12px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: 0.85rem;
  }

  .clickable-row {
    cursor: pointer;
  }

  .clickable-row:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .name-cell {
    color: var(--primary-color);
    font-weight: 600;
  }

  .alert-error {
    background: rgba(239, 68, 68, 0.15);
    color: #f87171;
    border: 1px solid rgba(239, 68, 68, 0.3);
    padding: 10px 14px;
    border-radius: var(--radius-sm);
  }

  .helm-subtab-wrapper {
    padding-top: 4px;
  }
</style>
