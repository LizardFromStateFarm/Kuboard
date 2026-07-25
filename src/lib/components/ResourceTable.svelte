<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { Search, RefreshCw, Trash2, X } from 'lucide-svelte';

  export let items: any[] = [];
  export let filteredItems: any[] = [];
  export let searchQuery: string = '';
  export let searchPlaceholder: string = 'Search...';
  export let noItemsMessage: string = 'No items available in this context.';
  export let noSearchResultsMessage: string = 'No items match your search query:';
  export let selectedCount: number = 0;

  const dispatch = createEventDispatcher();

  function triggerBulkDelete() {
    dispatch('bulkDelete');
  }

  function triggerBulkRestart() {
    dispatch('bulkRestart');
  }

  function triggerClearSelection() {
    dispatch('clearSelection');
  }
</script>

{#if items && items.length > 0}
  <!-- Search & Bulk Actions Bar -->
  <div class="search-bar-container">
    <div class="search-input-wrapper">
      <span class="search-icon"><Search size={14} /></span>
      <input
        type="text"
        class="search-input"
        placeholder={searchPlaceholder}
        bind:value={searchQuery}
        autocomplete="off"
      />
      {#if searchQuery}
        <button class="search-clear" onclick={() => searchQuery = ''} title="Clear search"><X size={14} /></button>
      {/if}
    </div>
    <div class="search-results-row">
      <div class="search-results-count">
        {#if searchQuery}
          Showing {filteredItems.length} of {items.length} items
        {:else}
          {items.length} items
        {/if}
      </div>

      {#if selectedCount > 0}
        <div class="bulk-action-bar">
          <span class="selected-badge">Selected: {selectedCount}</span>
          <button class="btn-bulk btn-restart" onclick={triggerBulkRestart}><RefreshCw size={13} /> Bulk Restart ({selectedCount})</button>
          <button class="btn-bulk btn-delete" onclick={triggerBulkDelete}><Trash2 size={13} /> Bulk Delete ({selectedCount})</button>
          <button class="btn-bulk btn-clear" onclick={triggerClearSelection}><X size={13} /> Clear</button>
        </div>
      {/if}
    </div>
  </div>

  {#if filteredItems.length === 0 && searchQuery}
    <div class="no-search-results">
      <div class="no-results-icon">🔍</div>
      <h5>No Results Found</h5>
      <p>{noSearchResultsMessage} <strong>"{searchQuery}"</strong></p>
      <button class="clear-search-button" onclick={() => searchQuery = ''}>Clear Search</button>
    </div>
  {:else}
    <div class="table-container">
      <slot name="table">
        <table class="resource-table">
          <thead>
            <slot name="header"></slot>
          </thead>
          <tbody>
            <slot name="rows"></slot>
          </tbody>
        </table>
      </slot>
    </div>
  {/if}
{:else}
  <!-- No Items Available -->
  <div class="no-items-message">
    <div class="no-items-icon">🟢</div>
    <h5>No Items Available</h5>
    <p>{noItemsMessage}</p>
  </div>
{/if}

<style>
  .search-bar-container {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-md);
  }

  .search-input-wrapper {
    position: relative;
    display: flex;
    align-items: center;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    padding: 0 var(--spacing-sm);
    transition: border-color 0.2s ease, background 0.2s ease;
  }

  .search-input-wrapper:focus-within {
    border-color: var(--primary-color);
    background: rgba(255, 255, 255, 0.08);
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
  }

  .search-icon {
    font-size: 1rem;
    color: var(--text-secondary);
    margin-right: var(--spacing-xs);
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: var(--text-primary);
    font-size: 0.9rem;
    padding: var(--spacing-sm) 0;
    width: 100%;
  }

  .search-input::placeholder {
    color: var(--text-muted);
    font-size: 0.85rem;
  }

  .search-clear {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 1.5rem;
    line-height: 1;
    padding: 0 var(--spacing-xs);
    transition: color 0.2s ease, transform 0.2s ease;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .search-clear:hover {
    color: var(--text-primary);
    transform: scale(1.1);
  }

  .search-results-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .search-results-count {
    font-size: 0.85rem;
    color: var(--text-muted);
    font-weight: 500;
    padding-left: var(--spacing-xs);
  }

  .bulk-action-bar {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .selected-badge {
    background: rgba(59, 130, 246, 0.15);
    color: #60a5fa;
    font-weight: 700;
    font-size: 0.8rem;
    padding: 3px 8px;
    border-radius: 12px;
  }

  .btn-bulk {
    border: none;
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    font-weight: 600;
    font-size: 0.8rem;
    cursor: pointer;
    transition: opacity 0.15s ease;
  }

  .btn-bulk.btn-restart { background: #eab308; color: black; }
  .btn-bulk.btn-delete { background: #ef4444; color: white; }
  .btn-bulk.btn-clear { background: rgba(255, 255, 255, 0.1); color: var(--text-secondary); }
  .btn-bulk:hover { opacity: 0.9; }

  .no-search-results {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-xl);
    text-align: center;
    background: rgba(255, 255, 255, 0.03);
    border-radius: var(--radius-md);
    border: 1px dashed var(--border-primary);
    margin-top: var(--spacing-md);
  }

  .no-results-icon {
    font-size: 3rem;
    opacity: 0.6;
    margin-bottom: var(--spacing-sm);
  }

  .no-search-results h5 {
    margin: 0 0 var(--spacing-sm) 0;
    color: var(--text-primary);
    font-size: 1.1rem;
  }

  .no-search-results p {
    margin: 0 0 var(--spacing-md) 0;
    color: var(--text-secondary);
    font-size: 0.9rem;
  }

  .no-search-results strong {
    color: var(--text-primary);
    font-weight: 600;
  }

  .clear-search-button {
    background: var(--primary-color);
    border: 1px solid var(--primary-color);
    border-radius: var(--radius-sm);
    color: white;
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 500;
    padding: var(--spacing-xs) var(--spacing-md);
    transition: all 0.2s ease;
  }

  .clear-search-button:hover {
    background: var(--accent-color);
    border-color: var(--accent-color);
    transform: translateY(-1px);
    box-shadow: 0 2px 4px rgba(0,0,0,0.2);
  }

  .table-container {
    overflow-x: auto;
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    background: var(--background-card);
  }

  .resource-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.88rem;
    table-layout: fixed;
  }

  :global(.resource-table th) {
    padding: 10px 14px;
    background: rgba(255, 255, 255, 0.04);
    border-bottom: 2px solid var(--border-primary);
    color: var(--text-secondary);
    font-weight: 600;
    text-align: left;
    white-space: nowrap;
    position: relative;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 50px;
    user-select: none;
    resize: horizontal;
  }

  :global(.resource-table th:hover) {
    background: rgba(255, 255, 255, 0.08);
  }

  :global(.resource-table td) {
    padding: 8px 14px;
    border-bottom: 1px solid var(--border-primary);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .no-items-message {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 60px 20px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px dashed var(--border-primary);
    border-radius: 8px;
    margin-top: 20px;
  }

  .no-items-icon {
    font-size: 48px;
    margin-bottom: 16px;
    opacity: 0.7;
  }
  
  .no-items-message h5 {
    margin: 0 0 8px 0;
    font-size: 1.2rem;
    color: var(--text-primary);
  }
  
  .no-items-message p {
    margin: 0;
    color: var(--text-secondary);
  }
</style>
