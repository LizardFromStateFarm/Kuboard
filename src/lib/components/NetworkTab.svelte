<!-- Kuboard Network Tab Component -->
<script lang="ts">
  import ServicesPanel from './ServicesPanel.svelte';
  import IngressesPanel from './IngressesPanel.svelte';
  import IngressClassesPanel from './IngressClassesPanel.svelte';
  import NetworkPoliciesPanel from './NetworkPoliciesPanel.svelte';
  import { Globe, ArrowRightLeft, Sliders, Shield } from 'lucide-svelte';

  // Props
  export let currentContext: any = null;
  export let namespace: string = 'all';
  export let tabSessionId: string = 'tab-default';

  // State
  let sessionTabMap: Record<string, 'services' | 'ingresses' | 'ingressclasses' | 'networkpolicies'> = {};
  $: activeTab = sessionTabMap[tabSessionId] || 'services';

  function setActiveTab(tab: any) {
    sessionTabMap[tabSessionId] = tab;
    sessionTabMap = { ...sessionTabMap };
  }
</script>

<div class="network-tab">
  <div class="tab-navigation">
    <button 
      class="nav-item" 
      class:active={activeTab === 'services'} 
      onclick={() => setActiveTab('services')}
    >
      <Globe size={15} /> Services
    </button>
    <button 
      class="nav-item" 
      class:active={activeTab === 'ingresses'} 
      onclick={() => setActiveTab('ingresses')}
    >
      <ArrowRightLeft size={15} /> Ingresses
    </button>
    <button 
      class="nav-item" 
      class:active={activeTab === 'ingressclasses'} 
      onclick={() => setActiveTab('ingressclasses')}
    >
      <Sliders size={15} /> Ingress Classes
    </button>
    <button 
      class="nav-item" 
      class:active={activeTab === 'networkpolicies'} 
      onclick={() => setActiveTab('networkpolicies')}
    >
      <Shield size={15} /> Network Policies
    </button>
  </div>

  <div class="tab-content">
    {#if activeTab === 'services'}
      <ServicesPanel {currentContext} {namespace} />
    {:else if activeTab === 'ingresses'}
      <IngressesPanel {currentContext} {namespace} />
    {:else if activeTab === 'ingressclasses'}
      <IngressClassesPanel {currentContext} />
    {:else if activeTab === 'networkpolicies'}
      <NetworkPoliciesPanel {currentContext} {namespace} />
    {/if}
  </div>
</div>

<style>
  .network-tab {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .tab-navigation {
    display: flex;
    gap: var(--spacing-sm);
    padding: var(--spacing-md);
    background: rgba(255, 255, 255, 0.02);
    border-bottom: 1px solid var(--border-primary);
    overflow-x: auto;
  }

  .nav-item {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    padding: var(--spacing-xs) var(--spacing-sm);
    cursor: pointer;
    font-size: 0.9rem;
    font-weight: 500;
    transition: all 0.2s ease;
    border-radius: var(--radius-sm);
    white-space: nowrap;
  }

  .nav-item:hover {
    color: var(--text-primary);
    background: rgba(255, 255, 255, 0.05);
  }

  .nav-item.active {
    color: var(--primary-color);
    background: rgba(59, 130, 246, 0.1);
  }

  .tab-content {
    flex: 1;
    overflow-y: auto;
  }
</style>
