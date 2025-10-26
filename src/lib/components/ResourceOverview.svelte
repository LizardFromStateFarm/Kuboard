<!-- Kuboard Resource Overview Component -->
<script lang="ts">
  import type { ExpandedPanel } from '../types/index.js';
  import { createEventDispatcher } from 'svelte';

  // Props
  export let nodes: any[] = [];
  export let namespaces: any[] = [];
  export let pods: any[] = [];
  export let deployments: any[] = [];
  export let expandedPanel: ExpandedPanel = null;
  export let resourceLoading: boolean = false;

  // Events
  const dispatch = createEventDispatcher();

  // Event handlers
  function togglePanel(panel: ExpandedPanel) {
    dispatch('panelToggle', panel);
  }

  function selectResource(type: string, resource: any) {
    dispatch('resourceSelect', { type, resource });
  }

  // Helper functions
  function formatDate(dateString: string): string {
    return new Date(dateString).toLocaleDateString();
  }

  function getStatusClass(status: string): string {
    return status.toLowerCase().replace(/\s+/g, '-');
  }
</script>

<section class="resource-overview">
  <h2>Resource Overview</h2>
  <div class="resource-grid">
    <!-- Nodes Panel -->
    <div 
      class="resource-panel" 
      class:expanded={expandedPanel === 'nodes'}
      role="button"
      tabindex="0"
      onclick={() => togglePanel('nodes')}
      onkeydown={(e) => e.key === 'Enter' || e.key === ' ' ? togglePanel('nodes') : null}
    >
      <div class="panel-content">
        <div class="panel-icon">🖥️</div>
        <div class="panel-title">Nodes</div>
        <div class="panel-count">{nodes.length}</div>
        <div class="panel-subtitle">
          {expandedPanel === 'nodes' ? 'Click to collapse' : 'Click to expand'}
        </div>
      </div>
      
      {#if expandedPanel === 'nodes'}
        <div class="resource-details">
          {#if resourceLoading}
            <div class="loading-state">Loading nodes...</div>
          {:else}
            <h4>Nodes ({nodes.length})</h4>
            <div class="resource-list">
              {#each nodes as node}
                <div 
                  class="resource-item" 
                  role="button"
                  tabindex="0"
                  onclick={() => selectResource('node', node)}
                  onkeydown={(e) => e.key === 'Enter' || e.key === ' ' ? selectResource('node', node) : null}
                >
                  <div class="resource-name">{node.metadata?.name || 'Unknown'}</div>
                  <div class="resource-info">
                    <span class="status-badge status-{getStatusClass(node.status?.conditions?.find(c => c.type === 'Ready')?.status || 'Unknown')}">
                      {node.status?.conditions?.find(c => c.type === 'Ready')?.status || 'Unknown'}
                    </span>
                    <span class="resource-detail">
                      Kubelet: {node.status?.nodeInfo?.kubeletVersion || 'Unknown'}
                    </span>
                    <span class="resource-detail">
                      OS: {node.status?.nodeInfo?.operatingSystem || 'Unknown'}
                    </span>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    </div>
    
    <!-- Namespaces Panel -->
    <div 
      class="resource-panel" 
      class:expanded={expandedPanel === 'namespaces'}
      role="button"
      tabindex="0"
      onclick={() => togglePanel('namespaces')}
      onkeydown={(e) => e.key === 'Enter' || e.key === ' ' ? togglePanel('namespaces') : null}
    >
      <div class="panel-content">
        <div class="panel-icon">📁</div>
        <div class="panel-title">Namespaces</div>
        <div class="panel-count">{namespaces.length}</div>
        <div class="panel-subtitle">
          {expandedPanel === 'namespaces' ? 'Click to collapse' : 'Click to expand'}
        </div>
      </div>
      
      {#if expandedPanel === 'namespaces'}
        <div class="resource-details">
          {#if resourceLoading}
            <div class="loading-state">Loading namespaces...</div>
          {:else}
            <h4>Namespaces ({namespaces.length})</h4>
            <div class="resource-list">
              {#each namespaces as namespace}
                <div 
                  class="resource-item" 
                  role="button"
                  tabindex="0"
                  onclick={() => selectResource('namespace', namespace)}
                  onkeydown={(e) => e.key === 'Enter' || e.key === ' ' ? selectResource('namespace', namespace) : null}
                >
                  <div class="resource-name">{namespace.metadata?.name || 'Unknown'}</div>
                  <div class="resource-info">
                    <span class="resource-detail">
                      Created: {formatDate(namespace.metadata?.creationTimestamp || '')}
                    </span>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    </div>
    
    <!-- Pods Panel -->
    <div 
      class="resource-panel" 
      class:expanded={expandedPanel === 'pods'}
      role="button"
      tabindex="0"
      onclick={() => togglePanel('pods')}
      onkeydown={(e) => e.key === 'Enter' || e.key === ' ' ? togglePanel('pods') : null}
    >
      <div class="panel-content">
        <div class="panel-icon">🚀</div>
        <div class="panel-title">Pods</div>
        <div class="panel-count">{pods.length}</div>
        <div class="panel-subtitle">
          {expandedPanel === 'pods' ? 'Click to collapse' : 'Click to expand'}
        </div>
      </div>
      
      {#if expandedPanel === 'pods'}
        <div class="resource-details">
          {#if resourceLoading}
            <div class="loading-state">Loading pods...</div>
          {:else}
            <h4>Pods ({pods.length})</h4>
            <div class="resource-list">
              {#each pods as pod}
                <div 
                  class="resource-item" 
                  role="button"
                  tabindex="0"
                  onclick={() => selectResource('pod', pod)}
                  onkeydown={(e) => e.key === 'Enter' || e.key === ' ' ? selectResource('pod', pod) : null}
                >
                  <div class="resource-name">{pod.metadata?.name || 'Unknown'}</div>
                  <div class="resource-info">
                    <span class="status-badge status-{getStatusClass(pod.status?.phase || 'Unknown')}">
                      {pod.status?.phase || 'Unknown'}
                    </span>
                    <span class="resource-detail">
                      Namespace: {pod.metadata?.namespace || 'Unknown'}
                    </span>
                    <span class="resource-detail">
                      Node: {pod.spec?.nodeName || 'Unknown'}
                    </span>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    </div>
    
    <!-- Deployments Panel -->
    <div 
      class="resource-panel" 
      class:expanded={expandedPanel === 'deployments'}
      role="button"
      tabindex="0"
      onclick={() => togglePanel('deployments')}
      onkeydown={(e) => e.key === 'Enter' || e.key === ' ' ? togglePanel('deployments') : null}
    >
      <div class="panel-content">
        <div class="panel-icon">⚙️</div>
        <div class="panel-title">Deployments</div>
        <div class="panel-count">{deployments.length}</div>
        <div class="panel-subtitle">
          {expandedPanel === 'deployments' ? 'Click to collapse' : 'Click to expand'}
        </div>
      </div>
      
      {#if expandedPanel === 'deployments'}
        <div class="resource-details">
          {#if resourceLoading}
            <div class="loading-state">Loading deployments...</div>
          {:else}
            <h4>Deployments ({deployments.length})</h4>
            <div class="resource-list">
              {#each deployments as deployment}
                <div 
                  class="resource-item" 
                  role="button"
                  tabindex="0"
                  onclick={() => selectResource('deployment', deployment)}
                  onkeydown={(e) => e.key === 'Enter' || e.key === ' ' ? selectResource('deployment', deployment) : null}
                >
                  <div class="resource-name">{deployment.metadata?.name || 'Unknown'}</div>
                  <div class="resource-info">
                    <span class="status-badge status-{getStatusClass(deployment.status?.conditions?.find(c => c.type === 'Available')?.status || 'Unknown')}">
                      {deployment.status?.conditions?.find(c => c.type === 'Available')?.status || 'Unknown'}
                    </span>
                    <span class="resource-detail">
                      Namespace: {deployment.metadata?.namespace || 'Unknown'}
                    </span>
                    <span class="resource-detail">
                      Replicas: {deployment.status?.readyReplicas || 0}/{deployment.spec?.replicas || 0}
                    </span>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </div>
</section>

<style>
  /* Import CSS variables */
  @import '../styles/variables.css';

  .resource-overview {
    margin: var(--spacing-xl) 0;
  }

  .resource-overview h2 {
    color: var(--text-color);
    margin-bottom: var(--spacing-lg);
    font-size: 1.5em;
    font-weight: 600;
  }

  .resource-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: var(--spacing-lg);
    margin: var(--spacing-lg) 0;
  }

  /* Resource Panel Styles */
  .resource-panel {
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: var(--transition-slow);
    overflow: hidden;
    background: var(--card-background);
    position: relative;
  }

  .resource-panel:hover {
    border-color: var(--primary-color);
    box-shadow: 0 2px 8px rgba(16, 185, 129, 0.2);
    transform: translateY(-2px);
  }

  .resource-panel.expanded {
    border-color: var(--primary-color);
    box-shadow: 0 4px 12px rgba(16, 185, 129, 0.3);
  }

  .panel-content {
    text-align: center;
    padding: var(--spacing-lg);
    position: relative;
  }

  .panel-icon {
    font-size: 2.5em;
    margin-bottom: var(--spacing-sm);
  }

  .panel-title {
    font-size: 1.2em;
    font-weight: 600;
    color: var(--text-color);
    margin-bottom: var(--spacing-sm);
  }

  .panel-count {
    font-size: 2em;
    font-weight: bold;
    color: var(--primary-color);
    margin-bottom: var(--spacing-sm);
  }

  .panel-subtitle {
    font-size: 0.8em;
    margin-top: var(--spacing-sm);
    opacity: 0.7;
    color: var(--text-muted);
  }

  .resource-details {
    border-top: 1px solid var(--border-color);
    background: var(--surface-color);
    max-height: 400px;
    overflow-y: auto;
    padding: var(--spacing-md);
  }

  .resource-details h4 {
    margin: 0 0 var(--spacing-md) 0;
    color: var(--text-color);
    font-size: 1.1em;
    font-weight: 600;
  }

  .loading-state {
    text-align: center;
    padding: var(--spacing-lg);
    color: var(--text-muted);
    font-style: italic;
  }

  .resource-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .resource-item {
    padding: var(--spacing-md);
    background: rgba(255, 255, 255, 0.03);
    border-radius: var(--radius-sm);
    border: 1px solid rgba(255, 255, 255, 0.05);
    cursor: pointer;
    transition: var(--transition-normal);
  }

  .resource-item:hover {
    background: rgba(255, 255, 255, 0.08);
    border-color: var(--primary-color);
    transform: translateX(4px);
  }

  .resource-name {
    font-weight: 600;
    color: var(--text-color);
    margin-bottom: var(--spacing-xs);
    font-size: 0.95em;
  }

  .resource-info {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .status-badge {
    display: inline-block;
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    font-size: 0.8em;
    font-weight: 600;
    text-transform: capitalize;
    width: fit-content;
  }

  .status-ready, .status-running, .status-available {
    background: rgba(16, 185, 129, 0.2);
    color: var(--success-color);
  }

  .status-not-ready, .status-pending, .status-unknown {
    background: rgba(245, 158, 11, 0.2);
    color: var(--warning-color);
  }

  .status-failed, .status-error {
    background: rgba(239, 68, 68, 0.2);
    color: var(--error-color);
  }

  .resource-detail {
    color: var(--text-muted);
    font-size: 0.85em;
    line-height: 1.4;
  }

  /* Scrollbar Styling */
  .resource-details::-webkit-scrollbar {
    width: 6px;
  }

  .resource-details::-webkit-scrollbar-track {
    background: rgba(255, 255, 255, 0.05);
    border-radius: 3px;
  }

  .resource-details::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.2);
    border-radius: 3px;
  }

  .resource-details::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.3);
  }

  /* Responsive Design */
  @media (max-width: 768px) {
    .resource-grid {
      grid-template-columns: 1fr;
    }
    
    .panel-content {
      padding: var(--spacing-md);
    }
    
    .panel-icon {
      font-size: 2em;
    }
    
    .panel-count {
      font-size: 1.5em;
    }
  }

  @media (max-width: 480px) {
    .resource-grid {
      grid-template-columns: 1fr;
      gap: var(--spacing-md);
    }
    
    .resource-details {
      max-height: 300px;
    }
  }
</style>
