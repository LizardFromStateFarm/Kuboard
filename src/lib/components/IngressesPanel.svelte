<!-- Kuboard Ingresses Panel Component -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import ResourceTable from './ResourceTable.svelte';
  import QuickActionsMenu from './QuickActionsMenu.svelte';
  import IngressDetails from './IngressDetails.svelte';

  // Props
  export let currentContext: any = null;
  export let namespace: string = 'all';

  // State
  let ingresses: any[] = [];
  let loading: boolean = true;
  let error: string | null = null;
  let searchQuery: string = '';
  let sortColumn: string = 'name';
  let sortDirection: 'asc' | 'desc' = 'asc';
  let refreshTimer: any;
  let selectedIngress: any = null;

  // Context Menu State
  let contextMenuVisible = false;
  let contextMenuPosition = { x: 0, y: 0 };
  let selectedResource: any = null;

  async function fetchIngresses() {
    if (!currentContext) return;
    
    try {
      loading = true;
      ingresses = await invoke('kuboard_list_ingresses', { namespace });
      error = null;
    } catch (e: any) {
      console.error('Failed to fetch Ingresses:', e);
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

  function getHosts(ingress: any): string {
    const rules = ingress.spec?.rules || [];
    if (rules.length === 0) return '*';
    const hosts = rules.map((r: any) => r.host || '*').filter((h: any, i: any, a: any) => a.indexOf(h) === i);
    if (hosts.length <= 2) return hosts.join(', ');
    return `${hosts[0]}, ${hosts[1]} +${hosts.length - 2} more`;
  }

  function getAddress(ingress: any): string {
    const ing = ingress.status?.loadBalancer?.ingress || [];
    if (ing.length === 0) return '-';
    return ing.map((i: any) => i.ip || i.hostname).join(', ');
  }

  function getPorts(ingress: any): string {
    const tls = ingress.spec?.tls || [];
    return tls.length > 0 ? '80, 443' : '80';
  }

  function handleContextMenu(event: MouseEvent, ing: any) {
    event.preventDefault();
    selectedResource = ing;
    contextMenuPosition = { x: event.clientX, y: event.clientY };
    contextMenuVisible = true;
  }

  function closeContextMenu() {
    contextMenuVisible = false;
    selectedResource = null;
  }

  $: filteredIngresses = ingresses
    .filter(ing => {
      const name = ing.metadata.name.toLowerCase();
      const query = searchQuery.toLowerCase();
      return name.includes(query);
    })
    .sort((a, b) => {
      let valA, valB;
      if (sortColumn === 'name') {
        valA = a.metadata.name;
        valB = b.metadata.name;
      } else if (sortColumn === 'namespace') {
        valA = a.metadata.namespace || '';
        valB = b.metadata.namespace || '';
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
    fetchIngresses();
    refreshTimer = setInterval(fetchIngresses, 15000); // Refresh every 15s
  });

  onDestroy(() => {
    if (refreshTimer) clearInterval(refreshTimer);
  });

  $: if (currentContext || namespace) {
    fetchIngresses();
  }
</script>

<div class="ingresses-panel">
  {#if selectedIngress}
    <IngressDetails ingress={selectedIngress} onBack={() => selectedIngress = null} />
  {:else}

    {#if loading && ingresses.length === 0}
      <div class="loading-state">
        <div class="spinner"></div>
        <p>Loading Ingresses...</p>
      </div>
    {:else if error}
      <div class="error-state">
        <span class="error-icon">⚠️</span>
        <p>{error}</p>
        <button onclick={fetchIngresses}>Retry</button>
      </div>
    {:else}
      <ResourceTable
        items={ingresses}
        filteredItems={filteredIngresses}
        bind:searchQuery
        searchPlaceholder="Search Ingresses..."
        noItemsMessage="No Ingresses found."
        noSearchResultsMessage="No Ingresses match your search query:"
      >
        <svelte:fragment slot="header">
          <th class="sortable" onclick={() => handleSort('name')}>Name</th>
          <th class="sortable" onclick={() => handleSort('namespace')}>Namespace</th>
          <th>Hosts</th>
          <th>Address</th>
          <th>Ports</th>
          <th class="sortable" onclick={() => handleSort('age')}>Age</th>
          <th>Actions</th>
        </svelte:fragment>

        <svelte:fragment slot="rows">
          {#each filteredIngresses as ing (ing.metadata.uid)}
            <tr class="resource-row clickable-row" onclick={() => selectedIngress = ing} oncontextmenu={(e) => handleContextMenu(e, ing)}>
              <td class="name-cell">{ing.metadata.name}</td>
              <td>{ing.metadata.namespace}</td>
              <td class="hosts-cell" title={getHosts(ing)}>{getHosts(ing)}</td>
              <td>{getAddress(ing)}</td>
              <td>{getPorts(ing)}</td>
              <td>{formatAge(ing.metadata.creationTimestamp)}</td>
              <td class="actions-cell">
                <button class="action-btn" onclick={(e) => { e.stopPropagation(); handleContextMenu(e, ing); }}>⚙️</button>
              </td>
            </tr>
          {/each}
        </svelte:fragment>
      </ResourceTable>
    {/if}
  {/if}
</div>

{#if contextMenuVisible}
  <QuickActionsMenu
    x={contextMenuPosition.x}
    y={contextMenuPosition.y}
    resource={selectedResource}
    resourceType="ingress"
    bind:visible={contextMenuVisible}
    on:close={closeContextMenu}
  />
{/if}

<style>
  .ingresses-panel {
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

  .hosts-cell {
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
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
