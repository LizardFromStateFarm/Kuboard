<!--
  PortForwardManager.svelte - Port forwarding management component
  This component manages port forwarding sessions for pods and services
-->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  // Props
  export let isOpen = false;
  export let onClose: () => void = () => {};
  export let pod: any = null; // Optional pod to pre-fill form
  export let service: any = null; // Optional service to pre-fill form

  // State
  let activeForwards: Array<{
    id: string;
    podName?: string;
    serviceName?: string;
    namespace: string;
    localPort: number;
    remotePort: number;
    status: 'active' | 'connecting' | 'error';
    url: string;
  }> = [];

  let loading = false;
  let error: string | null = null;
  let showAddDialog = false;

  // New port forward form
  let forwardType: 'pod' | 'service' = 'pod';
  let podName = '';
  let serviceName = '';
  let namespace = 'default';
  let localPort = '';
  let remotePort = '';
  let containerName = '';

  // Available ports from pod/service
  let availablePorts: Array<{port: number, name?: string, protocol?: string}> = [];

  // Extract available ports from pod
  function extractPodPorts(podData: any): Array<{port: number, name?: string, protocol?: string}> {
    if (!podData?.spec?.containers) {
      console.log('No containers found in pod spec');
      return [];
    }
    
    const ports: Array<{port: number, name?: string, protocol?: string}> = [];
    podData.spec.containers.forEach((container: any) => {
      if (container.ports && Array.isArray(container.ports)) {
        container.ports.forEach((port: any) => {
          const portNum = port.containerPort || port.port;
          if (portNum) {
            ports.push({
              port: portNum,
              name: port.name,
              protocol: port.protocol || 'TCP'
            });
          }
        });
      }
    });
    
    console.log('Extracted ports from pod:', ports);
    
    // Remove duplicates
    const uniquePorts = ports.filter((p, index, self) => 
      index === self.findIndex((p2) => p2.port === p.port)
    );
    
    return uniquePorts.sort((a, b) => a.port - b.port);
  }

  // Extract available ports from service
  function extractServicePorts(serviceData: any): Array<{port: number, name?: string, protocol?: string}> {
    if (!serviceData?.spec?.ports) return [];
    
    return serviceData.spec.ports.map((port: any) => ({
      port: port.port,
      name: port.name,
      protocol: port.protocol || 'TCP'
    })).sort((a: any, b: any) => a.port - b.port);
  }

  // Auto-fill form when pod or service is provided
  function autoFillForm() {
    console.log('Auto-filling form, pod:', pod, 'service:', service);
    if (pod) {
      forwardType = 'pod';
      podName = pod.metadata?.name || '';
      namespace = pod.metadata?.namespace || 'default';
      containerName = pod.spec?.containers?.[0]?.name || '';
      const extractedPorts = extractPodPorts(pod);
      availablePorts = extractedPorts;
      console.log('Available ports after extraction:', availablePorts);
      
      // Auto-select first port if available
      if (availablePorts.length > 0) {
        remotePort = availablePorts[0].port.toString();
        localPort = availablePorts[0].port.toString(); // Default to same port
      } else {
        remotePort = '';
        localPort = '';
      }
    } else if (service) {
      forwardType = 'service';
      serviceName = service.metadata?.name || '';
      namespace = service.metadata?.namespace || 'default';
      const extractedPorts = extractServicePorts(service);
      availablePorts = extractedPorts;
      console.log('Available ports after extraction:', availablePorts);
      
      // Auto-select first port if available
      if (availablePorts.length > 0) {
        remotePort = availablePorts[0].port.toString();
        localPort = availablePorts[0].port.toString(); // Default to same port
      } else {
        remotePort = '';
        localPort = '';
      }
    } else {
      // Reset to defaults
      forwardType = 'pod';
      podName = '';
      serviceName = '';
      namespace = 'default';
      localPort = '';
      remotePort = '';
      containerName = '';
      availablePorts = [];
    }
  }

  onMount(() => {
    if (isOpen) {
      autoFillForm();
      loadActiveForwards();
      // Auto-open add dialog if we have pod/service info
      if (pod || service) {
        showAddDialog = true;
      }
    }
  });

  $: if (isOpen) {
    autoFillForm();
    loadActiveForwards();
    // Auto-open add dialog if we have pod/service info
    if ((pod || service) && !showAddDialog) {
      showAddDialog = true;
    }
  }

  // Reactive statement to update ports when pod/service changes
  $: if (isOpen && (pod || service)) {
    autoFillForm();
  }

  async function loadActiveForwards() {
    loading = true;
    error = null;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const forwards = await invoke('kuboard_list_port_forwards');
      activeForwards = forwards as any[];
    } catch (err: any) {
      error = err.toString();
      console.error('Failed to load port forwards:', err);
    } finally {
      loading = false;
    }
  }

  async function createPortForward() {
    if (!localPort || !remotePort || !namespace) {
      error = 'Please fill in all required fields';
      return;
    }

    const localPortNum = parseInt(localPort);
    const remotePortNum = parseInt(remotePort);

    if (isNaN(localPortNum) || isNaN(remotePortNum)) {
      error = 'Ports must be valid numbers';
      return;
    }

    if (localPortNum < 1 || localPortNum > 65535 || remotePortNum < 1 || remotePortNum > 65535) {
      error = 'Ports must be between 1 and 65535';
      return;
    }

    loading = true;
    error = null;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const resourceName = forwardType === 'pod' ? podName : serviceName;
      if (!resourceName) {
        error = `Please provide a ${forwardType} name`;
        loading = false;
        return;
      }

      const result = await invoke('kuboard_port_forward', {
        resourceType: forwardType,
        resourceName: resourceName,
        namespace: namespace,
        localPort: localPortNum,
        remotePort: remotePortNum,
        containerName: containerName || null,
      }) as any;

      if (result.sessionId) {
        // Success - reload the list
        await loadActiveForwards();
        // Reset form
        resetForm();
        showAddDialog = false;
      } else {
        error = result.message || 'Failed to create port forward';
      }
    } catch (err: any) {
      error = err.toString();
      console.error('Failed to create port forward:', err);
    } finally {
      loading = false;
    }
  }

  async function stopPortForward(forwardId: string) {
    loading = true;
    error = null;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('kuboard_stop_port_forward', {
        forwardId: forwardId,
      });
      
      // Remove from list
      activeForwards = activeForwards.filter(f => f.id !== forwardId);
    } catch (err: any) {
      error = err.toString();
      console.error('Failed to stop port forward:', err);
    } finally {
      loading = false;
    }
  }

  function resetForm() {
    // Only reset if we don't have pod/service info
    if (!pod && !service) {
      forwardType = 'pod';
      podName = '';
      serviceName = '';
      namespace = 'default';
      localPort = '';
      remotePort = '';
      containerName = '';
      availablePorts = [];
    }
  }

  function openAddDialog() {
    // Don't reset if we have pod/service info - keep the auto-filled values
    if (!pod && !service) {
      resetForm();
    } else {
      autoFillForm();
    }
    showAddDialog = true;
    error = null;
  }

  function closeAddDialog() {
    showAddDialog = false;
    // Don't reset form if we have pod/service - keep the values for next time
    if (!pod && !service) {
      resetForm();
    }
  }

  function handleCloseManager() {
    showAddDialog = false;
    onClose();
  }

  function handleDialogClose() {
    // If opened from pod/service, close the entire manager
    // Otherwise just close the dialog
    if (pod || service) {
      handleCloseManager();
    } else {
      closeAddDialog();
    }
  }

  function getForwardUrl(forward: any): string {
    return `http://localhost:${forward.localPort}`;
  }
</script>

{#if isOpen}
  <div class="port-forward-manager">
    <div class="manager-header">
      <div class="header-left">
        <span class="manager-icon">🔌</span>
        <h3>Port Forwarding</h3>
      </div>
      <div class="header-right">
        <button class="add-button" onclick={openAddDialog} title="Add Port Forward">
          + Add Forward
        </button>
        <button class="close-button" onclick={handleCloseManager} title="Close">
          ✕
        </button>
      </div>
    </div>

    {#if error}
      <div class="error-message">
        <span>⚠️ {error}</span>
      </div>
    {/if}

    <div class="manager-content">
      {#if loading && activeForwards.length === 0}
        <div class="loading-state">
          <span>Loading port forwards...</span>
        </div>
      {:else if activeForwards.length === 0}
        <div class="empty-state">
          <div class="empty-icon">🔌</div>
          <p>No active port forwards</p>
          <p class="empty-hint">Click "Add Forward" to create a new port forward</p>
        </div>
      {:else}
        <div class="forwards-list">
          <div class="forwards-header">
            <div>Resource</div>
            <div>Namespace</div>
            <div>Local Port</div>
            <div>Remote Port</div>
            <div>Status</div>
            <div>Actions</div>
          </div>
          {#each activeForwards as forward}
            <div class="forward-item">
              <div class="forward-resource">
                {#if forward.podName}
                  <span class="resource-icon">🟢</span>
                  <span>{forward.podName}</span>
                {:else if forward.serviceName}
                  <span class="resource-icon">🌐</span>
                  <span>{forward.serviceName}</span>
                {/if}
              </div>
              <div class="forward-namespace">{forward.namespace}</div>
              <div class="forward-local-port">
                <a href={forward.url} target="_blank" rel="noopener noreferrer">
                  localhost:{forward.localPort}
                </a>
              </div>
              <div class="forward-remote-port">{forward.remotePort}</div>
              <div class="forward-status">
                {#if forward.status === 'active'}
                  <span class="status-badge active">● Active</span>
                {:else if forward.status === 'connecting'}
                  <span class="status-badge connecting">○ Connecting</span>
                {:else}
                  <span class="status-badge error">✕ Error</span>
                {/if}
              </div>
              <div class="forward-actions">
                <button 
                  class="stop-button" 
                  onclick={() => stopPortForward(forward.id)}
                  title="Stop Port Forward"
                >
                  Stop
                </button>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    {#if showAddDialog}
      <div class="dialog-overlay" onclick={(e) => {
        if (e.target === e.currentTarget) {
          handleDialogClose();
        }
      }}>
        <div class="dialog-content" onclick={(e) => e.stopPropagation()}>
          <div class="dialog-header">
            <h4>Add Port Forward</h4>
            <button class="dialog-close" onclick={handleDialogClose} type="button">✕</button>
          </div>
          
          <div class="dialog-body">
            <div class="form-group">
              <label>Type</label>
              <select 
                bind:value={forwardType}
                disabled={!!pod || !!service}
                class={(pod || service) ? 'auto-filled' : ''}
              >
                <option value="pod">Pod</option>
                <option value="service">Service</option>
              </select>
            </div>

            {#if forwardType === 'pod'}
              <div class="form-group">
                <label>Pod Name *</label>
                <input 
                  type="text" 
                  bind:value={podName} 
                  placeholder="my-pod" 
                  disabled={!!pod}
                  class={pod ? 'auto-filled' : ''}
                />
              </div>
              <div class="form-group">
                <label>Container Name (optional)</label>
                <input 
                  type="text" 
                  bind:value={containerName} 
                  placeholder="container-name"
                  disabled={!!pod}
                  class={pod ? 'auto-filled' : ''}
                />
              </div>
            {:else}
              <div class="form-group">
                <label>Service Name *</label>
                <input 
                  type="text" 
                  bind:value={serviceName} 
                  placeholder="my-service"
                  disabled={!!service}
                  class={service ? 'auto-filled' : ''}
                />
              </div>
            {/if}

            <div class="form-group">
              <label>Namespace *</label>
              <input 
                type="text" 
                bind:value={namespace} 
                placeholder="default"
                disabled={!!pod || !!service}
                class={(pod || service) ? 'auto-filled' : ''}
              />
            </div>

            {#if availablePorts.length > 0}
              <div class="form-group">
                <label>Available Ports ({availablePorts.length})</label>
                <div class="available-ports-list">
                  {#each availablePorts as portInfo}
                    <div class="port-info-item" onclick={() => {
                      remotePort = portInfo.port.toString();
                      localPort = portInfo.port.toString();
                    }}>
                      <span class="port-number">{portInfo.port}</span>
                      {#if portInfo.name}
                        <span class="port-name">({portInfo.name})</span>
                      {/if}
                      <span class="port-protocol">{portInfo.protocol || 'TCP'}</span>
                      <button 
                        class="port-select-button" 
                        onclick={(e) => {
                          e.stopPropagation();
                          remotePort = portInfo.port.toString();
                          localPort = portInfo.port.toString();
                        }}
                        title="Use this port"
                        type="button"
                      >
                        Use
                      </button>
                    </div>
                  {/each}
                </div>
              </div>
            {:else if pod || service}
              <div class="form-group">
                <div class="no-ports-message">
                  <span>⚠️ No exposed ports found in this {pod ? 'pod' : 'service'}</span>
                  <p class="hint">You can still enter a custom port number below</p>
                </div>
              </div>
            {/if}

            <div class="form-row">
              <div class="form-group">
                <label>Local Port *</label>
                <input type="number" bind:value={localPort} placeholder="8080" min="1" max="65535" />
              </div>
              <div class="form-group">
                <label>Remote Port *</label>
                {#if availablePorts.length > 0}
                  <select bind:value={remotePort} onchange={() => {
                    if (!localPort) {
                      localPort = remotePort;
                    }
                  }}>
                    <option value="">Select a port...</option>
                    {#each availablePorts as portInfo}
                      <option value={portInfo.port.toString()}>
                        {portInfo.port}{portInfo.name ? ` (${portInfo.name})` : ''} - {portInfo.protocol || 'TCP'}
                      </option>
                    {/each}
                  </select>
                  <div class="custom-port-hint">
                    <span>Or enter custom port:</span>
                    <input 
                      type="number" 
                      bind:value={remotePort} 
                      placeholder="Custom port" 
                      min="1" 
                      max="65535"
                      class="custom-port-input"
                    />
                  </div>
                {:else}
                  <input type="number" bind:value={remotePort} placeholder="80" min="1" max="65535" />
                {/if}
              </div>
            </div>
          </div>

          <div class="dialog-footer">
            <button class="cancel-button" onclick={handleDialogClose} type="button">Cancel</button>
            <button class="create-button" onclick={createPortForward} disabled={loading} type="button">
              {#if loading}
                Creating...
              {:else}
                Create Forward
              {/if}
            </button>
          </div>
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .port-forward-manager {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--background-card, #1a1a1a);
    border: 1px solid var(--border-color, #333);
    border-radius: 4px;
    overflow: hidden;
  }

  .manager-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: var(--background-secondary, #111);
    border-bottom: 1px solid var(--border-color, #333);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .manager-icon {
    font-size: 20px;
  }

  .manager-header h3 {
    margin: 0;
    color: var(--text-primary, #fff);
    font-size: 16px;
    font-weight: 600;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .add-button, .close-button {
    padding: 6px 12px;
    background: var(--primary-color, #2e91be);
    border: none;
    border-radius: 4px;
    color: white;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
  }

  .add-button:hover {
    background: var(--primary-dark, #1d4ed8);
  }

  .close-button {
    background: transparent;
    color: var(--text-primary, #fff);
    padding: 4px 8px;
  }

  .close-button:hover {
    background: var(--background-card, #1a1a1a);
  }

  .error-message {
    padding: 8px 16px;
    background: rgba(239, 68, 68, 0.1);
    border-bottom: 1px solid rgba(239, 68, 68, 0.3);
    color: var(--error-color, #ef4444);
    font-size: 13px;
  }

  .manager-content {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
  }

  .loading-state, .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px;
    text-align: center;
    color: var(--text-secondary, #aaa);
  }

  .empty-icon {
    font-size: 48px;
    margin-bottom: 16px;
  }

  .empty-state p {
    margin: 8px 0;
  }

  .empty-hint {
    font-size: 13px;
    color: var(--text-muted, #666);
  }

  .forwards-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .forwards-header {
    display: grid;
    grid-template-columns: 2fr 1fr 1fr 1fr 1fr 1fr;
    gap: 12px;
    padding: 8px 12px;
    background: var(--background-secondary, #111);
    border-radius: 4px;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary, #aaa);
  }

  .forward-item {
    display: grid;
    grid-template-columns: 2fr 1fr 1fr 1fr 1fr 1fr;
    gap: 12px;
    padding: 12px;
    background: var(--background-secondary, #111);
    border-radius: 4px;
    align-items: center;
    font-size: 13px;
  }

  .forward-resource {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text-primary, #fff);
  }

  .resource-icon {
    font-size: 14px;
  }

  .forward-namespace, .forward-local-port, .forward-remote-port {
    color: var(--text-secondary, #aaa);
  }

  .forward-local-port a {
    color: var(--primary-color, #2e91be);
    text-decoration: none;
  }

  .forward-local-port a:hover {
    text-decoration: underline;
  }

  .status-badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 11px;
    font-weight: 500;
  }

  .status-badge.active {
    background: rgba(16, 185, 129, 0.2);
    color: var(--success-color, #10b981);
  }

  .status-badge.connecting {
    background: rgba(251, 191, 36, 0.2);
    color: var(--warning-color, #f59e0b);
  }

  .status-badge.error {
    background: rgba(239, 68, 68, 0.2);
    color: var(--error-color, #ef4444);
  }

  .stop-button {
    padding: 4px 12px;
    background: rgba(239, 68, 68, 0.2);
    border: 1px solid rgba(239, 68, 68, 0.4);
    border-radius: 4px;
    color: var(--error-color, #ef4444);
    cursor: pointer;
    font-size: 12px;
  }

  .stop-button:hover {
    background: rgba(239, 68, 68, 0.3);
  }

  .dialog-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog-content {
    background: var(--background-card, #1a1a1a);
    border: 1px solid var(--border-color, #333);
    border-radius: 8px;
    width: 90%;
    max-width: 500px;
    max-height: 90vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .dialog-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    border-bottom: 1px solid var(--border-color, #333);
  }

  .dialog-header h4 {
    margin: 0;
    color: var(--text-primary, #fff);
    font-size: 16px;
    font-weight: 600;
  }

  .dialog-close {
    background: none;
    border: none;
    color: var(--text-primary, #fff);
    cursor: pointer;
    font-size: 20px;
    padding: 0;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .dialog-body {
    padding: 16px;
    overflow-y: auto;
  }

  .form-group {
    margin-bottom: 16px;
  }

  .form-group label {
    display: block;
    margin-bottom: 6px;
    color: var(--text-primary, #fff);
    font-size: 13px;
    font-weight: 500;
  }

  .form-group input, .form-group select {
    width: 100%;
    padding: 8px 12px;
    background: var(--background-primary, #000);
    border: 1px solid var(--border-color, #333);
    border-radius: 4px;
    color: var(--text-primary, #fff);
    font-size: 13px;
  }

  .form-group input:focus, .form-group select:focus {
    outline: none;
    border-color: var(--primary-color, #2e91be);
  }

  .form-group select:disabled {
    background: var(--background-secondary, #111) !important;
    color: var(--text-secondary, #aaa) !important;
    cursor: not-allowed;
  }

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .dialog-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 16px;
    border-top: 1px solid var(--border-color, #333);
  }

  .cancel-button, .create-button {
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
  }

  .cancel-button {
    background: transparent;
    color: var(--text-primary, #fff);
    border: 1px solid var(--border-color, #333);
  }

  .cancel-button:hover {
    background: var(--background-secondary, #111);
  }

  .create-button {
    background: var(--primary-color, #2e91be);
    color: white;
  }

  .create-button:hover:not(:disabled) {
    background: var(--primary-dark, #1d4ed8);
  }

  .create-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .available-ports-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 8px;
  }

  .port-info-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    background: var(--background-secondary, #111);
    border: 1px solid var(--border-color, #333);
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .port-info-item:hover {
    background: var(--background-card, #1a1a1a);
    border-color: var(--primary-color, #2e91be);
  }

  .port-number {
    font-weight: 600;
    color: var(--text-primary, #fff);
    min-width: 50px;
  }

  .port-name {
    color: var(--text-secondary, #aaa);
    font-size: 12px;
    flex: 1;
  }

  .port-protocol {
    color: var(--text-muted, #666);
    font-size: 11px;
    padding: 2px 6px;
    background: var(--background-primary, #000);
    border-radius: 3px;
  }

  .port-select-button {
    padding: 4px 12px;
    background: var(--primary-color, #2e91be);
    border: none;
    border-radius: 4px;
    color: white;
    cursor: pointer;
    font-size: 11px;
    font-weight: 500;
    transition: all 0.2s;
  }

  .port-select-button:hover {
    background: var(--primary-dark, #1d4ed8);
  }

  .custom-port-hint {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 8px;
    font-size: 12px;
    color: var(--text-secondary, #aaa);
  }

  .custom-port-hint span {
    white-space: nowrap;
  }

  .custom-port-input {
    flex: 1;
    max-width: 150px;
  }

  .auto-filled {
    background: var(--background-secondary, #111) !important;
    color: var(--text-secondary, #aaa) !important;
    cursor: not-allowed;
  }

  .auto-filled::placeholder {
    color: var(--text-muted, #666);
  }

  .no-ports-message {
    padding: 12px;
    background: rgba(251, 191, 36, 0.1);
    border: 1px solid rgba(251, 191, 36, 0.3);
    border-radius: 4px;
    color: var(--warning-color, #f59e0b);
    font-size: 13px;
  }

  .no-ports-message .hint {
    margin: 8px 0 0 0;
    font-size: 11px;
    color: var(--text-secondary, #aaa);
  }
</style>

