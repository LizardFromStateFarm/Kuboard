<!-- Kuboard ClusterRoleBindings Panel Component -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import ResourceTable from './ResourceTable.svelte';
  import QuickActionsMenu from './QuickActionsMenu.svelte';
  import RoleBindingDetails from './RoleBindingDetails.svelte';
  import { Link2 } from 'lucide-svelte';

  // Props
  export let currentContext: any = null;

  // State
  let clusterRoleBindings: any[] = [];
  let loading: boolean = true;
  let error: string | null = null;
  let searchQuery: string = '';
  let sortColumn: string = 'name';
  let sortDirection: 'asc' | 'desc' = 'asc';
  let refreshTimer: any;
  let selectedClusterRoleBinding: any = null;

  // Context Menu State
  let contextMenuVisible = false;
  let contextMenuPosition = { x: 0, y: 0 };
  let selectedResource: any = null;

  async function fetchClusterRoleBindings() {
    if (!currentContext) return;
    
    try {
      loading = true;
      clusterRoleBindings = await invoke('kuboard_list_cluster_role_bindings');
      error = null;
    } catch (e: any) {
      console.error('Failed to fetch ClusterRoleBindings:', e);
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

  function handleContextMenu(event: MouseEvent, crb: any) {
    event.preventDefault();
    selectedResource = crb;
    contextMenuPosition = { x: event.clientX, y: event.clientY };
    contextMenuVisible = true;
  }

  function closeContextMenu() {
    contextMenuVisible = false;
    selectedResource = null;
  }

  $: filteredBindings = clusterRoleBindings
    .filter(crb => {
      const name = crb.metadata.name.toLowerCase();
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
    fetchClusterRoleBindings();
    refreshTimer = setInterval(fetchClusterRoleBindings, 30000); // Refresh every 30s
  });

  onDestroy(() => {
    if (refreshTimer) clearInterval(refreshTimer);
  });

  $: if (currentContext) {
    fetchClusterRoleBindings();
  }
</script>

<div class="cluster-role-bindings-panel">
  <div class="panel-header">
    <h4><Link2 size={16} /> Cluster Role Bindings ({filteredBindings.length})</h4>
  </div>

  {#if selectedClusterRoleBinding}
    <RoleBindingDetails binding={selectedClusterRoleBinding} isClusterScoped={true} onBack={() => selectedClusterRoleBinding = null} />
  {:else if loading && clusterRoleBindings.length === 0}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Loading Cluster Role Bindings...</p>
    </div>
  {:else if error}
    <div class="error-state">
      <span class="error-icon">⚠️</span>
      <p>{error}</p>
      <button onclick={fetchClusterRoleBindings}>Retry</button>
    </div>
  {:else}
    <ResourceTable
      items={clusterRoleBindings}
      filteredItems={filteredBindings}
      bind:searchQuery
      searchPlaceholder="Search Cluster Role Bindings..."
      noItemsMessage="No Cluster Role Bindings found."
      noSearchResultsMessage="No Cluster Role Bindings match your search query:"
    >
      <svelte:fragment slot="header">
        <th class="sortable" onclick={() => handleSort('name')}>Name</th>
        <th>Role Ref</th>
        <th class="sortable" onclick={() => handleSort('age')}>Age</th>
        <th>Actions</th>
      </svelte:fragment>

      <svelte:fragment slot="rows">
        {#each filteredBindings as crb (crb.metadata.uid)}
          <tr class="resource-row clickable-row" onclick={() => selectedClusterRoleBinding = crb} oncontextmenu={(e) => handleContextMenu(e, crb)}>
            <td class="name-cell">{crb.metadata.name}</td>
            <td>
              <span class="role-ref">
                {crb.roleRef.kind}: {crb.roleRef.name}
              </span>
            </td>
            <td>{formatAge(crb.metadata.creationTimestamp)}</td>
            <td class="actions-cell">
              <button class="action-btn" onclick={(e) => { e.stopPropagation(); handleContextMenu(e, crb); }}>⚙️</button>
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
    resourceType="clusterrolebinding"
    bind:visible={contextMenuVisible}
    on:close={closeContextMenu}
  />
{/if}

<style>
  .cluster-role-bindings-panel {
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

  .role-ref {
    font-size: 0.8rem;
    color: var(--text-secondary);
    background: rgba(255, 255, 255, 0.05);
    padding: 2px 6px;
    border-radius: 4px;
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
