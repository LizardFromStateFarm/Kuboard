<!-- Kuboard Services Panel Component -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import ResourceTable from './ResourceTable.svelte';
  import QuickActionsMenu from './QuickActionsMenu.svelte';
  import ServiceDetails from './ServiceDetails.svelte';

  // Props
  export let currentContext: any = null;
  export let namespace: string = 'all';
  export let initialSelectedName: string | null = null;

  // State
  let services: any[] = [];
  let loading: boolean = true;
  let error: string | null = null;
  let searchQuery: string = '';
  let sortColumn: string = 'name';
  let sortDirection: 'asc' | 'desc' = 'asc';
  let refreshTimer: any;
  let selectedService: any = null;

  $: if (initialSelectedName && services && services.length > 0) {
    const found = services.find((s: any) => s.metadata?.name === initialSelectedName);
    if (found) {
      selectedService = found;
    } else if (!selectedService || selectedService?.metadata?.name !== initialSelectedName) {
      selectedService = { metadata: { name: initialSelectedName, namespace: currentContext?.namespace || 'default' } };
    }
  }

  // Context Menu State
  let contextMenuVisible = false;
  let contextMenuPosition = { x: 0, y: 0 };
  let selectedResource: any = null;

  async function fetchServices() {
    if (!currentContext) return;
    
    try {
      loading = true;
      // Note: We use the existing kuboard_get_services command
      services = await invoke('kuboard_get_services');
      
      // Filter by namespace if not 'all'
      if (namespace !== 'all' && namespace !== '') {
        services = services.filter(s => s.metadata.namespace === namespace);
      }
      
      error = null;
    } catch (e: any) {
      console.error('Failed to fetch Services:', e);
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function handleSort(column: string) {
    if (sortColumn === column) {
      sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
    } else {
      sortColumn = column;
      sortDirection = 'asc';
    }
  }

  function formatAge(timestamp: string): string {
    if (!timestamp) return '-';
    const date = new Date(timestamp);
    const now = new Date();
    const diff = Math.floor((now.getTime() - date.getTime()) / 1000);
    
    if (diff < 60) return `${diff}s`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h`;
    return `${Math.floor(diff / 86400)}d`;
  }

  function getServiceType(service: any): string {
    return service.spec?.type || 'ClusterIP';
  }

  function getClusterIP(service: any): string {
    return service.spec?.clusterIP || '-';
  }

  function getExternalIP(service: any): string {
    const ing = service.status?.loadBalancer?.ingress || [];
    if (ing.length === 0) return '-';
    return ing.map((i: any) => i.ip || i.hostname).join(', ');
  }

  function getPorts(service: any): string {
    const ports = service.spec?.ports || [];
    return ports.map((p: any) => `${p.port}/${p.protocol}`).join(', ');
  }

  function handleContextMenu(event: MouseEvent, service: any) {
    event.preventDefault();
    selectedResource = service;
    contextMenuPosition = { x: event.clientX, y: event.clientY };
    contextMenuVisible = true;
  }

  function closeContextMenu() {
    contextMenuVisible = false;
    selectedResource = null;
  }

  function handleServiceClick(service: any) {
    selectedService = service;
  }

  function handleBack() {
    selectedService = null;
  }

  $: filteredServices = services
    .filter(s => {
      const name = s.metadata.name.toLowerCase();
      const query = searchQuery.toLowerCase();
      return name.includes(query);
    })
    .sort((a, b) => {
      let valA, valB;
      if (sortColumn === 'name') {
        valA = a.metadata.name;
        valB = b.metadata.name;
      } else if (sortColumn === 'namespace') {
        valA = a.metadata.namespace || '';
        valB = b.metadata.namespace || '';
      } else if (sortColumn === 'age') {
        valA = new Date(a.metadata.creationTimestamp).getTime();
        valB = new Date(b.metadata.creationTimestamp).getTime();
      } else {
        valA = a.metadata.name;
        valB = b.metadata.name;
      }
      
      if (valA < valB) return sortDirection === 'asc' ? -1 : 1;
      if (valA > valB) return sortDirection === 'asc' ? 1 : -1;
      return 0;
    });

  onMount(() => {
    fetchServices();
    refreshTimer = setInterval(fetchServices, 10000); // Refresh every 10s
  });

  onDestroy(() => {
    if (refreshTimer) clearInterval(refreshTimer);
  });

  $: if (currentContext || namespace) {
    fetchServices();
  }
</script>

<div class="services-panel">
  {#if selectedService}
    <ServiceDetails service={selectedService} onBack={handleBack} on:navigateToWorkload />
  {:else}
    <div class="panel-header">
      <h4>🌐 Services ({filteredServices.length})</h4>
    </div>

    {#if loading && services.length === 0}
      <div class="loading-state">
        <div class="spinner"></div>
        <p>Loading Services...</p>
      </div>
    {:else if error}
      <div class="error-state">
        <span class="error-icon">⚠️</span>
        <p>{error}</p>
        <button onclick={fetchServices}>Retry</button>
      </div>
    {:else}
      <ResourceTable
        items={services}
        filteredItems={filteredServices}
        bind:searchQuery
        searchPlaceholder="Search Services..."
        noItemsMessage="No Services found."
        noSearchResultsMessage="No Services match your search query:"
      >
        <svelte:fragment slot="header">
          <th class="sortable" onclick={() => handleSort('name')}>Name</th>
          <th class="sortable" onclick={() => handleSort('namespace')}>Namespace</th>
          <th>Type</th>
          <th>Cluster IP</th>
          <th>External IP</th>
          <th>Ports</th>
          <th class="sortable" onclick={() => handleSort('age')}>Age</th>
          <th>Actions</th>
        </svelte:fragment>

        <svelte:fragment slot="rows">
          {#each filteredServices as service (service.metadata.uid)}
            <tr class="resource-row" 
                role="button"
                tabindex="0"
                onclick={() => handleServiceClick(service)}
                oncontextmenu={(e) => handleContextMenu(e, service)}>
              <td class="name-cell">{service.metadata.name}</td>
              <td>{service.metadata.namespace}</td>
              <td>
                <span class="type-badge type-{getServiceType(service).toLowerCase()}">
                  {getServiceType(service)}
                </span>
              </td>
              <td>{getClusterIP(service)}</td>
              <td>{getExternalIP(service)}</td>
              <td class="ports-cell">{getPorts(service)}</td>
              <td>{formatAge(service.metadata.creationTimestamp)}</td>
              <td class="actions-cell">
                <button class="action-btn" onclick={(e) => { e.stopPropagation(); handleContextMenu(e, service); }}>⚙️</button>
              </td>
            </tr>
          {/each}
        </svelte:fragment>
      </ResourceTable>
    {/if}
  {/if}
</div>

{#if contextMenuVisible}
  <QuickActionsMenu
    x={contextMenuPosition.x}
    y={contextMenuPosition.y}
    resource={selectedResource}
    resourceType="service"
    bind:visible={contextMenuVisible}
    on:close={closeContextMenu}
  />
{/if}

<style>
  .services-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: var(--spacing-md);
  }

  .panel-header {
    margin-bottom: var(--spacing-md);
  }

  .panel-header h4 {
    margin: 0;
    font-size: 1.2rem;
    color: var(--text-primary);
  }

  .loading-state, .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-xl);
    color: var(--text-secondary);
  }

  .spinner {
    width: 30px;
    height: 30px;
    border: 3px solid rgba(255, 255, 255, 0.1);
    border-top-color: var(--primary-color);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: var(--spacing-md);
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .name-cell {
    font-weight: 600;
    color: var(--primary-color);
  }

  .type-badge {
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  .type-clusterip { background: rgba(16, 185, 129, 0.1); color: #10b981; }
  .type-nodeport { background: rgba(245, 158, 11, 0.1); color: #f59e0b; }
  .type-loadbalancer { background: rgba(59, 130, 246, 0.1); color: #3b82f6; }
  .type-externalname { background: rgba(139, 92, 246, 0.1); color: #8b5cf6; }

  .ports-cell {
    max-width: 150px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .sortable {
    cursor: pointer;
  }

  .sortable:hover {
    color: var(--primary-color);
  }

  .action-btn {
    background: transparent;
    border: none;
    cursor: pointer;
    font-size: 1rem;
    padding: 4px;
    border-radius: 4px;
  }

  .action-btn:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  th {
    text-align: left;
    padding: var(--spacing-sm);
    color: var(--text-secondary);
    font-size: 0.85rem;
    font-weight: 600;
    border-bottom: 1px solid var(--border-primary);
  }

  td {
    padding: var(--spacing-sm);
    border-bottom: 1px solid var(--border-primary);
  }

  .resource-row {
    cursor: pointer;
  }

  .resource-row:hover {
    background: rgba(255, 255, 255, 0.03);
  }
</style>
