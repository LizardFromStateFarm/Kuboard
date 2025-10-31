<!-- Kuboard Config Tab Component -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  // Props
  export let currentContext: any = null;

  // State
  let configmaps: any[] = [];
  let secrets: any[] = [];
  let persistentvolumes: any[] = [];
  let persistentvolumeclaims: any[] = [];
  
  let loading: boolean = false;
  let error: string | null = null;
  let lastUpdate: string = '';

  // Load config data
  async function loadConfig() {
    if (!currentContext || loading) return;
    
    loading = true;
    error = null;
    
    try {
      // Load all config types in parallel
      const [configmapsData, secretsData] = await Promise.all([
        invoke('kuboard_get_configmaps').catch(() => []),
        invoke('kuboard_get_secrets').catch(() => [])
      ]);

      configmaps = configmapsData as any[] || [];
      secrets = secretsData as any[] || [];
      
      lastUpdate = new Date().toLocaleTimeString();
    } catch (err) {
      error = err as string;
      console.error('Failed to load config:', err);
    } finally {
      loading = false;
    }
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

  // Get configmap data count
  function getConfigmapDataCount(configmap: any): number {
    return Object.keys(configmap.data || {}).length;
  }

  // Get secret type
  function getSecretType(secret: any): string {
    return secret.type || 'Opaque';
  }

  // Get secret data count
  function getSecretDataCount(secret: any): number {
    return Object.keys(secret.data || {}).length;
  }

  // Lifecycle
  onMount(() => {
    loadConfig();
  });

  // Reactive updates
  $: if (currentContext) {
    loadConfig();
  }
</script>

<div class="config-tab">
  <div class="tab-header">
    <h4>⚙️ Configuration</h4>
    <div class="tab-controls">
      <button 
        class="refresh-button" 
        onclick={loadConfig}
        disabled={loading}
        title="Refresh config"
      >
        {#if loading}
          🔄
        {:else}
          ↻
        {/if}
      </button>
      {#if lastUpdate}
        <span class="last-update">Last: {lastUpdate}</span>
      {/if}
    </div>
  </div>

  {#if error}
    <div class="error-message">
      <div class="error-icon">⚠️</div>
      <div class="error-content">
        <h5>Failed to load configuration</h5>
        <p>{error}</p>
        <button class="retry-button" onclick={loadConfig}>
          Retry
        </button>
      </div>
    </div>
  {:else if loading}
    <div class="loading-message">
      <div class="loading-spinner">🔄</div>
      <p>Loading configuration...</p>
    </div>
  {:else}
    <div class="config-grid">
      <!-- ConfigMaps Section -->
      <div class="config-section">
        <div class="section-header">
          <h5>ConfigMaps ({configmaps.length})</h5>
        </div>
        <div class="config-list">
          {#each configmaps.slice(0, 10) as configmap}
            <div class="config-item">
              <div class="config-info">
                <span class="config-name">{configmap.metadata?.name || 'Unknown'}</span>
                <span class="config-namespace">{configmap.metadata?.namespace || 'default'}</span>
              </div>
              <div class="config-details">
                <span class="config-data-count">{getConfigmapDataCount(configmap)} keys</span>
                <span class="config-age">{formatAge(configmap.metadata?.creationTimestamp)}</span>
              </div>
            </div>
          {/each}
          {#if configmaps.length > 10}
            <div class="config-more">
              <span>... and {configmaps.length - 10} more</span>
            </div>
          {/if}
        </div>
      </div>

      <!-- Secrets Section -->
      <div class="config-section">
        <div class="section-header">
          <h5>Secrets ({secrets.length})</h5>
        </div>
        <div class="config-list">
          {#each secrets.slice(0, 10) as secret}
            <div class="config-item">
              <div class="config-info">
                <span class="config-name">{secret.metadata?.name || 'Unknown'}</span>
                <span class="config-namespace">{secret.metadata?.namespace || 'default'}</span>
              </div>
              <div class="config-details">
                <span class="config-type">{getSecretType(secret)}</span>
                <span class="config-data-count">{getSecretDataCount(secret)} keys</span>
                <span class="config-age">{formatAge(secret.metadata?.creationTimestamp)}</span>
              </div>
            </div>
          {/each}
          {#if secrets.length > 10}
            <div class="config-more">
              <span>... and {secrets.length - 10} more</span>
            </div>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  /* Import CSS variables */
  @import '../styles/variables.css';

  .config-tab {
    padding: 0;
  }

  .tab-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 5px;
    padding-bottom: var(--spacing-sm);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .tab-header h4 {
    margin: 0;
    color: white;
    font-size: 1.1rem;
    font-weight: 600;
  }

  .tab-controls {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .refresh-button {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-sm);
    color: white;
    cursor: pointer;
    font-size: 0.9rem;
    padding: 6px 12px;
    transition: var(--transition-normal);
  }

  .refresh-button:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.2);
    border-color: rgba(255, 255, 255, 0.3);
  }

  .refresh-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .last-update {
    font-size: 0.8rem;
    color: rgba(255, 255, 255, 0.6);
  }

  .error-message {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: var(--radius-md);
  }

  .error-icon {
    font-size: 1.2rem;
    flex-shrink: 0;
  }

  .error-content h5 {
    margin: 0 0 4px 0;
    color: white;
    font-size: 0.9rem;
    font-weight: 600;
  }

  .error-content p {
    margin: 0 0 8px 0;
    color: rgba(255, 255, 255, 0.8);
    font-size: 0.85rem;
  }

  .retry-button {
    background: var(--error-color);
    border: none;
    border-radius: var(--radius-sm);
    color: white;
    cursor: pointer;
    font-size: 0.8rem;
    padding: 4px 8px;
    transition: var(--transition-normal);
  }

  .retry-button:hover {
    background: #dc2626;
  }

  .loading-message {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-xl);
    color: rgba(255, 255, 255, 0.8);
  }

  .loading-spinner {
    font-size: 1.5rem;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .config-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: var(--spacing-lg);
  }

  .config-section {
    background: rgba(255, 255, 255, 0.02);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .section-header {
    margin-bottom: var(--spacing-md);
  }

  .section-header h5 {
    margin: 0;
    color: white;
    font-size: 1rem;
    font-weight: 600;
  }

  .config-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .config-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-sm);
    background: rgba(255, 255, 255, 0.03);
    border-radius: var(--radius-sm);
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .config-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .config-name {
    color: white;
    font-size: 0.9rem;
    font-weight: 600;
  }

  .config-namespace {
    color: rgba(255, 255, 255, 0.6);
    font-size: 0.8rem;
  }

  .config-details {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 2px;
  }

  .config-type {
    background: rgba(59, 130, 246, 0.2);
    color: #3b82f6;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .config-data-count {
    color: rgba(255, 255, 255, 0.8);
    font-size: 0.8rem;
    font-weight: 500;
  }

  .config-age {
    color: rgba(255, 255, 255, 0.6);
    font-size: 0.7rem;
  }

  .config-more {
    text-align: center;
    padding: var(--spacing-sm);
    color: rgba(255, 255, 255, 0.6);
    font-size: 0.8rem;
    font-style: italic;
  }

  /* Responsive Design */
  @media (max-width: 768px) {
    .config-grid {
      grid-template-columns: 1fr;
    }
    
    .config-item {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--spacing-sm);
    }
    
    .config-details {
      align-items: flex-start;
    }
  }
</style>
