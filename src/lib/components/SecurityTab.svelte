<!-- Kuboard Security Tab Component -->
<script lang="ts">
  import RolesPanel from './RolesPanel.svelte';
  import ClusterRolesPanel from './ClusterRolesPanel.svelte';
  import RoleBindingsPanel from './RoleBindingsPanel.svelte';
  import ClusterRoleBindingsPanel from './ClusterRoleBindingsPanel.svelte';
  import ServiceAccountsPanel from './ServiceAccountsPanel.svelte';
  import SecretsPanel from './SecretsPanel.svelte';

  // Props
  export let currentContext: any = null;
  export let namespace: string = 'all';

  // State
  let activeTab: 'secrets' | 'roles' | 'clusterroles' | 'rolebindings' | 'clusterrolebindings' | 'serviceaccounts' = 'secrets';

  function setActiveTab(tab: any) {
    activeTab = tab;
  }
</script>

<div class="security-tab">
  <div class="tab-navigation">
    <button 
      class="nav-item" 
      class:active={activeTab === 'secrets'} 
      onclick={() => setActiveTab('secrets')}
    >
      🔒 Secrets
    </button>
    <button 
      class="nav-item" 
      class:active={activeTab === 'roles'} 
      onclick={() => setActiveTab('roles')}
    >
      Roles
    </button>
    <button 
      class="nav-item" 
      class:active={activeTab === 'clusterroles'} 
      onclick={() => setActiveTab('clusterroles')}
    >
      Cluster Roles
    </button>
    <button 
      class="nav-item" 
      class:active={activeTab === 'rolebindings'} 
      onclick={() => setActiveTab('rolebindings')}
    >
      Role Bindings
    </button>
    <button 
      class="nav-item" 
      class:active={activeTab === 'clusterrolebindings'} 
      onclick={() => setActiveTab('clusterrolebindings')}
    >
      Cluster Role Bindings
    </button>
    <button 
      class="nav-item" 
      class:active={activeTab === 'serviceaccounts'} 
      onclick={() => setActiveTab('serviceaccounts')}
    >
      Service Accounts
    </button>
  </div>

  <div class="tab-content">
    {#if activeTab === 'secrets'}
      <SecretsPanel {currentContext} {namespace} />
    {:else if activeTab === 'roles'}
      <RolesPanel {currentContext} {namespace} />
    {:else if activeTab === 'clusterroles'}
      <ClusterRolesPanel {currentContext} />
    {:else if activeTab === 'rolebindings'}
      <RoleBindingsPanel {currentContext} {namespace} />
    {:else if activeTab === 'clusterrolebindings'}
      <ClusterRoleBindingsPanel {currentContext} />
    {:else if activeTab === 'serviceaccounts'}
      <ServiceAccountsPanel {currentContext} {namespace} />
    {/if}
  </div>
</div>

<style>
  .security-tab {
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
