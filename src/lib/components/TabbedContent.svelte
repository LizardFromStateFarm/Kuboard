<!-- Kuboard Tabbed Content Component -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import ResourceTabs from './ResourceTabs.svelte';
  import WorkloadsTab from './WorkloadsTab.svelte';
  import NodesTab from './NodesTab.svelte';
  import ConfigTab from './ConfigTab.svelte';
  import NetworkTab from './NetworkTab.svelte';
  import CustomResourcesTab from './CustomResourcesTab.svelte';
  import StorageTab from './StorageTab.svelte';
  import SecurityTab from './SecurityTab.svelte';
  import HelmTab from './HelmTab.svelte';
  import LinterTab from './LinterTab.svelte';
  import EventsPanel from './EventsPanel.svelte';
  import ClusterMetrics from './ClusterMetrics.svelte';
  import { navigationStore } from '../stores/nav';
  import { Activity, Boxes, Server, Sliders, Globe, Database, Wrench, Stethoscope, Shield } from 'lucide-svelte';

  // Props
  export let currentContext: any = null;
  export let nodes: any[] = [];
  export let tabSessionId: string = 'tab-default';

  // Per-session active tab map
  let sessionTabMap: Record<string, string> = {};
  $: activeTab = sessionTabMap[tabSessionId] || 'overview';
  let tabCounts: Record<string, number> = {};

  // Tab definitions
  const tabs = [
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

  // Tab change handler
  function handleTabChange(event: CustomEvent<{ tabId: string }>) {
    const newTab = event.detail.tabId;
    console.log(`🔄 TabbedContent tab change for session [${tabSessionId}]:`, newTab);
    sessionTabMap[tabSessionId] = newTab;
    sessionTabMap = { ...sessionTabMap };
    // Clear navigationStore so past resource cross-tab navigations don't override manual user tab clicks
    navigationStore.set(null);
  }

  // Update tab counts
  function updateTabCount(tabId: string, count: number) {
    tabCounts[tabId] = count;
  }

  // Load initial counts
  async function loadInitialCounts() {
    if (!currentContext) return;

    try {
      // Load counts for available tabs
      const [pods, deployments, services, nodes, configmaps, secrets, pvcs, helm] = await Promise.all([
        invoke('kuboard_get_pods').catch(() => []),
        invoke('kuboard_get_deployments').catch(() => []),
        invoke('kuboard_get_services').catch(() => []),
        invoke('kuboard_get_nodes').catch(() => []),
        invoke('kuboard_get_configmaps').catch(() => []),
        invoke('kuboard_get_secrets').catch(() => []),
        invoke('kuboard_list_persistent_volume_claims').catch(() => []),
        invoke('kuboard_list_helm_releases').catch(() => [])
      ]);

      // Update tab counts
      updateTabCount('workloads', (pods as any[]).length + (deployments as any[]).length + (services as any[]).length);
      updateTabCount('nodes', (nodes as any[]).length);
      updateTabCount('config', (configmaps as any[]).length + (secrets as any[]).length);
      updateTabCount('network', (services as any[]).length);
      updateTabCount('storage', (pvcs as any[]).length);
      updateTabCount('helm', (helm as any[]).length);
    } catch (error) {
      console.error('Failed to load initial counts:', error);
    }
  }

  // Reactive updates
  $: if (currentContext) {
    loadInitialCounts();
  }

  let lastProcessedNavTarget: any = null;

  // Watch navigation store
  $: if ($navigationStore && $navigationStore !== lastProcessedNavTarget) {
    lastProcessedNavTarget = $navigationStore;
    const nav = $navigationStore;
    if (nav.tab) {
      const targetSession = (nav as any).tabSessionId || tabSessionId;
      console.log(`🔄 TabbedContent: Navigating session [${targetSession}] to:`, nav.tab);
      const workloadSubTabs = ['pods', 'deployments', 'statefulsets', 'daemonsets', 'cronjobs', 'replicasets', 'services', 'pod', 'deployment', 'statefulset', 'daemonset', 'cronjob', 'replicaset', 'service'];
      const targetTabLower = nav.tab.toLowerCase();

      if (workloadSubTabs.includes(targetTabLower)) {
        sessionTabMap[targetSession] = 'workloads';
      } else {
        sessionTabMap[targetSession] = nav.tab;
      }
      sessionTabMap = { ...sessionTabMap };
    }
  }
</script>

<div class="tabbed-content">
  <ResourceTabs 
    {tabs}
    {activeTab}
    on:tabChange={handleTabChange}
  />

  <div class="tab-content">
    <!-- Use display: none instead of conditional rendering to prevent layout shifts -->
    <div class="tab-panel" class:active={activeTab === 'workloads'}>
      <WorkloadsTab {currentContext} {tabSessionId} />
    </div>
    <div class="tab-panel" class:active={activeTab === 'overview'}>
      <div class="cluster-details-tab-content">
        <ClusterMetrics {nodes} refreshInterval={10000} autoRefresh={true} />
        <div class="cluster-events-section" style="margin-top: 24px;">
          <EventsPanel {currentContext} />
        </div>
      </div>
    </div>
    <div class="tab-panel" class:active={activeTab === 'nodes'}>
      <NodesTab {currentContext} {nodes} />
    </div>
    <div class="tab-panel" class:active={activeTab === 'config'}>
      <ConfigTab {currentContext} {tabSessionId} />
    </div>
    <div class="tab-panel" class:active={activeTab === 'network'}>
      <NetworkTab {currentContext} {tabSessionId} />
    </div>
    <div class="tab-panel" class:active={activeTab === 'storage'}>
      <StorageTab {currentContext} {tabSessionId} />
    </div>
    <div class="tab-panel" class:active={activeTab === 'custom'}>
      <CustomResourcesTab {currentContext} />
    </div>
    <div class="tab-panel" class:active={activeTab === 'linter'}>
      <LinterTab {currentContext} />
    </div>
    <div class="tab-panel" class:active={activeTab === 'security'}>
      <SecurityTab {currentContext} {tabSessionId} />
    </div>
  </div>
</div>

<style>
  /* Import CSS variables */
  @import '../styles/variables.css';

  .tabbed-content {
    padding: 0;
    margin-bottom: 5px;
  }

  .tab-content {
    margin-top: 0;
    position: relative;
    min-height: 200px;
  }

  .tab-panel {
    display: none;
    opacity: 0;
    transition: opacity 0.2s ease-in-out;
    animation: fadeIn 0.2s ease-in-out;
    will-change: opacity;
    contain: layout style paint;
  }

  .tab-panel.active {
    display: block;
    opacity: 1;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .coming-soon-tab {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 300px;
  }

  .coming-soon-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-xl);
    text-align: center;
    color: rgba(255, 255, 255, 0.8);
  }

  .coming-soon-icon {
    font-size: 4rem;
    opacity: 0.7;
  }

  .coming-soon-content h4 {
    margin: 0;
    color: white;
    font-size: 1.5rem;
    font-weight: 600;
  }

  .coming-soon-content p {
    margin: 0;
    font-size: 1rem;
    line-height: 1.5;
    max-width: 500px;
  }

  /* Responsive Design */
  @media (max-width: 768px) {
    .tabbed-content {
      padding: var(--spacing-md);
    }
    
    .coming-soon-content {
      padding: var(--spacing-lg);
    }
    
    .coming-soon-icon {
      font-size: 3rem;
    }
    
    .coming-soon-content h4 {
      font-size: 1.3rem;
    }
    
    .coming-soon-content p {
      font-size: 0.9rem;
    }
  }
</style>
