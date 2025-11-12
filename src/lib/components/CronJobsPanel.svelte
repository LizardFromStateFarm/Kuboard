<!-- Kuboard CronJobs Panel Component -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import CronJobDetails from './CronJobDetails.svelte';
  import QuickActionsMenu from './QuickActionsMenu.svelte';

  // Props
  export let currentContext: any = null;
  export let cronjobs: any[] = [];

  // State
  let selectedCronJob: any = null;
  let showFullDetails: boolean = false;
  
  // Sorting state
  let sortColumn: string | null = null;
  let sortDirection: 'asc' | 'desc' | null = null;

  // Search state
  let searchQuery: string = '';

  // Quick Actions Menu state
  let contextMenuVisible = false;
  let contextMenuPosition = { x: 0, y: 0 };
  let contextMenuCronJob: any = null;
  
  // YAML Viewer/Editor state
  let yamlViewerVisible = false;
  let yamlContent = '';
  let yamlEditorVisible = false;
  let yamlEditorContent = '';
  let yamlEditorLoading = false;
  let yamlEditorError: string | null = null;

  function handleContextMenu(event: MouseEvent, cj: any) {
    event.preventDefault();
    event.stopPropagation();
    contextMenuCronJob = cj;
    contextMenuPosition = { x: event.clientX, y: event.clientY };
    contextMenuVisible = true;
  }

  function handleActionMenuClose() {
    contextMenuVisible = false;
    contextMenuCronJob = null;
  }

  function handleActionDeleted(event: CustomEvent) {
    // Reload cronjobs would be needed
    handleActionMenuClose();
  }

  function handleActionSuspended(event: CustomEvent) {
    // Reload cronjobs would be needed
    handleActionMenuClose();
  }

  function handleActionResumed(event: CustomEvent) {
    // Reload cronjobs would be needed
    handleActionMenuClose();
  }

  function handleActionTriggered(event: CustomEvent) {
    // Reload cronjobs would be needed
    handleActionMenuClose();
  }

  function handleViewYaml(event: any) {
    console.log('handleViewYaml called in CronJobsPanel', event);
    yamlContent = event.detail?.yaml || '';
    yamlViewerVisible = true;
  }

  function closeYamlViewer() {
    yamlViewerVisible = false;
    yamlContent = '';
    handleActionMenuClose();
  }

  function handleActionEdit(event: any) {
    console.log('handleActionEdit called in CronJobsPanel', event);
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
    if (!contextMenuCronJob?.metadata?.name || !contextMenuCronJob?.metadata?.namespace) return;
    
    yamlEditorLoading = true;
    yamlEditorError = null;
    
    try {
      const parsed = JSON.parse(yamlEditorContent);
      await invoke('kuboard_update_cronjob', {
        name: contextMenuCronJob.metadata.name,
        namespace: contextMenuCronJob.metadata.namespace,
        cronjob: parsed
      });
      closeYamlEditor();
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

  // Get CronJob status
  function getCronJobStatus(cj: any): string {
    const suspended = cj.spec?.suspend || false;
    if (suspended) return 'Suspended';
    
    const active = cj.status?.active?.length || 0;
    const lastScheduleTime = cj.status?.lastScheduleTime;
    const lastSuccessfulTime = cj.status?.lastSuccessfulTime;
    
    if (active > 0) return 'Running';
    if (lastSuccessfulTime) return 'Ready';
    if (lastScheduleTime) return 'Scheduled';
    return 'Unknown';
  }

  function getStatusClass(status: string): string {
    switch (status?.toLowerCase()) {
      case 'ready': return 'ready';
      case 'running': return 'running';
      case 'scheduled': return 'pending';
      case 'suspended': return 'failed';
      default: return 'unknown';
    }
  }

  // Get schedule
  function getSchedule(cj: any): string {
    return cj.spec?.schedule || 'Not set';
  }

  // Get concurrency policy
  function getConcurrencyPolicy(cj: any): string {
    return cj.spec?.concurrencyPolicy || 'Allow';
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

  function compareActiveJobs(a: any, b: any): number {
    const activeA = a.status?.active?.length || 0;
    const activeB = b.status?.active?.length || 0;
    return activeA - activeB;
  }

  function compareAge(a: any, b: any): number {
    const timeA = new Date(a.metadata?.creationTimestamp || 0).getTime();
    const timeB = new Date(b.metadata?.creationTimestamp || 0).getTime();
    return timeA - timeB;
  }

  // Reactive sorted and filtered cronjobs
  $: sortedCronJobs = (() => {
    if (!sortColumn || !sortDirection) {
      return cronjobs;
    }
    
    const sorted = [...cronjobs];
    sorted.sort((a, b) => {
      let comparison = 0;
      
      switch (sortColumn) {
        case 'name':
          comparison = compareName(a, b);
          break;
        case 'namespace':
          comparison = compareNamespace(a, b);
          break;
        case 'active':
          comparison = compareActiveJobs(a, b);
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

  $: filteredCronJobs = (() => {
    if (!searchQuery || !searchQuery.trim()) {
      return sortedCronJobs;
    }

    const query = searchQuery.toLowerCase().trim();
    return sortedCronJobs.filter(cj => {
      const name = (cj.metadata?.name || '').toLowerCase();
      const namespace = (cj.metadata?.namespace || '').toLowerCase();
      
      return name.includes(query) || namespace.includes(query);
    });
  })();

  // Show full details view
  function showFullCronJobDetails(cj: any) {
    selectedCronJob = cj;
    showFullDetails = true;
  }

  // Back to cronjobs list
  function backToCronJobsList() {
    showFullDetails = false;
    selectedCronJob = null;
  }

  // Loading state is managed by parent WorkloadsTab
  // This panel just displays the data it receives
</script>

{#if showFullDetails && selectedCronJob}
  <CronJobDetails cronJob={selectedCronJob} onBack={backToCronJobsList} />
{:else}
  <div class="cronjobs-panel">
    <div class="panel-header">
      <h4>📦 CronJobs ({filteredCronJobs.length})</h4>
      <div class="header-controls">
        <input
          type="text"
          class="search-input"
          placeholder="Search CronJobs..."
          bind:value={searchQuery}
        />
      </div>
    </div>

    {#if cronjobs.length === 0}
      <div class="empty-state">
        <div class="empty-icon">📭</div>
        <h5>No CronJobs Found</h5>
        <p>No CronJobs are currently in this cluster</p>
      </div>
    {:else if filteredCronJobs.length === 0}
      <div class="empty-state">
        <div class="empty-icon">🔍</div>
        <h5>No CronJobs Match</h5>
        <p>No CronJobs match your search query: "{searchQuery}"</p>
      </div>
    {:else}
      <div class="cronjobs-table">
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
          <div class="header-cell sortable" onclick={() => handleSort('active')} role="button" tabindex="0">
            Active Jobs
            {#if sortColumn === 'active'}
              <span class="sort-indicator">{sortDirection === 'asc' ? '↑' : '↓'}</span>
            {/if}
          </div>
          <div class="header-cell">Status</div>
          <div class="header-cell">Schedule</div>
          <div class="header-cell sortable" onclick={() => handleSort('age')} role="button" tabindex="0">
            Age
            {#if sortColumn === 'age'}
              <span class="sort-indicator">{sortDirection === 'asc' ? '↑' : '↓'}</span>
            {/if}
          </div>
          <div class="header-cell">Actions</div>
        </div>

        <div class="table-body">
          {#each filteredCronJobs as cj}
            {@const status = getCronJobStatus(cj)}
            {@const active = cj.status?.active?.length || 0}
            {@const schedule = getSchedule(cj)}
            <div
              class="table-row"
              role="button"
              tabindex="0"
              onclick={() => showFullCronJobDetails(cj)}
              onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && showFullCronJobDetails(cj)}
              oncontextmenu={(e) => handleContextMenu(e, cj)}
            >
              <div class="cell name-cell">
                <span class="resource-name">{cj.metadata?.name || 'Unknown'}</span>
              </div>
              <div class="cell namespace-cell">
                <span>{cj.metadata?.namespace || 'default'}</span>
              </div>
              <div class="cell active-jobs-cell">
                <span class="active-info">{active}</span>
              </div>
              <div class="cell status-cell">
                <span class="status-badge status-{getStatusClass(status)}">{status}</span>
              </div>
              <div class="cell schedule-cell">
                <span class="schedule-info" title={schedule}>{schedule}</span>
              </div>
              <div class="cell age-cell">
                <span>{formatAge(cj.metadata?.creationTimestamp)}</span>
              </div>
              <div class="cell actions-cell" onclick={(e) => { e.stopPropagation(); handleContextMenu(e, cj); }}>
                <button class="action-button" title="Actions">⚙️</button>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>

  <!-- Quick Actions Menu -->
  {#if contextMenuCronJob}
    <QuickActionsMenu
      x={contextMenuPosition.x}
      y={contextMenuPosition.y}
      resource={contextMenuCronJob}
      resourceType="cronjob"
      bind:visible={contextMenuVisible}
      on:close={handleActionMenuClose}
      on:deleted={handleActionDeleted}
      on:suspended={handleActionSuspended}
      on:resumed={handleActionResumed}
      on:triggered={handleActionTriggered}
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
          <h3>CronJob YAML: {contextMenuCronJob?.metadata?.name}</h3>
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
          <h3>Edit CronJob YAML</h3>
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

  .cronjobs-panel {
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

  .panel-header h4 {
    margin: 0;
    color: var(--text-primary);
    font-size: 1.1rem;
    font-weight: 600;
  }

  .header-controls {
    display: flex;
    gap: var(--spacing-sm);
    align-items: center;
  }

  .search-input {
    padding: 8px 12px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-sm);
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
    font-size: 0.9rem;
    width: 250px;
    transition: all 0.2s;
  }

  .search-input:focus {
    outline: none;
    border-color: var(--primary-color);
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
  }

  .loading-message, .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-xl);
    gap: var(--spacing-md);
    color: var(--text-secondary);
  }

  .loading-spinner {
    font-size: 2rem;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .empty-icon {
    font-size: 3rem;
    opacity: 0.7;
  }

  .empty-state h5 {
    margin: 0;
    color: var(--text-primary);
    font-size: 1.1rem;
  }

  .empty-state p {
    margin: 0;
    font-size: 0.9rem;
  }

  .cronjobs-table {
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

  /* YAML Viewer/Editor Styles */
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

