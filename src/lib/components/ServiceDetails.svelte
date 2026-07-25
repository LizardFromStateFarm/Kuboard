<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import QuickActionsMenu from './QuickActionsMenu.svelte';
  import PortForwardManager from './PortForwardManager.svelte';

  const dispatch = createEventDispatcher();

  export let service: any;
  export let onBack: () => void;

  let serviceDetails: any = null;
  let endpoints: any = null;
  let loading = false;
  let error: string | null = null;

  let actionsMenuVisible = false;
  let actionsMenuPosition = { x: 0, y: 0 };
  let yamlViewerVisible = false;
  let yamlContent = '';

  let portForwardManagerOpen = false;

  $: svc = serviceDetails || service;
  $: type = svc?.spec?.type || 'ClusterIP';

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

  async function loadServiceDetails() {
    if (!service?.metadata?.name || !service?.metadata?.namespace) return;
    loading = true; error = null;
    try {
      const details = await invoke('kuboard_get_service_details', {
        name: service.metadata.name,
        namespace: service.metadata.namespace
      });
      serviceDetails = details;
      await loadEndpoints();
    } catch (err: any) {
      console.warn('Failed to load service details via Tauri API:', err);
      serviceDetails = service;
    } finally {
      loading = false;
    }
  }

  async function loadEndpoints() {
    if (!service?.metadata?.name || !service?.metadata?.namespace) return;
    try {
      const ep = await invoke('kuboard_get_service_endpoints', {
        name: service.metadata.name,
        namespace: service.metadata.namespace
      });
      endpoints = ep;
    } catch (err: any) {
      console.warn('Failed to load endpoints:', err);
      endpoints = null;
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

  onMount(() => { loadServiceDetails(); });
</script>

<div class="resource-details-view">
  <!-- Top Action Bar -->
  <div class="details-nav-bar">
    <div class="nav-actions">
      <button class="btn-back" onclick={() => { if (onBack) onBack(); dispatch('back'); }}>← Back to Services</button>
      <button class="btn-subtle" onclick={() => portForwardManagerOpen = true}>🔌 Port Forward</button>
      <button class="btn-subtle" onclick={openActionsMenu} ondblclick={(e) => { e.stopPropagation(); e.preventDefault(); }}>⚙️ Actions</button>
    </div>
    <div class="nav-heading">
      <span class="status-pill status-ready">{type}</span>
      <h3 class="nav-title">{svc?.metadata?.name}</h3>
      <span class="namespace-pill">{svc?.metadata?.namespace}</span>
    </div>
  </div>

  <!-- Master Sheet -->
  <div class="details-sheet">
    <!-- Key Specs Summary Strip -->
    <div class="sheet-section specs-strip">
      <div class="spec-cell">
        <span class="spec-label">Service Type</span>
        <span class="spec-val">{type}</span>
      </div>
      <div class="spec-cell">
        <span class="spec-label">Cluster IP</span>
        <span class="spec-val font-mono">{svc?.spec?.clusterIP || '-'}</span>
      </div>
      <div class="spec-cell">
        <span class="spec-label">External IP / LB</span>
        <span class="spec-val font-mono">{svc?.status?.loadBalancer?.ingress?.[0]?.ip || svc?.spec?.externalIPs?.join(', ') || '-'}</span>
      </div>
      <div class="spec-cell">
        <span class="spec-label">Session Affinity</span>
        <span class="spec-val">{svc?.spec?.sessionAffinity || 'None'}</span>
      </div>
      <div class="spec-cell">
        <span class="spec-label">Age</span>
        <span class="spec-val">{formatAge(svc?.metadata?.creationTimestamp)}</span>
      </div>
    </div>

    <!-- Ports -->
    <div class="sheet-section">
      <h5>🔌 Service Ports ({svc?.spec?.ports?.length || 0})</h5>
      {#if svc?.spec?.ports && svc.spec.ports.length > 0}
        <div class="ports-table">
          <div class="pt-head">
            <div>Name</div>
            <div>Port</div>
            <div>Target Port</div>
            <div>Protocol</div>
            <div>Node Port</div>
          </div>
          {#each svc.spec.ports as p}
            <div class="pt-row">
              <div class="bold">{p.name || '-'}</div>
              <div class="font-mono">{p.port}</div>
              <div class="font-mono">{p.targetPort}</div>
              <div>{p.protocol || 'TCP'}</div>
              <div class="font-mono">{p.nodePort || '-'}</div>
            </div>
          {/each}
        </div>
      {:else}
        <p class="muted-text">No ports configured</p>
      {/if}
    </div>

    <!-- Selectors -->
    <div class="sheet-section">
      <h5>⚙️ Selectors</h5>
      <div class="kv-grid">
        <div class="kv-block">
          <span class="kv-title">Selector Labels</span>
          <div class="tag-wrap">
            {#if svc?.spec?.selector && Object.keys(svc.spec.selector).length > 0}
              {#each Object.entries(svc.spec.selector) as [k, v]}
                <span class="flat-tag"><strong class="k">{k}:</strong> {v}</span>
              {/each}
            {:else}
              <span class="muted-text">None (Headless / ExternalName)</span>
            {/if}
          </div>
        </div>
      </div>
    </div>

    <!-- Labels & Annotations -->
    <div class="sheet-section">
      <h5>🏷️ Labels & Annotations</h5>
      <div class="kv-grid">
        <div class="kv-block">
          <span class="kv-title">Labels</span>
          <div class="tag-wrap">
            {#if svc?.metadata?.labels && Object.keys(svc.metadata.labels).length > 0}
              {#each Object.entries(svc.metadata.labels) as [k, v]}
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
            {#if svc?.metadata?.annotations && Object.keys(svc.metadata.annotations).length > 0}
              {#each Object.entries(svc.metadata.annotations) as [k, v]}
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
  resource={serviceDetails || service}
  resourceType="service"
  bind:visible={actionsMenuVisible}
  on:close={handleActionMenuClose}
  on:deleted={handleActionDeleted}
  on:view-yaml={handleViewYaml}
/>

{#if portForwardManagerOpen}
  <div class="port-forward-overlay">
    <PortForwardManager
      bind:isOpen={portForwardManagerOpen}
      serviceName={service?.metadata?.name}
      namespace={service?.metadata?.namespace}
      onClose={() => portForwardManagerOpen = false}
    />
  </div>
{/if}

{#if yamlViewerVisible}
  <div class="modal-overlay" onclick={closeYamlViewer} role="button" tabindex="-1" onkeydown={(e) => e.key === 'Escape' && closeYamlViewer()}>
    <div class="modal-box" onclick={(e) => e.stopPropagation()} role="dialog" tabindex="-1">
      <div class="modal-hdr">
        <h4>Service YAML: {service?.metadata?.name}</h4>
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

  /* Details Sheet */
  .details-sheet { background: var(--background-secondary); border: 1px solid var(--border-primary); border-radius: var(--radius-md); display: flex; flex-direction: column; }
  .sheet-section { padding: 16px 20px; border-bottom: 1px solid rgba(255, 255, 255, 0.06); }
  .sheet-section:last-child { border-bottom: none; }
  .sheet-section h5 { margin: 0 0 12px 0; font-size: 0.95rem; font-weight: 700; color: var(--text-primary); letter-spacing: -0.2px; }
  .specs-strip { display: flex; align-items: center; gap: 24px; background: rgba(255, 255, 255, 0.02); overflow-x: auto; }
  .spec-cell { display: flex; flex-direction: column; gap: 2px; flex-shrink: 0; }
  .spec-label { font-size: 0.75rem; color: var(--text-muted); font-weight: 600; text-transform: uppercase; }
  .spec-val { font-size: 0.88rem; color: var(--text-primary); font-weight: 600; }

  .kv-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 16px; }
  .kv-block { display: flex; flex-direction: column; gap: 6px; }
  .kv-title { font-size: 0.8rem; color: var(--text-muted); font-weight: 600; }
  .tag-wrap { display: flex; flex-wrap: wrap; gap: 6px; }
  .flat-tag { background: rgba(255, 255, 255, 0.04); border: 1px solid var(--border-primary); border-radius: 4px; padding: 2px 6px; font-size: 0.78rem; color: var(--text-secondary); font-family: monospace; }
  .flat-tag .k { color: var(--text-primary); }
  .flat-tag.annotation { background: rgba(59, 130, 246, 0.05); border-color: rgba(59, 130, 246, 0.2); }
  .muted-text { color: var(--text-muted); font-size: 0.85rem; }

  .ports-table { border: 1px solid var(--border-primary); border-radius: var(--radius-sm); overflow: hidden; }
  .pt-head { display: grid; grid-template-columns: 1fr 100px 120px 100px 100px; background: rgba(255, 255, 255, 0.03); padding: 8px 12px; font-size: 0.8rem; font-weight: 600; color: var(--text-secondary); border-bottom: 1px solid var(--border-primary); }
  .pt-row { display: grid; grid-template-columns: 1fr 100px 120px 100px 100px; padding: 8px 12px; font-size: 0.85rem; border-bottom: 1px solid rgba(255, 255, 255, 0.04); align-items: center; }
  .pt-row:last-child { border-bottom: none; }
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
