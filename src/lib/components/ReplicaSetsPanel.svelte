<!-- Kuboard ReplicaSets Panel Component -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import ReplicaSetDetails from './ReplicaSetDetails.svelte';
  import QuickActionsMenu from './QuickActionsMenu.svelte';

  // Props
  export let currentContext: any = null;
  export let replicasets: any[] = [];

  // State
  let selectedReplicaSet: any = null;
  let showFullDetails: boolean = false;
  let isLoading: boolean = false;
  
  // Sorting state
  let sortColumn: string | null = null;
  let sortDirection: 'asc' | 'desc' | null = null;

  // Search state
  let searchQuery: string = '';

  // Quick Actions Menu state
  let contextMenuVisible = false;
  let contextMenuPosition = { x: 0, y: 0 };
  let contextMenuReplicaSet: any = null;

  function handleContextMenu(event: MouseEvent, rs: any) {
    event.preventDefault();
    event.stopPropagation();
    contextMenuReplicaSet = rs;
    contextMenuPosition = { x: event.clientX, y: event.clientY };
    contextMenuVisible = true;
  }

  function handleActionMenuClose() {
    contextMenuVisible = false;
    contextMenuReplicaSet = null;
  }

  function handleActionDeleted(event: CustomEvent) {
    // Reload replicasets would be needed
    handleActionMenuClose();
  }

  // Format age
  function formatAge(creationTimestamp: string): string {
    if (!creationTimestamp) return 'Unknown';
    const created = new Date(creationTimestamp);
    const now = new Date();
    const diffMs = now.getTime() - created.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMins / 60);
    const diffDays = Math.floor(diffHours / 24);
    
    if (diffDays > 0) return `${diffDays}d`;
    if (diffHours > 0) return `${diffHours}h`;
    return `${diffMins}m`;
  }

  // Get replica status
  function getReplicaStatus(rs: any): string {
    const desired = rs.spec?.replicas || 0;
    const ready = rs.status?.readyReplicas || 0;
    const current = rs.status?.replicas || 0;
    
    if (ready === desired && current === desired) return 'Ready';
    if (current < desired) return 'Scaling';
    if (ready < desired) return 'Not Ready';
    return 'Unknown';
  }

  function getStatusClass(status: string): string {
    switch (status?.toLowerCase()) {
      case 'ready': return 'ready';
      case 'scaling': return 'pending';
      case 'not ready': return 'failed';
      default: return 'unknown';
    }
  }

  // Get owner reference (e.g., Deployment)
  function getOwnerReference(rs: any): string {
    const ownerRefs = rs.metadata?.ownerReferences || [];
    if (ownerRefs.length === 0) return '-';
    const owner = ownerRefs[0];
    return `${owner.kind}/${owner.name}`;
  }

  // Sorting functions
  function handleSort(column: string, event?: Event) {
    if (event) {
      event.stopPropagation();
      event.preventDefault();
    }
    
    if (sortColumn === column) {
      if (sortDirection === 'asc') {
        sortDirection = 'desc';
      } else if (sortDirection === 'desc') {
        sortColumn = null;
        sortDirection = null;
      }
    } else {
      sortColumn = column;
      sortDirection = 'asc';
    }
  }

  // Comparison functions
  function compareName(a: any, b: any): number {
    const nameA = a.metadata?.name || '';
    const nameB = b.metadata?.name || '';
    return nameA.localeCompare(nameB);
  }

  function compareNamespace(a: any, b: any): number {
    const nsA = a.metadata?.namespace || '';
    const nsB = b.metadata?.namespace || '';
    return nsA.localeCompare(nsB);
  }

  function compareReplicas(a: any, b: any): number {
    const readyA = a.status?.readyReplicas || 0;
    const readyB = b.status?.readyReplicas || 0;
    return readyA - readyB;
  }

  function compareAge(a: any, b: any): number {
    const timeA = new Date(a.metadata?.creationTimestamp || 0).getTime();
    const timeB = new Date(b.metadata?.creationTimestamp || 0).getTime();
    return timeA - timeB;
  }

  // Reactive sorted and filtered replicasets
  $: sortedReplicaSets = (() => {
    if (!sortColumn || !sortDirection) {
      return replicasets;
    }
    
    const sorted = [...replicasets];
    sorted.sort((a, b) => {
      let comparison = 0;
      
      switch (sortColumn) {
        case 'name':
          comparison = compareName(a, b);
          break;
        case 'namespace':
          comparison = compareNamespace(a, b);
          break;
        case 'replicas':
          comparison = compareReplicas(a, b);
          break;
        case 'age':
          comparison = compareAge(a, b);
          break;
        default:
          return 0;
      }
      
      return sortDirection === 'asc' ? comparison : -comparison;
    });
    
    return sorted;
  })();

  $: filteredReplicaSets = (() => {
    if (!searchQuery || !searchQuery.trim()) {
      return sortedReplicaSets;
    }

    const query = searchQuery.toLowerCase().trim();
    return sortedReplicaSets.filter(rs => {
      const name = (rs.metadata?.name || '').toLowerCase();
      const namespace = (rs.metadata?.namespace || '').toLowerCase();
      const owner = getOwnerReference(rs).toLowerCase();
      
      return name.includes(query) || namespace.includes(query) || owner.includes(query);
    });
  })();

  // Show full details view
  function showFullReplicaSetDetails(rs: any) {
    selectedReplicaSet = rs;
    showFullDetails = true;
  }

  // Back to replicasets list
  function backToReplicaSetsList() {
    showFullDetails = false;
    selectedReplicaSet = null;
  }

  // Set loading state when context changes but replicasets haven't loaded yet
  $: if (currentContext && (!replicasets || replicasets.length === 0)) {
    isLoading = true;
  } else {
    isLoading = false;
  }
</script>

{#if showFullDetails && selectedReplicaSet}
  <ReplicaSetDetails replicaSet={selectedReplicaSet} onBack={backToReplicaSetsList} />
{:else}
  <div class="replicasets-panel">
    <div class="panel-header">
      <h4>📦 ReplicaSets ({filteredReplicaSets.length})</h4>
      <div class="header-controls">
        <input
          type="text"
          class="search-input"
          placeholder="Search ReplicaSets..."
          bind:value={searchQuery}
        />
      </div>
    </div>

    {#if isLoading}
      <div class="loading-message">
        <div class="loading-spinner">⏳</div>
        <p>Loading ReplicaSets...</p>
      </div>
    {:else if replicasets.length === 0}
      <div class="empty-state">
        <div class="empty-icon">📭</div>
        <h5>No ReplicaSets Found</h5>
        <p>No ReplicaSets are currently in this cluster</p>
      </div>
    {:else if filteredReplicaSets.length === 0}
      <div class="empty-state">
        <div class="empty-icon">🔍</div>
        <h5>No ReplicaSets Match</h5>
        <p>No ReplicaSets match your search query: "{searchQuery}"</p>
      </div>
    {:else}
      <div class="replicasets-table">
        <div class="table-header">
          <div class="header-cell sortable" onclick={() => handleSort('name')} role="button" tabindex="0">
            Name
            {#if sortColumn === 'name'}
              <span class="sort-indicator">{sortDirection === 'asc' ? '↑' : '↓'}</span>
            {/if}
          </div>
          <div class="header-cell sortable" onclick={() => handleSort('namespace')} role="button" tabindex="0">
            Namespace
            {#if sortColumn === 'namespace'}
              <span class="sort-indicator">{sortDirection === 'asc' ? '↑' : '↓'}</span>
            {/if}
          </div>
          <div class="header-cell sortable" onclick={() => handleSort('replicas')} role="button" tabindex="0">
            Replicas
            {#if sortColumn === 'replicas'}
              <span class="sort-indicator">{sortDirection === 'asc' ? '↑' : '↓'}</span>
            {/if}
          </div>
          <div class="header-cell">Status</div>
          <div class="header-cell">Owner</div>
          <div class="header-cell sortable" onclick={() => handleSort('age')} role="button" tabindex="0">
            Age
            {#if sortColumn === 'age'}
              <span class="sort-indicator">{sortDirection === 'asc' ? '↑' : '↓'}</span>
            {/if}
          </div>
          <div class="header-cell">Actions</div>
        </div>

        <div class="table-body">
          {#each filteredReplicaSets as rs}
            {@const status = getReplicaStatus(rs)}
            {@const desired = rs.spec?.replicas || 0}
            {@const ready = rs.status?.readyReplicas || 0}
            {@const current = rs.status?.replicas || 0}
            <div
              class="table-row"
              role="button"
              tabindex="0"
              onclick={() => showFullReplicaSetDetails(rs)}
              onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && showFullReplicaSetDetails(rs)}
              oncontextmenu={(e) => handleContextMenu(e, rs)}
            >
              <div class="cell name-cell">
                <span class="resource-name">{rs.metadata?.name || 'Unknown'}</span>
              </div>
              <div class="cell namespace-cell">
                <span>{rs.metadata?.namespace || 'default'}</span>
              </div>
              <div class="cell replicas-cell">
                <span class="replica-info">{ready}/{desired}</span>
                {#if current !== desired}
                  <span class="replica-warning">({current} current)</span>
                {/if}
              </div>
              <div class="cell status-cell">
                <span class="status-badge status-{getStatusClass(status)}">{status}</span>
              </div>
              <div class="cell owner-cell">
                <span>{getOwnerReference(rs)}</span>
              </div>
              <div class="cell age-cell">
                <span>{formatAge(rs.metadata?.creationTimestamp)}</span>
              </div>
              <div class="cell actions-cell" onclick={(e) => { e.stopPropagation(); handleContextMenu(e, rs); }}>
                <button class="action-button" title="Actions">⚙️</button>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>

  <!-- Quick Actions Menu -->
  {#if contextMenuVisible && contextMenuReplicaSet}
    <QuickActionsMenu
      x={contextMenuPosition.x}
      y={contextMenuPosition.y}
      resource={contextMenuReplicaSet}
      resourceType="replicaset"
      on:close={handleActionMenuClose}
      on:deleted={handleActionDeleted}
    />
  {/if}
{/if}

<style>
  @import '../styles/variables.css';

  .replicasets-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-md);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.05);
  }

  .panel-header h4 {
    margin: 0;
    color: var(--text-primary);
    font-size: 1.1rem;
    font-weight: 600;
  }

  .header-controls {
    display: flex;
    gap: var(--spacing-sm);
    align-items: center;
  }

  .search-input {
    padding: 8px 12px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-sm);
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
    font-size: 0.9rem;
    width: 250px;
    transition: all 0.2s;
  }

  .search-input:focus {
    outline: none;
    border-color: var(--primary-color);
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
  }

  .loading-message, .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-xl);
    gap: var(--spacing-md);
    color: var(--text-secondary);
  }

  .loading-spinner {
    font-size: 2rem;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .empty-icon {
    font-size: 3rem;
    opacity: 0.7;
  }

  .empty-state h5 {
    margin: 0;
    color: var(--text-primary);
    font-size: 1.1rem;
  }

  .empty-state p {
    margin: 0;
    font-size: 0.9rem;
  }

  .replicasets-table {
    flex: 1;
    overflow: auto;
    display: flex;
    flex-direction: column;
  }

  .table-header {
    display: grid;
    grid-template-columns: 2fr 1.5fr 1fr 1fr 1.5fr 1fr 80px;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    background: rgba(255, 255, 255, 0.05);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    position: sticky;
    top: 0;
    z-index: 10;
  }

  .header-cell {
    color: var(--text-secondary);
    font-size: 0.85rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .header-cell.sortable {
    cursor: pointer;
    user-select: none;
    transition: color 0.2s;
  }

  .header-cell.sortable:hover {
    color: var(--primary-color);
  }

  .sort-indicator {
    color: var(--primary-color);
    font-size: 0.8rem;
  }

  .table-body {
    display: flex;
    flex-direction: column;
  }

  .table-row {
    display: grid;
    grid-template-columns: 2fr 1.5fr 1fr 1fr 1.5fr 1fr 80px;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    cursor: pointer;
    transition: background 0.2s;
  }

  .table-row:hover {
    background: rgba(255, 255, 255, 0.03);
  }

  .table-row:focus {
    outline: 2px solid var(--primary-color);
    outline-offset: -2px;
  }

  .cell {
    display: flex;
    align-items: center;
    color: var(--text-primary);
    font-size: 0.9rem;
  }

  .name-cell .resource-name {
    font-weight: 600;
    color: var(--primary-color);
  }

  .namespace-cell {
    color: var(--text-secondary);
  }

  .replicas-cell {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .replica-info {
    font-weight: 500;
  }

  .replica-warning {
    font-size: 0.75rem;
    color: var(--warning-color);
  }

  .status-badge {
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .status-ready {
    background: rgba(16, 185, 129, 0.2);
    color: #10b981;
  }

  .status-pending {
    background: rgba(245, 158, 11, 0.2);
    color: #f59e0b;
  }

  .status-failed {
    background: rgba(239, 68, 68, 0.2);
    color: #ef4444;
  }

  .status-unknown {
    background: rgba(107, 114, 128, 0.2);
    color: #6b7280;
  }

  .owner-cell {
    color: var(--text-secondary);
    font-size: 0.85rem;
  }

  .age-cell {
    color: var(--text-secondary);
    font-size: 0.85rem;
  }

  .actions-cell {
    display: flex;
    justify-content: center;
  }

  .action-button {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    transition: all 0.2s;
    font-size: 0.9rem;
  }

  .action-button:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
  }
</style>

