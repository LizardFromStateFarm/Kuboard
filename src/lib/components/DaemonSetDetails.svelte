<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import QuickActionsMenu from './QuickActionsMenu.svelte';

  export let daemonSet: any;
  export let onBack: () => void;

  let daemonSetDetails: any = null;
  let managedPods: any[] = [];
  let loading = false;
  let error: string | null = null;
  let podsLoading = false;
  let podsError: string | null = null;
  let scaleLoading = false;
  let scaleError: string | null = null;
  let restartLoading = false;
  let restartError: string | null = null;

  // DaemonSets don't support manual scaling - they scale automatically based on node selectors

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
    pods: false,
    volumes: true,
    strategy: true,
    conditions: true,
    selectors: true,
    labels: true,
    yaml: true
  };

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

  function toggleSection(section: keyof typeof sectionsCollapsed) {
    sectionsCollapsed[section] = !sectionsCollapsed[section];
  }

  function formatObject(obj: any): string {
    if (typeof obj === 'string') return obj;
    if (typeof obj === 'number') return obj.toString();
    if (typeof obj === 'boolean') return obj.toString();
    if (Array.isArray(obj)) return obj.join(', ');
    if (obj && typeof obj === 'object') return JSON.stringify(obj, null, 2);
    return 'N/A';
  }

  function getDaemonSetStatus(ds: any): string {
    const desired = ds.status?.desiredNumberScheduled || 0;
    const ready = ds.status?.numberReady || 0;
    const current = ds.status?.currentNumberScheduled || 0;
    const available = ds.status?.numberAvailable || 0;
    
    if (ready === desired && current === desired && available === desired) return 'Ready';
    if (current < desired) return 'Rolling Out';
    if (ready < desired) return 'Not Ready';
    return 'Unknown';
  }

  function getStatusClass(status: string): string {
    switch (status?.toLowerCase()) {
      case 'ready': return 'ready';
      case 'rolling out': return 'pending';
      case 'not ready': return 'failed';
      default: return 'unknown';
    }
  }

  function getUpdateStrategy(ds: any): string {
    const strategy = ds.spec?.updateStrategy?.type || 'RollingUpdate';
    return strategy;
  }

  function getPodStatusClass(status: string): string {
    switch (status?.toLowerCase()) {
      case 'running': return 'running';
      case 'pending': return 'pending';
      case 'succeeded': return 'ready';
      case 'failed': return 'failed';
      case 'unknown': return 'unknown';
      default: return 'unknown';
    }
  }

  async function loadDaemonSetDetails() {
    if (!daemonSet?.metadata?.name || !daemonSet?.metadata?.namespace) return;
    
    loading = true;
    error = null;
    
    try {
      const dsData = await invoke('kuboard_get_daemonset', {
        name: daemonSet.metadata.name,
        namespace: daemonSet.metadata.namespace
      }).catch(() => null);
      
      daemonSetDetails = dsData || daemonSet;
      
      // Load managed pods
      await loadManagedPods();
    } catch (err) {
      error = String(err);
      daemonSetDetails = daemonSet; // Fallback to passed daemonSet
    } finally {
      loading = false;
    }
  }

  async function loadManagedPods() {
    if (!daemonSet?.metadata?.name || !daemonSet?.metadata?.namespace) return;
    
    podsLoading = true;
    podsError = null;
    
    try {
      const pods = await invoke('kuboard_get_daemonset_pods', {
        name: daemonSet.metadata.name,
        namespace: daemonSet.metadata.namespace
      });
      managedPods = Array.isArray(pods) ? pods : [];
    } catch (err) {
      podsError = String(err);
      managedPods = [];
    } finally {
      podsLoading = false;
    }
  }

  // DaemonSets don't support manual scaling - they scale automatically based on node selectors

  async function restartDaemonSet() {
    if (!daemonSet?.metadata?.name || !daemonSet?.metadata?.namespace) return;
    
    restartLoading = true;
    restartError = null;
    
    try {
      await invoke('kuboard_restart_daemonset', {
        name: daemonSet.metadata.name,
        namespace: daemonSet.metadata.namespace
      });
      
      // Reload details
      await loadDaemonSetDetails();
    } catch (err) {
      restartError = String(err);
      console.error('Failed to restart daemonset:', err);
    } finally {
      restartLoading = false;
    }
  }

  function openActionsMenu(event: MouseEvent) {
    actionsMenuPosition = { x: event.clientX, y: event.clientY };
    actionsMenuVisible = true;
  }

  function handleActionMenuClose() {
    actionsMenuVisible = false;
  }

  function handleAction(event: CustomEvent) {
    const action = event.detail.action;
    const resource = event.detail.resource;

    switch (action) {
      case 'view-yaml':
        openYamlViewer();
        break;
      case 'edit':
        openYamlEditor();
        break;
      case 'copy-name':
        copyToClipboard(daemonSet?.metadata?.name || '');
        break;
      case 'copy-namespace':
        copyToClipboard(daemonSet?.metadata?.namespace || '');
        break;
      default:
        console.log('Action not implemented:', action);
    }
    handleActionMenuClose();
  }

  async function openYamlViewer() {
    if (!daemonSet?.metadata?.name || !daemonSet?.metadata?.namespace) return;
    
    try {
      yamlContent = JSON.stringify(daemonSetDetails || daemonSet, null, 2);
      yamlViewerVisible = true;
    } catch (err) {
      console.error('Failed to load YAML:', err);
    }
  }

  function closeYamlViewer() {
    yamlViewerVisible = false;
    yamlContent = '';
    handleActionMenuClose();
  }

  function openYamlEditor() {
    yamlEditorContent = JSON.stringify(daemonSetDetails || daemonSet, null, 2);
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
    if (!daemonSet?.metadata?.name || !daemonSet?.metadata?.namespace) return;
    
    yamlEditorLoading = true;
    yamlEditorError = null;
    
    try {
      // TODO: Implement daemonset update command
      alert('DaemonSet update not yet implemented. Please use kubectl.');
      closeYamlEditor();
    } catch (error: any) {
      yamlEditorError = String(error);
      console.error('Failed to update daemonset:', error);
    } finally {
      yamlEditorLoading = false;
    }
  }

  onMount(() => {
    loadDaemonSetDetails();
  });
</script>

<div class="full-details-view">
  <div class="details-header">
    <div class="header-left">
      <button class="back-button" onclick={onBack}>← Back to DaemonSets</button>
      <button class="actions-button" onclick={openActionsMenu}>⚙️ Actions</button>
    </div>
    <div class="header-right">
      <h3>{daemonSet?.metadata?.name}</h3>
      <span class="daemonset-namespace">({daemonSet?.metadata?.namespace})</span>
    </div>
  </div>

  {#if loading}
    <div class="loading-message">
      <div class="loading-spinner">⏳</div>
      <p>Loading DaemonSet details...</p>
    </div>
  {:else if error}
    <div class="error-message">
      <div class="error-icon">⚠️</div>
      <div class="error-content">
        <h5>Failed to load DaemonSet details</h5>
        <p>{error}</p>
        <button class="retry-button" onclick={loadDaemonSetDetails}>Retry</button>
      </div>
    </div>
  {:else}
    {@const ds = daemonSetDetails || daemonSet}
    {@const status = getDaemonSetStatus(ds)}
    {@const desired = ds.status?.desiredNumberScheduled || 0}
    {@const ready = ds.status?.numberReady || 0}
    {@const current = ds.status?.currentNumberScheduled || 0}
    {@const available = ds.status?.numberAvailable || 0}
    {@const strategy = getUpdateStrategy(ds)}
    
    <div class="daemonset-details-content">
      <div class="details-section">
        <h6>Basic Information</h6>
        <div class="info-grid">
          <div class="info-item">
            <span class="info-label">Status:</span>
            <div class="info-value-container">
              <span class="info-value status-badge status-{getStatusClass(status)}">{status}</span>
            </div>
          </div>
          <div class="info-item">
            <span class="info-label">Desired Nodes:</span>
            <div class="info-value-container">
              <span class="info-value">{desired}</span>
            </div>
          </div>
          <div class="info-item">
            <span class="info-label">Ready Pods:</span>
            <div class="info-value-container">
              <span class="info-value">{ready}</span>
            </div>
          </div>
          <div class="info-item">
            <span class="info-label">Current Nodes:</span>
            <div class="info-value-container">
              <span class="info-value">{current}</span>
            </div>
          </div>
          <div class="info-item">
            <span class="info-label">Available Pods:</span>
            <div class="info-value-container">
              <span class="info-value">{available}</span>
            </div>
          </div>
          <div class="info-item">
            <span class="info-label">Update Strategy:</span>
            <div class="info-value-container">
              <span class="info-value">{strategy}</span>
            </div>
          </div>
          <div class="info-item">
            <span class="info-label">Age:</span>
            <div class="info-value-container">
              <span class="info-value" title={ds.metadata?.creationTimestamp}>{formatAge(ds.metadata?.creationTimestamp)}</span>
              <button class="copy-button" onclick={() => copyToClipboard(ds.metadata?.creationTimestamp || '')}>📋</button>
            </div>
          </div>
        </div>
      </div>

      <!-- DaemonSets don't support manual scaling - they scale automatically based on node selectors -->

      <div class="details-section">
        <h6>DaemonSet Actions</h6>
        <div class="action-buttons">
          <button class="restart-button" onclick={restartDaemonSet} disabled={restartLoading}>
            {restartLoading ? 'Restarting...' : '🔄 Restart'}
          </button>
          {#if restartError}
            <div class="action-error">Restart: {restartError}</div>
          {/if}
        </div>
      </div>

      <div class="details-section">
        <h6 class="section-header" onclick={() => toggleSection('strategy')} role="button" tabindex="0" onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSection('strategy')}>
          <span>Update Strategy</span>
          <span class="collapse-icon">{sectionsCollapsed.strategy ? '▶' : '▼'}</span>
        </h6>
        {#if !sectionsCollapsed.strategy}
          <div class="info-grid">
            <div class="info-item">
              <span class="info-label">Type:</span>
              <div class="info-value-container">
                <span class="info-value">{strategy}</span>
              </div>
            </div>
            <div class="info-item">
              <span class="info-label">Pod Management Policy:</span>
              <div class="info-value-container">
                <span class="info-value">{policy}</span>
              </div>
            </div>
            {#if strategy === 'RollingUpdate'}
              <div class="info-item">
                <span class="info-label">Partition (for canary updates):</span>
                <div class="info-value-container">
                  <span class="info-value">{ds.spec?.updateStrategy?.rollingUpdate?.partition?.toString() || 'Not set'}</span>
                </div>
              </div>
            {/if}
          </div>
        {/if}
      </div>

      <div class="details-section">
        <h6 class="section-header" onclick={() => toggleSection('conditions')} role="button" tabindex="0" onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSection('conditions')}>
          <span>Conditions</span>
          <span class="collapse-icon">{sectionsCollapsed.conditions ? '▶' : '▼'}</span>
        </h6>
        {#if !sectionsCollapsed.conditions}
          {#if ds.status?.conditions && ds.status.conditions.length > 0}
            <div class="conditions-table">
              <div class="conditions-header">
                <div>Type</div>
                <div>Status</div>
                <div>Reason</div>
                <div>Message</div>
                <div>Last Update</div>
              </div>
              {#each ds.status.conditions as condition}
                <div class="condition-row">
                  <div class="condition-type">{condition.type}</div>
                  <div class="condition-status">
                    <span class="status-badge status-{condition.status === 'True' ? 'ready' : 'failed'}">{condition.status}</span>
                  </div>
                  <div class="condition-reason">{condition.reason || '-'}</div>
                  <div class="condition-message">{condition.message || '-'}</div>
                  <div class="condition-time">{formatAge(condition.lastUpdateTime || condition.lastTransitionTime)}</div>
                </div>
              {/each}
            </div>
          {:else}
            <div class="conditions-placeholder"><p>No conditions available</p></div>
          {/if}
        {/if}
      </div>

      <div class="details-section">
        <h6 class="section-header" onclick={() => toggleSection('pods')} role="button" tabindex="0" onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSection('pods')}>
          <span>Managed Pods ({managedPods.length})</span>
          <span class="collapse-icon">{sectionsCollapsed.pods ? '▶' : '▼'}</span>
        </h6>
        {#if !sectionsCollapsed.pods}
          {#if podsLoading}
            <div class="pods-loading"><div class="loading-spinner">⏳</div><p>Loading pods...</p></div>
          {:else if podsError}
            <div class="pods-error"><div class="error-icon">⚠️</div><p>Failed to load pods: {podsError}</p><button class="retry-button" onclick={loadManagedPods}>Retry</button></div>
          {:else if managedPods.length > 0}
            <div class="pods-table">
              <div class="pods-header">
                <div>Name</div>
                <div>Status</div>
                <div>Node</div>
                <div>Pod IP</div>
                <div>Restarts</div>
                <div>Age</div>
              </div>
              {#each managedPods as pod}
                <div class="pod-row">
                  <div class="pod-name">
                    {pod.metadata?.name || 'Unknown'}
                  </div>
                  <div class="pod-status">
                    <span class="status-badge status-{getPodStatusClass(pod.status?.phase)}">{pod.status?.phase || 'Unknown'}</span>
                  </div>
                  <div class="pod-node">{pod.spec?.nodeName || '-'}</div>
                  <div class="pod-ip">{pod.status?.podIP || '-'}</div>
                  <div class="pod-restarts">
                    {pod.status?.containerStatuses?.[0]?.restartCount || 0}
                  </div>
                  <div class="pod-age">{formatAge(pod.metadata?.creationTimestamp)}</div>
                </div>
              {/each}
            </div>
          {:else}
            <div class="pods-placeholder"><p>No pods managed by this DaemonSet</p></div>
          {/if}
        {/if}
      </div>

      <div class="details-section">
        <h6 class="section-header" onclick={() => toggleSection('volumes')} role="button" tabindex="0" onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSection('volumes')}>
          <span>Node Selector & Tolerations</span>
          <span class="collapse-icon">{sectionsCollapsed.volumes ? '▶' : '▼'}</span>
        </h6>
        {#if !sectionsCollapsed.volumes}
          <div class="node-selector-section">
            <h7>Node Selector:</h7>
            {#if ds.spec?.template?.spec?.nodeSelector && Object.keys(ds.spec.template.spec.nodeSelector).length > 0}
              <div class="selector-list">
                {#each Object.entries(ds.spec.template.spec.nodeSelector) as [key, value]}
                  <div class="selector-item">
                    <span class="selector-key">{key}:</span>
                    <span class="selector-value">{value}</span>
                  </div>
                {/each}
              </div>
            {:else}
              <p class="no-selector">No node selector defined</p>
            {/if}
            
            <h7 style="margin-top: 1rem;">Tolerations:</h7>
            {#if ds.spec?.template?.spec?.tolerations && ds.spec.template.spec.tolerations.length > 0}
              <div class="tolerations-list">
                {#each ds.spec.template.spec.tolerations as tol}
                  <div class="toleration-item">
                    <div class="toleration-info">
                      {#if tol.key}
                        <span class="toleration-key">Key: {tol.key}</span>
                      {/if}
                      {#if tol.operator}
                        <span class="toleration-operator">Operator: {tol.operator}</span>
                      {/if}
                      {#if tol.value}
                        <span class="toleration-value">Value: {tol.value}</span>
                      {/if}
                      {#if tol.effect}
                        <span class="toleration-effect">Effect: {tol.effect}</span>
                      {/if}
                    </div>
                  </div>
                {/each}
              </div>
            {:else}
              <p class="no-toleration">No tolerations defined</p>
            {/if}
          </div>
        {/if}
      </div>

      <div class="details-section">
        <h6 class="section-header" onclick={() => toggleSection('selectors')} role="button" tabindex="0" onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSection('selectors')}>
          <span>Selectors</span>
          <span class="collapse-icon">{sectionsCollapsed.selectors ? '▶' : '▼'}</span>
        </h6>
        {#if !sectionsCollapsed.selectors}
          <div class="info-grid">
            <div class="info-item">
              <span class="info-label">Match Labels:</span>
              <div class="info-value-container">
                {#if ds.spec?.selector?.matchLabels && Object.keys(ds.spec.selector.matchLabels).length > 0}
                  <div class="kv-list">
                    {#each Object.entries(ds.spec.selector.matchLabels) as [k, v]}
                      <div class="kv"><span class="k">{k}</span><span class="v">{v}</span></div>
                    {/each}
                  </div>
                {:else}
                  <span class="info-value">-</span>
                {/if}
              </div>
            </div>
            <div class="info-item">
              <span class="info-label">Match Expressions:</span>
              <div class="info-value-container">
                {#if ds.spec?.selector?.matchExpressions && ds.spec.selector.matchExpressions.length > 0}
                  <div class="match-expressions-list">
                    {#each ds.spec.selector.matchExpressions as expr}
                      <div class="match-expression">
                        <span class="expr-key">{expr.key}</span>
                        <span class="expr-operator">{expr.operator}</span>
                        {#if expr.values}
                          <span class="expr-values">{expr.values.join(', ')}</span>
                        {/if}
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
                  {#if ds.metadata?.labels && Object.keys(ds.metadata.labels).length > 0}
                    {#each Object.entries(ds.metadata.labels) as [k, v]}
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
                  {#if ds.metadata?.annotations && Object.keys(ds.metadata.annotations).length > 0}
                    {#each Object.entries(ds.metadata.annotations) as [k, v]}
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
        <h6 class="section-header" onclick={() => toggleSection('yaml')} role="button" tabindex="0" onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSection('yaml')}>
          <span>YAML</span>
          <span class="collapse-icon">{sectionsCollapsed.yaml ? '▶' : '▼'}</span>
        </h6>
        {#if !sectionsCollapsed.yaml}
          <div class="yaml-section">
            <button class="view-yaml-button" onclick={openYamlViewer}>View YAML</button>
            <button class="edit-yaml-button" onclick={openYamlEditor}>Edit YAML</button>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

{#if actionsMenuVisible}
  <QuickActionsMenu
    x={actionsMenuPosition.x}
    y={actionsMenuPosition.y}
    resource={daemonSet}
    resourceType="daemonset"
    on:action={handleAction}
    on:close={handleActionMenuClose}
  />
{/if}

<!-- YAML Viewer/Editor modals - reuse ServiceDetails styles -->
{#if yamlViewerVisible}
  <div class="yaml-viewer-modal" onclick={(e) => e.target === e.currentTarget && closeYamlViewer()}>
    <div class="yaml-viewer-content" onclick={(e) => e.stopPropagation()}>
      <div class="yaml-viewer-header">
        <h3>DaemonSet YAML: {daemonSet?.metadata?.name}</h3>
        <button class="yaml-viewer-close" onclick={closeYamlViewer}>×</button>
      </div>
      <div class="yaml-viewer-body">
        <pre><code>{yamlContent}</code></pre>
      </div>
    </div>
  </div>
{/if}

{#if yamlEditorVisible}
  <div class="yaml-viewer-modal" onclick={(e) => e.target === e.currentTarget && closeYamlEditor()}>
    <div class="yaml-viewer-content yaml-editor-content" onclick={(e) => e.stopPropagation()}>
      <div class="yaml-viewer-header">
        <h3>Edit DaemonSet YAML: {daemonSet?.metadata?.name}</h3>
        <button class="yaml-viewer-close" onclick={closeYamlEditor}>×</button>
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
        ></textarea>
      </div>
      <div class="yaml-editor-footer">
        <button class="yaml-editor-button yaml-editor-cancel" onclick={closeYamlEditor} disabled={yamlEditorLoading}>
          Cancel
        </button>
        <button class="yaml-editor-button yaml-editor-save" onclick={saveYaml} disabled={yamlEditorLoading}>
          {yamlEditorLoading ? 'Saving...' : 'Save'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  @import '../styles/variables.css';

  /* Reuse ServiceDetails styles - same structure */
  .full-details-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .details-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-md);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.05);
  }

  .header-left {
    display: flex;
    gap: var(--spacing-sm);
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .header-right h3 {
    margin: 0;
    color: var(--text-primary);
    font-size: 1.2rem;
    font-weight: 600;
  }

  .daemonset-namespace {
    color: var(--text-secondary);
    font-size: 0.9rem;
  }

  .back-button, .actions-button {
    padding: 8px 16px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-sm);
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
    cursor: pointer;
    font-size: 0.9rem;
    transition: all 0.2s;
  }

  .back-button:hover, .actions-button:hover {
    background: rgba(255, 255, 255, 0.2);
    border-color: rgba(255, 255, 255, 0.3);
  }

  .daemonset-details-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-md);
  }

  .details-section {
    margin-bottom: var(--spacing-lg);
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
  }

  .details-section h6 {
    margin: 0 0 var(--spacing-md) 0;
    color: var(--text-primary);
    font-size: 1rem;
    font-weight: 600;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .section-header {
    cursor: pointer;
    user-select: none;
  }

  .section-header:hover {
    color: var(--primary-color);
  }

  .collapse-icon {
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  .info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: var(--spacing-md);
  }

  .info-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .info-label {
    color: var(--text-secondary);
    font-size: 0.85rem;
    font-weight: 500;
  }

  .info-value-container {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .info-value {
    color: var(--text-primary);
    font-size: 0.9rem;
  }

  .copy-button {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 2px 4px;
    font-size: 0.8rem;
    transition: color 0.2s;
  }

  .copy-button:hover {
    color: var(--primary-color);
  }

  .status-badge {
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    font-size: 0.8rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .status-ready {
    background: rgba(16, 185, 129, 0.2);
    color: #10b981;
  }

  .status-pending {
    background: rgba(245, 158, 11, 0.2);
    color: #f59e0b;
  }

  .status-failed {
    background: rgba(239, 68, 68, 0.2);
    color: #ef4444;
  }

  .status-running {
    background: rgba(16, 185, 129, 0.2);
    color: #10b981;
  }

  .status-unknown {
    background: rgba(107, 114, 128, 0.2);
    color: #6b7280;
  }

  /* Scale Controls */
  .scale-controls, .scale-display {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .scale-input-group {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .scale-input-group label {
    color: var(--text-secondary);
    font-size: 0.9rem;
  }

  .scale-input {
    width: 100px;
    padding: 8px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-sm);
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
    font-size: 0.9rem;
  }

  .scale-button, .scale-edit-button, .scale-cancel-button {
    padding: 8px 16px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-sm);
    background: var(--primary-color);
    color: white;
    cursor: pointer;
    font-size: 0.9rem;
    transition: all 0.2s;
  }

  .scale-button:hover:not(:disabled), .scale-edit-button:hover {
    background: var(--accent-color);
  }

  .scale-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .scale-cancel-button {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
  }

  .scale-cancel-button:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.2);
  }

  .scale-display {
    flex-direction: row;
    align-items: center;
    gap: var(--spacing-md);
  }

  .scale-current {
    color: var(--text-primary);
    font-weight: 500;
  }

  .scale-error {
    color: var(--error-color);
    font-size: 0.85rem;
    padding: 8px;
    background: rgba(239, 68, 68, 0.1);
    border-radius: var(--radius-sm);
  }

  /* DaemonSet Actions */
  .action-buttons {
    display: flex;
    gap: var(--spacing-sm);
    flex-wrap: wrap;
  }

  .restart-button {
    padding: 8px 16px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-sm);
    background: var(--primary-color);
    color: white;
    cursor: pointer;
    font-size: 0.9rem;
    transition: all 0.2s;
  }

  .restart-button:hover:not(:disabled) {
    background: var(--accent-color);
  }

  .restart-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .action-error {
    color: var(--error-color);
    font-size: 0.85rem;
    padding: 8px;
    background: rgba(239, 68, 68, 0.1);
    border-radius: var(--radius-sm);
    width: 100%;
  }

  /* Pod Ordinal Display */
  .pod-ordinal {
    color: var(--text-secondary);
    font-size: 0.75rem;
    margin-left: 4px;
    font-weight: normal;
  }

  /* Conditions Table */
  .conditions-table {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .conditions-header, .condition-row {
    display: grid;
    grid-template-columns: 1.5fr 1fr 1.5fr 2fr 1fr;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm);
  }

  .conditions-header {
    background: rgba(255, 255, 255, 0.05);
    font-weight: 600;
    color: var(--text-secondary);
    font-size: 0.85rem;
  }

  .condition-row {
    background: rgba(255, 255, 255, 0.02);
    border-radius: var(--radius-sm);
  }

  .condition-type {
    font-weight: 500;
    color: var(--text-primary);
  }

  .condition-status, .condition-reason, .condition-message, .condition-time {
    color: var(--text-secondary);
    font-size: 0.85rem;
  }

  .conditions-placeholder {
    display: flex;
    justify-content: center;
    padding: var(--spacing-md);
    color: var(--text-secondary);
    font-style: italic;
  }

  /* Pods Table */
  .pods-table {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .pods-header, .pod-row {
    display: grid;
    grid-template-columns: 2fr 1fr 1.5fr 1.5fr 1fr 1fr;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm);
  }

  .pods-header {
    background: rgba(255, 255, 255, 0.05);
    font-weight: 600;
    color: var(--text-secondary);
    font-size: 0.85rem;
  }

  .pod-row {
    background: rgba(255, 255, 255, 0.02);
    border-radius: var(--radius-sm);
  }

  .pod-name {
    font-weight: 500;
    color: var(--primary-color);
  }

  .pod-status, .pod-node, .pod-ip, .pod-restarts, .pod-age {
    color: var(--text-secondary);
    font-size: 0.85rem;
  }

  /* KV List */
  .kv-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .kv {
    display: flex;
    gap: var(--spacing-sm);
  }

  .kv .k {
    color: var(--text-secondary);
    font-weight: 500;
    min-width: 100px;
  }

  .kv .v {
    color: var(--text-primary);
  }

  .match-expressions-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .match-expression {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px;
    background: rgba(255, 255, 255, 0.03);
    border-radius: var(--radius-sm);
  }

  .expr-key {
    font-weight: 500;
    color: var(--text-primary);
  }

  .expr-operator {
    color: var(--text-secondary);
    font-size: 0.85rem;
  }

  .expr-values {
    color: var(--text-primary);
    font-size: 0.85rem;
  }

  .yaml-section {
    display: flex;
    gap: var(--spacing-sm);
  }

  .view-yaml-button, .edit-yaml-button {
    padding: 8px 16px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-sm);
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
    cursor: pointer;
    font-size: 0.9rem;
    transition: all 0.2s;
  }

  .view-yaml-button:hover, .edit-yaml-button:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .loading-message, .error-message, .pods-loading, .pods-error, .pods-placeholder {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-md);
  }

  .loading-message, .pods-loading {
    flex-direction: column;
    justify-content: center;
  }

  .error-message, .pods-error {
    flex-direction: row;
    align-items: flex-start;
  }

  .loading-spinner {
    font-size: 2rem;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .error-icon {
    font-size: 1.5rem;
  }

  .retry-button {
    padding: 6px 12px;
    background: var(--error-color);
    color: white;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: 0.85rem;
    margin-top: var(--spacing-sm);
  }

  .retry-button:hover {
    background: #dc2626;
  }

  .pods-placeholder {
    justify-content: center;
    color: var(--text-secondary);
    font-style: italic;
  }

  /* YAML Viewer/Editor - reuse ServiceDetails styles */
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

