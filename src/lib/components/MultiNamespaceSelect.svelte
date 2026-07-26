<!-- Kuboard Multi-Select Namespace Component -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Filter, Check, ChevronDown, Search, X } from 'lucide-svelte';

  import { createEventDispatcher } from 'svelte';
  const dispatch = createEventDispatcher();

  export let selectedNamespaces: string[] = ['all'];
  export let namespacesList: string[] = [];

  let isOpen = false;
  let searchTerm = '';
  let dropdownRef: HTMLDivElement;

  $: filteredNamespaces = namespacesList.filter(ns => 
    !searchTerm.trim() || ns.toLowerCase().includes(searchTerm.toLowerCase().trim())
  );

  $: isAllSelected = selectedNamespaces.length === 0 || selectedNamespaces.includes('all');

  function toggleOpen() {
    isOpen = !isOpen;
    if (isOpen) {
      searchTerm = '';
    }
  }

  function notifyChange() {
    dispatch('change', { selectedNamespaces: [...selectedNamespaces] });
  }

  function handleSelectAll() {
    selectedNamespaces = ['all'];
    notifyChange();
  }

  function handleClearAll() {
    selectedNamespaces = [];
    notifyChange();
  }

  function toggleNamespace(ns: string) {
    if (ns === 'all') {
      selectedNamespaces = ['all'];
      notifyChange();
      return;
    }

    let current = selectedNamespaces.filter(n => n !== 'all');
    if (current.includes(ns)) {
      current = current.filter(n => n !== ns);
    } else {
      current = [...current, ns];
    }

    if (current.length === 0) {
      selectedNamespaces = ['all'];
    } else {
      selectedNamespaces = [...current];
    }
    notifyChange();
  }

  function handleClickOutside(event: MouseEvent) {
    if (dropdownRef && !dropdownRef.contains(event.target as Node)) {
      isOpen = false;
    }
  }

  onMount(() => {
    document.addEventListener('click', handleClickOutside);
  });

  onDestroy(() => {
    document.removeEventListener('click', handleClickOutside);
  });

  $: buttonLabel = isAllSelected 
    ? 'All Namespaces' 
    : selectedNamespaces.length === 1 
      ? selectedNamespaces[0] 
      : `${selectedNamespaces.length} Namespaces`;
</script>

<div class="multi-ns-container" bind:this={dropdownRef}>
  <button 
    type="button" 
    class="multi-ns-btn" 
    class:active={!isAllSelected}
    onclick={toggleOpen}
    title="Filter by one or multiple namespaces"
  >
    <Filter size={13} class="ns-filter-icon" />
    <span class="ns-label-prefix">NS:</span>
    <span class="ns-label-text">{buttonLabel}</span>
    <ChevronDown size={13} class="ns-chevron {isOpen ? 'open' : ''}" />
  </button>

  {#if isOpen}
    <div class="multi-ns-popover">
      <div class="ns-header">
        <span class="ns-header-title">Select Namespaces</span>
        <div class="ns-header-actions">
          <button type="button" class="action-link" onclick={handleSelectAll}>All</button>
          <span class="sep">•</span>
          <button type="button" class="action-link" onclick={handleClearAll}>Clear</button>
        </div>
      </div>

      {#if namespacesList.length > 5}
        <div class="ns-search-wrapper">
          <Search size={12} class="ns-search-icon" />
          <input 
            type="text" 
            placeholder="Search namespaces..." 
            bind:value={searchTerm} 
            class="ns-search-input"
          />
          {#if searchTerm}
            <button type="button" class="ns-clear-btn" onclick={() => searchTerm = ''}><X size={12} /></button>
          {/if}
        </div>
      {/if}

      <div class="ns-options-list">
        <!-- All Namespaces Option -->
        <label class="ns-option-item" class:selected={isAllSelected}>
          <input 
            type="checkbox" 
            checked={isAllSelected} 
            onchange={() => toggleNamespace('all')} 
          />
          <span class="checkbox-custom">
            {#if isAllSelected}<Check size={11} />{/if}
          </span>
          <span class="ns-name">All Namespaces</span>
        </label>

        <div class="ns-divider"></div>

        <!-- Individual Namespaces -->
        {#if filteredNamespaces.length === 0}
          <div class="no-ns-found">No namespaces match filter</div>
        {:else}
          {#each filteredNamespaces as ns}
            {@const checked = !isAllSelected && selectedNamespaces.includes(ns)}
            <label class="ns-option-item" class:selected={checked}>
              <input 
                type="checkbox" 
                {checked} 
                onchange={() => toggleNamespace(ns)} 
              />
              <span class="checkbox-custom">
                {#if checked}<Check size={11} />{/if}
              </span>
              <span class="ns-name">{ns}</span>
            </label>
          {/each}
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .multi-ns-container {
    position: relative;
    display: inline-block;
  }

  .multi-ns-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    background: rgba(0, 0, 0, 0.4);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-sm);
    color: white;
    padding: 5px 10px;
    font-size: 0.85rem;
    cursor: pointer;
    transition: all 0.2s ease;
    white-space: nowrap;
  }

  .multi-ns-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.3);
  }

  .multi-ns-btn.active {
    background: rgba(46, 145, 190, 0.2);
    border-color: var(--primary-color);
    color: var(--primary-color);
  }

  .ns-label-prefix {
    color: var(--text-muted);
    font-weight: 500;
  }

  .ns-label-text {
    font-weight: 600;
    max-width: 140px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  :global(.ns-chevron) {
    transition: transform 0.2s ease;
  }

  :global(.ns-chevron.open) {
    transform: rotate(180deg);
  }

  .multi-ns-popover {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    width: 240px;
    background: var(--bg-surface, #1e2430);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: var(--radius-md);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
    z-index: 1000;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .ns-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 2px 4px;
    font-size: 0.8rem;
  }

  .ns-header-title {
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    font-size: 0.72rem;
  }

  .ns-header-actions {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .action-link {
    background: none;
    border: none;
    color: var(--primary-color);
    font-size: 0.75rem;
    cursor: pointer;
    padding: 0;
  }

  .action-link:hover {
    text-decoration: underline;
  }

  .sep {
    color: var(--text-muted);
    font-size: 0.7rem;
  }

  .ns-search-wrapper {
    position: relative;
    display: flex;
    align-items: center;
  }

  :global(.ns-search-icon) {
    position: absolute;
    left: 8px;
    color: var(--text-muted);
    pointer-events: none;
  }

  .ns-search-input {
    width: 100%;
    padding: 4px 22px 4px 26px;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: var(--radius-sm);
    color: white;
    font-size: 0.8rem;
    outline: none;
  }

  .ns-search-input:focus {
    border-color: var(--primary-color);
  }

  .ns-clear-btn {
    position: absolute;
    right: 4px;
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 2px;
    display: flex;
  }

  .ns-options-list {
    max-height: 200px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .ns-options-list::-webkit-scrollbar {
    width: 4px;
  }

  .ns-options-list::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.2);
    border-radius: 2px;
  }

  .ns-option-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 6px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: 0.82rem;
    color: var(--text-primary);
    user-select: none;
    transition: background 0.15s ease;
  }

  .ns-option-item:hover {
    background: rgba(255, 255, 255, 0.06);
  }

  .ns-option-item.selected {
    color: white;
  }

  .ns-option-item input {
    display: none;
  }

  .checkbox-custom {
    width: 14px;
    height: 14px;
    border: 1px solid rgba(255, 255, 255, 0.3);
    border-radius: 3px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.2);
    color: var(--primary-color);
    flex-shrink: 0;
  }

  .ns-option-item.selected .checkbox-custom {
    background: var(--primary-color);
    border-color: var(--primary-color);
    color: white;
  }

  .ns-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .ns-divider {
    height: 1px;
    background: rgba(255, 255, 255, 0.08);
    margin: 4px 0;
  }

  .no-ns-found {
    padding: 8px;
    font-size: 0.78rem;
    color: var(--text-muted);
    text-align: center;
  }
</style>
