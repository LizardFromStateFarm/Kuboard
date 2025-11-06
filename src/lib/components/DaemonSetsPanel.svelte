<!-- Kuboard DaemonSets Panel Component -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import DaemonSetDetails from './DaemonSetDetails.svelte';
  import QuickActionsMenu from './QuickActionsMenu.svelte';

  // Props
  export let currentContext: any = null;
  export let daemonsets: any[] = [];

  // State
  let selectedDaemonSet: any = null;
  let showFullDetails: boolean = false;
  
  // Sorting state
  let sortColumn: string | null = null;
  let sortDirection: 'asc' | 'desc' | null = null;

  // Search state
  let searchQuery: string = '';

  // Quick Actions Menu state
  let contextMenuVisible = false;
  let contextMenuPosition = { x: 0, y: 0 };
  let contextMenuDaemonSet: any = null;

  function handleContextMenu(event: MouseEvent, ds: any) {
    event.preventDefault();
    event.stopPropagation();
    contextMenuDaemonSet = ds;
    contextMenuPosition = { x: event.clientX, y: event.clientY };
    contextMenuVisible = true;
  }

  function handleActionMenuClose() {
    contextMenuVisible = false;
    contextMenuDaemonSet = null;
  }

  function handleActionDeleted(event: CustomEvent) {
    // Reload daemonsets would be needed
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

  // Get DaemonSet status
  function getDaemonSetStatus(ds: any): string {
    const desired = ds.status?.desiredNumberScheduled || 0;
    const ready = ds.status?.numberReady || 0;
    const current = ds.status?.currentNumberScheduled || 0;
    const available = ds.status?.numberAvailable || 0;
    
    if (ready === desired && current === desired && available === desired) return 'Ready';
    if (current < desired) return 'Rolling Out';
    if (ready < desired) return 'Not Ready';
    return 'Unknown';
  }

  function getStatusClass(status: string): string {
    switch (status?.toLowerCase()) {
      case 'ready': return 'ready';
      case 'rolling out': return 'pending';
      case 'not ready': return 'failed';
      default: return 'unknown';
    }
  }

  // Get update strategy
  function getUpdateStrategy(ds: any): string {
    const strategy = ds.spec?.updateStrategy?.type || 'RollingUpdate';
    return strategy;
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
    const readyA = a.status?.numberReady || 0;
    const readyB = b.status?.numberReady || 0;
    return readyA - readyB;
  }

  function compareAge(a: any, b: any): number {
    const timeA = new Date(a.metadata?.creationTimestamp || 0).getTime();
    const timeB = new Date(b.metadata?.creationTimestamp || 0).getTime();
    return timeA - timeB;
  }

  // Reactive sorted and filtered daemonsets
  $: sortedDaemonSets = (() => {
    if (!sortColumn || !sortDirection) {
      return daemonsets;
    }
    
    const sorted = [...daemonsets];
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

  $: filteredDaemonSets = (() => {
    if (!searchQuery || !searchQuery.trim()) {
      return sortedDaemonSets;
    }

    const query = searchQuery.toLowerCase().trim();
    return sortedDaemonSets.filter(ds => {
      const name = (ds.metadata?.name || '').toLowerCase();
      const namespace = (ds.metadata?.namespace || '').toLowerCase();
      
      return name.includes(query) || namespace.includes(query);
    });
  })();

  // Show full details view
  function showFullDaemonSetDetails(ds: any) {
    selectedDaemonSet = ds;
    showFullDetails = true;
  }

  // Back to daemonsets list
  function backToDaemonSetsList() {
    showFullDetails = false;
    selectedDaemonSet = null;
  }

  // Loading state is managed by parent WorkloadsTab
  // This panel just displays the data it receives
</script>

{#if showFullDetails && selectedDaemonSet}
  <DaemonSetDetails daemonSet={selectedDaemonSet} onBack={backToDaemonSetsList} />
{:else}
  <div class="daemonsets-panel">
    <div class="panel-header">
      <h4>📦 DaemonSets ({filteredDaemonSets.length})</h4>
      <div class="header-controls">
        <input
          type="text"
          class="search-input"
          placeholder="Search DaemonSets..."
          bind:value={searchQuery}
        />
      </div>
    </div>

    {#if daemonsets.length === 0}
      <div class="empty-state">
        <div class="empty-icon">📭</div>
        <h5>No DaemonSets Found</h5>
        <p>No DaemonSets are currently in this cluster</p>
      </div>
    {:else if filteredDaemonSets.length === 0}
      <div class="empty-state">
        <div class="empty-icon">🔍</div>
        <h5>No DaemonSets Match</h5>
        <p>No DaemonSets match your search query: "{searchQuery}"</p>
      </div>
    {:else}
      <div class="daemonsets-table">
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
          <div class="header-cell">Update Strategy</div>
          <div class="header-cell sortable" onclick={() => handleSort('age')} role="button" tabindex="0">
            Age
            {#if sortColumn === 'age'}
              <span class="sort-indicator">{sortDirection === 'asc' ? '↑' : '↓'}</span>
            {/if}
          </div>
          <div class="header-cell">Actions</div>
        </div>

        <div class="table-body">
          {#each filteredDaemonSets as ds}
            {@const status = getDaemonSetStatus(ds)}
            {@const desired = ds.status?.desiredNumberScheduled || 0}
            {@const ready = ds.status?.numberReady || 0}
            {@const current = ds.status?.currentNumberScheduled || 0}
            {@const available = ds.status?.numberAvailable || 0}
            <div
              class="table-row"
              role="button"
              tabindex="0"
              onclick={() => showFullDaemonSetDetails(ds)}
              onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && showFullDaemonSetDetails(ds)}
              oncontextmenu={(e) => handleContextMenu(e, ds)}
            >
              <div class="cell name-cell">
                <span class="resource-name">{ds.metadata?.name || 'Unknown'}</span>
              </div>
              <div class="cell namespace-cell">
                <span>{ds.metadata?.namespace || 'default'}</span>
              </div>
              <div class="cell replicas-cell">
                <span class="replica-info">{ready}/{desired}</span>
                {#if current !== desired}
                  <span class="replica-warning">({current} current)</span>
                {/if}
                {#if available !== ready}
                  <span class="replica-warning">({available} available)</span>
                {/if}
              </div>
              <div class="cell status-cell">
                <span class="status-badge status-{getStatusClass(status)}">{status}</span>
              </div>
              <div class="cell strategy-cell">
                <span>{getUpdateStrategy(ds)}</span>
              </div>
              <div class="cell age-cell">
                <span>{formatAge(ds.metadata?.creationTimestamp)}</span>
              </div>
              <div class="cell actions-cell" onclick={(e) => { e.stopPropagation(); handleContextMenu(e, ds); }}>
                <button class="action-button" title="Actions">⚙️</button>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>

  <!-- Quick Actions Menu -->
  {#if contextMenuVisible && contextMenuDaemonSet}
    <QuickActionsMenu
      x={contextMenuPosition.x}
      y={contextMenuPosition.y}
      resource={contextMenuDaemonSet}
      resourceType="daemonset"
      on:close={handleActionMenuClose}
      on:deleted={handleActionDeleted}
    />
  {/if}
{/if}

<style>
  @import '../styles/variables.css';

  .daemonsets-panel {
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

  .daemonsets-table {
    flex: 1;
    overflow: auto;
    display: flex;
    flex-direction: column;
  }

  .table-header {
    display: grid;
    grid-template-columns: 2fr 1.5fr 1fr 1fr 1.2fr 1fr 80px;
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
    grid-template-columns: 2fr 1.5fr 1fr 1fr 1.2fr 1fr 80px;
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

  .strategy-cell {
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

