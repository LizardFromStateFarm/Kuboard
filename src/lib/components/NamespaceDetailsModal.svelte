<!-- Kuboard Namespace Details Modal Component -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Folder, Tag, Activity, FileText, X, Loader2, Boxes, Box, HardDrive, ShieldCheck, Tag as TagIcon } from 'lucide-svelte';
  import YamlEditor from './YamlEditor.svelte';

  export let namespaceName: string;
  export let onClose: () => void = () => {};

  let loading: boolean = true;
  let error: string | null = null;
  let namespaceData: any = null;
  let podsCount: number = 0;
  let deploymentsCount: number = 0;
  let servicesCount: number = 0;
  let showYamlEditor: boolean = false;

  async function loadNamespaceDetails() {
    if (!namespaceName) return;
    loading = true;
    error = null;
    try {
      const namespaces = await invoke('kuboard_get_namespaces') as any[];
      const found = namespaces.find((ns: any) => ns.metadata?.name === namespaceName);
      namespaceData = found || { metadata: { name: namespaceName, creationTimestamp: new Date().toISOString() }, status: { phase: 'Active' } };

      // Load counts for this namespace
      const [allPods, allDeps, allSvcs] = await Promise.all([
        invoke('kuboard_get_pods').catch(() => []),
        invoke('kuboard_get_deployments').catch(() => []),
        invoke('kuboard_get_services').catch(() => [])
      ]);

      podsCount = (allPods as any[]).filter(p => p.metadata?.namespace === namespaceName).length;
      deploymentsCount = (allDeps as any[]).filter(d => d.metadata?.namespace === namespaceName).length;
      servicesCount = (allSvcs as any[]).filter(s => s.metadata?.namespace === namespaceName).length;
    } catch (err: any) {
      console.error('Failed to load namespace details:', err);
      error = String(err);
    } finally {
      loading = false;
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

  onMount(() => {
    loadNamespaceDetails();
  });
</script>

<div class="modal-overlay" onclick={(e) => { if (e.target === e.currentTarget) onClose(); }}>
  <div class="modal-card">
    <!-- Header Bar -->
    <div class="modal-header">
      <div class="title-wrap">
        <Folder size={20} class="inline-icon text-primary" />
        <div>
          <h4>Namespace: <span class="ns-highlight">{namespaceName}</span></h4>
          <span class="status-pill status-{namespaceData?.status?.phase?.toLowerCase() || 'active'}">
            {namespaceData?.status?.phase || 'Active'}
          </span>
        </div>
      </div>
      <div class="header-actions">
        <button class="action-btn" onclick={() => showYamlEditor = true}>
          <FileText size={14} class="inline-icon" /> Edit YAML
        </button>
        <button class="btn-close" onclick={onClose}><X size={18} /></button>
      </div>
    </div>

    <!-- Body -->
    <div class="modal-body">
      {#if loading}
        <div class="loading-box">
          <Loader2 size={24} class="spin" />
          <p>Loading Namespace Details...</p>
        </div>
      {:else if error}
        <div class="error-box">
          <p>{error}</p>
        </div>
      {:else}
        <!-- Specs Strip -->
        <div class="specs-grid">
          <div class="spec-cell">
            <span class="spec-lbl">Phase Status</span>
            <span class="spec-val font-bold text-success">{namespaceData?.status?.phase || 'Active'}</span>
          </div>
          <div class="spec-cell">
            <span class="spec-lbl">Active Pods</span>
            <span class="spec-val font-bold">{podsCount} pods</span>
          </div>
          <div class="spec-cell">
            <span class="spec-lbl">Deployments</span>
            <span class="spec-val font-bold">{deploymentsCount} deployments</span>
          </div>
          <div class="spec-cell">
            <span class="spec-lbl">Services</span>
            <span class="spec-val font-bold">{servicesCount} services</span>
          </div>
          <div class="spec-cell">
            <span class="spec-lbl">Age</span>
            <span class="spec-val">{formatAge(namespaceData?.metadata?.creationTimestamp)}</span>
          </div>
          <div class="spec-cell">
            <span class="spec-lbl">UID</span>
            <span class="spec-val font-mono">{namespaceData?.metadata?.uid || '-'}</span>
          </div>
        </div>

        <!-- Metadata & Labels -->
        {#if namespaceData?.metadata?.labels || namespaceData?.metadata?.annotations}
          <div class="meta-section">
            <h5><TagIcon size={16} class="inline-icon" /> Labels & Annotations</h5>
            {#if namespaceData?.metadata?.labels}
              <div class="meta-block">
                <span class="meta-title">Labels</span>
                <div class="pill-cloud">
                  {#each Object.entries(namespaceData.metadata.labels) as [k, v]}
                    <span class="label-pill"><strong>{k}:</strong> {v}</span>
                  {/each}
                </div>
              </div>
            {/if}
          </div>
        {/if}
      {/if}
    </div>
  </div>
</div>

{#if showYamlEditor}
  <YamlEditor 
    resource={namespaceData} 
    resourceType="namespace" 
    onSave={() => { showYamlEditor = false; loadNamespaceDetails(); }} 
    onCancel={() => showYamlEditor = false} 
  />
{/if}

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background: rgba(0, 0, 0, 0.65);
    backdrop-filter: blur(4px);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 9999;
  }

  .modal-card {
    background: var(--background-secondary, #12131a);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.12));
    border-radius: var(--radius-lg, 12px);
    width: 90%;
    max-width: 680px;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.5);
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    background: rgba(255, 255, 255, 0.03);
    border-bottom: 1px solid var(--border-primary);
  }

  .title-wrap {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .title-wrap h4 {
    margin: 0 0 2px 0;
    font-size: 16px;
    font-weight: 700;
  }

  .ns-highlight {
    color: var(--primary-color, #60a5fa);
  }

  .status-pill {
    font-size: 11px;
    font-weight: 700;
    padding: 2px 8px;
    border-radius: 10px;
    text-transform: uppercase;
  }
  .status-active {
    background: rgba(16, 185, 129, 0.2);
    color: #34d399;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .action-btn {
    background: var(--primary-color);
    color: white;
    border: none;
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
  }

  .btn-close {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .modal-body {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .loading-box {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 30px;
  }

  .specs-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 12px;
    background: rgba(0, 0, 0, 0.25);
    padding: 14px 16px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-primary);
  }

  .spec-cell {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .spec-lbl {
    font-size: 11px;
    color: var(--text-muted);
    font-weight: 600;
    text-transform: uppercase;
  }

  .spec-val {
    font-size: 13px;
    color: var(--text-primary);
  }

  .meta-section {
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--border-primary);
    padding: 14px 16px;
    border-radius: var(--radius-md);
  }

  .meta-section h5 {
    margin: 0 0 10px 0;
    font-size: 13px;
    font-weight: 700;
  }

  .pill-cloud {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .label-pill {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--border-primary);
    padding: 3px 8px;
    border-radius: 4px;
    font-size: 11px;
  }
</style>
