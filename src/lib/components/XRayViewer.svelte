<!-- Kuboard Resource X-Ray Viewer Component -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  // Props
  export let resource: any;
  export let resourceType: string;
  export let onClose: () => void = () => {};

  // State
  let graph: { nodes: any[], edges: any[] } = { nodes: [], edges: [] };
  let loading = true;
  let error: string | null = null;
  let svgContainer: SVGSVGElement;

  async function loadGraph() {
    loading = true;
    error = null;
    try {
      // For now, we fetch the whole namespace graph and filter it
      // In a more advanced version, we'd do a BFS/DFS from the root
      graph = await invoke('kuboard_get_resource_graph', {
        kind: resourceType,
        name: resource.metadata.name,
        namespace: resource.metadata.namespace
      });
      
      // Basic layout (very simple for now: columns by kind)
      layoutGraph();
    } catch (err: any) {
      error = String(err);
      console.error('Failed to load resource graph:', err);
    } finally {
      loading = false;
    }
  }

  // Simple column-based layout
  // Column 1: Deployments
  // Column 2: ReplicaSets
  // Column 3: Pods
  // Column 4: Services
  function layoutGraph() {
    const kindOrder = ['Deployment', 'StatefulSet', 'DaemonSet', 'ReplicaSet', 'Pod', 'Service', 'Ingress'];
    const columns: Record<string, any[]> = {};
    
    graph.nodes.forEach(node => {
      if (!columns[node.kind]) columns[node.kind] = [];
      columns[node.kind].push(node);
    });

    const activeKinds = kindOrder.filter(k => columns[k] && columns[k].length > 0);
    
    activeKinds.forEach((kind, colIndex) => {
      const nodes = columns[kind];
      const spacingY = 100;
      const offsetX = 100 + colIndex * 250;
      const startY = 100;
      
      nodes.forEach((node, i) => {
        node.x = offsetX;
        node.y = startY + i * spacingY;
      });
    });
    
    // Update graph to trigger reactivity
    graph = { ...graph };
  }

  onMount(() => {
    loadGraph();
  });

  function getKindIcon(kind: string) {
    const icons: Record<string, string> = {
      'Deployment': '🚀',
      'ReplicaSet': '🔄',
      'Pod': '📦',
      'Service': '🌐',
      'Ingress': '🌐',
      'StatefulSet': '💾',
      'DaemonSet': '👾'
    };
    return icons[kind] || '📄';
  }
</script>

<div class="xray-overlay">
  <div class="xray-container">
    <div class="xray-header">
      <div class="header-left">
        <h3>🔦 Resource X-Ray</h3>
        <span class="resource-info">{resourceType}: {resource.metadata.name}</span>
      </div>
      <button class="close-button" onclick={onClose}>×</button>
    </div>

    <div class="xray-body">
      {#if loading}
        <div class="loading-state">
          <div class="spinner"></div>
          <p>Analyzing resource relationships...</p>
        </div>
      {:else if error}
        <div class="error-state">
          <span class="error-icon">⚠️</span>
          <p>{error}</p>
        </div>
      {:else}
        <div class="graph-viewport">
          <svg bind:this={svgContainer} width="100%" height="100%" viewBox="0 0 1200 800">
            <!-- Defs for arrows -->
            <defs>
              <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
                <polygon points="0 0, 10 3.5, 0 7" fill="rgba(255,255,255,0.2)" />
              </marker>
            </defs>

            <!-- Edges -->
            {#each graph.edges as edge}
              {@const fromNode = graph.nodes.find(n => n.id === edge.from)}
              {@const toNode = graph.nodes.find(n => n.id === edge.to)}
              {#if fromNode && toNode && fromNode.x !== undefined && toNode.x !== undefined}
                <path 
                  d="M {fromNode.x + 80} {fromNode.y} C {fromNode.x + 160} {fromNode.y}, {toNode.x - 80} {toNode.y}, {toNode.x - 80} {toNode.y}"
                  stroke="rgba(255,255,255,0.1)"
                  stroke-width="2"
                  fill="none"
                  marker-end="url(#arrowhead)"
                />
              {/if}
            {/each}

            <!-- Nodes -->
            {#each graph.nodes as node}
              {#if node.x !== undefined}
                <g class="node-group" transform="translate({node.x - 80}, {node.y - 30})">
                  <rect 
                    width="160" 
                    height="60" 
                    rx="8" 
                    class="node-rect"
                    class:highlight={node.name === resource.metadata.name && node.kind.toLowerCase() === resourceType.toLowerCase()}
                  />
                  <text x="10" y="25" class="node-kind">{getKindIcon(node.kind)} {node.kind}</text>
                  <text x="10" y="45" class="node-name">{node.name.length > 18 ? node.name.substring(0, 15) + '...' : node.name}</text>
                  <circle cx="150" cy="30" r="4" class="status-dot status-{node.status.toLowerCase()}" />
                </g>
              {/if}
            {/each}
          </svg>
        </div>
      {/if}
    </div>

    <div class="xray-footer">
      <div class="legend">
        <div class="legend-item"><span class="line"></span> Relationship</div>
        <div class="legend-item"><span class="dot running"></span> Running</div>
        <div class="legend-item"><span class="rect highlight"></span> Current Resource</div>
      </div>
    </div>
  </div>
</div>

<style>
  @import '../styles/variables.css';

  .xray-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.85);
    backdrop-filter: blur(10px);
    z-index: 3000;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-xl);
  }

  .xray-container {
    width: 100%;
    max-width: 1400px;
    height: 85vh;
    background: var(--bg-secondary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-lg);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: var(--shadow-xxl);
  }

  .xray-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-md) var(--spacing-lg);
    border-bottom: 1px solid var(--border-primary);
    background: rgba(255, 255, 255, 0.03);
  }

  .header-left h3 {
    margin: 0;
    font-size: 1.2rem;
    color: white;
  }

  .resource-info {
    font-size: 0.85rem;
    color: var(--text-secondary);
    font-family: monospace;
  }

  .close-button {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 1.8rem;
    cursor: pointer;
    line-height: 1;
  }

  .xray-body {
    flex: 1;
    position: relative;
    overflow: hidden;
    background: radial-gradient(circle at center, #1a1a1a 0%, #0d0d0d 100%);
  }

  .graph-viewport {
    width: 100%;
    height: 100%;
    overflow: auto;
    cursor: grab;
  }

  .node-rect {
    fill: rgba(255, 255, 255, 0.05);
    stroke: rgba(255, 255, 255, 0.1);
    stroke-width: 1;
    transition: var(--transition-normal);
  }

  .node-rect.highlight {
    stroke: var(--primary-color);
    stroke-width: 2;
    fill: rgba(59, 130, 246, 0.1);
  }

  .node-group:hover .node-rect {
    fill: rgba(255, 255, 255, 0.1);
    stroke: rgba(255, 255, 255, 0.3);
  }

  .node-kind {
    fill: var(--text-secondary);
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
  }

  .node-name {
    fill: white;
    font-size: 12px;
    font-weight: 500;
  }

  .status-dot {
    stroke-width: 0;
  }

  .status-running, .status-ready, .status-active { fill: var(--success-color); }
  .status-failed, .status-error { fill: var(--error-color); }
  .status-pending { fill: var(--warning-color); }

  .loading-state, .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-secondary);
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid rgba(255, 255, 255, 0.1);
    border-top-color: var(--primary-color);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: var(--spacing-lg);
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .xray-footer {
    padding: var(--spacing-sm) var(--spacing-lg);
    background: rgba(0, 0, 0, 0.2);
    border-top: 1px solid var(--border-primary);
  }

  .legend {
    display: flex;
    gap: var(--spacing-xl);
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .legend-item .line {
    width: 20px;
    height: 2px;
    background: rgba(255, 255, 255, 0.2);
  }

  .legend-item .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .legend-item .dot.running { background: var(--success-color); }

  .legend-item .rect {
    width: 16px;
    height: 10px;
    border-radius: 2px;
    border: 1px solid rgba(255, 255, 255, 0.2);
  }

  .legend-item .rect.highlight { border-color: var(--primary-color); background: rgba(59, 130, 246, 0.1); }
</style>
