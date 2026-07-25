<!-- Kuboard PersistentVolumes Panel Component -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import ResourceTable from './ResourceTable.svelte';
  import QuickActionsMenu from './QuickActionsMenu.svelte';

  // Props
  export let currentContext: any = null;

  // State
  let pvs: any[] = [];
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

  async function fetchPVs() {
    if (!currentContext) return;
    
    try {
      loading = true;
      pvs = await invoke('kuboard_list_persistent_volumes');
      error = null;
    } catch (e: any) {
      console.error('Failed to fetch PVs:', e);
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

  function getStatusClass(status: string): string {
    switch (status?.toLowerCase()) {
      case 'available': return 'pending';
      case 'bound': return 'ready';
      case 'released': return 'unknown';
      case 'failed': return 'failed';
      default: return 'unknown';
    }
  }

  function handleContextMenu(event: MouseEvent, pv: any) {
    event.preventDefault();
    selectedResource = pv;
    contextMenuPosition = { x: event.clientX, y: event.clientY };
    contextMenuVisible = true;
  }

  function closeContextMenu() {
    contextMenuVisible = false;
    selectedResource = null;
  }

  $: filteredPVs = pvs
    .filter(pv => {
      const name = pv.metadata.name.toLowerCase();
      const query = searchQuery.toLowerCase();
      return name.includes(query);
    })
    .sort((a, b) => {
      let valA, valB;
      if (sortColumn === 'name') {
        valA = a.metadata.name;
        valB = b.metadata.name;
      } else if (sortColumn === 'status') {
        valA = a.status?.phase || '';
        valB = b.status?.phase || '';
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
    fetchPVs();
    refreshTimer = setInterval(fetchPVs, 10000); // Refresh every 10s
  });

  onDestroy(() => {
    if (refreshTimer) clearInterval(refreshTimer);
  });

  $: if (currentContext) {
    fetchPVs();
  }
</script>

<div class="pv-panel">

  {#if loading && pvs.length === 0}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Loading Persistent Volumes...</p>
    </div>
  {:else if error}
    <div class="error-state">
      <span class="error-icon">⚠️</span>
      <p>{error}</p>
      <button onclick={fetchPVs}>Retry</button>
    </div>
  {:else}
    <ResourceTable
      items={pvs}
      filteredItems={filteredPVs}
      bind:searchQuery
      searchPlaceholder="Search PVs..."
      noItemsMessage="No Persistent Volumes found."
      noSearchResultsMessage="No PVs match your search query:"
    >
      <svelte:fragment slot="header">
        <th class="sortable" onclick={() => handleSort('name')}>Name</th>
        <th>Capacity</th>
        <th>Access Modes</th>
        <th>Reclaim Policy</th>
        <th class="sortable" onclick={() => handleSort('status')}>Status</th>
        <th>Claim</th>
        <th>Storage Class</th>
        <th class="sortable" onclick={() => handleSort('age')}>Age</th>
        <th>Actions</th>
      </svelte:fragment>

      <svelte:fragment slot="rows">
        {#each filteredPVs as pv (pv.metadata.uid)}
          <tr class="resource-row" oncontextmenu={(e) => handleContextMenu(e, pv)}>
            <td class="name-cell">{pv.metadata.name}</td>
            <td>{pv.spec?.capacity?.storage || '-'}</td>
            <td>{pv.spec?.accessModes?.join(', ') || '-'}</td>
            <td>{pv.spec?.persistentVolumeReclaimPolicy || '-'}</td>
            <td>
              <span class="status-badge status-{getStatusClass(pv.status?.phase)}">
                {pv.status?.phase || 'Unknown'}
              </span>
            </td>
            <td>
              {#if pv.spec?.claimRef}
                {pv.spec.claimRef.namespace}/{pv.spec.claimRef.name}
              {:else}
                -
              {/if}
            </td>
            <td>{pv.spec?.storageClassName || '-'}</td>
            <td>{formatAge(pv.metadata.creationTimestamp)}</td>
            <td class="actions-cell">
              <button class="action-btn" onclick={(e) => handleContextMenu(e, pv)}>⚙️</button>
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
    resourceType="persistentvolume"
    bind:visible={contextMenuVisible}
    on:close={closeContextMenu}
  />
{/if}

<style>
  .pv-panel {
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

  .status-badge {
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  .status-ready { background: rgba(34, 197, 94, 0.2); color: #22c55e; }
  .status-pending { background: rgba(234, 179, 8, 0.2); color: #eab308; }
  .status-failed { background: rgba(239, 68, 68, 0.2); color: #ef4444; }
  .status-unknown { background: rgba(107, 114, 128, 0.2); color: #6b7280; }

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
