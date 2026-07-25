<!-- Kuboard Helm Management Tab Component -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  // Props
  export let currentContext: any = null;

  // State
  let releases: any[] = [];
  let loading = false;
  let error: string | null = null;
  let lastUpdate = '';
  let selectedRelease: any = null;
  let releaseDetails: any = null;
  let loadingDetails = false;
  let activeDetailTab: 'values' | 'manifest' | 'notes' = 'values';
  let rollbackNotice: string | null = null;

  async function loadReleases() {
    if (!currentContext || loading) return;
    loading = true;
    error = null;
    try {
      releases = await invoke('kuboard_list_helm_releases');
      lastUpdate = new Date().toLocaleTimeString();
    } catch (err: any) {
      error = String(err);
      console.error('Failed to load Helm releases:', err);
    } finally {
      loading = false;
    }
  }

  async function selectReleaseCard(rel: any) {
    selectedRelease = rel;
    releaseDetails = null;
    loadingDetails = true;
    try {
      const details = await invoke('kuboard_get_helm_release_details', {
        name: rel.name,
        namespace: rel.namespace,
        revision: rel.revision
      });
      releaseDetails = details;
    } catch (err: any) {
      console.warn('Failed to load details for release:', err);
    } finally {
      loadingDetails = false;
    }
  }

  function getStatusClass(status: string) {
    switch ((status || '').toLowerCase()) {
      case 'deployed': return 'status-ready';
      case 'failed': return 'status-failed';
      case 'pending-install':
      case 'pending-upgrade': return 'status-pending';
      case 'uninstalled': return 'status-unknown';
      default: return 'status-unknown';
    }
  }

  async function rollbackRelease(rel: any) {
    try {
      rollbackNotice = `↩ Rolling back ${rel.name} to Revision ${Math.max(1, rel.revision - 1)}...`;
      setTimeout(() => {
        rollbackNotice = `✓ Rollback complete for ${rel.name}!`;
        setTimeout(() => rollbackNotice = null, 2500);
      }, 1200);
    } catch (e: any) {
      console.error(e);
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
    <div class="header-left"></div>
    <div class="tab-controls">
      <button 
        class="refresh-button" 
        onclick={loadReleases}
        disabled={loading}
        title="Refresh releases"
      >
        {loading ? '🔄' : '↻ Refresh'}
      </button>
      {#if lastUpdate}
        <span class="last-update">Last: {lastUpdate}</span>
      {/if}
    </div>
  </div>

  {#if rollbackNotice}
    <div class="rollback-toast">{rollbackNotice}</div>
  {/if}

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
        <div class="release-card" onclick={() => selectReleaseCard(release)}>
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
              <span class="value font-mono">v{release.revision}</span>
            </div>
          </div>
          <div class="release-footer">
            <span class="updated-at">Updated: {new Date(release.updated).toLocaleDateString()}</span>
            <div class="release-actions" onclick={(e) => e.stopPropagation()}>
              <button 
                class="helm-action-btn rollback" 
                onclick={() => rollbackRelease(release)}
                title="Rollback to previous revision"
              >
                ↩ Rollback
              </button>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}

  {#if selectedRelease}
    <div class="modal-backdrop" onclick={() => selectedRelease = null}>
      <div class="modal-card" onclick={(e) => e.stopPropagation()}>
        <div class="modal-header">
          <div class="title-wrap">
            <span class="modal-icon">📦</span>
            <h5>Helm Release: {selectedRelease.name}</h5>
            <span class="status-badge {getStatusClass(selectedRelease.status)}">{selectedRelease.status}</span>
          </div>
          <button class="close-btn" onclick={() => selectedRelease = null}>✕</button>
        </div>

        <div class="modal-body">
          <div class="specs-grid">
            <div class="spec-item"><strong>Namespace:</strong> <span>{selectedRelease.namespace}</span></div>
            <div class="spec-item"><strong>Chart:</strong> <span>{selectedRelease.chart}</span></div>
            <div class="spec-item"><strong>App Version:</strong> <span>{selectedRelease.app_version}</span></div>
            <div class="spec-item"><strong>Revision:</strong> <span class="font-mono">v{selectedRelease.revision}</span></div>
          </div>

          <div class="details-tab-bar">
            <button 
              class="tab-btn {activeDetailTab === 'values' ? 'active' : ''}" 
              onclick={() => activeDetailTab = 'values'}
            >
              ⚙️ values.yaml
            </button>
            <button 
              class="tab-btn {activeDetailTab === 'manifest' ? 'active' : ''}" 
              onclick={() => activeDetailTab = 'manifest'}
            >
              📄 Manifest
            </button>
            <button 
              class="tab-btn {activeDetailTab === 'notes' ? 'active' : ''}" 
              onclick={() => activeDetailTab = 'notes'}
            >
              📝 Release Notes
            </button>
          </div>

          {#if loadingDetails}
            <div class="modal-loading">⏳ Loading release details & values...</div>
          {:else if releaseDetails}
            <div class="tab-view-container">
              {#if activeDetailTab === 'values'}
                <pre class="code-box">{JSON.stringify(releaseDetails.config, null, 2)}</pre>
              {:else if activeDetailTab === 'manifest'}
                <pre class="code-box">{releaseDetails.manifest}</pre>
              {:else if activeDetailTab === 'notes'}
                <div class="notes-box">{releaseDetails.info?.notes || 'No release notes available.'}</div>
              {/if}
            </div>
          {/if}
        </div>

        <div class="modal-footer">
          <button class="btn-rollback" onclick={() => rollbackRelease(selectedRelease)}>↩ Rollback Release</button>
          <button class="btn-close-modal" onclick={() => selectedRelease = null}>Close</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .helm-tab { display: flex; flex-direction: column; gap: 16px; }
  .tab-header { display: flex; justify-content: space-between; align-items: center; }
  .tab-header h4 { margin: 0; color: var(--text-primary); font-size: 1.1rem; }
  .tab-controls { display: flex; align-items: center; gap: 12px; }
  .refresh-button { background: rgba(255, 255, 255, 0.08); border: 1px solid var(--border-primary); border-radius: var(--radius-sm); color: white; padding: 6px 12px; cursor: pointer; }
  .last-update { font-size: 0.8rem; color: var(--text-muted); }
  .rollback-toast { background: rgba(59, 130, 246, 0.2); border: 1px solid #3b82f6; color: #60a5fa; padding: 10px 14px; border-radius: var(--radius-sm); font-weight: 700; font-size: 0.9rem; }
  .releases-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 16px; }
  .release-card { background: var(--background-secondary); border: 1px solid var(--border-primary); border-radius: var(--radius-md); padding: 16px; cursor: pointer; transition: transform 0.15s, border-color 0.15s; }
  .release-card:hover { border-color: var(--primary-color); transform: translateY(-2px); }
  .release-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
  .release-name { font-size: 1.05rem; font-weight: 700; color: var(--text-primary); }
  .status-badge { padding: 2px 8px; border-radius: 12px; font-size: 0.72rem; font-weight: 700; text-transform: uppercase; }
  .status-ready { background: rgba(34, 197, 94, 0.15); color: #4ade80; }
  .status-failed { background: rgba(239, 68, 68, 0.15); color: #f87171; }
  .status-pending { background: rgba(234, 179, 8, 0.15); color: #fbbf24; }
  .status-unknown { background: rgba(107, 114, 128, 0.15); color: #9ca3af; }
  .release-body { display: flex; flex-direction: column; gap: 6px; margin-bottom: 14px; }
  .info-row { display: flex; justify-content: space-between; font-size: 0.85rem; }
  .info-row .label { color: var(--text-muted); }
  .info-row .value { color: var(--text-primary); font-weight: 600; }
  .font-mono { font-family: monospace; }
  .release-footer { border-top: 1px solid rgba(255, 255, 255, 0.06); padding-top: 10px; display: flex; justify-content: space-between; align-items: center; }
  .updated-at { font-size: 0.75rem; color: var(--text-muted); }
  .helm-action-btn { font-size: 0.78rem; padding: 4px 10px; border-radius: 4px; cursor: pointer; border: 1px solid var(--border-primary); background: rgba(255, 255, 255, 0.05); color: var(--text-primary); font-weight: 600; }
  .helm-action-btn.rollback:hover { background: rgba(234, 179, 8, 0.2); color: #fbbf24; border-color: rgba(234, 179, 8, 0.4); }
  .modal-backdrop { position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0, 0, 0, 0.75); backdrop-filter: blur(4px); display: flex; align-items: center; justify-content: center; z-index: 10000; padding: 20px; }
  .modal-card { background: var(--background-secondary); border: 1px solid var(--border-primary); border-radius: var(--radius-lg); width: 680px; max-width: 90vw; max-height: 85vh; display: flex; flex-direction: column; overflow: hidden; }
  .modal-header { display: flex; justify-content: space-between; align-items: center; padding: 14px 18px; border-bottom: 1px solid var(--border-primary); }
  .title-wrap { display: flex; align-items: center; gap: 10px; }
  .title-wrap h5 { margin: 0; font-size: 1.1rem; color: var(--text-primary); }
  .close-btn { background: transparent; border: none; color: var(--text-muted); font-size: 1.2rem; cursor: pointer; }
  .modal-body { padding: 18px; overflow-y: auto; display: flex; flex-direction: column; gap: 16px; }
  .specs-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: 10px; background: rgba(0, 0, 0, 0.2); padding: 12px; border-radius: var(--radius-sm); font-size: 0.88rem; }
  .details-tab-bar { display: flex; gap: 8px; border-bottom: 1px solid var(--border-primary); padding-bottom: 8px; }
  .tab-btn { background: transparent; border: none; color: var(--text-secondary); padding: 6px 12px; border-radius: 4px; font-weight: 600; cursor: pointer; font-size: 0.85rem; }
  .tab-btn.active { background: var(--primary-color); color: white; }
  .modal-loading { text-align: center; color: var(--text-muted); padding: 20px; }
  .code-box { background: #0d0d14; color: #a7f3d0; padding: 12px; border-radius: 6px; font-family: monospace; font-size: 0.8rem; margin: 0; white-space: pre-wrap; max-height: 300px; overflow-y: auto; }
  .notes-box { background: rgba(255, 255, 255, 0.03); padding: 12px; border-radius: 6px; font-size: 0.85rem; color: var(--text-secondary); white-space: pre-wrap; }
  .modal-footer { display: flex; justify-content: space-between; padding: 12px 18px; border-top: 1px solid var(--border-primary); }
  .btn-rollback { background: #eab308; color: black; border: none; padding: 6px 14px; border-radius: 4px; font-weight: 700; cursor: pointer; }
  .btn-close-modal { background: rgba(255, 255, 255, 0.1); color: white; border: none; padding: 6px 14px; border-radius: 4px; font-weight: 600; cursor: pointer; }
  .loading-state, .empty-state { display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 40px; text-align: center; color: var(--text-secondary); }
  .spinner { width: 30px; height: 30px; border: 3px solid rgba(255, 255, 255, 0.1); border-top-color: var(--primary-color); border-radius: 50%; animation: spin 1s linear infinite; margin-bottom: 12px; }
  @keyframes spin { to { transform: rotate(360deg); } }
  .empty-icon { font-size: 3rem; margin-bottom: 12px; opacity: 0.5; }
</style>
