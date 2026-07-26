<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import QuickActionsMenu from './QuickActionsMenu.svelte';
  import { openGlobalPodLogs } from '../stores/logs';
  import { ArrowLeft, Sliders, FileText, Settings, ExternalLink, AlertTriangle, Box, Tag, Loader2 } from 'lucide-svelte';

  const dispatch = createEventDispatcher();

  export let replicaSet: any;
  export let onBack: () => void;

  let replicaSetDetails: any = null;
  let managedPods: any[] = [];
  let loading = false;
  let error: string | null = null;
  let podsLoading = false;
  let podsError: string | null = null;
  let scaleLoading = false;
  let scaleError: string | null = null;

  let scaleValue: number = 0;
  let showScaleInput = false;

  let actionsMenuVisible = false;
  let actionsMenuPosition = { x: 0, y: 0 };
  let yamlViewerVisible = false;
  let yamlContent = '';

  $: rs = replicaSetDetails || replicaSet;
  $: status = getReplicaStatus(rs);
  $: desired = rs?.spec?.replicas || 0;
  $: ready = rs?.status?.readyReplicas || 0;
  $: current = rs?.status?.replicas || 0;
  $: available = rs?.status?.availableReplicas || 0;
  $: owner = getOwnerReference(rs);

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

  function getReplicaStatus(r: any): string {
    if (!r) return 'Unknown';
    const des = r.spec?.replicas || 0;
    const rdy = r.status?.readyReplicas || 0;
    const cur = r.status?.replicas || 0;
    
    if (rdy === des && cur === des) return 'Ready';
    if (cur < des) return 'Scaling';
    if (rdy < des) return 'Not Ready';
    return 'Unknown';
  }

  function getStatusClass(st: string): string {
    switch (st?.toLowerCase()) {
      case 'ready': return 'ready';
      case 'scaling': return 'pending';
      case 'not ready': return 'failed';
      default: return 'unknown';
    }
  }

  function getOwnerReference(r: any): { type: string; name: string } | null {
    const ownerRefs = r?.metadata?.ownerReferences || [];
    if (ownerRefs.length === 0) return null;
    const o = ownerRefs[0];
    return { type: o.kind || 'Unknown', name: o.name || 'Unknown' };
  }

  function getPodStatusClass(st: string): string {
    switch (st?.toLowerCase()) {
      case 'running': return 'running';
      case 'pending': return 'pending';
      case 'succeeded': return 'ready';
      case 'failed': return 'failed';
      default: return 'unknown';
    }
  }

  async function loadReplicaSetDetails() {
    if (!replicaSet?.metadata?.name || !replicaSet?.metadata?.namespace) return;
    loading = true; error = null;
    try {
      const details = await invoke('kuboard_get_replicaset', {
        name: replicaSet.metadata.name,
        namespace: replicaSet.metadata.namespace
      });
      replicaSetDetails = details;
      scaleValue = replicaSetDetails?.spec?.replicas || replicaSet?.spec?.replicas || 0;
      await loadManagedPods();
    } catch (err: any) {
      console.warn('Failed to load replicaset details via Tauri API:', err);
      replicaSetDetails = replicaSet;
      scaleValue = replicaSet?.spec?.replicas || 0;
    } finally {
      loading = false;
    }
  }

  async function loadManagedPods() {
    if (!replicaSet?.metadata?.name || !replicaSet?.metadata?.namespace) return;
    podsLoading = true; podsError = null;
    try {
      const pods = await invoke('kuboard_get_replicaset_pods', {
        name: replicaSet.metadata.name,
        namespace: replicaSet.metadata.namespace
      }) as any[];
      managedPods = pods || [];
    } catch (err: any) {
      console.warn('Failed to load managed pods:', err);
      managedPods = [];
    } finally {
      podsLoading = false;
    }
  }

  async function scaleReplicaSet() {
    if (!replicaSet?.metadata?.name || !replicaSet?.metadata?.namespace) return;
    scaleLoading = true; scaleError = null;
    try {
      await invoke('kuboard_scale_replicaset', {
        name: replicaSet.metadata.name,
        namespace: replicaSet.metadata.namespace,
        replicas: scaleValue
      });
      showScaleInput = false;
      await loadReplicaSetDetails();
    } catch (err: any) {
      scaleError = String(err);
    } finally {
      scaleLoading = false;
    }
  }

  function openActionsMenu(event: MouseEvent) {
    event.stopPropagation();
    event.preventDefault();
    if (actionsMenuVisible) {
      actionsMenuVisible = false;
      return;
    }
    const btn = event.currentTarget as HTMLElement;
    if (btn && typeof btn.getBoundingClientRect === 'function') {
      const rect = btn.getBoundingClientRect();
      if (rect.width > 0 || rect.height > 0) {
        actionsMenuPosition = { x: rect.left, y: rect.bottom + 4 };
      } else if (event.clientX > 0 || event.clientY > 0) {
        actionsMenuPosition = { x: event.clientX, y: event.clientY };
      }
    } else if (event.clientX > 0 || event.clientY > 0) {
      actionsMenuPosition = { x: event.clientX, y: event.clientY };
    }
    actionsMenuVisible = true;
  }

  function handleActionMenuClose() { actionsMenuVisible = false; }
  function handleActionDeleted() { handleActionMenuClose(); onBack(); }
  function handleViewYaml(event: CustomEvent) {
    yamlContent = event.detail.yaml;
    yamlViewerVisible = true;
    handleActionMenuClose();
  }
  function closeYamlViewer() { yamlViewerVisible = false; yamlContent = ''; handleActionMenuClose(); }

  onMount(() => { loadReplicaSetDetails(); });
</script>

<div class="resource-details-view">
  <!-- Sleek Top Action Bar -->
  <div class="details-nav-bar">
    <div class="nav-actions">
      <button class="btn-back" onclick={() => { if (onBack) onBack(); dispatch('back'); }}><ArrowLeft size={14} class="inline-icon" /> Back to ReplicaSets</button>
      <button class="btn-subtle" onclick={() => { showScaleInput = !showScaleInput; scaleValue = desired; }}>
        <Sliders size={14} class="inline-icon" /> {showScaleInput ? 'Cancel Scale' : 'Scale'}
      </button>
      <button class="btn-subtle" onclick={() => openGlobalPodLogs(undefined, rs?.metadata?.name, rs?.metadata?.namespace)}>
        <FileText size={14} class="inline-icon" /> Logs
      </button>
      <button class="btn-subtle" onclick={openActionsMenu} ondblclick={(e) => { e.stopPropagation(); e.preventDefault(); }}><Settings size={14} class="inline-icon" /> Actions</button>
    </div>
    <div class="nav-heading">
      <span class="status-pill status-{getStatusClass(status)}">{status}</span>
      <h3 class="nav-title">{rs?.metadata?.name}</h3>
      <span class="namespace-pill">{rs?.metadata?.namespace}</span>
    </div>
  </div>

  <!-- Single Cohesive Master Sheet -->
  <div class="details-sheet">
    <!-- Key Specs Summary Strip -->
    <div class="sheet-section specs-strip">
      <div class="spec-cell">
        <span class="spec-label">Replicas (Ready / Desired)</span>
        <span class="spec-val">{ready} / {desired}</span>
      </div>
      <div class="spec-cell">
        <span class="spec-label">Current / Available</span>
        <span class="spec-val">{current} / {available}</span>
      </div>
      <div class="spec-cell">
        <span class="spec-label">Controlled By</span>
        {#if owner}
          <button class="controller-link-btn" onclick={() => dispatch('navigateToWorkload', { type: owner.type.toLowerCase(), name: owner.name })} title="Navigate to {owner.type} Details">
            <ExternalLink size={13} class="inline-icon" /> {owner.type} / {owner.name}
          </button>
        {:else}
          <span class="spec-val">None (Independent)</span>
        {/if}
      </div>
      <div class="spec-cell">
        <span class="spec-label">Age</span>
        <span class="spec-val">{formatAge(rs?.metadata?.creationTimestamp)}</span>
      </div>
    </div>

    <!-- Scale Control Strip (if active) -->
    {#if showScaleInput}
      <div class="sheet-section scale-inline-box">
        <span class="scale-lbl">Scale Replicas:</span>
        <input type="number" min="0" bind:value={scaleValue} class="input-num" disabled={scaleLoading} />
        <button class="btn-primary-sm" onclick={scaleReplicaSet} disabled={scaleLoading}>
          {scaleLoading ? 'Scaling...' : 'Apply Scale'}
        </button>
        {#if scaleError}
          <span class="err-txt"><AlertTriangle size={14} class="inline-icon" /> {scaleError}</span>
        {/if}
      </div>
    {/if}

    <!-- Managed Pods -->
    <div class="sheet-section">
      <h5><Box size={16} /> Managed Pods ({managedPods.length})</h5>
      {#if podsLoading}
        <div class="muted-text"><Loader2 size={14} class="spin inline-icon" /> Loading managed pods...</div>
      {:else if managedPods.length > 0}
        <div class="pods-table">
          <div class="p-head">
            <div>Name</div>
            <div>Status</div>
            <div>Node</div>
            <div>Pod IP</div>
            <div>Restarts</div>
            <div>Age</div>
          </div>
          {#each managedPods as pod}
            <div class="p-row">
              <div class="bold">{pod.metadata?.name || 'Unknown'}</div>
              <div><span class="status-pill status-{getPodStatusClass(pod.status?.phase)}">{pod.status?.phase || 'Unknown'}</span></div>
              <div>{pod.spec?.nodeName || '-'}</div>
              <div class="font-mono">{pod.status?.podIP || '-'}</div>
              <div>{pod.status?.containerStatuses?.[0]?.restartCount || 0}</div>
              <div>{formatAge(pod.metadata?.creationTimestamp)}</div>
            </div>
          {/each}
        </div>
      {:else}
        <p class="muted-text">No active pods managed by this ReplicaSet</p>
      {/if}
    </div>

    <!-- Selectors -->
    <div class="sheet-section">
      <h5><Sliders size={16} /> Selectors</h5>
      <div class="kv-grid">
        <div class="kv-block">
          <span class="kv-title">Match Labels</span>
          <div class="tag-wrap">
            {#if rs?.spec?.selector?.matchLabels && Object.keys(rs.spec.selector.matchLabels).length > 0}
              {#each Object.entries(rs.spec.selector.matchLabels) as [k, v]}
                <span class="flat-tag"><strong class="k">{k}:</strong> {v}</span>
              {/each}
            {:else}
              <span class="muted-text">-</span>
            {/if}
          </div>
        </div>
      </div>
    </div>

    <!-- Labels & Annotations -->
    <div class="sheet-section">
      <h5><Tag size={16} /> Labels & Annotations</h5>
      <div class="kv-grid">
        <div class="kv-block">
          <span class="kv-title">Labels</span>
          <div class="tag-wrap">
            {#if rs?.metadata?.labels && Object.keys(rs.metadata.labels).length > 0}
              {#each Object.entries(rs.metadata.labels) as [k, v]}
                <span class="flat-tag"><strong class="k">{k}:</strong> {v}</span>
              {/each}
            {:else}
              <span class="muted-text">-</span>
            {/if}
          </div>
        </div>
        <div class="kv-block">
          <span class="kv-title">Annotations</span>
          <div class="tag-wrap">
            {#if rs?.metadata?.annotations && Object.keys(rs.metadata.annotations).length > 0}
              {#each Object.entries(rs.metadata.annotations) as [k, v]}
                <span class="flat-tag annotation"><strong class="k">{k}:</strong> {v}</span>
              {/each}
            {:else}
              <span class="muted-text">-</span>
            {/if}
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<QuickActionsMenu
  x={actionsMenuPosition.x}
  y={actionsMenuPosition.y}
  position={actionsMenuPosition}
  resource={replicaSetDetails || replicaSet}
  resourceType="replicaset"
  bind:visible={actionsMenuVisible}
  on:close={handleActionMenuClose}
  on:deleted={handleActionDeleted}
  on:view-yaml={handleViewYaml}
/>

{#if yamlViewerVisible}
  <div class="modal-overlay" onclick={closeYamlViewer} role="button" tabindex="-1" onkeydown={(e) => e.key === 'Escape' && closeYamlViewer()}>
    <div class="modal-box" onclick={(e) => e.stopPropagation()} role="dialog" tabindex="-1">
      <div class="modal-hdr">
        <h4>ReplicaSet YAML: {replicaSet?.metadata?.name}</h4>
        <button class="btn-close" onclick={closeYamlViewer}>×</button>
      </div>
      <div class="modal-bdy">
        <pre class="yaml-code">{yamlContent}</pre>
      </div>
    </div>
  </div>
{/if}

<style>
  .resource-details-view { display: flex; flex-direction: column; gap: 12px; }
  .details-nav-bar { display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; background: var(--background-secondary); border: 1px solid var(--border-primary); border-radius: var(--radius-md); }
  .nav-actions { display: flex; align-items: center; gap: 8px; }
  .btn-back { background: var(--primary-color); border: none; color: white; padding: 5px 12px; border-radius: var(--radius-sm); font-size: 0.85rem; font-weight: 600; cursor: pointer; }
  .btn-subtle { background: rgba(255, 255, 255, 0.05); border: 1px solid var(--border-primary); color: var(--text-primary); padding: 5px 10px; border-radius: var(--radius-sm); font-size: 0.85rem; cursor: pointer; transition: background 0.15s; }
  .btn-subtle:hover { background: rgba(255, 255, 255, 0.1); }
  .nav-heading { display: flex; align-items: center; gap: 8px; }
  .nav-title { margin: 0; font-size: 1.1rem; font-weight: 700; color: var(--text-primary); }
  .namespace-pill { background: rgba(255, 255, 255, 0.08); color: var(--text-secondary); font-size: 0.8rem; padding: 2px 8px; border-radius: 12px; }
  .status-pill { padding: 2px 8px; font-size: 0.75rem; font-weight: 700; border-radius: 12px; text-transform: uppercase; }
  .status-ready { background: rgba(34, 197, 94, 0.15); color: #4ade80; }
  .status-pending { background: rgba(245, 158, 11, 0.15); color: #fbbf24; }
  .status-failed { background: rgba(239, 68, 68, 0.15); color: #f87171; }
  .status-unknown { background: rgba(156, 163, 175, 0.15); color: #9ca3af; }

  /* Details Sheet */
  .details-sheet { background: var(--background-secondary); border: 1px solid var(--border-primary); border-radius: var(--radius-md); display: flex; flex-direction: column; }
  .sheet-section { padding: 16px 20px; border-bottom: 1px solid rgba(255, 255, 255, 0.06); }
  .sheet-section:last-child { border-bottom: none; }
  .sheet-section h5 { margin: 0 0 12px 0; font-size: 0.95rem; font-weight: 700; color: var(--text-primary); letter-spacing: -0.2px; }
  .specs-strip { display: flex; align-items: center; gap: 24px; background: rgba(255, 255, 255, 0.02); overflow-x: auto; }
  .spec-cell { display: flex; flex-direction: column; gap: 2px; flex-shrink: 0; }
  .spec-label { font-size: 0.75rem; color: var(--text-muted); font-weight: 600; text-transform: uppercase; }
  .spec-val {
    font-size: 0.88rem;
    color: var(--text-primary);
    font-weight: 600;
  }

  .controller-link-btn {
    background: rgba(59, 130, 246, 0.12);
    border: 1px solid rgba(59, 130, 246, 0.3);
    color: #60a5fa;
    padding: 3px 8px;
    border-radius: 4px;
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
    width: fit-content;
  }
  .controller-link-btn:hover {
    background: rgba(59, 130, 246, 0.25);
    color: #93c5fd;
    border-color: rgba(59, 130, 246, 0.5);
  }

  .scale-inline-box { display: flex; align-items: center; gap: 10px; background: rgba(59, 130, 246, 0.08); }
  .scale-lbl { font-size: 0.85rem; font-weight: 600; color: var(--text-primary); }
  .input-num { background: rgba(0, 0, 0, 0.4); border: 1px solid var(--border-primary); color: white; padding: 4px 8px; border-radius: 4px; width: 70px; }
  .btn-primary-sm { background: var(--primary-color); border: none; color: white; padding: 4px 12px; border-radius: 4px; font-size: 0.82rem; font-weight: 600; cursor: pointer; }
  .err-txt { color: #f87171; font-size: 0.82rem; }

  .kv-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 16px; }
  .kv-block { display: flex; flex-direction: column; gap: 6px; }
  .kv-title { font-size: 0.8rem; color: var(--text-muted); font-weight: 600; }
  .tag-wrap { display: flex; flex-wrap: wrap; gap: 6px; }
  .flat-tag { background: rgba(255, 255, 255, 0.04); border: 1px solid var(--border-primary); border-radius: 4px; padding: 2px 6px; font-size: 0.78rem; color: var(--text-secondary); font-family: monospace; }
  .flat-tag .k { color: var(--text-primary); }
  .flat-tag.annotation { background: rgba(59, 130, 246, 0.05); border-color: rgba(59, 130, 246, 0.2); }
  .muted-text { color: var(--text-muted); font-size: 0.85rem; }

  .pods-table { border: 1px solid var(--border-primary); border-radius: var(--radius-sm); overflow: hidden; }
  .p-head { display: grid; grid-template-columns: 1fr 100px 140px 130px 80px 80px; background: rgba(255, 255, 255, 0.03); padding: 8px 12px; font-size: 0.8rem; font-weight: 600; color: var(--text-secondary); border-bottom: 1px solid var(--border-primary); }
  .p-row { display: grid; grid-template-columns: 1fr 100px 140px 130px 80px 80px; padding: 8px 12px; font-size: 0.85rem; border-bottom: 1px solid rgba(255, 255, 255, 0.04); align-items: center; }
  .p-row:last-child { border-bottom: none; }
  .bold { font-weight: 600; color: var(--text-primary); }
  .font-mono { font-family: monospace; font-size: 0.8rem; }

  .modal-overlay { position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0, 0, 0, 0.75); backdrop-filter: blur(4px); z-index: 2100; display: flex; align-items: center; justify-content: center; padding: 20px; }
  .modal-box { background: #181824; border: 1px solid var(--border-primary); border-radius: var(--radius-lg); width: 100%; max-width: 650px; max-height: 80vh; display: flex; flex-direction: column; overflow: hidden; }
  .modal-hdr { display: flex; justify-content: space-between; align-items: center; padding: 12px 16px; border-bottom: 1px solid var(--border-primary); }
  .modal-hdr h4 { margin: 0; color: white; font-size: 1rem; }
  .btn-close { background: transparent; border: none; color: var(--text-secondary); font-size: 1.2rem; cursor: pointer; }
  .modal-bdy { padding: 16px; overflow-y: auto; }
  .yaml-code { background: #0d0d14; color: #a7f3d0; padding: 12px; border-radius: 6px; font-family: monospace; font-size: 0.8rem; margin: 0; white-space: pre-wrap; }
</style>
