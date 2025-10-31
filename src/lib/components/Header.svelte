<!-- Kuboard Header Component -->
<script lang="ts">
  import type { KubeContext } from '../types/index.js';

  // Props
  export let contexts: KubeContext[] = [];
  export let currentContext: KubeContext | null = null;
  export let loading: boolean = false;
  export let isTauriAvailable: boolean = false;

  // Events
  import { createEventDispatcher } from 'svelte';
  const dispatch = createEventDispatcher();

  let selectedContextName = '';

  // Update selectedContextName when currentContext changes
  $: if (currentContext?.name) {
    selectedContextName = currentContext.name;
  }

  // Handle context change from dropdown
  function handleContextChange() {
    if (selectedContextName) {
      dispatch('contextChange', selectedContextName);
    }
  }

  // Handle refresh button click
  function handleRefresh() {
    dispatch('refresh');
  }
</script>

<header class="modern-header">
  <div class="header-content">
    <div class="header-left">
      <h1 class="app-title">🚢 Kuboard</h1>
      <div class="app-subtitle">Kubernetes Dashboard</div>
    </div>
    
    <div class="header-center">
      <div class="context-selector">
        <label for="context-dropdown">Context:</label>
        <select 
          id="context-dropdown" 
          bind:value={selectedContextName}
          onchange={handleContextChange}
          class="context-dropdown"
          disabled={contexts.length === 0}
        >
          {#if contexts.length === 0}
            <option value="">Loading contexts...</option>
          {:else}
            <option value="">🔍 Select a context to begin</option>
            {#each contexts as context}
              <option value={context.name}>{context.name}</option>
            {/each}
          {/if}
        </select>
      </div>
    </div>
    
    <div class="header-right">
      <div class="app-version">v1.0.0</div>
      <button onclick={handleRefresh} disabled={loading} class="header-refresh-button" title="Refresh cluster data">
        {loading ? "⏳" : "↻"}
      </button>
    </div>
  </div>
</header>

<style>
  /* Modern Header Styles */
  .modern-header {
    background: linear-gradient(135deg, var(--primary-color) 0%, var(--accent-color) 100%);
    color: var(--text-color);
    padding: 20px 0;
    margin-bottom: 20px;
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    border: 1px solid var(--primary-color);
  }

  .header-content {
    display: flex;
    align-items: center;
    justify-content: space-between;
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 20px;
  }

  .header-left {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
  }

  .app-title {
    font-size: 2.5em;
    font-weight: 700;
    margin: 0;
    text-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
  }

  .app-subtitle {
    font-size: 1em;
    opacity: 0.9;
    margin-top: -5px;
    font-weight: 300;
  }

  .header-center {
    flex: 1;
    display: flex;
    justify-content: center;
    margin: 0 40px;
  }

  .context-selector {
    display: flex;
    align-items: center;
    gap: 12px;
    background: rgba(16, 185, 129, 0.2);
    padding: 12px 20px;
    border-radius: var(--radius-md);
    backdrop-filter: blur(10px);
    border: 1px solid var(--primary-color);
  }

  .context-selector label {
    color: var(--text-color);
    font-weight: 600;
    font-size: 0.95em;
    white-space: nowrap;
  }

  .context-dropdown {
    background: rgba(30, 30, 30, 0.8);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 0.95em;
    font-weight: 600;
    padding: 8px 16px;
    min-width: 200px;
    cursor: pointer;
    transition: var(--transition-normal);
  }

  .context-dropdown:hover {
    background: rgba(30, 30, 30, 0.9);
    border-color: var(--accent-color);
    transform: translateY(-1px);
  }

  .context-dropdown:focus {
    outline: none;
    border-color: var(--accent-color);
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.3);
  }

  .context-dropdown:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .app-version {
    background: var(--card-background);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    color: var(--text-color);
    font-size: 0.9em;
    font-weight: 500;
    padding: 8px 12px;
    opacity: 0.8;
  }

  .header-refresh-button {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-md);
    color: var(--text-color);
    font-size: 1.2em;
    font-weight: 600;
    padding: 8px 12px;
    cursor: pointer;
    transition: var(--transition-normal);
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
  }

  .header-refresh-button:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.2);
    transform: rotate(180deg);
  }

  .header-refresh-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
  }
</style>
