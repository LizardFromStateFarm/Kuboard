<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import QuickActionsMenu from './QuickActionsMenu.svelte';
  import PortForwardManager from './PortForwardManager.svelte';

  export let service: any;
  export let onBack: () => void;

  let serviceDetails: any = null;
  let endpoints: any = null;
  let loading = false;
  let error: string | null = null;
  let endpointsLoading = false;
  let endpointsError: string | null = null;

  // Quick Actions Menu state
  let actionsMenuVisible = false;
  let actionsMenuPosition = { x: 0, y: 0 };
  let yamlViewerVisible = false;
  let yamlContent = '';
  let yamlEditorVisible = false;
  let yamlEditorContent = '';
  let yamlEditorLoading = false;
  let yamlEditorError: string | null = null;

  // Port Forward state
  let portForwardManagerOpen = false;

  // Section collapse state
  let sectionsCollapsed = {
    ports: false,
    selectors: true,
    endpoints: false,
    labels: true,
    affinity: true,
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

  function getServiceTypeClass(type: string): string {
    switch (type?.toLowerCase()) {
      case 'clusterip': return 'type-clusterip';
      case 'loadbalancer': return 'type-loadbalancer';
      case 'nodeport': return 'type-nodeport';
      case 'externalname': return 'type-externalname';
      default: return 'type-unknown';
    }
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

  async function loadServiceDetails() {
    if (!service?.metadata?.name || !service?.metadata?.namespace) return;
    
    loading = true;
    error = null;
    
    try {
      const [serviceData, endpointsData] = await Promise.all([
        invoke('kuboard_get_service', {
          name: service.metadata.name,
          namespace: service.metadata.namespace
        }).catch(() => null),
        loadEndpoints()
      ]);
      
      serviceDetails = serviceData || service;
    } catch (err) {
      error = String(err);
      serviceDetails = service; // Fallback to passed service
    } finally {
      loading = false;
    }
  }

  async function loadEndpoints() {
    if (!service?.metadata?.name || !service?.metadata?.namespace) return;
    
    endpointsLoading = true;
    endpointsError = null;
    
    try {
      const endpointsData = await invoke('kuboard_get_service_endpoints', {
        name: service.metadata.name,
        namespace: service.metadata.namespace
      });
      endpoints = endpointsData;
    } catch (err) {
      endpointsError = String(err);
      endpoints = null;
    } finally {
      endpointsLoading = false;
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
        copyToClipboard(service?.metadata?.name || '');
        break;
      case 'copy-namespace':
        copyToClipboard(service?.metadata?.namespace || '');
        break;
      default:
        console.log('Action not implemented:', action);
    }
    handleActionMenuClose();
  }

  async function openYamlViewer() {
    if (!service?.metadata?.name || !service?.metadata?.namespace) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      // For now, use JSON.stringify - in future could fetch YAML from backend
      yamlContent = JSON.stringify(serviceDetails || service, null, 2);
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
    yamlEditorContent = JSON.stringify(serviceDetails || service, null, 2);
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
    if (!service?.metadata?.name || !service?.metadata?.namespace) return;
    
    yamlEditorLoading = true;
    yamlEditorError = null;
    
    try {
      // TODO: Implement service update command
      alert('Service update not yet implemented. Please use kubectl.');
      closeYamlEditor();
    } catch (error: any) {
      yamlEditorError = String(error);
      console.error('Failed to update service:', error);
    } finally {
      yamlEditorLoading = false;
    }
  }

  function getExternalIPs(service: any): string[] {
    if (!service) return [];
    
    // LoadBalancer type
    if (service.status?.loadBalancer?.ingress) {
      return service.status.loadBalancer.ingress.map((ing: any) => 
        ing.ip || ing.hostname || 'Pending'
      );
    }
    
    // External IPs field
    if (service.spec?.externalIPs) {
      return service.spec.externalIPs;
    }
    
    return [];
  }

  onMount(() => {
    loadServiceDetails();
  });
</script>

<div class="full-details-view">
  <div class="details-header">
    <div class="header-left">
      <button class="back-button" onclick={onBack}>← Back to Services</button>
      <button class="port-forward-button" onclick={() => portForwardManagerOpen = true} title="Port Forward">🔌 Port Forward</button>
      <button class="actions-button" onclick={openActionsMenu}>⚙️ Actions</button>
    </div>
    <div class="header-right">
      <h3>{service?.metadata?.name}</h3>
      <span class="service-namespace">({service?.metadata?.namespace})</span>
    </div>
  </div>

  {#if loading}
    <div class="loading-message">
      <div class="loading-spinner">⏳</div>
      <p>Loading service details...</p>
    </div>
  {:else if error}
    <div class="error-message">
      <div class="error-icon">⚠️</div>
      <div class="error-content">
        <h5>Failed to load service details</h5>
        <p>{error}</p>
        <button class="retry-button" onclick={loadServiceDetails}>Retry</button>
      </div>
    </div>
  {:else}
    {@const svc = serviceDetails || service}
    <div class="service-details-content">
      <div class="details-section">
        <h6>Basic Information</h6>
        <div class="info-grid">
          <div class="info-item">
            <span class="info-label">Type:</span>
            <div class="info-value-container">
              <span class="info-value type-badge {getServiceTypeClass(svc.spec?.type)}">{svc.spec?.type || 'ClusterIP'}</span>
            </div>
          </div>
          <div class="info-item">
            <span class="info-label">Cluster IP:</span>
            <div class="info-value-container">
              <span class="info-value" title={svc.spec?.clusterIP}>{svc.spec?.clusterIP || 'None'}</span>
              <button class="copy-button" onclick={() => copyToClipboard(svc.spec?.clusterIP || '')}>📋</button>
            </div>
          </div>
          {#if svc.spec?.type === 'ExternalName'}
            <div class="info-item">
              <span class="info-label">External Name:</span>
              <div class="info-value-container">
                <span class="info-value" title={svc.spec?.externalName}>{svc.spec?.externalName || 'None'}</span>
              </div>
            </div>
          {/if}
          {#if getExternalIPs(svc).length > 0}
            <div class="info-item">
              <span class="info-label">External IPs:</span>
              <div class="info-value-container">
                <div class="ip-list">
                  {#each getExternalIPs(svc) as ip}
                    <span class="ip-badge" title={ip}>{ip}</span>
                  {/each}
                </div>
              </div>
            </div>
          {/if}
          <div class="info-item">
            <span class="info-label">Age:</span>
            <div class="info-value-container">
              <span class="info-value" title={svc.metadata?.creationTimestamp}>{formatAge(svc.metadata?.creationTimestamp)}</span>
              <button class="copy-button" onclick={() => copyToClipboard(svc.metadata?.creationTimestamp || '')}>📋</button>
            </div>
          </div>
          {#if svc.spec?.sessionAffinity}
            <div class="info-item">
              <span class="info-label">Session Affinity:</span>
              <div class="info-value-container">
                <span class="info-value">{svc.spec.sessionAffinity}</span>
                {#if svc.spec.sessionAffinityConfig?.clientIP?.timeoutSeconds}
                  <span class="info-value">(Timeout: {svc.spec.sessionAffinityConfig.clientIP.timeoutSeconds}s)</span>
                {/if}
              </div>
            </div>
          {/if}
        </div>
      </div>

      <div class="details-section">
        <h6 class="section-header" onclick={() => toggleSection('ports')} role="button" tabindex="0" onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSection('ports')}>
          <span>Ports ({svc.spec?.ports?.length || 0})</span>
          <span class="collapse-icon">{sectionsCollapsed.ports ? '▶' : '▼'}</span>
        </h6>
        {#if !sectionsCollapsed.ports}
          {#if svc.spec?.ports && svc.spec.ports.length > 0}
            <div class="ports-table">
              <div class="ports-header">
                <div>Name</div>
                <div>Protocol</div>
                <div>Port</div>
                <div>Target Port</div>
                <div>Node Port</div>
              </div>
              {#each svc.spec.ports as port}
                <div class="port-row">
                  <div class="p-name">{port.name || '-'}</div>
                  <div class="p-protocol">{port.protocol || 'TCP'}</div>
                  <div class="p-port">{port.port}</div>
                  <div class="p-target">{formatObject(port.targetPort)}</div>
                  <div class="p-nodeport">{port.nodePort || '-'}</div>
                </div>
              {/each}
            </div>
          {:else}
            <div class="ports-placeholder"><p>No ports configured</p></div>
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
              <span class="info-label">Selectors:</span>
              <div class="info-value-container">
                {#if svc.spec?.selector && Object.keys(svc.spec.selector).length > 0}
                  <div class="kv-list">
                    {#each Object.entries(svc.spec.selector) as [k, v]}
                      <div class="kv"><span class="k">{k}</span><span class="v">{v}</span></div>
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
        <h6 class="section-header" onclick={() => toggleSection('endpoints')} role="button" tabindex="0" onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSection('endpoints')}>
          <span>Endpoints</span>
          <span class="collapse-icon">{sectionsCollapsed.endpoints ? '▶' : '▼'}</span>
        </h6>
        {#if !sectionsCollapsed.endpoints}
          {#if endpointsLoading}
            <div class="endpoints-loading"><div class="loading-spinner">⏳</div><p>Loading endpoints...</p></div>
          {:else if endpointsError}
            <div class="endpoints-error"><div class="error-icon">⚠️</div><p>Failed to load endpoints: {endpointsError}</p><button class="retry-button" onclick={loadEndpoints}>Retry</button></div>
          {:else if endpoints?.subsets && endpoints.subsets.length > 0}
            <div class="endpoints-list">
              {#each endpoints.subsets as subset}
                <div class="endpoint-subset">
                  {#if subset.addresses && subset.addresses.length > 0}
                    <h6>Addresses ({subset.addresses.length})</h6>
                    <div class="addresses-list">
                      {#each subset.addresses as addr}
                        <div class="address-item">
                          <span class="address-ip">{addr.ip}</span>
                          {#if addr.nodeName}
                            <span class="address-node">Node: {addr.nodeName}</span>
                          {/if}
                          {#if addr.targetRef}
                            <span class="address-target">{addr.targetRef.kind}/{addr.targetRef.name}</span>
                          {/if}
                        </div>
                      {/each}
                    </div>
                  {/if}
                  {#if subset.ports && subset.ports.length > 0}
                    <h6>Ports</h6>
                    <div class="endpoint-ports-list">
                      {#each subset.ports as port}
                        <div class="endpoint-port-item">
                          <span class="port-name">{port.name || '-'}</span>
                          <span class="port-value">{port.port}/{port.protocol || 'TCP'}</span>
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
          {:else}
            <div class="endpoints-placeholder"><p>No endpoints available</p></div>
          {/if}
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
                  {#if svc.metadata?.labels && Object.keys(svc.metadata.labels).length > 0}
                    {#each Object.entries(svc.metadata.labels) as [k, v]}
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
                  {#if svc.metadata?.annotations && Object.keys(svc.metadata.annotations).length > 0}
                    {#each Object.entries(svc.metadata.annotations) as [k, v]}
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
    resource={service}
    resourceType="service"
    on:action={handleAction}
    on:close={handleActionMenuClose}
  />
{/if}

{#if portForwardManagerOpen}
  <div class="port-forward-overlay">
    <PortForwardManager
      isOpen={portForwardManagerOpen}
      service={service}
      onClose={() => portForwardManagerOpen = false}
    />
  </div>
{/if}

{#if yamlViewerVisible}
  <div class="yaml-viewer-modal" onclick={(e) => e.target === e.currentTarget && closeYamlViewer()}>
    <div class="yaml-viewer-content" onclick={(e) => e.stopPropagation()}>
      <div class="yaml-viewer-header">
        <h3>Service YAML: {service?.metadata?.name}</h3>
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
        <h3>Edit Service YAML: {service?.metadata?.name}</h3>
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

  .service-namespace {
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

  .service-details-content {
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

  .type-badge {
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    font-size: 0.8rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  .type-clusterip {
    background: rgba(16, 185, 129, 0.2);
    color: #10b981;
  }

  .type-loadbalancer {
    background: rgba(59, 130, 246, 0.2);
    color: #3b82f6;
  }

  .type-nodeport {
    background: rgba(245, 158, 11, 0.2);
    color: #f59e0b;
  }

  .type-externalname {
    background: rgba(139, 92, 246, 0.2);
    color: #8b5cf6;
  }

  .ip-list {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .ip-badge {
    padding: 4px 8px;
    background: rgba(59, 130, 246, 0.2);
    color: #3b82f6;
    border-radius: var(--radius-sm);
    font-size: 0.8rem;
  }

  .ports-table {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .ports-header, .port-row {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr 1fr 1fr;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm);
  }

  .ports-header {
    background: rgba(255, 255, 255, 0.05);
    font-weight: 600;
    color: var(--text-secondary);
    font-size: 0.85rem;
  }

  .port-row {
    background: rgba(255, 255, 255, 0.02);
    border-radius: var(--radius-sm);
  }

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

  .endpoints-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .endpoint-subset {
    background: rgba(255, 255, 255, 0.02);
    border-radius: var(--radius-sm);
    padding: var(--spacing-sm);
  }

  .endpoint-subset h6 {
    margin: 0 0 var(--spacing-sm) 0;
    font-size: 0.9rem;
    color: var(--text-secondary);
  }

  .addresses-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: var(--spacing-sm);
  }

  .address-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: 6px;
    background: rgba(255, 255, 255, 0.03);
    border-radius: var(--radius-sm);
  }

  .address-ip {
    color: var(--text-primary);
    font-weight: 500;
  }

  .address-node, .address-target {
    color: var(--text-secondary);
    font-size: 0.85rem;
  }

  .endpoint-ports-list {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .endpoint-port-item {
    display: flex;
    gap: 6px;
    padding: 4px 8px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
  }

  .port-name {
    color: var(--text-secondary);
  }

  .port-value {
    color: var(--text-primary);
    font-weight: 500;
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

  .loading-message, .error-message {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-xl);
  }

  .loading-spinner {
    font-size: 2rem;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .error-message {
    flex-direction: row;
    align-items: flex-start;
  }

  .error-icon {
    font-size: 1.5rem;
  }

  .error-content {
    flex: 1;
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

  .ports-placeholder, .endpoints-placeholder {
    padding: var(--spacing-md);
    text-align: center;
    color: var(--text-secondary);
    font-style: italic;
  }

  .endpoints-loading, .endpoints-error {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-md);
  }

  .endpoints-error {
    flex-direction: column;
    align-items: flex-start;
  }

  /* YAML Viewer Modal - reuse PodDetails styles */
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

  .port-forward-button {
    padding: 6px 12px;
    background: var(--background-secondary, #111);
    border: 1px solid var(--border-color, #333);
    border-radius: 4px;
    color: var(--text-primary, #fff);
    cursor: pointer;
    font-size: 13px;
    transition: all 0.2s;
  }

  .port-forward-button:hover {
    background: var(--background-card, #1a1a1a);
    border-color: var(--primary-color, #2e91be);
  }

  .port-forward-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.8);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 20px;
  }

  .port-forward-overlay > :global(*) {
    width: 100%;
    max-width: 1200px;
    height: 80vh;
    max-height: 800px;
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

