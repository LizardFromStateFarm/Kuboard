<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import QuickActionsMenu from './QuickActionsMenu.svelte';

  const dispatch = createEventDispatcher();

  export let cronJob: any;
  export let onBack: () => void;

  let cronJobDetails: any = null;
  let managedJobs: any[] = [];
  let loading = false;
  let error: string | null = null;
  let jobsLoading = false;
  let jobsError: string | null = null;

  let actionsMenuVisible = false;
  let actionsMenuPosition = { x: 0, y: 0 };
  let yamlViewerVisible = false;
  let yamlContent = '';

  $: cj = cronJobDetails || cronJob;
  $: status = getCronJobStatus(cj);

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

  function getCronJobStatus(c: any): string {
    return c?.spec?.suspend ? 'Suspended' : 'Active';
  }

  function getStatusClass(st: string): string {
    switch (st?.toLowerCase()) {
      case 'active': return 'ready';
      case 'suspended': return 'pending';
      default: return 'unknown';
    }
  }

  async function loadCronJobDetails() {
    if (!cronJob?.metadata?.name || !cronJob?.metadata?.namespace) return;
    loading = true; error = null;
    try {
      const details = await invoke('kuboard_get_cronjob_details', {
        name: cronJob.metadata.name,
        namespace: cronJob.metadata.namespace
      });
      cronJobDetails = details;
      await loadManagedJobs();
    } catch (err: any) {
      console.warn('Failed to load cronjob details via Tauri API:', err);
      cronJobDetails = cronJob;
    } finally {
      loading = false;
    }
  }

  async function loadManagedJobs() {
    if (!cronJob?.metadata?.name || !cronJob?.metadata?.namespace) return;
    jobsLoading = true; jobsError = null;
    try {
      const jobs = await invoke('kuboard_get_cronjob_jobs', {
        name: cronJob.metadata.name,
        namespace: cronJob.metadata.namespace
      }) as any[];
      managedJobs = jobs || [];
    } catch (err: any) {
      console.warn('Failed to load managed jobs:', err);
      managedJobs = [];
    } finally {
      jobsLoading = false;
    }
  }

  function openActionsMenu(event: MouseEvent) {
    event.stopPropagation();
    const btn = event.currentTarget as HTMLElement;
    if (btn && btn.getBoundingClientRect) {
      const rect = btn.getBoundingClientRect();
      actionsMenuPosition = { x: Math.max(12, rect.right - 220), y: rect.bottom + 6 };
    } else {
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

  onMount(() => { loadCronJobDetails(); });
</script>

<div class="resource-details-view">
  <!-- Top Action Bar -->
  <div class="details-nav-bar">
    <div class="nav-actions">
      <button class="btn-back" onclick={onBack}>← Back to CronJobs</button>
      <button class="btn-subtle" onclick={openActionsMenu}>⚙️ Actions</button>
    </div>
    <div class="nav-heading">
      <span class="status-pill status-{getStatusClass(status)}">{status}</span>
      <h3 class="nav-title">{cj?.metadata?.name}</h3>
      <span class="namespace-pill">{cj?.metadata?.namespace}</span>
    </div>
  </div>

  <!-- Master Sheet -->
  <div class="details-sheet">
    <!-- Key Specs Summary Strip -->
    <div class="sheet-section specs-strip">
      <div class="spec-cell">
        <span class="spec-label">Schedule</span>
        <span class="spec-val font-mono">{cj?.spec?.schedule || '-'}</span>
      </div>
      <div class="spec-cell">
        <span class="spec-label">Suspend</span>
        <span class="spec-val">{cj?.spec?.suspend ? 'True (Paused)' : 'False (Active)'}</span>
      </div>
      <div class="spec-cell">
        <span class="spec-label">Concurrency Policy</span>
        <span class="spec-val">{cj?.spec?.concurrencyPolicy || 'Allow'}</span>
      </div>
      <div class="spec-cell">
        <span class="spec-label">Last Schedule Time</span>
        <span class="spec-val">{formatAge(cj?.status?.lastScheduleTime)}</span>
      </div>
      <div class="spec-cell">
        <span class="spec-label">Age</span>
        <span class="spec-val">{formatAge(cj?.metadata?.creationTimestamp)}</span>
      </div>
    </div>

    <!-- Active/Recent Jobs -->
    <div class="sheet-section">
      <h5>⚡ Recent Jobs ({managedJobs.length})</h5>
      {#if jobsLoading}
        <div class="muted-text">⏳ Loading jobs...</div>
      {:else if managedJobs.length > 0}
        <div class="jobs-table">
          <div class="j-head">
            <div>Name</div>
            <div>Completions</div>
            <div>Duration</div>
            <div>Age</div>
          </div>
          {#each managedJobs as job}
            <div class="j-row">
              <div class="bold">{job.metadata?.name || 'Unknown'}</div>
              <div>{job.status?.succeeded || 0} / {job.spec?.completions || 1}</div>
              <div>{job.status?.completionTime ? formatAge(job.status.startTime) : 'Running'}</div>
              <div>{formatAge(job.metadata?.creationTimestamp)}</div>
            </div>
          {/each}
        </div>
      {:else}
        <p class="muted-text">No recent execution jobs found for this CronJob</p>
      {/if}
    </div>

    <!-- Labels & Annotations -->
    <div class="sheet-section">
      <h5>🏷️ Labels & Annotations</h5>
      <div class="kv-grid">
        <div class="kv-block">
          <span class="kv-title">Labels</span>
          <div class="tag-wrap">
            {#if cj?.metadata?.labels && Object.keys(cj.metadata.labels).length > 0}
              {#each Object.entries(cj.metadata.labels) as [k, v]}
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
            {#if cj?.metadata?.annotations && Object.keys(cj.metadata.annotations).length > 0}
              {#each Object.entries(cj.metadata.annotations) as [k, v]}
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
  resource={cronJobDetails || cronJob}
  resourceType="cronjob"
  bind:visible={actionsMenuVisible}
  on:close={handleActionMenuClose}
  on:deleted={handleActionDeleted}
  on:view-yaml={handleViewYaml}
/>

{#if yamlViewerVisible}
  <div class="modal-overlay" onclick={closeYamlViewer} role="button" tabindex="-1" onkeydown={(e) => e.key === 'Escape' && closeYamlViewer()}>
    <div class="modal-box" onclick={(e) => e.stopPropagation()} role="dialog" tabindex="-1">
      <div class="modal-hdr">
        <h4>CronJob YAML: {cronJob?.metadata?.name}</h4>
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
  .status-unknown { background: rgba(156, 163, 175, 0.15); color: #9ca3af; }

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

  .jobs-table { border: 1px solid var(--border-primary); border-radius: var(--radius-sm); overflow: hidden; }
  .j-head { display: grid; grid-template-columns: 1fr 120px 120px 80px; background: rgba(255, 255, 255, 0.03); padding: 8px 12px; font-size: 0.8rem; font-weight: 600; color: var(--text-secondary); border-bottom: 1px solid var(--border-primary); }
  .j-row { display: grid; grid-template-columns: 1fr 120px 120px 80px; padding: 8px 12px; font-size: 0.85rem; border-bottom: 1px solid rgba(255, 255, 255, 0.04); align-items: center; }
  .j-row:last-child { border-bottom: none; }
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
