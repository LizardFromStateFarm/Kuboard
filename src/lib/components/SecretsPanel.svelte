<!-- Kuboard Secrets Panel Component -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import ResourceTable from './ResourceTable.svelte';
  import QuickActionsMenu from './QuickActionsMenu.svelte';
  import SecretDetails from './SecretDetails.svelte';
  import { Key } from 'lucide-svelte';

  // Props
  export let currentContext: any = null;
  export let namespace: string = 'all';

  // State
  let secrets: any[] = [];
  let loading: boolean = true;
  let error: string | null = null;
  let searchQuery: string = '';
  let sortColumn: string = 'name';
  let sortDirection: 'asc' | 'desc' = 'asc';
  let selectedSecret: any = null;
  let showDecoded: Record<string, boolean> = {};

  // Context Menu State
  let contextMenuVisible = false;
  let contextMenuPosition = { x: 0, y: 0 };
  let contextMenuResource: any = null;

  async function fetchSecrets() {
    if (!currentContext) return;
    try {
      loading = true;
      secrets = await invoke('kuboard_get_secrets');
      error = null;
    } catch (e: any) {
      console.error('Failed to fetch Secrets:', e);
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

  function toggleDecode(key: string) {
    showDecoded[key] = !showDecoded[key];
    showDecoded = { ...showDecoded };
  }

  function decodeBase64(str: string): string {
    try {
      return atob(str);
    } catch {
      return str;
    }
  }

  function handleContextMenu(event: MouseEvent, sec: any) {
    event.preventDefault();
    event.stopPropagation();
    contextMenuResource = sec;
    contextMenuPosition = { x: event.clientX, y: event.clientY };
    contextMenuVisible = true;
  }

  $: filteredSecrets = secrets.filter(sec => {
    const matchesNamespace = namespace === 'all' || sec.metadata?.namespace === namespace;
    const q = searchQuery.toLowerCase();
    const matchesSearch = !q || 
      (sec.metadata?.name || '').toLowerCase().includes(q) ||
      (sec.metadata?.namespace || '').toLowerCase().includes(q) ||
      (sec.type || '').toLowerCase().includes(q);
    return matchesNamespace && matchesSearch;
  });

  $: sortedSecrets = [...filteredSecrets].sort((a, b) => {
    let comp = 0;
    if (sortColumn === 'name') {
      comp = (a.metadata?.name || '').localeCompare(b.metadata?.name || '');
    } else if (sortColumn === 'namespace') {
      comp = (a.metadata?.namespace || '').localeCompare(b.metadata?.namespace || '');
    } else if (sortColumn === 'type') {
      comp = (a.type || '').localeCompare(b.type || '');
    } else if (sortColumn === 'keys') {
      const keysA = Object.keys(a.data || a.stringData || {}).length;
      const keysB = Object.keys(b.data || b.stringData || {}).length;
      comp = keysA - keysB;
    }
    return sortDirection === 'asc' ? comp : -comp;
  });

  onMount(() => {
    fetchSecrets();
  });

  $: if (currentContext) {
    fetchSecrets();
  }
</script>

<div class="secrets-panel">
  <div class="panel-header">
    <h4><Key size={16} /> Secrets ({filteredSecrets.length})</h4>
    <button class="refresh-btn" onclick={fetchSecrets} disabled={loading}>
      {loading ? '🔄' : '↻ Refresh'}
    </button>
  </div>

  {#if selectedSecret}
    <SecretDetails secret={selectedSecret} onBack={() => selectedSecret = null} />
  {:else}
    <ResourceTable
      items={secrets}
      filteredItems={sortedSecrets}
      bind:searchQuery
      searchPlaceholder="Search secrets by name, namespace, or type..."
      noItemsMessage="No secrets found in this context."
      noSearchResultsMessage="No secrets match your search:"
    >
      <svelte:fragment slot="header">
        <tr>
          <th class="sortable" onclick={() => handleSort('name')}>
            Name {sortColumn === 'name' ? (sortDirection === 'asc' ? '▲' : '▼') : ''}
          </th>
          <th class="sortable" onclick={() => handleSort('namespace')}>
            Namespace {sortColumn === 'namespace' ? (sortDirection === 'asc' ? '▲' : '▼') : ''}
          </th>
          <th class="sortable" onclick={() => handleSort('type')}>
            Type {sortColumn === 'type' ? (sortDirection === 'asc' ? '▲' : '▼') : ''}
          </th>
          <th class="sortable" onclick={() => handleSort('keys')}>
            Data Keys {sortColumn === 'keys' ? (sortDirection === 'asc' ? '▲' : '▼') : ''}
          </th>
          <th>Age</th>
        </tr>
      </svelte:fragment>

      <svelte:fragment slot="rows">
        {#each sortedSecrets as sec}
          <tr 
            class="clickable-row" 
            onclick={() => selectedSecret = sec}
            oncontextmenu={(e) => handleContextMenu(e, sec)}
          >
            <td class="name-cell"><Key size={14} /> {sec.metadata?.name}</td>
            <td>{sec.metadata?.namespace || 'default'}</td>
            <td><span class="type-badge">{sec.type || 'Opaque'}</span></td>
            <td>{Object.keys(sec.data || sec.stringData || {}).length} keys</td>
            <td>{formatAge(sec.metadata?.creationTimestamp)}</td>
          </tr>
        {/each}
      </svelte:fragment>
    </ResourceTable>
  {/if}
</div>

{#if contextMenuResource}
  <QuickActionsMenu
    resource={contextMenuResource}
    resourceType="secret"
    position={contextMenuPosition}
    bind:visible={contextMenuVisible}
    on:close={() => contextMenuResource = null}
    on:deleted={fetchSecrets}
  />
{/if}

<style>
  .secrets-panel {
    padding: 12px;
  }
  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }
  .panel-header h4 {
    margin: 0;
    color: white;
  }
  .refresh-btn {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-sm);
    color: white;
    padding: 4px 10px;
    cursor: pointer;
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
  .type-badge {
    padding: 2px 6px;
    background: rgba(59, 130, 246, 0.15);
    color: #60a5fa;
    border-radius: var(--radius-sm);
    font-size: 0.8rem;
  }
  .secret-details-view {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .details-header {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .back-btn {
    background: rgba(255, 255, 255, 0.1);
    border: none;
    color: white;
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    cursor: pointer;
  }
  .details-card {
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    padding: 16px;
  }
  .details-card h5 {
    margin: 0 0 12px 0;
    color: white;
  }
  .meta-row {
    margin-bottom: 6px;
    color: var(--text-secondary);
  }
  .data-keys-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .data-key-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: rgba(0, 0, 0, 0.2);
    padding: 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-primary);
  }
  .key-value-wrapper {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .masked-val {
    color: var(--text-muted);
    font-family: monospace;
  }
  .decoded-val {
    margin: 0;
    font-family: monospace;
    color: var(--success-color);
    background: rgba(0, 0, 0, 0.4);
    padding: 4px 8px;
    border-radius: 4px;
    max-width: 400px;
    overflow-x: auto;
  }
  .toggle-decode-btn {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    color: white;
    font-size: 0.8rem;
    padding: 4px 8px;
    border-radius: 4px;
    cursor: pointer;
  }
</style>
