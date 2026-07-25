<!-- Kuboard Roles Panel Component -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import ResourceTable from './ResourceTable.svelte';
  import QuickActionsMenu from './QuickActionsMenu.svelte';
  import RoleDetails from './RoleDetails.svelte';
  import { ShieldCheck } from 'lucide-svelte';

  // Props
  export let currentContext: any = null;
  export let namespace: string = 'all';

  // State
  let roles: any[] = [];
  let loading: boolean = true;
  let error: string | null = null;
  let searchQuery: string = '';
  let sortColumn: string = 'name';
  let sortDirection: 'asc' | 'desc' = 'asc';
  let refreshTimer: any;
  let selectedRole: any = null;

  // Context Menu State
  let contextMenuVisible = false;
  let contextMenuPosition = { x: 0, y: 0 };
  let selectedResource: any = null;

  async function fetchRoles() {
    if (!currentContext) return;
    
    try {
      loading = true;
      roles = await invoke('kuboard_list_roles', { namespace });
      error = null;
    } catch (e: any) {
      console.error('Failed to fetch Roles:', e);
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

  function handleContextMenu(event: MouseEvent, role: any) {
    event.preventDefault();
    selectedResource = role;
    contextMenuPosition = { x: event.clientX, y: event.clientY };
    contextMenuVisible = true;
  }

  function closeContextMenu() {
    contextMenuVisible = false;
    selectedResource = null;
  }

  $: filteredRoles = roles
    .filter(role => {
      const name = role.metadata.name.toLowerCase();
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
    fetchRoles();
    refreshTimer = setInterval(fetchRoles, 15000); // Refresh every 15s
  });

  onDestroy(() => {
    if (refreshTimer) clearInterval(refreshTimer);
  });

  $: if (currentContext || namespace) {
    fetchRoles();
  }
</script>

<div class="roles-panel">
  {#if selectedRole}
    <RoleDetails role={selectedRole} onBack={() => selectedRole = null} />
  {:else}
    <div class="panel-header">
      <h4><ShieldCheck size={16} /> Roles ({filteredRoles.length})</h4>
    </div>

    {#if loading && roles.length === 0}
      <div class="loading-state">
        <div class="spinner"></div>
        <p>Loading Roles...</p>
      </div>
    {:else if error}
      <div class="error-state">
        <span class="error-icon">⚠️</span>
        <p>{error}</p>
        <button onclick={fetchRoles}>Retry</button>
      </div>
    {:else}
      <ResourceTable
        items={roles}
        filteredItems={filteredRoles}
        bind:searchQuery
        searchPlaceholder="Search Roles..."
        noItemsMessage="No Roles found."
        noSearchResultsMessage="No Roles match your search query:"
      >
        <svelte:fragment slot="header">
          <th class="sortable" onclick={() => handleSort('name')}>Name</th>
          <th class="sortable" onclick={() => handleSort('namespace')}>Namespace</th>
          <th class="sortable" onclick={() => handleSort('age')}>Age</th>
          <th>Actions</th>
        </svelte:fragment>

        <svelte:fragment slot="rows">
          {#each filteredRoles as role (role.metadata.uid)}
            <tr class="resource-row clickable-row" onclick={() => selectedRole = role} oncontextmenu={(e) => handleContextMenu(e, role)}>
              <td class="name-cell">{role.metadata.name}</td>
              <td>{role.metadata.namespace || 'Cluster-wide'}</td>
              <td>{formatAge(role.metadata.creationTimestamp)}</td>
              <td class="actions-cell">
                <button class="action-btn" onclick={(e) => { e.stopPropagation(); handleContextMenu(e, role); }}>⚙️</button>
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
    resourceType="role"
    bind:visible={contextMenuVisible}
    on:close={closeContextMenu}
  />
{/if}

<style>
  .roles-panel {
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
