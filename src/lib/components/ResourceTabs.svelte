<!-- Kuboard Resource Tabs Component -->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { 
    Activity, Boxes, Server, Sliders, Globe, Database, Wrench, Stethoscope, Shield, Layers 
  } from 'lucide-svelte';

  // Props
  export let activeTab: string = 'overview';
  export let tabs: Array<{
    id: string;
    label: string;
    icon: any;
    count?: number;
    disabled?: boolean;
  }> = [
    { id: 'overview', label: 'Cluster Details', icon: Activity, count: 0 },
    { id: 'workloads', label: 'Workloads', icon: Boxes, count: 0 },
    { id: 'nodes', label: 'Nodes', icon: Server, count: 0 },
    { id: 'config', label: 'Config', icon: Sliders, count: 0 },
    { id: 'network', label: 'Network', icon: Globe, count: 0 },
    { id: 'storage', label: 'Storage', icon: Database, count: 0 },
    { id: 'custom', label: 'Custom Resources', icon: Wrench, count: 0 },
    { id: 'linter', label: 'Linter', icon: Stethoscope, count: 0 },
    { id: 'security', label: 'Security', icon: Shield, count: 0 }
  ];

  // Events
  const dispatch = createEventDispatcher();

  // Tab click handler
  function selectTab(tabId: string) {
    if (tabs.find(t => t.id === tabId)?.disabled) {
      return;
    }
    
    dispatch('tabChange', { tabId });
  }

  // Update tab counts
  export function updateTabCount(tabId: string, count: number) {
    const tab = tabs.find(t => t.id === tabId);
    if (tab) {
      tab.count = count;
      tabs = [...tabs]; // Trigger reactivity
    }
  }
</script>

<div class="resource-tabs-container">
  <div class="tabs-list">
    {#each tabs as tab}
      <button
        class="tab-button"
        class:active={activeTab === tab.id}
        class:disabled={tab.disabled}
        onclick={() => selectTab(tab.id)}
        title={tab.disabled ? 'Coming soon' : tab.label}
      >
        <span class="tab-icon">
          {#if typeof tab.icon === 'string'}
            {tab.icon}
          {:else}
            <svelte:component this={tab.icon} size={16} />
          {/if}
        </span>
        <span class="tab-label">{tab.label}</span>
        {#if tab.count !== undefined && tab.count > 0}
          <span class="tab-count">{tab.count}</span>
        {/if}
      </button>
    {/each}
  </div>
</div>

<style>
  /* Import CSS variables */
  @import '../styles/variables.css';

  .resource-tabs-container {
    padding: 2px 5px;
    margin-bottom: 5px;
  }

  .tabs-list {
    display: flex;
    flex-wrap: nowrap;
    gap: 6px;
    width: 100%;
    overflow-x: auto;
    scrollbar-width: thin;
  }

  .tabs-list::-webkit-scrollbar {
    height: 3px;
  }

  .tabs-list::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.2);
    border-radius: 2px;
  }

  .tab-button {
    flex: 1 1 0px;
    min-width: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    padding: 6px 10px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: var(--radius-md);
    color: rgba(255, 255, 255, 0.8);
    cursor: pointer;
    transition: var(--transition-normal);
    font-size: 0.85rem;
    font-weight: 500;
    white-space: nowrap;
  }

  .tab-button:hover:not(.disabled) {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.2);
    color: white;
    transform: translateY(-1px);
  }

  .tab-button.active {
    background: var(--primary-color);
    border-color: var(--primary-color);
    color: white;
    box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
  }

  .tab-button.disabled {
    opacity: 0.5;
    cursor: not-allowed;
    background: rgba(255, 255, 255, 0.02);
    border-color: rgba(255, 255, 255, 0.05);
  }

  .tab-icon {
    font-size: 1rem;
    flex-shrink: 0;
    display: flex;
    align-items: center;
  }

  .tab-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
    text-align: center;
  }

  .tab-count {
    background: rgba(255, 255, 255, 0.2);
    color: white;
    padding: 1px 5px;
    border-radius: var(--radius-sm);
    font-size: 0.75rem;
    font-weight: 600;
    min-width: 18px;
    text-align: center;
    flex-shrink: 0;
  }

  .tab-button.active .tab-count {
    background: rgba(255, 255, 255, 0.3);
  }

  /* Responsive Design */
  @media (max-width: 768px) {
    .tabs-header {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--spacing-sm);
    }
  }

  @media (max-width: 480px) {
    .resource-tabs-container {
      padding: var(--spacing-md);
    }
    
    .tab-button {
      padding: var(--spacing-sm) var(--spacing-md);
      font-size: 0.85rem;
    }
  }
</style>
