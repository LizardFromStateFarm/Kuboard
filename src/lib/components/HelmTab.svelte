<!-- Kuboard Helm Management Tab Component -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import ResourceTable from './ResourceTable.svelte';

  // Props
  export let currentContext: any = null;

  // State
  let releases: any[] = [];
  let loading = false;
  let error: string | null = null;
  let lastUpdate = '';
  let selectedRelease: any = null;

  async function loadReleases() {
    if (!currentContext || loading) return;
    
    loading = true;
    error = null;
    
    try {
      releases = await invoke('kuboard_list_helm_releases');
      lastUpdate = new Date().toLocaleTimeString();
    } catch (err: any) {
      error = err;
      console.error('Failed to load Helm releases:', err);
    } finally {
      loading = false;
    }
  }

  function getStatusClass(status: string) {
    switch (status.toLowerCase()) {
      case 'deployed': return 'status-ready';
      case 'failed': return 'status-failed';
      case 'pending-install':
      case 'pending-upgrade': return 'status-pending';
      case 'uninstalled': return 'status-unknown';
      default: return 'status-unknown';
    }
  }

  onMount(() => {
    loadReleases();
  });

  $: if (currentContext) {
    loadReleases();
  }
</script>

<div class="helm-tab">
  <div class="tab-header">
    <h4>📦 Helm Releases</h4>
    <div class="tab-controls">
      <button 
        class="refresh-button" 
        onclick={loadReleases}
        disabled={loading}
        title="Refresh releases"
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
    <div class="error-banner">
      <span class="error-icon">⚠️</span>
      <p>{error}</p>
      <button class="retry-button" onclick={loadReleases}>Retry</button>
    </div>
  {:else if loading && releases.length === 0}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Discovering Helm releases...</p>
    </div>
  {:else if releases.length === 0}
    <div class="empty-state">
      <div class="empty-icon">📦</div>
      <h5>No Helm Releases Found</h5>
      <p>We couldn't find any Helm 3 releases (secrets with owner=helm) in this cluster.</p>
    </div>
  {:else}
    <div class="releases-grid">
      {#each releases as release}
        <div class="release-card" onclick={() => selectedRelease = release}>
          <div class="release-header">
            <span class="release-name">{release.name}</span>
            <span class="status-badge {getStatusClass(release.status)}">{release.status}</span>
          </div>
          <div class="release-body">
            <div class="info-row">
              <span class="label">Namespace:</span>
              <span class="value">{release.namespace}</span>
            </div>
            <div class="info-row">
              <span class="label">Chart:</span>
              <span class="value">{release.chart}</span>
            </div>
            <div class="info-row">
              <span class="label">App Version:</span>
              <span class="value">{release.app_version}</span>
            </div>
            <div class="info-row">
              <span class="label">Revision:</span>
              <span class="value">{release.revision}</span>
            </div>
          </div>
          <div class="release-footer">
            <span class="updated-at">Updated: {new Date(release.updated).toLocaleString()}</span>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  @import '../styles/variables.css';

  .helm-tab {
    padding: 0;
  }

  .tab-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-lg);
    padding-bottom: var(--spacing-sm);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .tab-header h4 {
    margin: 0;
    color: white;
    font-size: 1.1rem;
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
    padding: 6px 12px;
    cursor: pointer;
  }

  .last-update {
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .releases-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
    gap: var(--spacing-lg);
  }

  .release-card {
    background: var(--card-background);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
    cursor: pointer;
    transition: var(--transition-normal);
  }

  .release-card:hover {
    border-color: var(--primary-color);
    transform: translateY(-2px);
    box-shadow: var(--shadow-lg);
  }

  .release-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-md);
  }

  .release-name {
    font-size: 1.1rem;
    font-weight: 700;
    color: white;
  }

  .status-badge {
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  .status-ready { background: rgba(16, 185, 129, 0.1); color: #10b981; border: 1px solid rgba(16, 185, 129, 0.3); }
  .status-failed { background: rgba(239, 68, 68, 0.1); color: #ef4444; border: 1px solid rgba(239, 68, 68, 0.3); }
  .status-pending { background: rgba(245, 158, 11, 0.1); color: #f59e0b; border: 1px solid rgba(245, 158, 11, 0.3); }
  .status-unknown { background: rgba(107, 114, 128, 0.1); color: #6b7280; border: 1px solid rgba(107, 114, 128, 0.3); }

  .release-body {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-lg);
  }

  .info-row {
    display: flex;
    justify-content: space-between;
    font-size: 0.9rem;
  }

  .label {
    color: var(--text-secondary);
  }

  .value {
    color: var(--text-primary);
    font-weight: 500;
  }

  .release-footer {
    border-top: 1px solid rgba(255, 255, 255, 0.05);
    padding-top: var(--spacing-sm);
    text-align: right;
  }

  .updated-at {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .loading-state, .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-xxl);
    text-align: center;
    color: var(--text-secondary);
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid rgba(255, 255, 255, 0.1);
    border-top-color: var(--primary-color);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: var(--spacing-lg);
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .empty-icon {
    font-size: 3rem;
    margin-bottom: var(--spacing-lg);
    opacity: 0.5;
  }
</style>
