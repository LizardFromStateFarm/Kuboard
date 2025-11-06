<script lang="ts">
  import { onMount } from 'svelte';
  import MetricsGraph from './MetricsGraph.svelte';
  import QuickActionsMenu from './QuickActionsMenu.svelte';

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

  // Section collapse state
  let sectionsCollapsed = {
    labels: true,
    qos: true,
    tolerations: true,
    scheduling: true,
    volumes: true,
    containers: true,
    controller: true,
    events: true,
    conditions: true
  };

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

  async function copyToClipboard(text: string) {
    try { await navigator.clipboard.writeText(text); } catch {}
  }

  function getQoSClassClass(qosClass: string): string {
    switch (qosClass?.toLowerCase()) {
      case 'guaranteed': return 'qos-guaranteed';
      case 'burstable': return 'qos-burstable';
      case 'besteffort': return 'qos-besteffort';
      default: return 'qos-unknown';
    }
  }

  function getConditionStatusClass(status: string): string {
    switch (status?.toLowerCase()) {
      case 'true': return 'condition-true';
      case 'false': return 'condition-false';
      case 'unknown': return 'condition-unknown';
      default: return 'condition-unknown';
    }
  }

  function formatObject(obj: any): string {
    if (typeof obj === 'string') return obj;
    if (typeof obj === 'number') return obj.toString();
    if (typeof obj === 'boolean') return obj.toString();
    if (Array.isArray(obj)) return obj.join(', ');
    if (obj && typeof obj === 'object') return JSON.stringify(obj, null, 2);
    return 'N/A';
  }

  function formatEventTime(timestamp: string): string {
    if (!timestamp) return 'Unknown';
    try { return new Date(timestamp).toLocaleString(); } catch { return timestamp; }
  }

  function getEventTypeClass(type: string): string {
    switch (type?.toLowerCase()) {
      case 'normal': return 'event-normal';
      case 'warning': return 'event-warning';
      case 'error': return 'event-error';
      default: return 'event-unknown';
    }
  }

  function getEventReasonClass(reason: string): string {
    switch (reason?.toLowerCase()) {
      case 'created': return 'reason-created';
      case 'scheduled': return 'reason-scheduled';
      case 'pulling': return 'reason-pulling';
      case 'pulled': return 'reason-pulled';
      case 'started': return 'reason-started';
      case 'killing': return 'reason-killing';
      case 'killed': return 'reason-killed';
      case 'failed': return 'reason-failed';
      case 'backoff': return 'reason-backoff';
      case 'unhealthy': return 'reason-unhealthy';
      default: return 'reason-unknown';
    }
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

  function toggleSection(section: keyof typeof sectionsCollapsed) {
    sectionsCollapsed[section] = !sectionsCollapsed[section];
    sectionsCollapsed = { ...sectionsCollapsed };
  }

  function openActionsMenu(event: MouseEvent) {
    event.stopPropagation();
    actionsMenuPosition = { x: event.clientX, y: event.clientY };
    actionsMenuVisible = true;
  }

  function handleActionMenuClose() {
    actionsMenuVisible = false;
  }

  function handleActionDeleted(event: CustomEvent) {
    // After deletion, go back to pods list
    handleActionMenuClose();
    if (onBack) {
      setTimeout(() => onBack(), 500);
    }
  }

  function handleActionRestarted(event: CustomEvent) {
    // Refresh pod details
    handleActionMenuClose();
    // Reload pod events/metrics
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

  function handleActionCopied(event: CustomEvent) {
    console.log(`Copied ${event.detail.type}: ${event.detail.value}`);
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
      
      // Close editor and refresh pod details
      closeYamlEditor();
      
      // Reload pod details - would need to trigger a refresh
      // For now, just show success message
      alert('Pod updated successfully! Please refresh the view.');
    } catch (error: any) {
      yamlEditorError = String(error);
      console.error('Failed to update pod:', error);
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
    return { type: owner.kind || 'Unknown', name: owner.name || 'Unknown', uid: owner.uid || 'Unknown' };
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
  
  // Reactive statement to reload metrics when time range changes (only after initial load)
  $: if (selectedTimeRange && pod?.metadata?.name && metricsInitialized) {
    loadPodMetrics();
  }
  function selectContainer(container: any) { selectedContainer = container; }

  onMount(async () => { 
    await loadPodMetrics(); 
    await loadPodEvents(); 
    metricsInitialized = true;
  });
</script>

<div class="full-details-view">
  <div class="details-header">
    <div class="header-left">
      <button class="back-button" onclick={onBack}>← Back to Pods</button>
      <button class="logs-button" onclick={() => onOpenLogs(pod)}>📋 View Logs</button>
      <button class="actions-button" onclick={openActionsMenu}>⚙️ Actions</button>
    </div>
    <div class="header-right">
      <h3>{pod.metadata?.name}</h3>
      <span class="pod-namespace">({pod.metadata?.namespace})</span>
    </div>
  </div>

  <div class="pod-details-content">
    <div class="details-section">
      <h6>Basic Information</h6>
      <div class="info-grid">
        <div class="info-item">
          <span class="info-label">Status:</span>
          <div class="info-value-container">
            <span class="info-value status-badge status-{getStatusClass(pod.status?.phase)}">{pod.status?.phase}</span>
          </div>
        </div>
        <div class="info-item">
          <span class="info-label">Node:</span>
          <div class="info-value-container">
            <span class="info-value" title={pod.spec?.nodeName}>{pod.spec?.nodeName}</span>
            <button class="copy-button" onclick={() => copyToClipboard(pod.spec?.nodeName || '')}>📋</button>
          </div>
        </div>
        <div class="info-item">
          <span class="info-label">Pod IP:</span>
          <div class="info-value-container">
            <span class="info-value" title={pod.status?.podIP}>{pod.status?.podIP}</span>
            <button class="copy-button" onclick={() => copyToClipboard(pod.status?.podIP || '')}>📋</button>
          </div>
        </div>
        <div class="info-item">
          <span class="info-label">Age:</span>
          <div class="info-value-container">
            <span class="info-value" title={pod.metadata?.creationTimestamp}>{formatAge(pod.metadata?.creationTimestamp)}</span>
            <button class="copy-button" onclick={() => copyToClipboard(pod.metadata?.creationTimestamp || '')}>📋</button>
          </div>
        </div>
        <div class="info-item">
          <span class="info-label">Controlled By:</span>
          <div class="info-value-container">
            <span class="info-value">{getControllerInfo(pod).name}</span>
          </div>
        </div>
      </div>
    </div>

    <div class="details-section">
      <h6>Resource Usage Metrics</h6>
      {#if metricsLoading}
        <div class="metrics-loading"><div class="loading-spinner">⏳</div><p>Loading metrics...</p></div>
      {:else if metricsError}
        <div class="metrics-error"><div class="error-icon">⚠️</div><p>Failed to load metrics: {metricsError}</p><button class="retry-button" onclick={loadPodMetrics}>Retry</button></div>
      {:else if podMetrics}
        <div class="metrics-controls">
          <div class="metrics-header">
            <h6>📊 Resource Usage</h6>
            <div class="controls-row">
              <div class="time-range-selector">
                <label for="timeRange">Time Range:</label>
                <select id="timeRange" bind:value={selectedTimeRange} class="time-range-select">
                  <option value={30}>30 minutes</option>
                  <option value={60}>1 hour</option>
                  <option value={120}>2 hours</option>
                  <option value={240}>4 hours</option>
                  <option value={480}>8 hours</option>
                  <option value={720}>12 hours</option>
                </select>
              </div>
              <div class="resource-tabs">
                <button class="resource-tab {selectedResourceType === 'cpu' ? 'active' : ''}" onclick={() => changeResourceType('cpu')}>CPU</button>
                <button class="resource-tab {selectedResourceType === 'memory' ? 'active' : ''}" onclick={() => changeResourceType('memory')}>Memory</button>
              </div>
            </div>
          </div>
        </div>
        <div class="metrics-graph-container">
          <MetricsGraph data={podMetrics} type={selectedResourceType} duration={selectedTimeRange} loading={metricsLoading} error={metricsError} maxCpuCores={1} maxMemoryBytes={1024 * 1024 * 1024} isPodMetrics={true} />
        </div>
      {:else}
        <div class="metrics-placeholder"><p>No metrics data available</p></div>
      {/if}
    </div>

    <div class="details-section">
      <h6 class="section-header" onclick={() => toggleSection('labels')} role="button" tabindex="0" onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSection('labels')}>
        <span>Labels & Annotations</span>
        <span class="collapse-icon">{sectionsCollapsed.labels ? '▶' : '▼'}</span>
      </h6>
      {#if !sectionsCollapsed.labels}
        <div class="info-grid">
        <div class="info-item">
          <span class="info-label">Labels:</span>
          <div class="info-value-container">
            <div class="kv-list">
              {#if pod.metadata?.labels && Object.keys(pod.metadata.labels).length > 0}
                {#each Object.entries(pod.metadata.labels) as [k, v]}
                  <div class="kv"><span class="k">{k}</span><span class="v">{v}</span></div>
                {/each}
              {:else}
                <span class="info-value">-</span>
              {/if}
            </div>
          </div>
        </div>
        <div class="info-item">
          <span class="info-label">Annotations:</span>
          <div class="info-value-container">
            <div class="kv-list">
              {#if pod.metadata?.annotations && Object.keys(pod.metadata.annotations).length > 0}
                {#each Object.entries(pod.metadata.annotations) as [k, v]}
                  <div class="kv"><span class="k">{k}</span><span class="v">{v}</span></div>
                {/each}
              {:else}
                <span class="info-value">-</span>
              {/if}
            </div>
          </div>
        </div>
        </div>
      {/if}
    </div>

    <div class="details-section">
      <h6 class="section-header" onclick={() => toggleSection('qos')} role="button" tabindex="0" onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSection('qos')}>
        <span>QoS Class</span>
        <span class="collapse-icon">{sectionsCollapsed.qos ? '▶' : '▼'}</span>
      </h6>
      {#if !sectionsCollapsed.qos}
        <div class="info-grid">
          <div class="info-item">
            <span class="info-label">QoS Class:</span>
            <div class="info-value-container">
              <span class="info-value {getQoSClassClass(pod.status?.qosClass)}">{pod.status?.qosClass || 'Unknown'}</span>
            </div>
          </div>
        </div>
      {/if}
    </div>

    <div class="details-section">
      <h6 class="section-header" onclick={() => toggleSection('tolerations')} role="button" tabindex="0" onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSection('tolerations')}>
        <span>Tolerations</span>
        <span class="collapse-icon">{sectionsCollapsed.tolerations ? '▶' : '▼'}</span>
      </h6>
      {#if !sectionsCollapsed.tolerations}
        <div class="info-grid">
          <div class="info-item">
            <span class="info-label">Tolerations:</span>
            <div class="info-value-container">
              {#if pod.spec?.tolerations && pod.spec.tolerations.length > 0}
                <div class="tolerations-list">
                  {#each pod.spec.tolerations as t}
                    <div class="toleration-badge">
                      <span title="Key">{t.key}</span>{t.operator ? `:${t.operator}` : ''}{t.value ? `=${t.value}` : ''}{t.effect ? ` (${t.effect})` : ''}
                    </div>
                  {/each}
                </div>
              {:else}
                <span class="info-value">-</span>
              {/if}
            </div>
          </div>
        </div>
      {/if}
    </div>

    <div class="details-section">
      <h6 class="section-header" onclick={() => toggleSection('scheduling')} role="button" tabindex="0" onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSection('scheduling')}>
        <span>Scheduling & Affinity</span>
        <span class="collapse-icon">{sectionsCollapsed.scheduling ? '▶' : '▼'}</span>
      </h6>
      {#if !sectionsCollapsed.scheduling}
        <div class="info-grid">
          <div class="info-item">
            <span class="info-label">Node Selector:</span>
            <div class="info-value-container">
              {#if pod.spec?.nodeSelector && Object.keys(pod.spec.nodeSelector).length > 0}
                <div class="kv-list">
                  {#each Object.entries(pod.spec.nodeSelector) as [k, v]}
                    <div class="kv"><span class="k">{k}</span><span class="v">{v}</span></div>
                  {/each}
                </div>
              {:else}
                <span class="info-value">-</span>
              {/if}
            </div>
          </div>
          <div class="info-item">
            <span class="info-label">Affinity:</span>
            <div class="info-value-container">
              <pre class="info-value">{formatObject(pod.spec?.affinity)}</pre>
            </div>
          </div>
        </div>
      {/if}
    </div>

    <div class="details-section">
      <h6 class="section-header" onclick={() => toggleSection('volumes')} role="button" tabindex="0" onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSection('volumes')}>
        <span>Volumes</span>
        <span class="collapse-icon">{sectionsCollapsed.volumes ? '▶' : '▼'}</span>
      </h6>
      {#if !sectionsCollapsed.volumes}
        {#if pod.spec?.volumes && pod.spec.volumes.length > 0}
          <div class="volumes-list">
            {#each pod.spec.volumes as v}
              <div class="volume-item">
                <span class="volume-name">{v.name}</span>
                <span class="volume-source">{Object.keys(v).find(k => k !== 'name')}</span>
              </div>
            {/each}
          </div>
        {:else}
          <div class="events-placeholder"><p>No volumes</p></div>
        {/if}
      {/if}
    </div>

    <div class="details-section">
      <h6 class="section-header" onclick={() => toggleSection('containers')} role="button" tabindex="0" onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSection('containers')}>
        <span>Containers</span>
        <span class="collapse-icon">{sectionsCollapsed.containers ? '▶' : '▼'}</span>
      </h6>
      {#if !sectionsCollapsed.containers}
        {#if pod.spec?.containers && pod.spec.containers.length > 0}
        <div class="containers-table">
          <div class="containers-header">
            <div>Name</div>
            <div>Status</div>
            <div>Image</div>
            <div>Resources</div>
            <div>Ports</div>
            <div>Restarts</div>
            <div>Probes</div>
            <div>Actions</div>
          </div>
          {#each pod.spec.containers as c, i}
            {@const containerStatus = pod.status?.containerStatuses?.find((cs: any) => cs.name === c.name)}
            {@const isSelected = selectedContainer?.name === c.name}
            <div class="container-row {isSelected ? 'selected' : ''}" role="button" tabindex="0" onclick={() => selectContainer(c)} onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && selectContainer(c)}>
              <div class="c-name">{c.name}</div>
              <div class="c-status">
                {#if containerStatus}
                  <span class="status-badge status-{getContainerStatusClass(containerStatus)}">{getContainerStatus(containerStatus)}</span>
                {:else}
                  <span class="status-badge status-unknown">Unknown</span>
                {/if}
              </div>
              <div class="c-image" title={c.image}>{c.image}</div>
              <div class="c-resources">
                {#if c.resources?.requests || c.resources?.limits}
                  <div class="resource-item">
                    <span class="resource-label">CPU:</span>
                    <span class="resource-value">{formatResourceValue(c.resources?.requests?.cpu)} / {formatResourceValue(c.resources?.limits?.cpu)}</span>
                  </div>
                  <div class="resource-item">
                    <span class="resource-label">Memory:</span>
                    <span class="resource-value">{formatResourceValue(c.resources?.requests?.memory)} / {formatResourceValue(c.resources?.limits?.memory)}</span>
                  </div>
                {:else}
                  <span class="no-resources">No limits</span>
                {/if}
              </div>
              <div class="c-ports">
                {#if c.ports && c.ports.length > 0}
                  {#each c.ports as port}
                    <div class="port-item">{port.containerPort}/{port.protocol || 'TCP'}{port.name ? ` (${port.name})` : ''}</div>
                  {/each}
                {:else}
                  <span class="no-ports">-</span>
                {/if}
              </div>
              <div class="c-restarts">
                {#if containerStatus}
                  <span class="restart-count">{containerStatus.restartCount || 0}</span>
                {:else}
                  <span class="restart-count">-</span>
                {/if}
              </div>
              <div class="c-probes">
                {#if c.livenessProbe || c.readinessProbe || c.startupProbe}
                  <div class="probes-list">
                    {#if c.livenessProbe}<span class="probe-badge liveness">L</span>{/if}
                    {#if c.readinessProbe}<span class="probe-badge readiness">R</span>{/if}
                    {#if c.startupProbe}<span class="probe-badge startup">S</span>{/if}
                  </div>
                {:else}
                  <span class="no-probes">-</span>
                {/if}
              </div>
              <div class="c-actions">
                <button class="action-button logs-button" onclick={(e) => { e.stopPropagation(); onOpenLogs(pod); }} title="View Logs">
                  📋
                </button>
                <button class="action-button details-button" onclick={(e) => { e.stopPropagation(); selectContainer(c); }} title="View Details">
                  🔍
                </button>
              </div>
            </div>
          {/each}
        </div>
        
        {#if selectedContainer}
          <div class="container-details">
            <h6>Container Details: {selectedContainer.name}</h6>
            <div class="container-info-grid">
              <div class="container-info-item">
                <span class="info-label">Image:</span>
                <span class="info-value">{selectedContainer.image}</span>
              </div>
              <div class="container-info-item">
                <span class="info-label">Command:</span>
                <span class="info-value">{selectedContainer.command ? selectedContainer.command.join(' ') : '-'}</span>
              </div>
              <div class="container-info-item">
                <span class="info-label">Args:</span>
                <span class="info-value">{selectedContainer.args ? selectedContainer.args.join(' ') : '-'}</span>
              </div>
              <div class="container-info-item">
                <span class="info-label">Working Dir:</span>
                <span class="info-value">{selectedContainer.workingDir || '-'}</span>
              </div>
              {#if selectedContainer.env && selectedContainer.env.length > 0}
                <div class="container-info-item full-width">
                  <span class="info-label">Environment Variables:</span>
                  <div class="env-list">
                    {#each selectedContainer.env as env}
                      <div class="env-item">
                        <span class="env-name">{env.name}</span>
                        <span class="env-value">{env.value || (env.valueFrom ? 'From: ' + Object.keys(env.valueFrom)[0] : '-')}</span>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}
              {#if selectedContainer.volumeMounts && selectedContainer.volumeMounts.length > 0}
                <div class="container-info-item full-width">
                  <span class="info-label">Volume Mounts:</span>
                  <div class="volume-mounts-list">
                    {#each selectedContainer.volumeMounts as vm}
                      <div class="volume-mount-item">
                        <span class="vm-name">{vm.name}</span>
                        <span class="vm-path">{vm.mountPath}</span>
                        {#if vm.readOnly}<span class="vm-readonly">RO</span>{/if}
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}
            </div>
          </div>
        {/if}
        {:else}
          <div class="events-placeholder"><p>No containers found</p></div>
        {/if}
      {/if}
    </div>

    <div class="details-section">
      <h6 class="section-header" onclick={() => toggleSection('controller')} role="button" tabindex="0" onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSection('controller')}>
        <span>Controller Information</span>
        <span class="collapse-icon">{sectionsCollapsed.controller ? '▶' : '▼'}</span>
      </h6>
      {#if !sectionsCollapsed.controller}
        {#if pod}
          {#key pod.metadata?.uid}
            {@const controller = getControllerInfo(pod)}
            <div class="info-grid">
              <div class="info-item">
                <span class="info-label">Controller Type:</span>
                <div class="info-value-container">
                  <span class="info-value">{controller.type}</span>
                </div>
              </div>
              <div class="info-item">
                <span class="info-label">Controller Name:</span>
                <div class="info-value-container">
                  <span class="info-value">{controller.name}</span>
                </div>
              </div>
            </div>
          {/key}
        {/if}
      {/if}
    </div>

    <div class="details-section">
      <h6 class="section-header" onclick={() => toggleSection('events')} role="button" tabindex="0" onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSection('events')}>
        <span>Pod Events</span>
        <span class="collapse-icon">{sectionsCollapsed.events ? '▶' : '▼'}</span>
      </h6>
      {#if !sectionsCollapsed.events}
        {#if eventsLoading}
        <div class="events-loading"><div class="loading-spinner">⏳</div><p>Loading events...</p></div>
      {:else if eventsError}
        <div class="events-error"><div class="error-icon">⚠️</div><p>Failed to load events: {eventsError}</p><button class="retry-button" onclick={loadPodEvents}>Retry</button></div>
      {:else if podEvents && podEvents.length > 0}
        <div class="events-list">
          {#each podEvents as event}
            <div class="event-item">
              <div class="event-header">
                <div class="event-type-badge event-{getEventTypeClass(event.type)}">{event.type}</div>
                <div class="event-reason reason-{getEventReasonClass(event.reason)}">{event.reason}</div>
                <div class="event-time">{formatEventTime(event.firstTimestamp || event.eventTime)}</div>
              </div>
              <div class="event-message">{event.message}</div>
            </div>
          {/each}
        </div>
        {:else}
          <div class="events-placeholder"><p>No events found for this pod</p></div>
        {/if}
      {/if}
    </div>

    <div class="details-section">
      <h6 class="section-header" onclick={() => toggleSection('conditions')} role="button" tabindex="0" onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSection('conditions')}>
        <span>Conditions</span>
        <span class="collapse-icon">{sectionsCollapsed.conditions ? '▶' : '▼'}</span>
      </h6>
      {#if !sectionsCollapsed.conditions}
        {#if pod.status?.conditions && pod.status.conditions.length > 0}
          <div class="conditions-list">
            {#each pod.status.conditions as c}
              <div class="condition-item">
                <span class="condition-type">{c.type}</span>
                <span class="condition-status {getConditionStatusClass(c.status)}">{c.status}</span>
                <span class="condition-reason">{c.reason}</span>
                <span class="condition-message">{c.message}</span>
              </div>
            {/each}
          </div>
        {:else}
          <div class="events-placeholder"><p>No conditions available</p></div>
        {/if}
      {/if}
    </div>
  </div>
</div>

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

<!-- YAML Viewer Modal -->
{#if yamlViewerVisible}
  <div class="yaml-viewer-modal" onclick={closeYamlViewer}>
    <div class="yaml-viewer-content" onclick={(e) => e.stopPropagation()}>
      <div class="yaml-viewer-header">
        <h3>Pod YAML</h3>
        <button class="yaml-viewer-close" onclick={closeYamlViewer}>✕</button>
      </div>
      <div class="yaml-viewer-body">
        <pre><code>{yamlContent}</code></pre>
      </div>
    </div>
  </div>
{/if}

<!-- YAML Editor Modal -->
{#if yamlEditorVisible}
  <div class="yaml-viewer-modal" onclick={closeYamlEditor}>
    <div class="yaml-viewer-content yaml-editor-content" onclick={(e) => e.stopPropagation()}>
      <div class="yaml-viewer-header">
        <h3>Edit Pod YAML</h3>
        <button class="yaml-viewer-close" onclick={closeYamlEditor} disabled={yamlEditorLoading}>✕</button>
      </div>
      <div class="yaml-editor-body">
        {#if yamlEditorError}
          <div class="yaml-editor-error">
            <span class="error-icon">⚠️</span>
            <span class="error-text">{yamlEditorError}</span>
          </div>
        {/if}
        <textarea
          class="yaml-editor-textarea"
          bind:value={yamlEditorContent}
          disabled={yamlEditorLoading}
          placeholder="Edit YAML content here..."
        ></textarea>
      </div>
      <div class="yaml-editor-footer">
        <button 
          class="yaml-editor-button yaml-editor-cancel" 
          onclick={closeYamlEditor}
          disabled={yamlEditorLoading}
        >
          Cancel
        </button>
        <button 
          class="yaml-editor-button yaml-editor-save" 
          onclick={saveYaml}
          disabled={yamlEditorLoading || !yamlEditorContent.trim()}
        >
          {yamlEditorLoading ? '⏳ Saving...' : '💾 Save'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  @import '../styles/variables.css';
  @import '../styles/color-palette.css';

  .full-details-view {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .details-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-sm) var(--spacing-md);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    background: var(--background-card);
  }

  .header-left {
    display: flex;
    gap: var(--spacing-sm);
    align-items: center;
  }

  .back-button,
  .logs-button,
  .actions-button {
    padding: 6px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-primary);
    background: transparent;
    color: var(--text-primary);
    cursor: pointer;
    font-size: 0.9rem;
    transition: all 0.2s;
  }

  .back-button:hover,
  .logs-button:hover,
  .actions-button:hover {
    background: rgba(255, 255, 255, 0.06);
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .pod-namespace {
    color: var(--text-secondary);
    font-weight: 500;
  }

  .pod-details-content {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .details-section {
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    background: var(--background-card);
    padding: var(--spacing-md);
  }

  .details-section h6 {
    margin: 0 0 var(--spacing-sm) 0;
    color: var(--text-primary);
    font-weight: 700;
    letter-spacing: .2px;
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    cursor: pointer;
    user-select: none;
    transition: color 0.2s ease;
    padding: 8px 0;
    margin: 0 0 var(--spacing-sm) 0;
  }

  .section-header:hover {
    color: var(--primary-color);
  }

  .collapse-icon {
    font-size: 12px;
    color: var(--text-muted);
    transition: transform 0.2s ease;
  }

  .info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
    gap: var(--spacing-md);
  }

  .info-item {
    display: grid;
    grid-template-columns: 110px 1fr;
    align-items: start;
    gap: var(--spacing-sm);
    min-height: 32px;
  }

  .info-label {
    color: var(--text-secondary);
    font-weight: 600;
  }

  .info-value-container {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .info-value {
    color: var(--text-primary);
    font-weight: 600;
    word-break: break-word;
    overflow-wrap: break-word;
    hyphens: auto;
    max-width: 100%;
  }

  .copy-button {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--border-primary);
    color: var(--text-secondary);
    border-radius: 6px;
    padding: 2px 6px;
    cursor: pointer;
  }

  .status-badge {
    padding: 2px 8px;
    font-size: 12px;
    border-radius: 999px;
    font-weight: 700;
    text-transform: uppercase;
    border: 1px solid transparent;
  }

  .status-running { background: rgba(34, 197, 94, 0.12); color: #22c55e; border-color: rgba(34, 197, 94, .24); }
  .status-ready { background: rgba(59, 130, 246, 0.12); color: #3b82f6; border-color: rgba(59, 130, 246, .24); }
  .status-pending { background: rgba(245, 158, 11, 0.12); color: #f59e0b; border-color: rgba(245, 158, 11, .24); }
  .status-failed { background: rgba(239, 68, 68, 0.12); color: #ef4444; border-color: rgba(239, 68, 68, .24); }
  .status-unknown { background: rgba(156, 163, 175, 0.12); color: #9ca3af; border-color: rgba(156, 163, 175, .24); }

  .kv-list { 
    display: flex; 
    flex-wrap: wrap; 
    gap: 6px; 
    max-width: 100%;
    overflow: hidden;
  }
  .kv { 
    display: inline-flex; 
    align-items: center; 
    gap: 6px; 
    padding: 4px 8px; 
    border: 1px solid var(--border-primary); 
    border-radius: 8px; 
    background: rgba(255,255,255,0.04);
    max-width: 100%;
    overflow: hidden;
  }
  .kv .k { 
    color: var(--text-secondary); 
    font-weight: 600; 
    flex-shrink: 0;
  }
  .kv .v { 
    color: var(--text-primary); 
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace; 
    font-size: 12px;
    word-break: break-all;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 200px;
  }

  .conditions-list { 
    display: grid; 
    gap: 8px; 
    max-width: 100%;
  }
  .condition-item { 
    display: grid; 
    grid-template-columns: 160px 90px 1fr; 
    align-items: start; 
    gap: 8px; 
    padding: 8px 10px; 
    border: 1px solid var(--border-primary); 
    border-radius: 8px; 
    background: rgba(255,255,255,0.03);
    min-height: 40px;
  }
  .condition-type { 
    font-weight: 700; 
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .condition-status { 
    padding: 2px 8px; 
    border-radius: 999px; 
    font-size: 12px; 
    font-weight: 700; 
    text-transform: uppercase; 
    text-align: center;
    flex-shrink: 0;
  }
  .condition-true { background: rgba(34, 197, 94, 0.12); color: #22c55e; border: 1px solid rgba(34, 197, 94, .24); }
  .condition-false { background: rgba(239, 68, 68, 0.12); color: #ef4444; border: 1px solid rgba(239, 68, 68, .24); }
  .condition-unknown { background: rgba(156, 163, 175, 0.12); color: #9ca3af; border: 1px solid rgba(156, 163, 175, .24); }
  .condition-reason { 
    color: var(--text-secondary); 
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .condition-message { 
    grid-column: 1 / -1; 
    color: var(--text-muted);
    word-break: break-word;
    overflow-wrap: break-word;
    margin-top: 4px;
  }

  .tolerations-list { 
    display: flex; 
    flex-wrap: wrap; 
    gap: 6px; 
    max-width: 100%;
    overflow: hidden;
  }
  .toleration-badge { 
    padding: 4px 8px; 
    border: 1px solid var(--border-primary); 
    border-radius: 999px; 
    background: rgba(255,255,255,0.04); 
    color: var(--text-secondary); 
    font-weight: 600;
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex-shrink: 1;
    font-size: 11px;
  }

  .volumes-list { display: grid; gap: 8px; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); }
  .volume-item { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 8px 10px; border: 1px solid var(--border-primary); border-radius: 8px; background: rgba(255,255,255,0.03); }
  .volume-name { font-weight: 700; color: var(--text-primary); }
  .volume-source { color: var(--text-secondary); font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace; font-size: 12px; }

  .containers-table { display: grid; gap: 6px; }
  .containers-header { 
    display: grid; 
    grid-template-columns: 1.2fr 0.8fr 1.8fr 1.4fr 1fr 0.6fr 0.8fr 100px; 
    gap: 8px; 
    padding: 8px 10px; 
    background: rgba(255,255,255,0.04); 
    border: 1px solid var(--border-primary); 
    border-radius: 8px; 
    font-weight: 700; 
    color: var(--text-secondary); 
    font-size: 12px;
  }
  .container-row { 
    display: grid; 
    grid-template-columns: 1.2fr 0.8fr 1.8fr 1.4fr 1fr 0.6fr 0.8fr 100px; 
    gap: 8px; 
    padding: 10px; 
    border: 1px solid var(--border-primary); 
    border-radius: 8px; 
    background: rgba(255,255,255,0.02); 
    cursor: pointer; 
    transition: all 0.2s ease;
  }
  .container-row:hover { background: rgba(255,255,255,0.05); }
  .container-row.selected { 
    background: rgba(59, 130, 246, 0.1); 
    border-color: rgba(59, 130, 246, 0.3); 
  }
  .c-name { font-weight: 700; color: var(--text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .c-status { display: flex; align-items: center; }
  .c-image { color: var(--text-secondary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace; font-size: 11px; }
  .c-resources { color: var(--text-secondary); font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace; font-size: 11px; display: grid; gap: 2px; }
  .resource-item { display: flex; gap: 4px; }
  .resource-label { color: var(--text-muted); font-size: 10px; }
  .resource-value { color: var(--text-secondary); }
  .no-resources { color: var(--text-muted); font-size: 11px; }
  .c-ports { color: var(--text-secondary); font-size: 11px; }
  .port-item { font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace; font-size: 10px; }
  .no-ports { color: var(--text-muted); }
  .c-restarts { display: flex; align-items: center; justify-content: center; }
  .restart-count { font-weight: 700; color: var(--text-primary); font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace; }
  .c-probes { display: flex; align-items: center; }
  .probes-list { display: flex; gap: 4px; }
  .probe-badge { 
    padding: 2px 6px; 
    border-radius: 4px; 
    font-size: 10px; 
    font-weight: 700; 
    text-transform: uppercase;
  }
  .probe-badge.liveness { background: rgba(239, 68, 68, 0.12); color: #ef4444; border: 1px solid rgba(239, 68, 68, 0.24); }
  .probe-badge.readiness { background: rgba(34, 197, 94, 0.12); color: #22c55e; border: 1px solid rgba(34, 197, 94, 0.24); }
  .probe-badge.startup { background: rgba(59, 130, 246, 0.12); color: #3b82f6; border: 1px solid rgba(59, 130, 246, 0.24); }
  .no-probes { color: var(--text-muted); font-size: 11px; }
  .c-actions { display: flex; gap: 4px; justify-content: flex-end; }
  .action-button { 
    padding: 4px 6px; 
    border: 1px solid var(--border-primary); 
    border-radius: 4px; 
    background: transparent; 
    color: var(--text-secondary); 
    cursor: pointer; 
    font-size: 12px;
    transition: all 0.2s ease;
  }
  .action-button:hover { 
    background: rgba(255,255,255,0.06); 
    color: var(--text-primary); 
  }

  .container-details {
    margin-top: var(--spacing-md);
    padding: var(--spacing-md);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    background: rgba(255,255,255,0.02);
  }

  .container-details h6 {
    margin: 0 0 var(--spacing-sm) 0;
    color: var(--text-primary);
    font-weight: 700;
  }

  .container-info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: var(--spacing-sm);
  }

  .container-info-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .container-info-item.full-width {
    grid-column: 1 / -1;
  }

  .env-list, .volume-mounts-list {
    display: grid;
    gap: 6px;
  }

  .env-item, .volume-mount-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border: 1px solid var(--border-primary);
    border-radius: 6px;
    background: rgba(255,255,255,0.02);
  }

  .env-name, .vm-name {
    font-weight: 700;
    color: var(--text-primary);
    min-width: 120px;
  }

  .env-value, .vm-path {
    color: var(--text-secondary);
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
    font-size: 11px;
  }

  .vm-readonly {
    padding: 2px 6px;
    background: rgba(245, 158, 11, 0.12);
    color: #f59e0b;
    border: 1px solid rgba(245, 158, 11, 0.24);
    border-radius: 4px;
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
  }

  .metrics-controls {
    margin-bottom: var(--spacing-md);
  }

  .metrics-header {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .controls-row {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    flex-wrap: wrap;
  }

  .time-range-selector {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .time-range-selector label {
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
    white-space: nowrap;
  }

  .time-range-select {
    padding: 6px 10px;
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    background: var(--background-card);
    color: var(--text-primary);
    font-size: 12px;
    min-width: 120px;
    cursor: pointer;
  }

  .time-range-select:hover {
    border-color: var(--primary-color);
  }

  .time-range-select:focus {
    outline: none;
    border-color: var(--primary-color);
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
  }

  .resource-tabs {
    display: flex;
    gap: 4px;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    padding: 2px;
  }

  .resource-tab {
    padding: 6px 12px;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s ease;
    white-space: nowrap;
  }

  .resource-tab:hover {
    background: rgba(255, 255, 255, 0.06);
    color: var(--text-primary);
  }

  .resource-tab.active {
    background: var(--primary-color);
    color: white;
  }

  .metrics-graph-container {
    margin-top: var(--spacing-sm);
  }

  .metrics-loading, .metrics-error, .events-loading, .events-error, .events-placeholder, .metrics-placeholder {
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--text-secondary);
  }

  .events-list {
    display: grid;
    gap: 8px;
    max-width: 100%;
  }

  .event-item {
    padding: 8px 10px;
    border: 1px solid var(--border-primary);
    border-radius: 8px;
    background: rgba(255,255,255,0.02);
    max-width: 100%;
  }

  .event-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
    flex-wrap: wrap;
  }

  .event-type-badge {
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    flex-shrink: 0;
  }

  .event-normal { background: rgba(34, 197, 94, 0.12); color: #22c55e; }
  .event-warning { background: rgba(245, 158, 11, 0.12); color: #f59e0b; }
  .event-error { background: rgba(239, 68, 68, 0.12); color: #ef4444; }
  .event-unknown { background: rgba(156, 163, 175, 0.12); color: #9ca3af; }

  .event-reason {
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 10px;
    font-weight: 600;
    flex-shrink: 0;
  }

  .reason-created, .reason-scheduled, .reason-pulled, .reason-started { background: rgba(34, 197, 94, 0.12); color: #22c55e; }
  .reason-pulling, .reason-killing, .reason-backoff { background: rgba(245, 158, 11, 0.12); color: #f59e0b; }
  .reason-failed, .reason-killed, .reason-unhealthy { background: rgba(239, 68, 68, 0.12); color: #ef4444; }
  .reason-unknown { background: rgba(156, 163, 175, 0.12); color: #9ca3af; }

  .event-time {
    color: var(--text-muted);
    font-size: 11px;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
    margin-left: auto;
    flex-shrink: 0;
  }

  .event-message {
    color: var(--text-primary);
    font-size: 12px;
    word-break: break-word;
    overflow-wrap: break-word;
    line-height: 1.4;
  }

  .retry-button {
    padding: 4px 8px;
    background: var(--primary-color);
    border: 1px solid var(--primary-color);
    color: white;
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
    font-weight: 600;
  }

  .retry-button:hover {
    background: var(--accent-color);
    border-color: var(--accent-color);
  }

  .loading-spinner {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  /* YAML Viewer Modal */
  .yaml-viewer-modal {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 10001;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 20px;
  }

  .yaml-viewer-content {
    background: var(--bg-secondary);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    width: 90%;
    max-width: 900px;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .yaml-viewer-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.05);
  }

  .yaml-viewer-header h3 {
    margin: 0;
    color: var(--text-primary);
    font-size: 1.1rem;
    font-weight: 600;
  }

  .yaml-viewer-close {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 1.2rem;
    line-height: 1;
    transition: all 0.2s;
  }

  .yaml-viewer-close:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
  }

  .yaml-viewer-body {
    flex: 1;
    overflow: auto;
    padding: 20px;
  }

  .yaml-viewer-body pre {
    margin: 0;
    padding: 0;
    background: transparent;
    color: var(--text-primary);
    font-family: 'Courier New', Courier, monospace;
    font-size: 0.85rem;
    line-height: 1.6;
    white-space: pre-wrap;
    word-wrap: break-word;
  }

  .yaml-viewer-body code {
    color: var(--text-primary);
  }

  /* YAML Editor Styles */
  .yaml-editor-content {
    display: flex;
    flex-direction: column;
    height: 90vh;
  }

  .yaml-editor-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: 20px;
  }

  .yaml-editor-error {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px;
    margin-bottom: 12px;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: 6px;
    color: #ef4444;
    font-size: 0.9rem;
  }

  .yaml-editor-error .error-icon {
    font-size: 1.2rem;
  }

  .yaml-editor-error .error-text {
    flex: 1;
  }

  .yaml-editor-textarea {
    flex: 1;
    width: 100%;
    padding: 12px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    background: var(--bg-secondary);
    color: var(--text-primary);
    font-family: 'Courier New', Courier, monospace;
    font-size: 0.85rem;
    line-height: 1.6;
    resize: none;
    outline: none;
    overflow-y: auto;
    white-space: pre;
    tab-size: 2;
  }

  .yaml-editor-textarea:focus {
    border-color: var(--primary-color);
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
  }

  .yaml-editor-textarea:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .yaml-editor-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    padding: 16px 20px;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.05);
  }

  .yaml-editor-button {
    padding: 10px 20px;
    border: none;
    border-radius: 6px;
    font-size: 0.9rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .yaml-editor-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .yaml-editor-cancel {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
  }

  .yaml-editor-cancel:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.2);
  }

  .yaml-editor-save {
    background: var(--primary-color);
    color: white;
  }

  .yaml-editor-save:hover:not(:disabled) {
    background: var(--accent-color);
    transform: translateY(-1px);
  }
</style>


