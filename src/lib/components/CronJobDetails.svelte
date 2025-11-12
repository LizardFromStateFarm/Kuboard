<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import QuickActionsMenu from './QuickActionsMenu.svelte';

  export let cronJob: any;
  export let onBack: () => void;

  let cronJobDetails: any = null;
  let managedJobs: any[] = [];
  let loading = false;
  let error: string | null = null;
  let jobsLoading = false;
  let jobsError: string | null = null;
  let suspendLoading = false;
  let suspendError: string | null = null;
  let resumeLoading = false;
  let resumeError: string | null = null;
  let triggerLoading = false;
  let triggerError: string | null = null;

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
    jobs: false,
    schedule: true,
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

  function getSchedule(cj: any): string {
    return cj.spec?.schedule || 'Not set';
  }

  function getConcurrencyPolicy(cj: any): string {
    return cj.spec?.concurrencyPolicy || 'Allow';
  }

  function getSuspendState(cj: any): boolean {
    return cj.spec?.suspend || false;
  }

  function formatNextScheduleTime(cj: any): string {
    // Calculate next schedule time from cron expression
    // For now, just show if scheduled
    if (cj.status?.lastScheduleTime) {
      return 'Scheduled';
    }
    return 'Not scheduled';
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

  async function loadCronJobDetails() {
    if (!cronJob?.metadata?.name || !cronJob?.metadata?.namespace) return;
    
    loading = true;
    error = null;
    
    try {
      const cjData = await invoke('kuboard_get_cronjob', {
        name: cronJob.metadata.name,
        namespace: cronJob.metadata.namespace
      }).catch(() => null);
      
      cronJobDetails = cjData || cronJob;
      
      // Load managed jobs
      await loadManagedJobs();
    } catch (err) {
      error = String(err);
      cronJobDetails = cronJob; // Fallback to passed cronJob
    } finally {
      loading = false;
    }
  }

  async function loadManagedJobs() {
    if (!cronJob?.metadata?.name || !cronJob?.metadata?.namespace) return;
    
    jobsLoading = true;
    jobsError = null;
    
    try {
      const jobs = await invoke('kuboard_get_cronjob_jobs', {
        name: cronJob.metadata.name,
        namespace: cronJob.metadata.namespace
      });
      managedJobs = Array.isArray(jobs) ? jobs : [];
    } catch (err) {
      jobsError = String(err);
      managedJobs = [];
    } finally {
      jobsLoading = false;
    }
  }

  async function suspendCronJob() {
    if (!cronJob?.metadata?.name || !cronJob?.metadata?.namespace) return;
    
    suspendLoading = true;
    suspendError = null;
    
    try {
      await invoke('kuboard_suspend_cronjob', {
        name: cronJob.metadata.name,
        namespace: cronJob.metadata.namespace
      });
      
      // Reload details
      await loadCronJobDetails();
    } catch (err) {
      suspendError = String(err);
      console.error('Failed to suspend cronjob:', err);
    } finally {
      suspendLoading = false;
    }
  }

  async function resumeCronJob() {
    if (!cronJob?.metadata?.name || !cronJob?.metadata?.namespace) return;
    
    resumeLoading = true;
    resumeError = null;
    
    try {
      await invoke('kuboard_resume_cronjob', {
        name: cronJob.metadata.name,
        namespace: cronJob.metadata.namespace
      });
      
      // Reload details
      await loadCronJobDetails();
    } catch (err) {
      resumeError = String(err);
      console.error('Failed to resume cronjob:', err);
    } finally {
      resumeLoading = false;
    }
  }

  async function triggerCronJob() {
    if (!cronJob?.metadata?.name || !cronJob?.metadata?.namespace) return;
    
    triggerLoading = true;
    triggerError = null;
    
    try {
      await invoke('kuboard_trigger_cronjob', {
        name: cronJob.metadata.name,
        namespace: cronJob.metadata.namespace
      });
      
      // Reload details to show the new job
      await loadCronJobDetails();
    } catch (err) {
      triggerError = String(err);
      console.error('Failed to trigger cronjob:', err);
    } finally {
      triggerLoading = false;
    }
  }

  function openActionsMenu(event: MouseEvent) {
    actionsMenuPosition = { x: event.clientX, y: event.clientY };
    actionsMenuVisible = true;
  }

  function handleActionMenuClose() {
    actionsMenuVisible = false;
  }

  function handleActionDeleted(event: CustomEvent) {
    handleActionMenuClose();
    onBack();
  }

  function handleActionSuspended(event: CustomEvent) {
    handleActionMenuClose();
    loadCronJobDetails();
  }

  function handleActionResumed(event: CustomEvent) {
    handleActionMenuClose();
    loadCronJobDetails();
  }

  function handleActionTriggered(event: CustomEvent) {
    handleActionMenuClose();
    loadCronJobDetails();
  }

  function handleViewYaml(event: CustomEvent) {
    console.log('handleViewYaml called', event.detail);
    yamlContent = event.detail.yaml;
    yamlViewerVisible = true;
    handleActionMenuClose();
  }

  function handleActionEdit(event: CustomEvent) {
    console.log('handleActionEdit called', event.detail);
    yamlEditorContent = event.detail.yaml;
    yamlEditorVisible = true;
    yamlEditorError = null;
    handleActionMenuClose();
  }

  function handleActionCopied(event: CustomEvent) {
    console.log('Copied:', event.detail.type, event.detail.value);
  }

  async function openYamlViewer() {
    if (!cronJob?.metadata?.name || !cronJob?.metadata?.namespace) return;
    
    try {
      yamlContent = JSON.stringify(cronJobDetails || cronJob, null, 2);
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
    yamlEditorContent = JSON.stringify(cronJobDetails || cronJob, null, 2);
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
    if (!cronJob?.metadata?.name || !cronJob?.metadata?.namespace) return;
    
    yamlEditorLoading = true;
    yamlEditorError = null;
    
    try {
      // TODO: Implement cronjob update command
      alert('CronJob update not yet implemented. Please use kubectl.');
      closeYamlEditor();
    } catch (error: any) {
      yamlEditorError = String(error);
      console.error('Failed to update cronjob:', error);
    } finally {
      yamlEditorLoading = false;
    }
  }

  onMount(() => {
    loadCronJobDetails();
  });
</script>

<div class="full-details-view">
  <div class="details-header">
    <div class="header-left">
      <button class="back-button" onclick={onBack}>← Back to CronJobs</button>
      <button class="actions-button" onclick={openActionsMenu}>⚙️ Actions</button>
    </div>
    <div class="header-right">
      <h3>{cronJob?.metadata?.name}</h3>
      <span class="cronjob-namespace">({cronJob?.metadata?.namespace})</span>
    </div>
  </div>

  {#if loading}
    <div class="loading-message">
      <div class="loading-spinner">⏳</div>
      <p>Loading CronJob details...</p>
    </div>
  {:else if error}
    <div class="error-message">
      <div class="error-icon">⚠️</div>
      <div class="error-content">
        <h5>Failed to load CronJob details</h5>
        <p>{error}</p>
        <button class="retry-button" onclick={loadCronJobDetails}>Retry</button>
      </div>
    </div>
  {:else}
    {@const cj = cronJobDetails || cronJob}
    {@const status = getCronJobStatus(cj)}
    {@const schedule = getSchedule(cj)}
    {@const suspended = getSuspendState(cj)}
    {@const activeJobs = cj.status?.active?.length || 0}
    {@const lastScheduleTime = cj.status?.lastScheduleTime}
    {@const lastSuccessfulTime = cj.status?.lastSuccessfulTime}
    {@const concurrencyPolicy = getConcurrencyPolicy(cj)}
    
    <div class="cronjob-details-content">
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
            <span class="info-label">Schedule:</span>
            <div class="info-value-container">
              <span class="info-value">{schedule}</span>
            </div>
          </div>
          <div class="info-item">
            <span class="info-label">Suspended:</span>
            <div class="info-value-container">
              <span class="info-value">{suspended ? 'Yes' : 'No'}</span>
            </div>
          </div>
          <div class="info-item">
            <span class="info-label">Active Jobs:</span>
            <div class="info-value-container">
              <span class="info-value">{activeJobs}</span>
            </div>
          </div>
          <div class="info-item">
            <span class="info-label">Concurrency Policy:</span>
            <div class="info-value-container">
              <span class="info-value">{concurrencyPolicy}</span>
            </div>
          </div>
          <div class="info-item">
            <span class="info-label">Last Scheduled:</span>
            <div class="info-value-container">
              <span class="info-value" title={lastScheduleTime}>{lastScheduleTime ? formatAge(lastScheduleTime) : 'Never'}</span>
            </div>
          </div>
          <div class="info-item">
            <span class="info-label">Last Successful:</span>
            <div class="info-value-container">
              <span class="info-value" title={lastSuccessfulTime}>{lastSuccessfulTime ? formatAge(lastSuccessfulTime) : 'Never'}</span>
            </div>
          </div>
          <div class="info-item">
            <span class="info-label">Age:</span>
            <div class="info-value-container">
              <span class="info-value" title={cj.metadata?.creationTimestamp}>{formatAge(cj.metadata?.creationTimestamp)}</span>
              <button class="copy-button" onclick={() => copyToClipboard(cj.metadata?.creationTimestamp || '')}>📋</button>
            </div>
          </div>
        </div>
      </div>

      <div class="details-section">
        <h6>CronJob Actions</h6>
        <div class="action-buttons">
          {#if suspended}
            <button class="resume-button" onclick={resumeCronJob} disabled={resumeLoading}>
              {resumeLoading ? 'Resuming...' : '▶ Resume'}
            </button>
            {#if resumeError}
              <div class="action-error">Resume: {resumeError}</div>
            {/if}
          {:else}
            <button class="suspend-button" onclick={suspendCronJob} disabled={suspendLoading}>
              {suspendLoading ? 'Suspending...' : '⏸ Suspend'}
            </button>
            {#if suspendError}
              <div class="action-error">Suspend: {suspendError}</div>
            {/if}
          {/if}
          <button class="trigger-button" onclick={triggerCronJob} disabled={triggerLoading || suspended}>
            {triggerLoading ? 'Triggering...' : '▶ Trigger Now'}
          </button>
          {#if triggerError}
            <div class="action-error">Trigger: {triggerError}</div>
          {/if}
        </div>
      </div>

      <div class="details-section">
        <h6 class="section-header" onclick={() => toggleSection('schedule')} role="button" tabindex="0" onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSection('schedule')}>
          <span>Schedule Configuration</span>
          <span class="collapse-icon">{sectionsCollapsed.schedule ? '▶' : '▼'}</span>
        </h6>
        {#if !sectionsCollapsed.schedule}
          <div class="info-grid">
            <div class="info-item">
              <span class="info-label">Schedule (Cron):</span>
              <div class="info-value-container">
                <span class="info-value">{schedule}</span>
              </div>
            </div>
            <div class="info-item">
              <span class="info-label">Concurrency Policy:</span>
              <div class="info-value-container">
                <span class="info-value">{concurrencyPolicy}</span>
              </div>
            </div>
            <div class="info-item">
              <span class="info-label">Starting Deadline Seconds:</span>
              <div class="info-value-container">
                <span class="info-value">{cj.spec?.startingDeadlineSeconds?.toString() || 'Not set'}</span>
              </div>
            </div>
            <div class="info-item">
              <span class="info-label">Successful Jobs History Limit:</span>
              <div class="info-value-container">
                <span class="info-value">{cj.spec?.successfulJobsHistoryLimit?.toString() || '3'}</span>
              </div>
            </div>
            <div class="info-item">
              <span class="info-label">Failed Jobs History Limit:</span>
              <div class="info-value-container">
                <span class="info-value">{cj.spec?.failedJobsHistoryLimit?.toString() || '1'}</span>
              </div>
            </div>
          </div>
        {/if}
      </div>

      <div class="details-section">
        <h6 class="section-header" onclick={() => toggleSection('conditions')} role="button" tabindex="0" onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSection('conditions')}>
          <span>Conditions</span>
          <span class="collapse-icon">{sectionsCollapsed.conditions ? '▶' : '▼'}</span>
        </h6>
        {#if !sectionsCollapsed.conditions}
          {#if cj.status?.conditions && cj.status.conditions.length > 0}
            <div class="conditions-table">
              <div class="conditions-header">
                <div>Type</div>
                <div>Status</div>
                <div>Reason</div>
                <div>Message</div>
                <div>Last Update</div>
              </div>
              {#each cj.status?.conditions || [] as condition}
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
        <h6 class="section-header" onclick={() => toggleSection('jobs')} role="button" tabindex="0" onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSection('jobs')}>
          <span>Managed Jobs ({managedJobs.length})</span>
          <span class="collapse-icon">{sectionsCollapsed.jobs ? '▶' : '▼'}</span>
        </h6>
        {#if !sectionsCollapsed.jobs}
          {#if jobsLoading}
            <div class="jobs-loading"><div class="loading-spinner">⏳</div><p>Loading jobs...</p></div>
          {:else if jobsError}
            <div class="jobs-error"><div class="error-icon">⚠️</div><p>Failed to load jobs: {jobsError}</p><button class="retry-button" onclick={loadManagedJobs}>Retry</button></div>
          {:else if managedJobs.length > 0}
            <div class="jobs-table">
              <div class="jobs-header">
                <div>Name</div>
                <div>Status</div>
                <div>Completions</div>
                <div>Age</div>
              </div>
              {#each managedJobs as job}
                {@const jobStatus = job.status?.succeeded ? 'Succeeded' : (job.status?.failed ? 'Failed' : (job.status?.active ? 'Active' : 'Unknown'))}
                <div class="job-row">
                  <div class="job-name">
                    {job.metadata?.name || 'Unknown'}
                  </div>
                  <div class="job-status">
                    <span class="status-badge status-{getPodStatusClass(jobStatus)}">{jobStatus}</span>
                  </div>
                  <div class="job-completions">
                    {job.status?.succeeded || 0}/{job.spec?.completions || 1}
                  </div>
                  <div class="job-age">{formatAge(job.metadata?.creationTimestamp)}</div>
                </div>
              {/each}
            </div>
          {:else}
            <div class="jobs-placeholder"><p>No jobs created by this CronJob</p></div>
          {/if}
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
                {#if cj.spec?.jobTemplate?.spec?.selector?.matchLabels && Object.keys(cj.spec.jobTemplate.spec.selector.matchLabels).length > 0}
                  <div class="kv-list">
                    {#each Object.entries(cj.spec.jobTemplate.spec.selector.matchLabels) as [k, v]}
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
                {#if cj.spec?.jobTemplate?.spec?.selector?.matchExpressions && cj.spec.jobTemplate.spec.selector.matchExpressions.length > 0}
                  <div class="match-expressions-list">
                    {#each cj.spec.jobTemplate.spec.selector.matchExpressions as expr}
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
                  {#if cj.metadata?.labels && Object.keys(cj.metadata.labels).length > 0}
                    {#each Object.entries(cj.metadata.labels) as [k, v]}
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
                  {#if cj.metadata?.annotations && Object.keys(cj.metadata.annotations).length > 0}
                    {#each Object.entries(cj.metadata.annotations) as [k, v]}
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

<QuickActionsMenu
  x={actionsMenuPosition.x}
  y={actionsMenuPosition.y}
  resource={cronJobDetails || cronJob}
  resourceType="cronjob"
  bind:visible={actionsMenuVisible}
  on:close={handleActionMenuClose}
  on:deleted={handleActionDeleted}
  on:suspended={handleActionSuspended}
  on:resumed={handleActionResumed}
  on:triggered={handleActionTriggered}
  on:view-yaml={handleViewYaml}
  on:edit={handleActionEdit}
  on:copied={handleActionCopied}
/>

<!-- YAML Viewer/Editor modals - reuse ServiceDetails styles -->
{#if yamlViewerVisible}
  <div class="yaml-viewer-modal" onclick={(e) => e.target === e.currentTarget && closeYamlViewer()}>
    <div class="yaml-viewer-content" onclick={(e) => e.stopPropagation()}>
      <div class="yaml-viewer-header">
        <h3>CronJob YAML: {cronJob?.metadata?.name}</h3>
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
        <h3>Edit CronJob YAML: {cronJob?.metadata?.name}</h3>
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

  .cronjob-namespace {
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

  .cronjob-details-content {
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

  /* CronJob Actions */
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

