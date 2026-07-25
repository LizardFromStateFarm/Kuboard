<!-- Kuboard Deployments Panel Component -->
<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import ResourceTable from './ResourceTable.svelte';
  import DeploymentDetails from './DeploymentDetails.svelte';
  import QuickActionsMenu from './QuickActionsMenu.svelte';
  
  const dispatch = createEventDispatcher();

  // Props
  export let currentContext: any = null;
  export let deployments: any[] = [];
  export let initialSelectedName: string | null = null;

  // State
  let selectedDeployment: any = null;
  let showFullDetails: boolean = false;

  let lastProcessedInitialName: string | null = null;

  $: if (initialSelectedName && initialSelectedName !== lastProcessedInitialName) {
    lastProcessedInitialName = initialSelectedName;
    const found = deployments?.find((d: any) => d.metadata?.name === initialSelectedName);
    selectedDeployment = found || { metadata: { name: initialSelectedName, namespace: currentContext?.namespace || 'default' } };
    showFullDetails = true;
  }
  
  // Sorting state
  let sortColumn: string | null = null;
  let sortDirection: 'asc' | 'desc' | null = null;

  // Search state
  let searchQuery: string = '';

  // Quick Actions Menu state
  let contextMenuVisible = false;
  let contextMenuPosition = { x: 0, y: 0 };
  let contextMenuDeployment: any = null;
  
  // YAML Viewer/Editor state
  let yamlViewerVisible = false;
  let yamlContent = '';
  let yamlEditorVisible = false;
  let yamlEditorContent = '';
  let yamlEditorLoading = false;
  let yamlEditorError: string | null = null;
  
  // Auto-refresh state
  let refreshInterval: any = null;
  let lastRestartTime: number | null = null;

  // Watch-based live update state
  let liveDeployments: any[] | null = null;
  let watchError: string | null = null;
  let watchActive = false;
  let deploymentsMap = new Map<string, any>(); // Track deployments by key for efficient updates

  function handleContextMenu(event: MouseEvent, dep: any) {
    event.preventDefault();
    event.stopPropagation();
    contextMenuDeployment = dep;
    contextMenuPosition = { x: event.clientX, y: event.clientY };
    contextMenuVisible = true;
  }

  function handleActionMenuClose() {
    contextMenuVisible = false;
    contextMenuDeployment = null;
  }

  function handleActionDeleted(event: CustomEvent) {
    // Reload deployments would be needed
    handleActionMenuClose();
  }

  function handleActionRestarted(event: CustomEvent) {
    console.log('handleActionRestarted called in DeploymentsPanel', event);
    console.log('Event detail:', event.detail);
    // Reload deployments
    console.log('Dispatching reload event');
    dispatch('reload');
    handleActionMenuClose();
    
    // Start auto-refresh for 30 seconds after restart
    lastRestartTime = Date.now();
    startAutoRefresh();
  }
  
  function startAutoRefresh() {
    // Clear any existing interval
    if (refreshInterval) {
      clearInterval(refreshInterval);
    }
    
    // Refresh every 2 seconds for 30 seconds after restart
    refreshInterval = setInterval(() => {
      const now = Date.now();
      if (lastRestartTime && (now - lastRestartTime) < 30000) {
        console.log('Auto-refreshing deployments after restart');
        dispatch('reload');
      } else {
        // Stop refreshing after 30 seconds
        stopAutoRefresh();
      }
    }, 2000);
  }
  
  function stopAutoRefresh() {
    if (refreshInterval) {
      clearInterval(refreshInterval);
      refreshInterval = null;
    }
    lastRestartTime = null;
  }
  
  // Get deployment key for tracking
  function getDeploymentKey(deployment: any): string {
    const namespace = deployment.metadata?.namespace || 'default';
    const name = deployment.metadata?.name || 'unknown';
    return `${namespace}/${name}`;
  }

  // Handle watch events
  function handleWatchEvent(event: any) {
    console.log('📡 Deployment watch event received:', event);
    const { event_type, deployment } = event;
    
    if (!deployment || !deployment.metadata) {
      console.error('⚠️ Invalid watch event: missing deployment or metadata', event);
      return;
    }
    
    const key = getDeploymentKey(deployment);
    const eventTypeStr = String(event_type);
    
    console.log(`📡 Watch event: ${eventTypeStr} for deployment ${key}`);
    
    switch (eventTypeStr) {
      case 'Added':
        if (!deploymentsMap.has(key)) {
          deploymentsMap.set(key, deployment);
          liveDeployments = [...Array.from(deploymentsMap.values())];
          console.log(`✅ Added deployment: ${key}, total: ${liveDeployments.length}`);
        }
        break;
        
      case 'Modified':
        deploymentsMap.set(key, deployment);
        liveDeployments = [...Array.from(deploymentsMap.values())];
        console.log(`🔄 Modified deployment: ${key}, total: ${liveDeployments.length}`);
        break;
        
      case 'Deleted':
        if (deploymentsMap.has(key)) {
          deploymentsMap.delete(key);
          liveDeployments = [...Array.from(deploymentsMap.values())];
          console.log(`🗑️ Deleted deployment: ${key}, total: ${liveDeployments.length}`);
        }
        break;
    }
    
    watchError = null;
  }

  function handleWatchError(error: any) {
    console.error('Deployment watch error:', error);
    watchError = error?.error || String(error) || 'Watch connection error';
  }

  // Start watch stream
  async function startWatch() {
    if (!currentContext) return;
    
    try {
      await invoke('kuboard_stop_deployment_watch'); // Stop any existing watch
      await invoke('kuboard_start_deployment_watch');
      watchActive = true;
      watchError = null;
      console.log('✅ Deployment watch started');
    } catch (e: any) {
      console.error('Failed to start deployment watch:', e);
      watchError = String(e);
      watchActive = false;
    }
  }

  // Stop watch stream
  async function stopWatch() {
    try {
      await invoke('kuboard_stop_deployment_watch');
      watchActive = false;
      console.log('🛑 Deployment watch stopped');
    } catch (e: any) {
      console.error('Failed to stop deployment watch:', e);
    }
  }

  // Initialize deployments from initial list
  function initializeDeployments() {
    if (deployments && deployments.length > 0) {
      deploymentsMap.clear();
      for (const deployment of deployments) {
        deploymentsMap.set(getDeploymentKey(deployment), deployment);
      }
      liveDeployments = [...Array.from(deploymentsMap.values())];
      console.log(`📋 Initialized ${liveDeployments.length} deployments`);
    }
  }

  // Lifecycle
  let watchEventListenerUnsubscribe: (() => Promise<void>) | null = null;
  let watchErrorListenerUnsubscribe: (() => Promise<void>) | null = null;
  let lastContext: string | null = null;

  onMount(async () => {
    // Initialize deployments from props
    initializeDeployments();
    
    // Listen to watch events
    const { listen } = await import('@tauri-apps/api/event');
    
    watchEventListenerUnsubscribe = await listen('deployment-watch-event', (event: any) => {
      handleWatchEvent(event.payload);
    });
    
    watchErrorListenerUnsubscribe = await listen('deployment-watch-error', (event: any) => {
      handleWatchError(event.payload);
    });
    
    // Start watch when context is available
    if (currentContext && !watchActive) {
      startWatch();
    }
    
    return async () => {
      if (watchEventListenerUnsubscribe) await watchEventListenerUnsubscribe();
      if (watchErrorListenerUnsubscribe) await watchErrorListenerUnsubscribe();
      await stopWatch();
    };
  });

  onDestroy(async () => {
    stopAutoRefresh();
    if (watchEventListenerUnsubscribe) await watchEventListenerUnsubscribe();
    if (watchErrorListenerUnsubscribe) await watchErrorListenerUnsubscribe();
    await stopWatch();
  });

  // Restart watch only when context actually changes
  $: if (currentContext && currentContext !== lastContext) {
    lastContext = currentContext;
    if (watchActive) {
      stopWatch().then(() => {
        deploymentsMap.clear();
        liveDeployments = [];
        initializeDeployments();
        startWatch();
      });
    } else {
      deploymentsMap.clear();
      liveDeployments = [];
      initializeDeployments();
      startWatch();
    }
  }

  function handleViewYaml(event: any) {
    console.log('handleViewYaml called in DeploymentsPanel', event);
    yamlContent = event.detail?.yaml || '';
    yamlViewerVisible = true;
  }

  function closeYamlViewer() {
    yamlViewerVisible = false;
    yamlContent = '';
    handleActionMenuClose();
  }

  function handleActionEdit(event: any) {
    console.log('handleActionEdit called in DeploymentsPanel', event);
    yamlEditorContent = event.detail?.yaml || '';
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
    if (!contextMenuDeployment?.metadata?.name || !contextMenuDeployment?.metadata?.namespace) return;
    
    yamlEditorLoading = true;
    yamlEditorError = null;
    
    try {
      // Parse and validate YAML
      const parsed = JSON.parse(yamlEditorContent);
      
      // Update the deployment using the Kubernetes API
      await invoke('kuboard_update_deployment', {
        name: contextMenuDeployment.metadata.name,
        namespace: contextMenuDeployment.metadata.namespace,
        deployment: parsed
      });
      
      closeYamlEditor();
      // Reload deployments would be needed
    } catch (err: any) {
      yamlEditorError = err?.toString() || 'Failed to save YAML';
      console.error('Failed to save YAML:', err);
    } finally {
      yamlEditorLoading = false;
    }
  }

  function handleActionCopied(event: CustomEvent) {
    console.log('Copied:', event.detail.type, event.detail.value);
    handleActionMenuClose();
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

  // Get deployment status
  function getDeploymentStatus(dep: any): string {
    const available = dep.status?.conditions?.find((c: any) => c.type === 'Available');
    const progressing = dep.status?.conditions?.find((c: any) => c.type === 'Progressing');
    
    if (available?.status === 'True' && progressing?.status === 'True') {
      return 'Available';
    }
    if (progressing?.status === 'True') {
      return 'Progressing';
    }
    if (available?.status === 'False') {
      return 'Not Available';
    }
    return 'Unknown';
  }

  function getStatusClass(status: string): string {
    switch (status?.toLowerCase()) {
      case 'available': return 'ready';
      case 'progressing': return 'pending';
      case 'not available': return 'failed';
      default: return 'unknown';
    }
  }

  // Get update strategy
  function getUpdateStrategy(dep: any): string {
    const strategy = dep.spec?.strategy?.type || 'RollingUpdate';
    return strategy;
  }

  // Sorting functions
  function handleSort(column: string, event?: Event) {
    if (event) {
      event.stopPropagation();
      event.preventDefault();
    }
    
    if (sortColumn === column) {
      if (sortDirection === 'asc') {
        sortDirection = 'desc';
      } else if (sortDirection === 'desc') {
        sortColumn = null;
        sortDirection = null;
      }
    } else {
      sortColumn = column;
      sortDirection = 'asc';
    }
  }

  // Comparison functions
  function compareName(a: any, b: any): number {
    const nameA = a.metadata?.name || '';
    const nameB = b.metadata?.name || '';
    return nameA.localeCompare(nameB);
  }

  function compareNamespace(a: any, b: any): number {
    const nsA = a.metadata?.namespace || '';
    const nsB = b.metadata?.namespace || '';
    return nsA.localeCompare(nsB);
  }

  function compareReplicas(a: any, b: any): number {
    const readyA = a.status?.readyReplicas || 0;
    const readyB = b.status?.readyReplicas || 0;
    return readyA - readyB;
  }

  function compareAge(a: any, b: any): number {
    const timeA = new Date(a.metadata?.creationTimestamp || 0).getTime();
    const timeB = new Date(b.metadata?.creationTimestamp || 0).getTime();
    return timeA - timeB;
  }

  function getRenderDeployments(): any[] {
    if (liveDeployments !== null && liveDeployments.length > 0) {
      return liveDeployments;
    }
    return deployments ?? [];
  }

  // Reactive: Initialize deployments when props change
  $: if (deployments) {
    initializeDeployments();
  }

  // Reactive sorted and filtered deployments
  $: sortedDeployments = (() => {
    const deps = (liveDeployments && liveDeployments.length > 0) ? liveDeployments : (deployments || []);
    if (!sortColumn || !sortDirection) {
      return deps;
    }
    
    const sorted = [...deps];
    sorted.sort((a, b) => {
      let comparison = 0;
      
      switch (sortColumn) {
        case 'name':
          comparison = compareName(a, b);
          break;
        case 'namespace':
          comparison = compareNamespace(a, b);
          break;
        case 'replicas':
          comparison = compareReplicas(a, b);
          break;
        case 'age':
          comparison = compareAge(a, b);
          break;
        default:
          return 0;
      }
      
      return sortDirection === 'asc' ? comparison : -comparison;
    });
    
    return sorted;
  })();

  $: filteredDeployments = (() => {
    if (!searchQuery || !searchQuery.trim()) {
      return sortedDeployments;
    }

    const query = searchQuery.toLowerCase().trim();
    return sortedDeployments.filter(dep => {
      const name = (dep.metadata?.name || '').toLowerCase();
      const namespace = (dep.metadata?.namespace || '').toLowerCase();
      
      return name.includes(query) || namespace.includes(query);
    });
  })();

  // Show full details view
  function showFullDeploymentDetails(dep: any) {
    selectedDeployment = dep;
    showFullDetails = true;
  }

  // Back to deployments list
  function backToDeploymentsList() {
    initialSelectedName = null;
    lastProcessedInitialName = null;
    showFullDetails = false;
    selectedDeployment = null;
  }

  // Loading state is managed by parent WorkloadsTab
  // This panel just displays the data it receives
</script>

{#if showFullDetails && selectedDeployment}
  <DeploymentDetails deployment={selectedDeployment} onBack={backToDeploymentsList} currentContext={currentContext} on:navigateToWorkload />
{:else}
  <div class="deployments-panel">
    <div class="panel-header">
      <div class="panel-controls">
        <span class="live-indicator {watchError ? 'error' : watchActive ? 'active' : ''}">
          {watchError ? 'Watch Error' : watchActive ? '🟢 Live' : '⏸️ Paused'}
        </span>
      </div>
    </div>
      <ResourceTable
        items={getRenderDeployments()}
        filteredItems={filteredDeployments}
        bind:searchQuery
        searchPlaceholder="Search Deployments..."
        noItemsMessage="No Deployments are currently in this cluster"
        noSearchResultsMessage="No Deployments match your search query:"
      >
        <svelte:fragment slot="table">      <div class="deployments-table">
        <div class="table-header">
          <div class="header-cell sortable" onclick={() => handleSort('name')} role="button" tabindex="0">
            Name
            {#if sortColumn === 'name'}
              <span class="sort-indicator">{sortDirection === 'asc' ? '↑' : '↓'}</span>
            {/if}
          </div>
          <div class="header-cell sortable" onclick={() => handleSort('namespace')} role="button" tabindex="0">
            Namespace
            {#if sortColumn === 'namespace'}
              <span class="sort-indicator">{sortDirection === 'asc' ? '↑' : '↓'}</span>
            {/if}
          </div>
          <div class="header-cell sortable" onclick={() => handleSort('replicas')} role="button" tabindex="0">
            Replicas
            {#if sortColumn === 'replicas'}
              <span class="sort-indicator">{sortDirection === 'asc' ? '↑' : '↓'}</span>
            {/if}
          </div>
          <div class="header-cell">Status</div>
          <div class="header-cell">Strategy</div>
          <div class="header-cell sortable" onclick={() => handleSort('age')} role="button" tabindex="0">
            Age
            {#if sortColumn === 'age'}
              <span class="sort-indicator">{sortDirection === 'asc' ? '↑' : '↓'}</span>
            {/if}
          </div>
          <div class="header-cell">Actions</div>
        </div>

        <div class="table-body">
          {#each filteredDeployments as dep}
            {@const status = getDeploymentStatus(dep)}
            {@const desired = dep.spec?.replicas || 0}
            {@const ready = dep.status?.readyReplicas || 0}
            {@const current = dep.status?.replicas || 0}
            {@const available = dep.status?.availableReplicas || 0}
            <div
              class="table-row"
              role="button"
              tabindex="0"
              onclick={() => showFullDeploymentDetails(dep)}
              onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && showFullDeploymentDetails(dep)}
              oncontextmenu={(e) => handleContextMenu(e, dep)}
            >
              <div class="cell name-cell">
                <span class="resource-name">{dep.metadata?.name || 'Unknown'}</span>
              </div>
              <div class="cell namespace-cell">
                <span>{dep.metadata?.namespace || 'default'}</span>
              </div>
              <div class="cell replicas-cell">
                <span class="replica-info">{ready}/{desired}</span>
                {#if available !== ready}
                  <span class="replica-warning">({available} available)</span>
                {/if}
                {#if current !== desired}
                  <span class="replica-warning">({current} current)</span>
                {/if}
              </div>
              <div class="cell status-cell">
                <span class="status-badge status-{getStatusClass(status)}">{status}</span>
              </div>
              <div class="cell strategy-cell">
                <span>{getUpdateStrategy(dep)}</span>
              </div>
              <div class="cell age-cell">
                <span>{formatAge(dep.metadata?.creationTimestamp)}</span>
              </div>
              <div class="cell actions-cell" onclick={(e) => { e.stopPropagation(); handleContextMenu(e, dep); }}>
                <button class="action-button" title="Actions">⚙️</button>
              </div>
            </div>
          {/each}
        </div>
      </div>
        </svelte:fragment>
      </ResourceTable>
  </div>

  <!-- Quick Actions Menu -->
  {#if contextMenuDeployment}
    <QuickActionsMenu
      x={contextMenuPosition.x}
      y={contextMenuPosition.y}
      resource={contextMenuDeployment}
      resourceType="deployment"
      bind:visible={contextMenuVisible}
      on:close={handleActionMenuClose}
      on:deleted={handleActionDeleted}
      on:restarted={handleActionRestarted}
      on:view-yaml={handleViewYaml}
      on:edit={handleActionEdit}
      on:copied={handleActionCopied}
    />
  {/if}

  <!-- YAML Viewer Modal -->
  {#if yamlViewerVisible}
    <div class="yaml-viewer-modal" onclick={closeYamlViewer}>
      <div class="yaml-viewer-content" onclick={(e) => e.stopPropagation()}>
        <div class="yaml-viewer-header">
          <h3>Deployment YAML: {contextMenuDeployment?.metadata?.name}</h3>
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
          <h3>Edit Deployment YAML</h3>
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
            {yamlEditorLoading ? 'Saving...' : 'Save'}
          </button>
        </div>
      </div>
    </div>
  {/if}
{/if}

<style>
  @import '../styles/variables.css';

  .deployments-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-md);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.05);
  }

  .panel-controls {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .live-indicator {
    font-size: 0.85rem;
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid rgba(34, 197, 94, 0.3);
    background: rgba(34, 197, 94, 0.12);
    color: #22c55e;
  }

  .live-indicator.active {
    border-color: rgba(34, 197, 94, 0.3);
    background: rgba(34, 197, 94, 0.12);
    color: #22c55e;
  }

  .live-indicator.error {
    border-color: rgba(239, 68, 68, 0.3);
    background: rgba(239, 68, 68, 0.12);
    color: #ef4444;
  }

  .panel-header h4 {
    margin: 0;
    color: var(--text-primary);
    font-size: 1.1rem;
    font-weight: 600;
  }

  .deployments-table {
    flex: 1;
    overflow: auto;
    display: flex;
    flex-direction: column;
  }

  .table-header {
    display: grid;
    grid-template-columns: 2fr 1.5fr 1fr 1fr 1.2fr 1fr 80px;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    background: rgba(255, 255, 255, 0.05);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    position: sticky;
    top: 0;
    z-index: 10;
  }

  .header-cell {
    color: var(--text-secondary);
    font-size: 0.85rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .header-cell.sortable {
    cursor: pointer;
    user-select: none;
    transition: color 0.2s;
  }

  .header-cell.sortable:hover {
    color: var(--primary-color);
  }

  .sort-indicator {
    color: var(--primary-color);
    font-size: 0.8rem;
  }

  .table-body {
    display: flex;
    flex-direction: column;
  }

  .table-row {
    display: grid;
    grid-template-columns: 2fr 1.5fr 1fr 1fr 1.2fr 1fr 80px;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    cursor: pointer;
    transition: background 0.2s;
  }

  .table-row:hover {
    background: rgba(255, 255, 255, 0.03);
  }

  .table-row:focus {
    outline: 2px solid var(--primary-color);
    outline-offset: -2px;
  }

  .cell {
    display: flex;
    align-items: center;
    color: var(--text-primary);
    font-size: 0.9rem;
  }

  .name-cell .resource-name {
    font-weight: 600;
    color: var(--primary-color);
  }

  .namespace-cell {
    color: var(--text-secondary);
  }

  .replicas-cell {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .replica-info {
    font-weight: 500;
  }

  .replica-warning {
    font-size: 0.75rem;
    color: var(--warning-color);
  }

  .status-badge {
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    font-size: 0.75rem;
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

  .status-unknown {
    background: rgba(107, 114, 128, 0.2);
    color: #6b7280;
  }

  .strategy-cell {
    color: var(--text-secondary);
    font-size: 0.85rem;
  }

  .age-cell {
    color: var(--text-secondary);
    font-size: 0.85rem;
  }

  .actions-cell {
    display: flex;
    justify-content: center;
  }

  .action-button {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    transition: all 0.2s;
    font-size: 0.9rem;
  }

  .action-button:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
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
    background: var(--primary-color-hover);
  }
</style>

