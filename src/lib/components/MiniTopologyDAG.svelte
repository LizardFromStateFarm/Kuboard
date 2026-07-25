<!-- Kuboard Resource Mini-Topology DAG Component -->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let resource: any;
  export let resourceType: string = 'pod';

  const dispatch = createEventDispatcher();

  interface NodeItem {
    id: string;
    kind: string;
    name: string;
    namespace?: string;
    status?: string;
    icon: string;
    role: 'parent' | 'current' | 'child';
  }

  $: nodes = buildTopologyNodes(resource, resourceType);

  function buildTopologyNodes(res: any, type: string): NodeItem[] {
    if (!res) return [];
    const result: NodeItem[] = [];
    const ns = res.metadata?.namespace || 'default';
    const name = res.metadata?.name || 'unknown';

    if (type === 'pod') {
      // Parents (Owner References)
      const owners = res.metadata?.ownerReferences || [];
      if (owners.length > 0) {
        owners.forEach((owner: any) => {
          result.push({
            id: `parent-${owner.uid || owner.name}`,
            kind: owner.kind,
            name: owner.name,
            namespace: ns,
            status: 'Active',
            icon: owner.kind === 'ReplicaSet' ? '🔄' : owner.kind === 'StatefulSet' ? '📦' : '⚙️',
            role: 'parent'
          });
        });
      } else {
        result.push({
          id: 'parent-none',
          kind: 'Standalone',
          name: 'Unmanaged Pod',
          namespace: ns,
          status: 'Active',
          icon: '📌',
          role: 'parent'
        });
      }

      // Current Pod
      result.push({
        id: `current-${name}`,
        kind: 'Pod',
        name,
        namespace: ns,
        status: res.status?.phase || 'Running',
        icon: '🐳',
        role: 'current'
      });

      // Children / Bound Dependencies (Volumes, ConfigMaps, Secrets, PVCs)
      const volumes = res.spec?.volumes || [];
      volumes.forEach((v: any) => {
        if (v.configMap) {
          result.push({
            id: `cm-${v.configMap.name}`,
            kind: 'ConfigMap',
            name: v.configMap.name,
            namespace: ns,
            icon: '⚙️',
            role: 'child'
          });
        } else if (v.secret) {
          result.push({
            id: `sec-${v.secret.secretName}`,
            kind: 'Secret',
            name: v.secret.secretName,
            namespace: ns,
            icon: '🔒',
            role: 'child'
          });
        } else if (v.persistentVolumeClaim) {
          result.push({
            id: `pvc-${v.persistentVolumeClaim.claimName}`,
            kind: 'PVC',
            name: v.persistentVolumeClaim.claimName,
            namespace: ns,
            icon: '💾',
            role: 'child'
          });
        }
      });
    } else if (type === 'deployment') {
      // Deployment -> ReplicaSet -> Pods
      result.push({
        id: `current-${name}`,
        kind: 'Deployment',
        name,
        namespace: ns,
        status: `${res.status?.readyReplicas || 0}/${res.status?.replicas || 0} Ready`,
        icon: '🚀',
        role: 'current'
      });
    } else if (type === 'service') {
      // Service -> Selector Pods / Endpoints
      result.push({
        id: `current-${name}`,
        kind: 'Service',
        name,
        namespace: ns,
        status: res.spec?.type || 'ClusterIP',
        icon: '🔌',
        role: 'current'
      });
    }

    return result;
  }

  function handleNodeClick(node: NodeItem) {
    dispatch('navigate', { kind: node.kind, name: node.name, namespace: node.namespace });
  }
</script>

<div class="mini-topology-dag">
  <div class="dag-header">
    <h5>🌐 Resource Topology DAG & Live Links</h5>
    <span class="dag-subtitle">Owner hierarchy & bound volume/config dependencies</span>
  </div>

  <div class="dag-graph-view">
    <!-- Parent Nodes Section -->
    <div class="dag-column parents-col">
      <span class="col-title">Controller Parent</span>
      {#each nodes.filter(n => n.role === 'parent') as node}
        <button class="node-card parent-card" onclick={() => handleNodeClick(node)}>
          <span class="node-icon">{node.icon}</span>
          <div class="node-meta">
            <span class="node-kind">{node.kind}</span>
            <span class="node-name" title={node.name}>{node.name}</span>
          </div>
        </button>
      {/each}
    </div>

    <!-- Arrow connector -->
    <div class="connector-arrow">➔</div>

    <!-- Current Resource Node -->
    <div class="dag-column current-col">
      <span class="col-title">Current Resource</span>
      {#each nodes.filter(n => n.role === 'current') as node}
        <div class="node-card current-card">
          <span class="node-icon">{node.icon}</span>
          <div class="node-meta">
            <span class="node-kind">{node.kind}</span>
            <span class="node-name" title={node.name}>{node.name}</span>
            {#if node.status}
              <span class="node-status">{node.status}</span>
            {/if}
          </div>
        </div>
      {/each}
    </div>

    <!-- Arrow connector -->
    <div class="connector-arrow">➔</div>

    <!-- Children / Dependencies Column -->
    <div class="dag-column children-col">
      <span class="col-title">Bound Config & Volume Dependencies</span>
      {#each nodes.filter(n => n.role === 'child') as node}
        <button class="node-card child-card" onclick={() => handleNodeClick(node)}>
          <span class="node-icon">{node.icon}</span>
          <div class="node-meta">
            <span class="node-kind">{node.kind}</span>
            <span class="node-name" title={node.name}>{node.name}</span>
          </div>
        </button>
      {/each}

      {#if nodes.filter(n => n.role === 'child').length === 0}
        <span class="no-deps-text">No external ConfigMap/Secret/PVC mounts attached.</span>
      {/if}
    </div>
  </div>
</div>

<style>
  .mini-topology-dag {
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .dag-header { display: flex; justify-content: space-between; align-items: center; }
  .dag-header h5 { margin: 0; color: var(--text-primary); font-size: 0.95rem; font-weight: 700; }
  .dag-subtitle { font-size: 0.8rem; color: var(--text-muted); }
  .dag-graph-view {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    overflow-x: auto;
    padding: 8px 0;
  }
  .dag-column {
    display: flex;
    flex-direction: column;
    gap: 8px;
    flex: 1;
    min-width: 180px;
  }
  .col-title { font-size: 0.75rem; color: var(--text-muted); text-transform: uppercase; font-weight: 700; }
  .connector-arrow { color: var(--primary-color); font-size: 1.2rem; font-weight: 700; user-select: none; }
  .node-card {
    display: flex;
    align-items: center;
    gap: 10px;
    background: var(--background-secondary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    padding: 10px 12px;
    text-align: left;
    transition: transform 0.15s ease, border-color 0.15s ease;
  }
  button.node-card { cursor: pointer; }
  button.node-card:hover { transform: translateY(-2px); border-color: var(--primary-color); }
  .node-card.current-card { border-color: var(--primary-color); background: rgba(59, 130, 246, 0.12); }
  .node-icon { font-size: 1.2rem; }
  .node-meta { display: flex; flex-direction: column; gap: 2px; overflow: hidden; }
  .node-kind { font-size: 0.72rem; color: var(--text-muted); text-transform: uppercase; font-weight: 700; }
  .node-name { font-size: 0.85rem; color: var(--text-primary); font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .node-status { font-size: 0.75rem; color: #4ade80; font-weight: 700; }
  .no-deps-text { font-size: 0.8rem; color: var(--text-muted); font-style: italic; }
</style>
