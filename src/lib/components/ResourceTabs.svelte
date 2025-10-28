<!-- Kuboard Resource Tabs Component -->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  // Props
  export let activeTab: string = 'workloads';
  export let tabs: Array<{
    id: string;
    label: string;
    icon: string;
    count?: number;
    disabled?: boolean;
  }> = [
    { id: 'workloads', label: 'Workloads', icon: '⚙️', count: 0 },
    { id: 'config', label: 'Config', icon: '⚙️', count: 0 },
    { id: 'network', label: 'Network', icon: '🌐', count: 0 },
    { id: 'storage', label: 'Storage', icon: '💾', count: 0 },
    { id: 'custom', label: 'Custom Resources', icon: '🔧', count: 0 },
    { id: 'security', label: 'Security', icon: '🔒', count: 0 }
  ];

  // Events
  const dispatch = createEventDispatcher();

  // Tab click handler
  function selectTab(tabId: string) {
    if (tabs.find(t => t.id === tabId)?.disabled) return;
    
    activeTab = tabId;
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
  <div class="tabs-header">
    <h3>📋 Resource Management</h3>
    <div class="tabs-summary">
      <span class="summary-text">
        {tabs.filter(t => !t.disabled).length} categories available
      </span>
    </div>
  </div>
  
  <div class="tabs-list">
    {#each tabs as tab}
      <button
        class="tab-button"
        class:active={activeTab === tab.id}
        class:disabled={tab.disabled}
        onclick={() => selectTab(tab.id)}
        title={tab.disabled ? 'Coming soon' : tab.label}
      >
        <span class="tab-icon">{tab.icon}</span>
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
    background: rgba(255, 255, 255, 0.05);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
    border: 1px solid rgba(255, 255, 255, 0.1);
    margin-bottom: var(--spacing-lg);
  }

  .tabs-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-md);
    padding-bottom: var(--spacing-sm);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .tabs-header h3 {
    margin: 0;
    color: white;
    font-size: 1.2rem;
    font-weight: 600;
  }

  .tabs-summary {
    display: flex;
    align-items: center;
  }

  .summary-text {
    color: rgba(255, 255, 255, 0.7);
    font-size: 0.9rem;
    font-weight: 500;
  }

  .tabs-list {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-sm);
  }

  .tab-button {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-md) var(--spacing-lg);
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: var(--radius-md);
    color: rgba(255, 255, 255, 0.8);
    cursor: pointer;
    transition: var(--transition-normal);
    font-size: 0.9rem;
    font-weight: 500;
    min-width: 140px;
    justify-content: center;
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
    font-size: 1.1rem;
    flex-shrink: 0;
  }

  .tab-label {
    flex: 1;
    text-align: center;
  }

  .tab-count {
    background: rgba(255, 255, 255, 0.2);
    color: white;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    font-size: 0.8rem;
    font-weight: 600;
    min-width: 20px;
    text-align: center;
  }

  .tab-button.active .tab-count {
    background: rgba(255, 255, 255, 0.3);
  }

  /* Responsive Design */
  @media (max-width: 768px) {
    .tabs-list {
      flex-direction: column;
    }
    
    .tab-button {
      min-width: auto;
      width: 100%;
    }
    
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
