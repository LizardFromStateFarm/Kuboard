<!-- Kuboard Network Tab Component -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import ServiceDetails from './ServiceDetails.svelte';

  // Props
  export let currentContext: any = null;

  // State
  let services: any[] = [];
  let ingresses: any[] = [];
  let networkpolicies: any[] = [];
  let endpoints: any[] = [];
  
  let loading: boolean = false;
  let error: string | null = null;
  let lastUpdate: string = '';
  
  // Service detail view state
  let selectedService: any = null;

  // Load network data
  async function loadNetwork() {
    if (!currentContext || loading) return;
    
    loading = true;
    error = null;
    
    try {
      // Load all network types in parallel
      const [servicesData] = await Promise.all([
        invoke('kuboard_get_services').catch(() => [])
      ]);

      services = servicesData as any[] || [];
      
      lastUpdate = new Date().toLocaleTimeString();
    } catch (err) {
      error = err as string;
      console.error('Failed to load network resources:', err);
    } finally {
      loading = false;
    }
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

  // Get service type
  function getServiceType(service: any): string {
    return service.spec?.type || 'ClusterIP';
  }

  // Get service ports
  function getServicePorts(service: any): string {
    const ports = service.spec?.ports || [];
    if (ports.length === 0) return 'No ports';
    if (ports.length === 1) {
      const port = ports[0];
      return `${port.port}${port.targetPort ? `:${port.targetPort}` : ''}${port.protocol ? `/${port.protocol}` : ''}`;
    }
    return `${ports.length} ports`;
  }

  // Get service cluster IP
  function getServiceClusterIP(service: any): string {
    return service.spec?.clusterIP || 'None';
  }

  // Get service external IP
  function getServiceExternalIP(service: any): string {
    const externalIPs = service.status?.loadBalancer?.ingress || [];
    if (externalIPs.length > 0) {
      return externalIPs[0].ip || externalIPs[0].hostname || 'Pending';
    }
    return 'None';
  }

  // Lifecycle
  onMount(() => {
    loadNetwork();
  });

  // Reactive updates
  $: if (currentContext) {
    loadNetwork();
  }

  function handleServiceClick(service: any) {
    selectedService = service;
  }

  function handleBack() {
    selectedService = null;
  }
</script>

<div class="network-tab">
  {#if selectedService}
    <ServiceDetails service={selectedService} onBack={handleBack} />
  {:else}
    <div class="tab-header">
      <h4>🌐 Network</h4>
      <div class="tab-controls">
        <button 
          class="refresh-button" 
          onclick={loadNetwork}
          disabled={loading}
          title="Refresh network resources"
        >
          {#if loading}
            🔄
          {:else}
            ↻
          {/if}
        </button>
        {#if lastUpdate}
          <span class="last-update">Last: {lastUpdate}</span>
        {/if}
      </div>
    </div>

    {#if error}
    <div class="error-message">
      <div class="error-icon">⚠️</div>
      <div class="error-content">
        <h5>Failed to load network resources</h5>
        <p>{error}</p>
        <button class="retry-button" onclick={loadNetwork}>
          Retry
        </button>
      </div>
    </div>
  {:else if loading}
    <div class="loading-message">
      <div class="loading-spinner">🔄</div>
      <p>Loading network resources...</p>
    </div>
  {:else}
    <div class="network-grid">
      <!-- Services Section -->
      <div class="network-section">
        <div class="section-header">
          <h5>Services ({services.length})</h5>
        </div>
        <div class="network-list">
          {#each services.slice(0, 10) as service}
            <div class="network-item" role="button" tabindex="0" onclick={() => handleServiceClick(service)} onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && handleServiceClick(service)}>
              <div class="network-info">
                <span class="network-name">{service.metadata?.name || 'Unknown'}</span>
                <span class="network-namespace">{service.metadata?.namespace || 'default'}</span>
              </div>
              <div class="network-details">
                <div class="network-type">
                  <span class="type-badge type-{getServiceType(service).toLowerCase()}">
                    {getServiceType(service)}
                  </span>
                </div>
                <div class="network-ports">
                  <span class="port-info">{getServicePorts(service)}</span>
                </div>
                <div class="network-ips">
                  <span class="ip-info">Cluster: {getServiceClusterIP(service)}</span>
                  {#if getServiceExternalIP(service) !== 'None'}
                    <span class="ip-info">External: {getServiceExternalIP(service)}</span>
                  {/if}
                </div>
                <div class="network-age">
                  <span class="age-info">{formatAge(service.metadata?.creationTimestamp)}</span>
                </div>
              </div>
            </div>
          {/each}
          {#if services.length > 10}
            <div class="network-more">
              <span>... and {services.length - 10} more</span>
            </div>
          {/if}
        </div>
      </div>

      <!-- Coming Soon Section -->
      <div class="network-section">
        <div class="section-header">
          <h5>Ingress (Coming Soon)</h5>
        </div>
        <div class="coming-soon">
          <div class="coming-soon-icon">🚧</div>
          <p>Ingress resources will be available in a future update</p>
        </div>
      </div>

      <div class="network-section">
        <div class="section-header">
          <h5>Network Policies (Coming Soon)</h5>
        </div>
        <div class="coming-soon">
          <div class="coming-soon-icon">🚧</div>
          <p>Network policies will be available in a future update</p>
        </div>
      </div>
    </div>
  {/if}
  {/if}
</div>

<style>
  /* Import CSS variables */
  @import '../styles/variables.css';

  .network-tab {
    padding: 0;
  }

  .tab-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 5px;
    padding-bottom: var(--spacing-sm);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .tab-header h4 {
    margin: 0;
    color: white;
    font-size: 1.1rem;
    font-weight: 600;
  }

  .tab-controls {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .refresh-button {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-sm);
    color: white;
    cursor: pointer;
    font-size: 0.9rem;
    padding: 6px 12px;
    transition: var(--transition-normal);
  }

  .refresh-button:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.2);
    border-color: rgba(255, 255, 255, 0.3);
  }

  .refresh-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .last-update {
    font-size: 0.8rem;
    color: rgba(255, 255, 255, 0.6);
  }

  .error-message {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: var(--radius-md);
  }

  .error-icon {
    font-size: 1.2rem;
    flex-shrink: 0;
  }

  .error-content h5 {
    margin: 0 0 4px 0;
    color: white;
    font-size: 0.9rem;
    font-weight: 600;
  }

  .error-content p {
    margin: 0 0 8px 0;
    color: rgba(255, 255, 255, 0.8);
    font-size: 0.85rem;
  }

  .retry-button {
    background: var(--error-color);
    border: none;
    border-radius: var(--radius-sm);
    color: white;
    cursor: pointer;
    font-size: 0.8rem;
    padding: 4px 8px;
    transition: var(--transition-normal);
  }

  .retry-button:hover {
    background: #dc2626;
  }

  .loading-message {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-xl);
    color: rgba(255, 255, 255, 0.8);
  }

  .loading-spinner {
    font-size: 1.5rem;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .network-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: var(--spacing-lg);
  }

  .network-section {
    background: rgba(255, 255, 255, 0.02);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .section-header {
    margin-bottom: var(--spacing-md);
  }

  .section-header h5 {
    margin: 0;
    color: white;
    font-size: 1rem;
    font-weight: 600;
  }

  .network-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .network-item {
    padding: var(--spacing-sm);
    background: rgba(255, 255, 255, 0.03);
    border-radius: var(--radius-sm);
    border: 1px solid rgba(255, 255, 255, 0.05);
    cursor: pointer;
    transition: all 0.2s;
  }

  .network-item:hover {
    background: rgba(255, 255, 255, 0.05);
    border-color: rgba(255, 255, 255, 0.1);
    transform: translateY(-1px);
  }

  .network-item:focus {
    outline: 2px solid var(--primary-color);
    outline-offset: 2px;
  }

  .network-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-xs);
  }

  .network-name {
    color: white;
    font-size: 0.9rem;
    font-weight: 600;
  }

  .network-namespace {
    color: rgba(255, 255, 255, 0.6);
    font-size: 0.8rem;
  }

  .network-details {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .network-type {
    margin-bottom: var(--spacing-xs);
  }

  .type-badge {
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .type-clusterip {
    background: rgba(16, 185, 129, 0.2);
    color: var(--primary-color);
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

  .network-ports {
    margin-bottom: 2px;
  }

  .port-info {
    color: rgba(255, 255, 255, 0.8);
    font-size: 0.8rem;
    font-weight: 500;
  }

  .network-ips {
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin-bottom: 2px;
  }

  .ip-info {
    color: rgba(255, 255, 255, 0.7);
    font-size: 0.75rem;
  }

  .network-age {
    margin-top: 2px;
  }

  .age-info {
    color: rgba(255, 255, 255, 0.6);
    font-size: 0.7rem;
  }

  .network-more {
    text-align: center;
    padding: var(--spacing-sm);
    color: rgba(255, 255, 255, 0.6);
    font-size: 0.8rem;
    font-style: italic;
  }

  .coming-soon {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-xl);
    color: rgba(255, 255, 255, 0.6);
    text-align: center;
  }

  .coming-soon-icon {
    font-size: 2rem;
    opacity: 0.7;
  }

  .coming-soon p {
    margin: 0;
    font-size: 0.9rem;
    font-style: italic;
  }

  /* Responsive Design */
  @media (max-width: 768px) {
    .network-grid {
      grid-template-columns: 1fr;
    }
    
    .network-info {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--spacing-xs);
    }
  }
</style>
