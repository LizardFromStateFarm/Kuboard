<!-- Kuboard Nodes Tab Component - Overhauled 2-Tier Architecture -->
<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import { formatMemory, formatCPU } from '$lib/utils/formatters';
  import MetricsGraph from './MetricsGraph.svelte';
  import { navigationStore } from '../stores/nav';
  import { 
    Server, Cpu, HardDrive, Layers, Search, Copy, 
    ExternalLink, RefreshCw, Sliders, CheckCircle2, AlertTriangle, 
    ArrowLeft, FileText, Activity, Loader2, Filter, X, Box, Check
  } from 'lucide-svelte';

  // Props
  export let currentContext: any = null;
  export let nodes: any[] = [];

  // State
  let selectedNode: any = null;
  let showFullDetails: boolean = false;
  let refreshTimer: any;
  let isLoading: boolean = false;
  let searchQuery: string = '';
  let selectedPoolFilter: string | null = null;
  let selectedPoolDetails: NodePoolItem | null = null;
  let poolNodeSearchQuery: string = '';
  let copiedNotice: string | null = null;

  // Metrics state
  let nodeMetrics: any = null;
  let metricsLoading: boolean = false;
  let metricsError: string | null = null;
  let selectedResourceType: 'cpu' | 'memory' | 'disk' = 'cpu';
  let selectedTimeRange: number = 30; // Default to 30 minutes

  const dispatch = createEventDispatcher();

  interface NodePoolItem {
    name: string;
    roleOrPool: string;
    nodeCount: number;
    readyCount: number;
    totalCpu: number;
    totalMemoryBytes: number;
    instanceType: string;
    nodes: any[];
  }

  $: if (currentContext && (!nodes || nodes.length === 0)) {
    isLoading = true;
  } else {
    isLoading = false;
  }

  $: if ($navigationStore && $navigationStore.tab === 'nodes' && $navigationStore.resourceName && nodes && nodes.length > 0) {
    const targetNodeName = $navigationStore.resourceName;
    const targetNode = nodes.find(n => n.metadata?.name === targetNodeName);
    if (targetNode) {
      selectNode(targetNode);
      showFullDetails = true;
    }
  }

  function getNodePool(node: any): string {
    const labels = node.metadata?.labels || {};
    return (
      labels['agentpool'] ||
      labels['eks.amazonaws.com/nodegroup'] ||
      labels['kops.k8s.io/instance_group'] ||
      labels['topology.kubernetes.io/zone'] ||
      labels['node.kubernetes.io/instance-type'] ||
      (labels['node-role.kubernetes.io/control-plane'] !== undefined ? 'Control Plane' : 'Worker Pool')
    );
  }

  function getNodeRole(node: any): string {
    const labels = node.metadata?.labels || {};
    if (labels['node-role.kubernetes.io/control-plane'] !== undefined || labels['node-role.kubernetes.io/master'] !== undefined) {
      return 'Control Plane';
    }
    return 'Worker';
  }

  function parseCpuCores(cpuStr: string | undefined): number {
    if (!cpuStr) return 0;
    if (cpuStr.endsWith('m')) return parseFloat(cpuStr.replace('m', '')) / 1000;
    return parseFloat(cpuStr) || 0;
  }

  function parseMemoryBytes(memStr: string | undefined): number {
    if (!memStr) return 0;
    if (memStr.endsWith('Ki')) return parseInt(memStr.replace('Ki', '')) * 1024;
    if (memStr.endsWith('Mi')) return parseInt(memStr.replace('Mi', '')) * 1024 * 1024;
    if (memStr.endsWith('Gi')) return parseInt(memStr.replace('Gi', '')) * 1024 * 1024 * 1024;
    return parseInt(memStr) || 0;
  }

  $: nodePools = (() => {
    if (!nodes || nodes.length === 0) return [];
    const map = new Map<string, NodePoolItem>();
    nodes.forEach(n => {
      const poolName = getNodePool(n);
      const isReady = n.status?.conditions?.some((c: any) => c.type === 'Ready' && c.status === 'True');
      const cpu = parseCpuCores(n.status?.capacity?.cpu);
      const mem = parseMemoryBytes(n.status?.capacity?.memory);
      const inst = n.metadata?.labels?.['node.kubernetes.io/instance-type'] || n.status?.nodeInfo?.architecture || 'Standard';

      if (!map.has(poolName)) {
        map.set(poolName, {
          name: poolName,
          roleOrPool: getNodeRole(n),
          nodeCount: 0,
          readyCount: 0,
          totalCpu: 0,
          totalMemoryBytes: 0,
          instanceType: inst,
          nodes: []
        });
      }
      const pool = map.get(poolName)!;
      pool.nodeCount += 1;
      if (isReady) pool.readyCount += 1;
      pool.totalCpu += cpu;
      pool.totalMemoryBytes += mem;
      pool.nodes.push(n);
    });
    return Array.from(map.values());
  })();

  $: filteredNodes = (nodes || []).filter(n => {
    const matchesPool = !selectedPoolFilter || getNodePool(n) === selectedPoolFilter;
    const q = searchQuery.toLowerCase();
    const name = (n.metadata?.name || '').toLowerCase();
    const pool = getNodePool(n).toLowerCase();
    const role = getNodeRole(n).toLowerCase();
    const ip = (n.status?.addresses?.find((a: any) => a.type === 'InternalIP')?.address || '').toLowerCase();
    const os = (n.status?.nodeInfo?.operatingSystem || '').toLowerCase();
    const kubelet = (n.status?.nodeInfo?.kubeletVersion || '').toLowerCase();

    const matchesSearch = !q || name.includes(q) || pool.includes(q) || role.includes(q) || ip.includes(q) || os.includes(q) || kubelet.includes(q);
    return matchesPool && matchesSearch;
  });

  async function copyText(text: string | undefined, label: string = 'Text') {
    if (!text) return;
    try {
      await navigator.clipboard.writeText(text);
      copiedNotice = `${label} copied!`;
      setTimeout(() => copiedNotice = null, 1500);
    } catch (err) {
      console.error('Copy failed:', err);
    }
  }

  function selectNode(node: any) {
    selectedNode = {
      name: node.metadata?.name || 'Unknown',
      status: node.status?.conditions?.find((c: any) => c.type === 'Ready')?.status || 'Unknown',
      os: node.status?.nodeInfo?.operatingSystem || 'Unknown',
      kernelVersion: node.status?.nodeInfo?.kernelVersion || 'Unknown',
      kubeletVersion: node.status?.nodeInfo?.kubeletVersion || 'Unknown',
      containerRuntime: node.status?.nodeInfo?.containerRuntimeVersion || 'Unknown',
      diskCapacity: parseInt(node.status?.capacity?.['ephemeral-storage']?.replace('Ki', '') || '0') * 1024,
      diskAllocatable: parseInt(node.status?.allocatable?.['ephemeral-storage']?.replace('Ki', '') || '0') * 1024,
      cpuCapacity: node.status?.capacity?.cpu || '0',
      memoryCapacity: parseInt(node.status?.capacity?.memory?.replace('Ki', '') || '0') * 1024,
      cpuAllocatable: node.status?.allocatable?.cpu || '0',
      memoryAllocatable: parseInt(node.status?.allocatable?.memory?.replace('Ki', '') || '0') * 1024,
      architecture: node.status?.nodeInfo?.architecture || 'Unknown',
      creationTimestamp: node.metadata?.creationTimestamp || 'Unknown',
      raw: node
    };
    dispatch('nodeSelect', selectedNode);
  }

  async function loadNodeMetrics(node: any) {
    if (!node?.metadata?.name) return;
    metricsLoading = true;
    metricsError = null;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const metrics = await invoke('kuboard_get_node_metrics_history', {
        nodeName: node.metadata.name,
        durationMinutes: selectedTimeRange
      });
      nodeMetrics = metrics;
    } catch (err: any) {
      metricsError = String(err);
    } finally {
      metricsLoading = false;
    }
  }

  function showFullNodeDetails(node: any) {
    selectNode(node);
    showFullDetails = true;
    loadNodeMetrics(node);
  }

  function openNodePoolDetails(pool: NodePoolItem) {
    selectedPoolDetails = pool;
    poolNodeSearchQuery = '';
  }

  function getFilteredPoolNodes(pool: NodePoolItem): any[] {
    if (!pool || !pool.nodes) return [];
    if (!poolNodeSearchQuery) return pool.nodes;
    const q = poolNodeSearchQuery.toLowerCase();
    return pool.nodes.filter(n => {
      const name = (n.metadata?.name || '').toLowerCase();
      const ip = (n.status?.addresses?.find((a: any) => a.type === 'InternalIP')?.address || '').toLowerCase();
      const kubelet = (n.status?.nodeInfo?.kubeletVersion || '').toLowerCase();
      const role = getNodeRole(n).toLowerCase();
      return name.includes(q) || ip.includes(q) || kubelet.includes(q) || role.includes(q);
    });
  }

  function backToNodesList() {
    showFullDetails = false;
    selectedNode = null;
    nodeMetrics = null;
    selectedResourceType = 'cpu';
  }

  function changeResourceType(type: 'cpu' | 'memory' | 'disk') { selectedResourceType = type; }
  function changeTimeRange(minutes: number) {
    selectedTimeRange = minutes;
    if (selectedNode) loadNodeMetrics(selectedNode);
  }

  function getStatusClass(status: string): string {
    switch (status?.toLowerCase()) {
      case 'ready':
      case 'true': return 'ready';
      case 'notready':
      case 'false': return 'not-ready';
      default: return 'unknown';
    }
  }

  onDestroy(() => {
    if (refreshTimer) clearInterval(refreshTimer);
  });
</script>

<div class="nodes-tab">
  {#if copiedNotice}
    <div class="copy-toast"><Check size={14} /> {copiedNotice}</div>
  {/if}

  {#if showFullDetails && selectedNode}
    <!-- Detailed Node View -->
    <div class="full-details-view">
      <div class="details-header">
        <button class="btn-back" onclick={backToNodesList}>
          <ArrowLeft size={14} class="inline-icon" /> {selectedPoolDetails ? `Back to Pool (${selectedPoolDetails.name})` : 'Back to All Nodes'}
        </button>
        <h3 class="node-details-title" onclick={() => copyText(selectedNode.name, 'Node Name')} title="Click to copy node name">
          <Server size={18} class="inline-icon" /> {selectedNode.name}
        </h3>
        <span class="status-badge status-{getStatusClass(selectedNode.status)}">
          {selectedNode.status === 'True' || selectedNode.status === 'Ready' ? 'Ready' : 'Not Ready'}
        </span>
      </div>
      
      <div class="node-details-content">
        <div class="details-section">
          <h6><Server size={14} class="inline-icon" /> System Information</h6>
          <div class="info-grid">
            <div class="info-item">
              <span class="info-label">OS</span>
              <span class="info-val">{selectedNode.os} ({selectedNode.architecture})</span>
            </div>
            <div class="info-item">
              <span class="info-label">Kernel</span>
              <span class="info-val">{selectedNode.kernelVersion}</span>
            </div>
            <div class="info-item">
              <span class="info-label">Kubelet</span>
              <span class="info-val">{selectedNode.kubeletVersion}</span>
            </div>
            <div class="info-item">
              <span class="info-label">Runtime</span>
              <span class="info-val">{selectedNode.containerRuntime}</span>
            </div>
          </div>
        </div>

        <div class="details-section">
          <h6><Activity size={14} class="inline-icon" /> Resource Allocation</h6>
          <div class="resource-grid">
            <div class="resource-item">
              <span class="resource-label">CPU Capacity / Allocatable</span>
              <span class="resource-val">{formatCPU(selectedNode.cpuAllocatable)} / {formatCPU(selectedNode.cpuCapacity)}</span>
            </div>
            <div class="resource-item">
              <span class="resource-label">Memory Capacity / Allocatable</span>
              <span class="resource-val">{formatMemory(selectedNode.memoryAllocatable)} / {formatMemory(selectedNode.memoryCapacity)}</span>
            </div>
            <div class="resource-item">
              <span class="resource-label">Disk Storage</span>
              <span class="resource-val">{formatMemory(selectedNode.diskAllocatable)} / {formatMemory(selectedNode.diskCapacity)}</span>
            </div>
          </div>
        </div>

        <!-- Historical Utilization Graph -->
        <div class="details-section">
          <div class="section-header-row">
            <h6><Activity size={14} class="inline-icon" /> Node Historical Utilization Graph</h6>
            <div class="controls-row">
              <select class="select-sm" bind:value={selectedTimeRange} onchange={() => changeTimeRange(selectedTimeRange)}>
                <option value={15}>15m</option>
                <option value={30}>30m</option>
                <option value={60}>1h</option>
                <option value={360}>6h</option>
                <option value={1440}>24h</option>
              </select>
              <div class="pill-toggle">
                <button class="pill-btn" class:active={selectedResourceType === 'cpu'} onclick={() => changeResourceType('cpu')}>CPU</button>
                <button class="pill-btn" class:active={selectedResourceType === 'memory'} onclick={() => changeResourceType('memory')}>RAM</button>
                <button class="pill-btn" class:active={selectedResourceType === 'disk'} onclick={() => changeResourceType('disk')}>Disk</button>
              </div>
            </div>
          </div>
          {#if metricsLoading}
            <div class="metrics-loading"><Loader2 size={16} class="spin" /><p>Loading node metrics...</p></div>
          {:else if nodeMetrics}
            <MetricsGraph 
              data={nodeMetrics} 
              type={selectedResourceType} 
              duration={selectedTimeRange} 
              loading={metricsLoading} 
              error={metricsError}
              maxCpuCores={parseFloat(selectedNode.cpuCapacity || '0')}
              maxMemoryBytes={selectedNode.memoryCapacity || 0}
              maxDiskBytes={selectedNode.diskCapacity || 0}
            />
          {:else}
            <p class="muted-text">No metrics history available for this node.</p>
          {/if}
        </div>
      </div>
    </div>
  {:else if selectedPoolDetails}
    <!-- Node Pool Details View -->
    <div class="full-details-view pool-details-view">
      <div class="details-header">
        <button class="btn-back" onclick={() => selectedPoolDetails = null}>
          <ArrowLeft size={14} class="inline-icon" /> Back to Node Pools
        </button>
        <h3 class="node-details-title clickable" onclick={() => copyText(selectedPoolDetails?.name, 'Node Pool Name')} title="Click to copy pool name">
          <Layers size={18} class="inline-icon" /> Node Pool: {selectedPoolDetails.name}
        </h3>
        <span class="status-badge status-ready">
          {selectedPoolDetails.readyCount} / {selectedPoolDetails.nodeCount} Ready
        </span>
      </div>

      <div class="node-details-content">
        <div class="details-section">
          <h6><Layers size={14} class="inline-icon" /> Node Pool Capacity & Summary</h6>
          <div class="resource-grid">
            <div class="resource-item">
              <span class="resource-label">Total Nodes</span>
              <span class="resource-val">{selectedPoolDetails.nodeCount} ({selectedPoolDetails.readyCount} Healthy)</span>
            </div>
            <div class="resource-item">
              <span class="resource-label">Aggregate CPU Capacity</span>
              <span class="resource-val">{selectedPoolDetails.totalCpu.toFixed(1)} Cores</span>
            </div>
            <div class="resource-item">
              <span class="resource-label">Aggregate RAM Capacity</span>
              <span class="resource-val">{formatMemory(selectedPoolDetails.totalMemoryBytes)}</span>
            </div>
            <div class="resource-item">
              <span class="resource-label">Role / Instance Type</span>
              <span class="resource-val">{selectedPoolDetails.roleOrPool} • {selectedPoolDetails.instanceType}</span>
            </div>
          </div>
        </div>

        <div class="details-section">
          <div class="section-header-row">
            <h6><Server size={14} class="inline-icon" /> Pool Member Nodes ({selectedPoolDetails.nodes.length})</h6>
            <div class="search-input-wrap compact-search">
              <Search size={14} class="search-icon" />
              <input 
                type="text" 
                bind:value={poolNodeSearchQuery} 
                placeholder="Search member nodes..." 
                class="node-search-input"
              />
              {#if poolNodeSearchQuery}
                <button class="clear-search-btn" onclick={() => poolNodeSearchQuery = ''}><X size={13} /></button>
              {/if}
            </div>
          </div>

          <div class="nodes-table-container">
            <table class="nodes-table">
              <thead>
                <tr>
                  <th>Status</th>
                  <th>Node Name</th>
                  <th>Internal IP</th>
                  <th>Kubelet</th>
                  <th>CPU Allocatable</th>
                  <th>RAM Allocatable</th>
                  <th class="actions-col">Actions</th>
                </tr>
              </thead>
              <tbody>
                {#each getFilteredPoolNodes(selectedPoolDetails) as node}
                  {@const isReady = node.status?.conditions?.some((c: any) => c.type === 'Ready' && c.status === 'True')}
                  {@const internalIp = node.status?.addresses?.find((a: any) => a.type === 'InternalIP')?.address || 'N/A'}
                  {@const cpuAlloc = parseCpuCores(node.status?.allocatable?.cpu)}
                  {@const cpuCap = parseCpuCores(node.status?.capacity?.cpu)}
                  {@const cpuPct = cpuCap > 0 ? Math.round((cpuAlloc / cpuCap) * 100) : 0}
                  {@const memAlloc = parseMemoryBytes(node.status?.allocatable?.memory)}
                  {@const memCap = parseMemoryBytes(node.status?.capacity?.memory)}
                  {@const memPct = memCap > 0 ? Math.round((memAlloc / memCap) * 100) : 0}

                  <tr class="node-row" onclick={() => showFullNodeDetails(node)}>
                    <td>
                      <span class="status-indicator {isReady ? 'ready' : 'not-ready'}">
                        {isReady ? 'Ready' : 'Not Ready'}
                      </span>
                    </td>
                    <td class="font-mono node-name-cell">
                      <Server size={14} class="inline-icon" /> {node.metadata?.name}
                    </td>
                    <td class="font-mono text-muted">
                      <span class="copyable-ip" onclick={(e) => { e.stopPropagation(); copyText(internalIp, 'Internal IP'); }} title="Click to copy IP">
                        {internalIp}
                      </span>
                    </td>
                    <td class="text-muted font-mono">{node.status?.nodeInfo?.kubeletVersion || '-'}</td>
                    <td>
                      <div class="capacity-bar-wrap">
                        <div class="capacity-bar">
                          <div class="capacity-fill cpu" style="width: {cpuPct}%"></div>
                        </div>
                        <span class="capacity-text">{formatCPU(node.status?.allocatable?.cpu)} / {formatCPU(node.status?.capacity?.cpu)} ({cpuPct}%)</span>
                      </div>
                    </td>
                    <td>
                      <div class="capacity-bar-wrap">
                        <div class="capacity-bar">
                          <div class="capacity-fill ram" style="width: {memPct}%"></div>
                        </div>
                        <span class="capacity-text">{formatMemory(memAlloc)} / {formatMemory(memCap)} ({memPct}%)</span>
                      </div>
                    </td>
                    <td class="actions-col" onclick={(e) => e.stopPropagation()}>
                      <button class="action-btn" onclick={() => showFullNodeDetails(node)} title="Inspect Node Details">
                        <FileText size={14} /> Details
                      </button>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>
  {:else}
    <!-- 2-Tier Node View -->
    <div class="nodes-master-view">
      <!-- Tier 1: Node Pools Cards -->
      <div class="pools-section">
        <div class="section-title-row">
          <h5><Layers size={16} class="inline-icon" /> Node Pools ({nodePools.length})</h5>
          {#if selectedPoolFilter}
            <button class="filter-pill-btn" onclick={() => selectedPoolFilter = null}>
              <Filter size={12} /> Filter: {selectedPoolFilter} <X size={12} />
            </button>
          {/if}
        </div>

        <div class="pools-grid">
          {#each nodePools as pool}
            <div 
              class="pool-card" 
              class:active={selectedPoolFilter === pool.name}
              onclick={() => openNodePoolDetails(pool)}
              role="button"
              tabindex="0"
              onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && openNodePoolDetails(pool)}
            >
              <div class="pool-card-header">
                <span class="pool-name" title={pool.name}><Server size={14} class="inline-icon" /> {pool.name}</span>
                <span class="pool-badge">{pool.readyCount}/{pool.nodeCount} Ready</span>
              </div>
              <div class="pool-card-metrics">
                <div class="pool-metric">
                  <span class="lbl"><Cpu size={12} /> CPU Cores</span>
                  <span class="val">{pool.totalCpu} Cores</span>
                </div>
                <div class="pool-metric">
                  <span class="lbl"><HardDrive size={12} /> RAM Capacity</span>
                  <span class="val">{formatMemory(pool.totalMemoryBytes)}</span>
                </div>
                <div class="pool-metric">
                  <span class="lbl"><Box size={12} /> Instance Type</span>
                  <span class="val">{pool.instanceType}</span>
                </div>
              </div>
              <div class="pool-card-footer">
                <button class="btn-inspect-pool" onclick={(e) => { e.stopPropagation(); openNodePoolDetails(pool); }}>
                  <Layers size={12} class="inline-icon" /> Inspect Pool Details →
                </button>
              </div>
            </div>
          {/each}
        </div>
      </div>

      <!-- Tier 2: Searchable All Nodes Table -->
      <div class="nodes-table-section">
        <div class="table-header-controls">
          <h5><Server size={16} class="inline-icon" /> Cluster Nodes ({filteredNodes.length})</h5>
          <div class="search-input-wrap">
            <Search size={14} class="search-icon" />
            <input 
              type="text" 
              bind:value={searchQuery} 
              placeholder="Search nodes by name, pool, role, IP, OS..." 
              class="node-search-input"
            />
            {#if searchQuery}
              <button class="clear-search-btn" onclick={() => searchQuery = ''}><X size={13} /></button>
            {/if}
          </div>
        </div>

        {#if isLoading}
          <div class="loading-nodes">
            <Loader2 size={24} class="spin" />
            <h5>Loading Cluster Nodes...</h5>
          </div>
        {:else if filteredNodes.length > 0}
          <div class="nodes-table-wrap">
            <table class="nodes-table">
              <thead>
                <tr>
                  <th>Node Name</th>
                  <th>Status</th>
                  <th>Node Pool / Role</th>
                  <th>Internal IP</th>
                  <th>CPU Allocatable</th>
                  <th>RAM Allocatable</th>
                  <th>Kubelet</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                {#each filteredNodes as node}
                  {@const isReady = node.status?.conditions?.some((c: any) => c.type === 'Ready' && c.status === 'True')}
                  {@const ip = node.status?.addresses?.find((a: any) => a.type === 'InternalIP')?.address || '-'}
                  {@const cpuAlloc = parseCpuCores(node.status?.allocatable?.cpu)}
                  {@const cpuCap = parseCpuCores(node.status?.capacity?.cpu)}
                  {@const memAlloc = parseMemoryBytes(node.status?.allocatable?.memory)}
                  {@const memCap = parseMemoryBytes(node.status?.capacity?.memory)}
                  <tr class="node-row" onclick={() => showFullNodeDetails(node)}>
                    <td class="node-name-cell">
                      <span class="node-click-link" title="Click to view details for {node.metadata?.name}">
                        <Server size={14} class="inline-icon" /> {node.metadata?.name || 'Unknown'}
                      </span>
                    </td>
                    <td>
                      <span class="status-pill status-{isReady ? 'ready' : 'not-ready'}">
                        {isReady ? 'Ready' : 'Not Ready'}
                      </span>
                    </td>
                    <td>
                      <span class="pool-tag">{getNodePool(node)}</span>
                    </td>
                    <td>
                      <button 
                        class="btn-copy-ip" 
                        onclick={(e) => { e.stopPropagation(); copyText(ip, 'Node IP'); }} 
                        title="Click to copy IP"
                      >
                        <Copy size={12} class="inline-icon" /> {ip}
                      </button>
                    </td>
                    <td>
                      <div class="gauge-bar-wrap">
                        <span class="gauge-label">{formatCPU(node.status?.allocatable?.cpu || '0')} / {formatCPU(node.status?.capacity?.cpu || '0')}</span>
                        <div class="gauge-bar"><div class="gauge-fill" style="width: {cpuCap > 0 ? (cpuAlloc / cpuCap) * 100 : 0}%"></div></div>
                      </div>
                    </td>
                    <td>
                      <div class="gauge-bar-wrap">
                        <span class="gauge-label">{formatMemory(memAlloc)} / {formatMemory(memCap)}</span>
                        <div class="gauge-bar"><div class="gauge-fill ram" style="width: {memCap > 0 ? (memAlloc / memCap) * 100 : 0}%"></div></div>
                      </div>
                    </td>
                    <td>{node.status?.nodeInfo?.kubeletVersion || '-'}</td>
                    <td>
                      <button 
                        class="btn-table-action" 
                        onclick={(e) => { e.stopPropagation(); showFullNodeDetails(node); }}
                        title="View Node Details"
                      >
                        <ExternalLink size={13} /> Details
                      </button>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {:else}
          <div class="no-nodes-message">
            <Server size={32} class="muted-icon" />
            <h5>No Nodes Match Search</h5>
            <p>Try clearing filters or search query.</p>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  @import '../styles/variables.css';
  @import '../styles/color-palette.css';

  .nodes-tab {
    padding: var(--spacing-sm) 0;
    display: flex;
    flex-direction: column;
    gap: 16px;
    position: relative;
  }

  .copy-toast {
    position: fixed;
    bottom: 20px;
    right: 20px;
    background: #10b981;
    color: #ffffff;
    padding: 8px 14px;
    border-radius: 6px;
    font-size: 0.85rem;
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 6px;
    z-index: 9999;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  .nodes-master-view {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  /* Pools Tier */
  .pools-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .section-title-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .section-title-row h5 {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 700;
    color: var(--text-primary);
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .filter-pill-btn {
    background: rgba(59, 130, 246, 0.15);
    border: 1px solid var(--primary-color);
    color: var(--primary-color);
    border-radius: 14px;
    padding: 4px 10px;
    font-size: 0.78rem;
    font-weight: 600;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .pools-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 12px;
  }
  .pool-card {
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    padding: 14px;
    cursor: pointer;
    transition: transform 0.15s ease, border-color 0.15s ease, background 0.15s ease;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .pool-card:hover, .pool-card.active {
    border-color: var(--primary-color);
    background: rgba(59, 130, 246, 0.08);
    transform: translateY(-2px);
  }
  .pool-card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .pool-name {
    font-size: 0.88rem;
    font-weight: 700;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .pool-badge {
    background: rgba(16, 185, 129, 0.2);
    color: #34d399;
    border-radius: 10px;
    padding: 2px 8px;
    font-size: 0.72rem;
    font-weight: 700;
  }
  .pool-card-metrics {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .pool-metric {
    display: flex;
    justify-content: space-between;
    font-size: 0.78rem;
  }
  .pool-metric .lbl { color: var(--text-muted); display: flex; align-items: center; gap: 4px; }
  .pool-metric .val { color: var(--text-primary); font-weight: 600; }

  .pool-card-footer {
    margin-top: 4px;
    padding-top: 8px;
    border-top: 1px dashed rgba(255, 255, 255, 0.08);
  }
  .btn-inspect-pool {
    width: 100%;
    background: rgba(59, 130, 246, 0.12);
    border: 1px solid rgba(59, 130, 246, 0.25);
    color: #60a5fa;
    padding: 5px 8px;
    border-radius: 4px;
    font-size: 0.78rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
  }
  .btn-inspect-pool:hover {
    background: rgba(59, 130, 246, 0.25);
    border-color: rgba(59, 130, 246, 0.5);
    color: #93c5fd;
  }

  .compact-search {
    width: 240px;
  }

  /* Nodes Table Section */
  .nodes-table-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .table-header-controls {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }
  .table-header-controls h5 {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 700;
    color: var(--text-primary);
  }
  .search-input-wrap {
    position: relative;
    display: flex;
    align-items: center;
    min-width: 280px;
  }
  .search-input-wrap .search-icon {
    position: absolute;
    left: 10px;
    color: var(--text-muted);
  }
  .node-search-input {
    width: 100%;
    background: var(--background-secondary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    padding: 6px 30px 6px 30px;
    color: var(--text-primary);
    font-size: 0.82rem;
  }
  .node-search-input:focus {
    border-color: var(--primary-color);
    outline: none;
  }
  .clear-search-btn {
    position: absolute;
    right: 8px;
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
  }

  .nodes-table-wrap {
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    overflow-x: auto;
  }
  .nodes-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.84rem;
  }
  .nodes-table th {
    background: var(--background-secondary);
    padding: 10px 12px;
    text-align: left;
    color: var(--text-muted);
    font-weight: 700;
    font-size: 0.76rem;
    text-transform: uppercase;
    border-bottom: 1px solid var(--border-primary);
  }
  .nodes-table td {
    padding: 10px 12px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    color: var(--text-primary);
  }
  .node-row {
    cursor: pointer;
    transition: background 0.15s ease;
  }
  .node-row:hover {
    background: rgba(255, 255, 255, 0.04);
  }
  .node-click-link {
    color: var(--primary-color);
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .status-pill {
    padding: 2px 8px;
    border-radius: 10px;
    font-size: 0.74rem;
    font-weight: 700;
  }
  .status-pill.status-ready { background: rgba(16, 185, 129, 0.15); color: #34d399; }
  .status-pill.status-not-ready { background: rgba(239, 68, 68, 0.15); color: #f87171; }
  .pool-tag {
    background: var(--background-secondary);
    border: 1px solid var(--border-primary);
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 0.76rem;
    color: var(--text-muted);
  }
  .btn-copy-ip {
    background: transparent;
    border: none;
    color: var(--text-primary);
    cursor: pointer;
    font-family: monospace;
    font-size: 0.8rem;
  }
  .btn-copy-ip:hover { color: var(--primary-color); }
  .btn-table-action {
    background: var(--background-secondary);
    border: 1px solid var(--border-primary);
    color: var(--text-primary);
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 0.78rem;
    cursor: pointer;
  }
  .btn-table-action:hover {
    border-color: var(--primary-color);
    color: var(--primary-color);
  }

  .gauge-bar-wrap { display: flex; flex-direction: column; gap: 3px; min-width: 110px; }
  .gauge-label { font-size: 0.72rem; color: var(--text-muted); }
  .gauge-bar { width: 100%; height: 5px; background: rgba(255, 255, 255, 0.1); border-radius: 3px; overflow: hidden; }
  .gauge-fill { height: 100%; background: #3b82f6; border-radius: 3px; }
  .gauge-fill.ram { background: #8b5cf6; }

  /* Full Details View */
  .full-details-view { display: flex; flex-direction: column; gap: 16px; }
  .details-header { display: flex; align-items: center; gap: 14px; }
  .btn-back {
    background: var(--background-secondary);
    border: 1px solid var(--border-primary);
    color: var(--text-primary);
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    font-size: 0.82rem;
    cursor: pointer;
  }
  .node-details-title { margin: 0; font-size: 1.1rem; color: var(--text-primary); cursor: pointer; }
  .details-section {
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .details-section h6 { margin: 0; font-size: 0.9rem; font-weight: 700; color: var(--text-primary); }
  .info-grid, .resource-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 12px;
  }
  .info-item, .resource-item { display: flex; flex-direction: column; gap: 2px; }
  .info-label, .resource-label { font-size: 0.74rem; color: var(--text-muted); text-transform: uppercase; font-weight: 700; }
  .info-val, .resource-val { font-size: 0.86rem; color: var(--text-primary); font-weight: 600; }
  .section-header-row { display: flex; justify-content: space-between; align-items: center; }
  .controls-row { display: flex; align-items: center; gap: 10px; }
  .pill-toggle { display: flex; background: var(--background-secondary); padding: 2px; border-radius: 6px; }
  .pill-btn { background: transparent; border: none; color: var(--text-muted); padding: 4px 10px; font-size: 0.78rem; cursor: pointer; border-radius: 4px; }
  .pill-btn.active { background: var(--primary-color); color: #ffffff; font-weight: 700; }
  .no-nodes-message, .loading-nodes, .metrics-loading {
    display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 40px 20px; gap: 10px; color: var(--text-muted);
  }
  .inline-icon { display: inline-block; vertical-align: middle; }
</style>