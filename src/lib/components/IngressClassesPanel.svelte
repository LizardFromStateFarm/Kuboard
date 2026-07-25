<!-- Kuboard IngressClasses Panel Component -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import ResourceTable from './ResourceTable.svelte';
  import QuickActionsMenu from './QuickActionsMenu.svelte';

  // Props
  export let currentContext: any = null;

  // State
  let ingressClasses: any[] = [];
  let loading: boolean = true;
  let error: string | null = null;
  let searchQuery: string = '';
  let sortColumn: string = 'name';
  let sortDirection: 'asc' | 'desc' = 'asc';
  let refreshTimer: any;

  // Context Menu State
  let contextMenuVisible = false;
  let contextMenuPosition = { x: 0, y: 0 };
  let selectedResource: any = null;

  async function fetchIngressClasses() {
    if (!currentContext) return;
    
    try {
      loading = true;
      ingressClasses = await invoke('kuboard_list_ingress_classes');
      error = null;
    } catch (e: any) {
      console.error('Failed to fetch IngressClasses:', e);
      error = String(e);
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
    const date = new Date(timestamp);
    const now = new Date();
    const diff = Math.floor((now.getTime() - date.getTime()) / 1000);
    
    if (diff < 60) return `${diff}s`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h`;
    return `${Math.floor(diff / 86400)}d`;
  }

  function handleContextMenu(event: MouseEvent, ic: any) {
    event.preventDefault();
    selectedResource = ic;
    contextMenuPosition = { x: event.clientX, y: event.clientY };
    contextMenuVisible = true;
  }

  function closeContextMenu() {
    contextMenuVisible = false;
    selectedResource = null;
  }

  $: filteredClasses = ingressClasses
    .filter(ic => {
      const name = ic.metadata.name.toLowerCase();
      const query = searchQuery.toLowerCase();
      return name.includes(query);
    })
    .sort((a, b) => {
      let valA, valB;
      if (sortColumn === 'name') {
        valA = a.metadata.name;
        valB = b.metadata.name;
      } else if (sortColumn === 'age') {
        valA = new Date(a.metadata.creationTimestamp).getTime();
        valB = new Date(b.metadata.creationTimestamp).getTime();
      } else {
        valA = a.metadata.name;
        valB = b.metadata.name;
      }
      
      if (valA < valB) return sortDirection === 'asc' ? -1 : 1;
      if (valA > valB) return sortDirection === 'asc' ? 1 : -1;
      return 0;
    });

  onMount(() => {
    fetchIngressClasses();
    refreshTimer = setInterval(fetchIngressClasses, 30000); // Refresh every 30s
  });

  onDestroy(() => {
    if (refreshTimer) clearInterval(refreshTimer);
  });

  $: if (currentContext) {
    fetchIngressClasses();
  }
</script>

<div class="ingress-classes-panel">
  <div class="panel-header">
    <h4>🏗️ Ingress Classes ({filteredClasses.length})</h4>
  </div>

  {#if loading && ingressClasses.length === 0}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Loading Ingress Classes...</p>
    </div>
  {:else if error}
    <div class="error-state">
      <span class="error-icon">⚠️</span>
      <p>{error}</p>
      <button onclick={fetchIngressClasses}>Retry</button>
    </div>
  {:else}
    <ResourceTable
      items={ingressClasses}
      filteredItems={filteredClasses}
      bind:searchQuery
      searchPlaceholder="Search Ingress Classes..."
      noItemsMessage="No Ingress Classes found."
      noSearchResultsMessage="No Ingress Classes match your search query:"
    >
      <svelte:fragment slot="header">
        <th class="sortable" onclick={() => handleSort('name')}>Name</th>
        <th>Controller</th>
        <th class="sortable" onclick={() => handleSort('age')}>Age</th>
        <th>Actions</th>
      </svelte:fragment>

      <svelte:fragment slot="rows">
        {#each filteredClasses as ic (ic.metadata.uid)}
          <tr class="resource-row" oncontextmenu={(e) => handleContextMenu(e, ic)}>
            <td class="name-cell">{ic.metadata.name}</td>
            <td>{ic.spec?.controller || '-'}</td>
            <td>{formatAge(ic.metadata.creationTimestamp)}</td>
            <td class="actions-cell">
              <button class="action-btn" onclick={(e) => handleContextMenu(e, ic)}>⚙️</button>
            </td>
          </tr>
        {/each}
      </svelte:fragment>
    </ResourceTable>
  {/if}
</div>

{#if contextMenuVisible}
  <QuickActionsMenu
    x={contextMenuPosition.x}
    y={contextMenuPosition.y}
    resource={selectedResource}
    resourceType="ingressclass"
    bind:visible={contextMenuVisible}
    on:close={closeContextMenu}
  />
{/if}

<style>
  .ingress-classes-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: var(--spacing-md);
  }

  .panel-header {
    margin-bottom: var(--spacing-md);
  }

  .panel-header h4 {
    margin: 0;
    font-size: 1.2rem;
    color: var(--text-primary);
  }

  .loading-state, .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-xl);
    color: var(--text-secondary);
  }

  .spinner {
    width: 30px;
    height: 30px;
    border: 3px solid rgba(255, 255, 255, 0.1);
    border-top-color: var(--primary-color);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: var(--spacing-md);
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .name-cell {
    font-weight: 600;
    color: var(--primary-color);
  }

  .sortable {
    cursor: pointer;
  }

  .sortable:hover {
    color: var(--primary-color);
  }

  .action-btn {
    background: transparent;
    border: none;
    cursor: pointer;
    font-size: 1rem;
    padding: 4px;
    border-radius: 4px;
  }

  .action-btn:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  th {
    text-align: left;
    padding: var(--spacing-sm);
    color: var(--text-secondary);
    font-size: 0.85rem;
    font-weight: 600;
    border-bottom: 1px solid var(--border-primary);
  }

  td {
    padding: var(--spacing-sm);
    border-bottom: 1px solid var(--border-primary);
  }

  .resource-row:hover {
    background: rgba(255, 255, 255, 0.03);
  }
</style>
