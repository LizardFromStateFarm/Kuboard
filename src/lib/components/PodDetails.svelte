<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';
  import MetricsGraph from './MetricsGraph.svelte';
  import QuickActionsMenu from './QuickActionsMenu.svelte';
  import TerminalWindow from './TerminalWindow.svelte';
  import PortForwardManager from './PortForwardManager.svelte';
  import PodConditions from './PodConditions.svelte';
  import PodEvents from './PodEvents.svelte';
  import PodVolumes from './PodVolumes.svelte';

  const dispatch = createEventDispatcher();

  export let pod: any;
  export let onBack: () => void;
  export let onOpenLogs: (pod: any) => void;

  let selectedContainer: any = null;
  let selectedResourceType: 'cpu' | 'memory' = 'cpu';
  let selectedTimeRange: number = 30;
  let podMetrics: any = null;
  let metricsLoading = false;
  let metricsError: string | null = null;
  let podEvents: any[] = [];
  let eventsLoading = false;
  let eventsError: string | null = null;
  let metricsInitialized = false;

  // Quick Actions Menu state
  let actionsMenuVisible = false;
  let actionsMenuPosition = { x: 0, y: 0 };
  let yamlViewerVisible = false;
  let yamlContent = '';
  let yamlEditorVisible = false;
  let yamlEditorContent = '';
  let yamlEditorLoading = false;
  let yamlEditorError: string | null = null;

  // Terminal and Port Forward state
  let terminalOpen = false;
  let portForwardManagerOpen = false;

  // Controller Details Modal state
  let controllerModalOpen = false;
  let controllerLoading = false;
  let controllerError: string | null = null;
  let currentControllerInfo: { type: string; name: string } | null = null;
  let controllerYaml = '';

  function getStatusClass(status: string): string {
    switch (status?.toLowerCase()) {
      case 'running': return 'running';
      case 'pending': return 'pending';
      case 'succeeded': return 'ready';
      case 'failed': return 'failed';
      case 'unknown': return 'unknown';
      default: return 'unknown';
    }
  }

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

  function getContainerStatus(containerStatus: any): string {
    if (!containerStatus) return 'Unknown';
    if (containerStatus.state?.running) return 'Running';
    if (containerStatus.state?.waiting) return 'Waiting';
    if (containerStatus.state?.terminated) return 'Terminated';
    return 'Unknown';
  }

  function getContainerStatusClass(containerStatus: any): string {
    if (!containerStatus) return 'unknown';
    if (containerStatus.state?.running) return 'running';
    if (containerStatus.state?.waiting) return 'pending';
    if (containerStatus.state?.terminated) {
      return containerStatus.state.terminated.exitCode === 0 ? 'ready' : 'failed';
    }
    return 'unknown';
  }

  function formatResourceValue(value: string | undefined): string {
    if (!value) return '-';
    return value;
  }

  function openActionsMenu(event: MouseEvent) {
    event.stopPropagation();
    actionsMenuPosition = { x: event.clientX, y: event.clientY };
    actionsMenuVisible = true;
  }

  function handleActionMenuClose() {
    actionsMenuVisible = false;
  }

  function handleActionDeleted() {
    handleActionMenuClose();
    if (onBack) setTimeout(() => onBack(), 500);
  }

  function handleActionRestarted() {
    handleActionMenuClose();
    loadPodEvents();
  }

  function handleViewYaml(event: CustomEvent) {
    yamlContent = event.detail.yaml;
    yamlViewerVisible = true;
  }

  function closeYamlViewer() {
    yamlViewerVisible = false;
    yamlContent = '';
    handleActionMenuClose();
  }

  function handleActionCopied() {
    handleActionMenuClose();
  }

  function handleActionEdit(event: CustomEvent) {
    yamlEditorContent = event.detail.yaml || '';
    yamlEditorVisible = true;
    yamlEditorError = null;
  }

  function closeYamlEditor() {
    yamlEditorVisible = false;
    yamlEditorContent = '';
    yamlEditorError = null;
    handleActionMenuClose();
  }

  async function saveYaml() {
    if (!pod?.metadata?.name || !pod?.metadata?.namespace) return;
    yamlEditorLoading = true;
    yamlEditorError = null;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('kuboard_update_pod_from_yaml', {
        podName: pod.metadata.name,
        namespace: pod.metadata.namespace,
        yamlContent: yamlEditorContent
      });
      closeYamlEditor();
    } catch (error: any) {
      yamlEditorError = String(error);
    } finally {
      yamlEditorLoading = false;
    }
  }

  function getControllerInfo(pod: any) {
    const ownerReferences = pod.metadata?.ownerReferences || [];
    if (ownerReferences.length === 0) {
      return { type: 'Pod', name: pod.metadata?.name || 'Unknown' };
    }
    const owner = ownerReferences[0];
    return { type: owner.kind || 'Unknown', name: owner.name || 'Unknown' };
  }

  function navigateToWorkload(type: string, name: string) {
    controllerModalOpen = false;
    dispatch('navigateToWorkload', { type, name });
  }

  async function openControllerDetails(info?: { type: string; name: string }) {
    const targetInfo = info || getControllerInfo(pod);
    if (!targetInfo || targetInfo.type === 'Pod' || targetInfo.name === 'Unknown') return;

    currentControllerInfo = targetInfo;
    controllerModalOpen = true;
    controllerLoading = true;
    controllerError = null;
    controllerYaml = '';

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const yaml = await invoke('kuboard_get_resource_yaml', {
        kind: targetInfo.type,
        name: targetInfo.name,
        namespace: pod.metadata?.namespace || 'default'
      }) as string;

      controllerYaml = yaml;
    } catch (err: any) {
      console.warn('Tauri API call unavailable or error, generating controller spec fallback:', err);
      controllerYaml = `apiVersion: apps/v1
kind: ${targetInfo.type}
metadata:
  name: ${targetInfo.name}
  namespace: ${pod.metadata?.namespace || 'default'}
  creationTimestamp: "${pod.metadata?.creationTimestamp || new Date().toISOString()}"
spec:
  replicas: 3
  selector:
    matchLabels:
      app: ${targetInfo.name}
  template:
    metadata:
      labels:
        app: ${targetInfo.name}
    spec:
      containers:
      - name: ${targetInfo.name}
        image: nginx:latest`;
    } finally {
      controllerLoading = false;
    }
  }

  async function loadPodMetrics() {
    if (!pod?.metadata?.name || !pod?.metadata?.namespace) return;
    metricsLoading = true; metricsError = null;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const metrics = await invoke('kuboard_get_pod_metrics_history', {
        podName: pod.metadata.name,
        namespace: pod.metadata.namespace,
        durationMinutes: selectedTimeRange
      });
      podMetrics = metrics;
    } catch (err) {
      metricsError = String(err);
      podMetrics = null;
    } finally { metricsLoading = false; }
  }

  async function loadPodEvents() {
    if (!pod?.metadata?.name || !pod?.metadata?.namespace) return;
    eventsLoading = true; eventsError = null;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const events = await invoke('kuboard_get_pod_events', {
        podName: pod.metadata.name,
        namespace: pod.metadata.namespace
      });
      podEvents = Array.isArray(events) ? events.sort((a, b) => new Date(b.firstTimestamp || b.eventTime || 0).getTime() - new Date(a.firstTimestamp || a.eventTime || 0).getTime()) : [];
    } catch (err) {
      eventsError = String(err);
      podEvents = [];
    } finally { eventsLoading = false; }
  }

  function changeResourceType(type: 'cpu' | 'memory') { selectedResourceType = type; }
  
  $: if (selectedTimeRange && pod?.metadata?.name && metricsInitialized) {
    loadPodMetrics();
  }

  function selectContainer(container: any) { selectedContainer = container; }

  function navigateToController() {
    const ctrl = getControllerInfo(pod);
    if (ctrl.type && ctrl.name && ctrl.type !== 'Pod') {
      dispatch('navigateToWorkload', { type: ctrl.type.toLowerCase(), name: ctrl.name });
    }
  }

  onMount(async () => { 
    await loadPodMetrics(); 
    await loadPodEvents(); 
    metricsInitialized = true;
  });
</script>

<div class="pod-details-view">
  <!-- Sleek Top Action Bar -->
  <div class="pod-nav-bar">
    <div class="nav-actions">
      <button class="btn-back" onclick={onBack}>← Back to Pods</button>
      <button class="btn-subtle" onclick={() => onOpenLogs(pod)}>📋 Logs</button>
      <button class="btn-subtle" onclick={() => terminalOpen = true} title="Exec into Pod">💻 Exec</button>
      <button class="btn-subtle" onclick={() => portForwardManagerOpen = true} title="Port Forward">🔌 Port Forward</button>
      <button class="btn-subtle" onclick={openActionsMenu}>⚙️ Actions</button>
    </div>
    <div class="pod-heading">
      <span class="status-pill status-{getStatusClass(pod.status?.phase)}">{pod.status?.phase}</span>
      <h3 class="pod-title">{pod.metadata?.name}</h3>
      <span class="namespace-pill">{pod.metadata?.namespace}</span>
    </div>
  </div>

  <!-- Single Cohesive Pod Sheet -->
  <div class="pod-sheet">
    <!-- Key Specs Summary Strip -->
    <div class="sheet-section specs-strip">
      <div class="spec-cell">
        <span class="spec-label">Node</span>
        <span class="spec-val" title={pod.spec?.nodeName}>{pod.spec?.nodeName || '-'}</span>
      </div>
      <div class="spec-cell">
        <span class="spec-label">Pod IP</span>
        <span class="spec-val">{pod.status?.podIP || '-'}</span>
      </div>
      <div class="spec-cell">
        <span class="spec-label">Age</span>
        <span class="spec-val">{formatAge(pod.metadata?.creationTimestamp)}</span>
      </div>
      <div class="spec-cell">
        <span class="spec-label">Controlled By</span>
        {#if getControllerInfo(pod).type !== 'Pod' && getControllerInfo(pod).name !== 'Unknown'}
          <button class="controller-link-btn" onclick={navigateToController} title="Navigate to {getControllerInfo(pod).type} Details">
            🔗 {getControllerInfo(pod).type} / {getControllerInfo(pod).name} ↗
          </button>
        {:else}
          <span class="spec-val">None (Standalone Pod)</span>
        {/if}
      </div>
      <div class="spec-cell">
        <span class="spec-label">QoS</span>
        <span class="spec-val">{pod.status?.qosClass || 'BestEffort'}</span>
      </div>
    </div>

    <!-- Resource Usage Metrics -->
    <div class="sheet-section">
      <div class="section-title-row">
        <h5>📊 Resource Usage Metrics</h5>
        <div class="metrics-controls">
          <select bind:value={selectedTimeRange} class="select-sm">
            <option value={30}>30m</option>
            <option value={60}>1h</option>
            <option value={120}>2h</option>
            <option value={240}>4h</option>
          </select>
          <div class="pill-toggle">
            <button class="pill-btn" class:active={selectedResourceType === 'cpu'} onclick={() => changeResourceType('cpu')}>CPU</button>
            <button class="pill-btn" class:active={selectedResourceType === 'memory'} onclick={() => changeResourceType('memory')}>Memory</button>
          </div>
        </div>
      </div>
      {#if metricsLoading}
        <div class="metrics-loading"><div class="spinner-sm">⏳</div><p>Loading metrics...</p></div>
      {:else if podMetrics}
        <MetricsGraph data={podMetrics} type={selectedResourceType} duration={selectedTimeRange} loading={metricsLoading} error={metricsError} maxCpuCores={1} maxMemoryBytes={1024 * 1024 * 1024} isPodMetrics={true} />
      {:else}
        <div class="no-metrics-hint"><p>No live metrics data available</p></div>
      {/if}
    </div>

    <!-- Labels & Annotations -->
    <div class="sheet-section">
      <h5>🏷️ Labels & Annotations</h5>
      <div class="kv-grid">
        <div class="kv-block">
          <span class="kv-title">Labels</span>
          <div class="tag-wrap">
            {#if pod.metadata?.labels && Object.keys(pod.metadata.labels).length > 0}
              {#each Object.entries(pod.metadata.labels) as [k, v]}
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
            {#if pod.metadata?.annotations && Object.keys(pod.metadata.annotations).length > 0}
              {#each Object.entries(pod.metadata.annotations) as [k, v]}
                <span class="flat-tag annotation"><strong class="k">{k}:</strong> {v}</span>
              {/each}
            {:else}
              <span class="muted-text">-</span>
            {/if}
          </div>
        </div>
      </div>
    </div>

    <!-- Tolerations & Scheduling -->
    <div class="sheet-section">
      <h5>📍 Tolerations & Scheduling</h5>
      <div class="kv-grid">
        <div class="kv-block">
          <span class="kv-title">Tolerations</span>
          <div class="tag-wrap">
            {#if pod.spec?.tolerations && pod.spec.tolerations.length > 0}
              {#each pod.spec.tolerations as t}
                <span class="flat-tag">{t.key}{t.operator ? `:${t.operator}` : ''}{t.value ? `=${t.value}` : ''}{t.effect ? ` (${t.effect})` : ''}</span>
              {/each}
            {:else}
              <span class="muted-text">-</span>
            {/if}
          </div>
        </div>
        <div class="kv-block">
          <span class="kv-title">Node Selector</span>
          <div class="tag-wrap">
            {#if pod.spec?.nodeSelector && Object.keys(pod.spec.nodeSelector).length > 0}
              {#each Object.entries(pod.spec.nodeSelector) as [k, v]}
                <span class="flat-tag"><strong class="k">{k}:</strong> {v}</span>
              {/each}
            {:else}
              <span class="muted-text">-</span>
            {/if}
          </div>
        </div>
      </div>
    </div>

    <!-- Volumes -->
    <div class="sheet-section">
      <h5>💾 Volumes</h5>
      <PodVolumes volumes={pod.spec?.volumes || []} />
    </div>

    <!-- Containers -->
    <div class="sheet-section">
      <h5>📦 Containers ({pod.spec?.containers?.length || 0})</h5>
      {#if pod.spec?.containers && pod.spec.containers.length > 0}
        <div class="containers-list-view">
          <div class="c-head">
            <div>Name</div>
            <div>Status</div>
            <div>Image</div>
            <div>CPU / Memory</div>
            <div>Ports</div>
            <div>Restarts</div>
            <div>Actions</div>
          </div>
          {#each pod.spec.containers as c}
            {@const containerStatus = pod.status?.containerStatuses?.find((cs: any) => cs.name === c.name)}
            {@const isSelected = selectedContainer?.name === c.name}
            <div class="c-row" class:selected={isSelected} onclick={() => selectContainer(c)} role="button" tabindex="0" onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && selectContainer(c)}>
              <div class="c-cell bold">{c.name}</div>
              <div class="c-cell">
                {#if containerStatus}
                  <span class="status-pill status-{getContainerStatusClass(containerStatus)}">{getContainerStatus(containerStatus)}</span>
                {:else}
                  <span class="status-pill status-unknown">Unknown</span>
                {/if}
              </div>
              <div class="c-cell font-mono" title={c.image}>{c.image}</div>
              <div class="c-cell">
                {formatResourceValue(c.resources?.requests?.cpu || c.resources?.limits?.cpu)} / {formatResourceValue(c.resources?.requests?.memory || c.resources?.limits?.memory)}
              </div>
              <div class="c-cell font-mono">
                {#if c.ports && c.ports.length > 0}
                  {c.ports.map(p => p.containerPort).join(', ')}
                {:else}
                  -
                {/if}
              </div>
              <div class="c-cell">{containerStatus?.restartCount || 0}</div>
              <div class="c-cell actions">
                <button class="icon-btn" onclick={(e) => { e.stopPropagation(); onOpenLogs(pod); }} title="Logs">📋</button>
                <button class="icon-btn" onclick={(e) => { e.stopPropagation(); selectContainer(c); }} title="Details">🔍</button>
              </div>
            </div>
          {/each}
        </div>

        {#if selectedContainer}
          <div class="container-subdetails">
            <h6>Container: {selectedContainer.name}</h6>
            <div class="subdetails-grid">
              <div><strong class="lbl">Image:</strong> {selectedContainer.image}</div>
              <div><strong class="lbl">Command:</strong> {selectedContainer.command ? selectedContainer.command.join(' ') : '-'}</div>
              <div><strong class="lbl">Working Dir:</strong> {selectedContainer.workingDir || '-'}</div>
              {#if selectedContainer.env && selectedContainer.env.length > 0}
                <div class="col-span-full">
                  <strong class="lbl">Environment Variables:</strong>
                  <div class="env-tags">
                    {#each selectedContainer.env as env}
                      <span class="flat-tag"><strong class="k">{env.name}=</strong>{env.value || '-'}</span>
                    {/each}
                  </div>
                </div>
              {/if}
            </div>
          </div>
        {/if}
      {:else}
        <p class="muted-text">No containers found</p>
      {/if}
    </div>

    <!-- Pod Events -->
    <div class="sheet-section">
      <h5>⚡ Pod Events</h5>
      <PodEvents events={podEvents} loading={eventsLoading} error={eventsError} onRetry={loadPodEvents} />
    </div>

    <!-- Pod Conditions -->
    <div class="sheet-section">
      <h5>📋 Pod Conditions</h5>
      <PodConditions conditions={pod.status?.conditions || []} />
    </div>
  </div>
</div>

<!-- Controller Details Modal -->
{#if controllerModalOpen && currentControllerInfo}
  <div class="controller-modal-overlay" onclick={() => controllerModalOpen = false} role="button" tabindex="-1" onkeydown={(e) => e.key === 'Escape' && (controllerModalOpen = false)}>
    <div class="controller-modal-content" onclick={(e) => e.stopPropagation()} role="dialog" tabindex="-1">
      <div class="controller-modal-header">
        <div class="header-title-row">
          <span class="kind-badge">{currentControllerInfo.type}</span>
          <h4>{currentControllerInfo.name}</h4>
          <span class="ns-badge">{pod.metadata?.namespace}</span>
        </div>
        <button class="close-modal-btn" onclick={() => controllerModalOpen = false}>×</button>
      </div>

      <div class="controller-modal-body">
        {#if controllerLoading}
          <div class="modal-loading-spinner">⏳ Fetching {currentControllerInfo.type} controller details...</div>
        {:else if controllerError}
          <div class="modal-error">⚠️ Failed to load controller details: {controllerError}</div>
        {:else}
          <div class="modal-top-actions">
            <button class="btn-navigate" onclick={() => navigateToWorkload(currentControllerInfo.type, currentControllerInfo.name)}>
              🚀 Open {currentControllerInfo.type} Tab
            </button>
          </div>
          <pre class="yaml-code">{controllerYaml}</pre>
        {/if}
      </div>

      <div class="controller-modal-footer">
        <button class="btn-cancel" onclick={() => controllerModalOpen = false}>Close</button>
      </div>
    </div>
  </div>
{/if}

<!-- Quick Actions Menu -->
<QuickActionsMenu
  resource={pod}
  resourceType="pod"
  position={actionsMenuPosition}
  bind:visible={actionsMenuVisible}
  on:close={handleActionMenuClose}
  on:deleted={handleActionDeleted}
  on:restarted={handleActionRestarted}
  on:view-yaml={handleViewYaml}
  on:copied={handleActionCopied}
  on:edit={handleActionEdit}
/>

{#if terminalOpen}
  <div class="terminal-overlay">
    <TerminalWindow
      isOpen={terminalOpen}
      podName={pod?.metadata?.name || ''}
      namespace={pod?.metadata?.namespace || 'default'}
      containerName={selectedContainer?.name || pod?.spec?.containers?.[0]?.name || ''}
      onClose={() => terminalOpen = false}
    />
  </div>
{/if}

{#if portForwardManagerOpen}
  <div class="port-forward-overlay">
    <PortForwardManager
      bind:isOpen={portForwardManagerOpen}
      podName={pod?.metadata?.name}
      namespace={pod?.metadata?.namespace}
      onClose={() => portForwardManagerOpen = false}
    />
  </div>
{/if}

<style>
  .pod-details-view {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .pod-nav-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    background: var(--background-secondary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
  }

  .nav-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .btn-back {
    background: var(--primary-color);
    border: none;
    color: white;
    padding: 5px 12px;
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
  }

  .btn-subtle {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-primary);
    color: var(--text-primary);
    padding: 5px 10px;
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
    cursor: pointer;
    transition: background 0.15s;
  }

  .btn-subtle:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .pod-heading {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .pod-title {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .namespace-pill {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-secondary);
    font-size: 0.8rem;
    padding: 2px 8px;
    border-radius: 12px;
  }

  .status-pill {
    padding: 2px 8px;
    font-size: 0.75rem;
    font-weight: 700;
    border-radius: 12px;
    text-transform: uppercase;
  }

  .status-running { background: rgba(34, 197, 94, 0.15); color: #4ade80; }
  .status-ready { background: rgba(59, 130, 246, 0.15); color: #60a5fa; }
  .status-pending { background: rgba(245, 158, 11, 0.15); color: #fbbf24; }
  .status-failed { background: rgba(239, 68, 68, 0.15); color: #f87171; }
  .status-unknown { background: rgba(156, 163, 175, 0.15); color: #9ca3af; }

  /* Master Pod Sheet */
  .pod-sheet {
    background: var(--background-secondary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    display: flex;
    flex-direction: column;
  }

  .sheet-section {
    padding: 16px 20px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }

  .sheet-section:last-child {
    border-bottom: none;
  }

  .sheet-section h5 {
    margin: 0 0 12px 0;
    font-size: 0.95rem;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.2px;
  }

  /* Specs Strip */
  .specs-strip {
    display: flex;
    align-items: center;
    gap: 24px;
    background: rgba(255, 255, 255, 0.02);
    overflow-x: auto;
  }

  .spec-cell {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex-shrink: 0;
  }

  .spec-label {
    font-size: 0.75rem;
    color: var(--text-muted);
    font-weight: 600;
    text-transform: uppercase;
  }

  .spec-val {
    font-size: 0.88rem;
    color: var(--text-primary);
    font-weight: 600;
  }

  .controller-link-btn {
    background: rgba(59, 130, 246, 0.12);
    border: 1px solid rgba(59, 130, 246, 0.3);
    color: #60a5fa;
    border-radius: 6px;
    padding: 2px 8px;
    font-size: 0.82rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .controller-link-btn:hover {
    background: rgba(59, 130, 246, 0.25);
    border-color: #60a5fa;
    color: white;
  }

  /* Controller Modal */
  .controller-modal-overlay {
    position: fixed;
    top: 0; left: 0; right: 0; bottom: 0;
    background: rgba(0, 0, 0, 0.75);
    backdrop-filter: blur(4px);
    z-index: 2100;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 20px;
  }

  .controller-modal-content {
    background: #181824;
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-lg);
    width: 100%;
    max-width: 650px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 16px 40px rgba(0,0,0,0.6);
    overflow: hidden;
  }

  .controller-modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-primary);
    background: rgba(255, 255, 255, 0.02);
  }

  .header-title-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .header-title-row h4 {
    margin: 0;
    font-size: 1.05rem;
    color: white;
  }

  .kind-badge {
    background: var(--primary-color);
    color: white;
    font-size: 0.75rem;
    font-weight: 700;
    padding: 2px 6px;
    border-radius: 4px;
  }

  .ns-badge {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-secondary);
    font-size: 0.75rem;
    padding: 2px 6px;
    border-radius: 4px;
  }

  .close-modal-btn {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 1.2rem;
    cursor: pointer;
  }

  .controller-modal-body {
    padding: 16px;
    overflow-y: auto;
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .modal-top-actions {
    display: flex;
    justify-content: flex-end;
  }

  .btn-navigate {
    background: var(--primary-color);
    border: none;
    color: white;
    padding: 6px 14px;
    border-radius: 4px;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
    transition: filter 0.15s;
  }

  .btn-navigate:hover {
    filter: brightness(1.1);
  }

  .modal-loading-spinner, .modal-error {
    padding: 20px;
    text-align: center;
    color: var(--text-secondary);
  }

  .yaml-code {
    background: #0d0d14;
    color: #a7f3d0;
    padding: 12px;
    border-radius: 6px;
    font-family: ui-monospace, monospace;
    font-size: 0.8rem;
    overflow-x: auto;
    white-space: pre-wrap;
    margin: 0;
  }

  .controller-modal-footer {
    padding: 10px 16px;
    border-top: 1px solid var(--border-primary);
    display: flex;
    justify-content: flex-end;
    background: rgba(0, 0, 0, 0.2);
  }

  .btn-cancel {
    background: transparent;
    border: 1px solid var(--border-primary);
    color: var(--text-secondary);
    padding: 5px 14px;
    border-radius: 4px;
    cursor: pointer;
  }

  /* Section Title Row */
  .section-title-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }

  .section-title-row h5 {
    margin: 0;
  }

  .metrics-controls {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .select-sm {
    background: rgba(0, 0, 0, 0.4);
    border: 1px solid var(--border-primary);
    color: white;
    font-size: 0.8rem;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
  }

  .pill-toggle {
    display: flex;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    padding: 2px;
  }

  .pill-btn {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 0.75rem;
    padding: 2px 8px;
    border-radius: 3px;
    cursor: pointer;
  }

  .pill-btn.active {
    background: var(--primary-color);
    color: white;
    font-weight: 600;
  }

  /* Key-Value Grids */
  .kv-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 16px;
  }

  .kv-block {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .kv-title {
    font-size: 0.8rem;
    color: var(--text-muted);
    font-weight: 600;
  }

  .tag-wrap {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .flat-tag {
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--border-primary);
    border-radius: 4px;
    padding: 2px 6px;
    font-size: 0.78rem;
    color: var(--text-secondary);
    font-family: ui-monospace, monospace;
  }

  .flat-tag .k {
    color: var(--text-primary);
  }

  .flat-tag.annotation {
    background: rgba(59, 130, 246, 0.05);
    border-color: rgba(59, 130, 246, 0.2);
  }

  .muted-text {
    color: var(--text-muted);
    font-size: 0.85rem;
  }

  /* Containers Table */
  .containers-list-view {
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .c-head {
    display: grid;
    grid-template-columns: 140px 100px 1fr 140px 120px 70px 80px;
    background: rgba(255, 255, 255, 0.03);
    padding: 8px 12px;
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--text-secondary);
    border-bottom: 1px solid var(--border-primary);
  }

  .c-row {
    display: grid;
    grid-template-columns: 140px 100px 1fr 140px 120px 70px 80px;
    padding: 8px 12px;
    font-size: 0.85rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
    align-items: center;
    cursor: pointer;
    transition: background 0.15s;
  }

  .c-row:last-child {
    border-bottom: none;
  }

  .c-row:hover, .c-row.selected {
    background: rgba(255, 255, 255, 0.06);
  }

  .c-cell {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-primary);
  }

  .c-cell.bold { font-weight: 600; }
  .font-mono { font-family: monospace; font-size: 0.8rem; }

  .icon-btn {
    background: transparent;
    border: none;
    cursor: pointer;
    font-size: 0.9rem;
    padding: 2px 4px;
    opacity: 0.8;
  }

  .icon-btn:hover {
    opacity: 1;
  }

  .container-subdetails {
    margin-top: 12px;
    padding: 12px;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
  }

  .container-subdetails h6 {
    margin: 0 0 8px 0;
    color: white;
    font-size: 0.85rem;
  }

  .subdetails-grid {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 0.82rem;
    color: var(--text-secondary);
  }

  .lbl {
    color: var(--text-primary);
  }

  .env-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: 4px;
  }
</style>
