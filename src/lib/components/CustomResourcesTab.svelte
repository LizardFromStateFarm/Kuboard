<!-- Kuboard Custom Resources Tab Component -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  // Props
  export let currentContext: any = null;

  // State
  let customResourceDefinitions: any[] = [];
  let customResources: any[] = [];
  
  let loading: boolean = false;
  let error: string | null = null;
  let lastUpdate: string = '';

  // Load custom resources data
  async function loadCustomResources() {
    if (!currentContext || loading) return;
    
    loading = true;
    error = null;
    
    try {
      // For now, we'll show a placeholder since custom resources require more complex API calls
      // In a real implementation, you'd call APIs to get CRDs and custom resources
      customResourceDefinitions = [];
      customResources = [];
      
      lastUpdate = new Date().toLocaleTimeString();
    } catch (err) {
      error = err as string;
      console.error('Failed to load custom resources:', err);
    } finally {
      loading = false;
    }
  }

  // Lifecycle
  onMount(() => {
    loadCustomResources();
  });

  // Reactive updates
  $: if (currentContext) {
    loadCustomResources();
  }
</script>

<div class="custom-resources-tab">
  <div class="tab-header">
    <h4>🔧 Custom Resources</h4>
    <div class="tab-controls">
      <button 
        class="refresh-button" 
        onclick={loadCustomResources}
        disabled={loading}
        title="Refresh custom resources"
      >
        {#if loading}
          🔄
        {:else}
          ↻
        {/if}
      </button>
      {#if lastUpdate}
        <span class="last-update">Last: {lastUpdate}</span>
      {/if}
    </div>
  </div>

  {#if error}
    <div class="error-message">
      <div class="error-icon">⚠️</div>
      <div class="error-content">
        <h5>Failed to load custom resources</h5>
        <p>{error}</p>
        <button class="retry-button" onclick={loadCustomResources}>
          Retry
        </button>
      </div>
    </div>
  {:else if loading}
    <div class="loading-message">
      <div class="loading-spinner">🔄</div>
      <p>Loading custom resources...</p>
    </div>
  {:else}
    <div class="custom-resources-content">
      <!-- Custom Resource Definitions Section -->
      <div class="crd-section">
        <div class="section-header">
          <h5>Custom Resource Definitions (CRDs)</h5>
        </div>
        <div class="coming-soon">
          <div class="coming-soon-icon">🚧</div>
          <h6>Coming Soon</h6>
          <p>Custom Resource Definitions will be available in a future update. This will allow you to view and manage custom resources defined in your cluster.</p>
          <div class="feature-list">
            <div class="feature-item">
              <span class="feature-icon">📋</span>
              <span>View CRD definitions and schemas</span>
            </div>
            <div class="feature-item">
              <span class="feature-icon">🔍</span>
              <span>Browse custom resource instances</span>
            </div>
            <div class="feature-item">
              <span class="feature-icon">⚙️</span>
              <span>Manage custom resource lifecycle</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Custom Resources Section -->
      <div class="custom-resources-section">
        <div class="section-header">
          <h5>Custom Resource Instances</h5>
        </div>
        <div class="coming-soon">
          <div class="coming-soon-icon">🚧</div>
          <h6>Coming Soon</h6>
          <p>Custom resource instances will be available in a future update. This will allow you to view and manage instances of custom resources defined by CRDs.</p>
          <div class="feature-list">
            <div class="feature-item">
              <span class="feature-icon">📊</span>
              <span>View custom resource status and details</span>
            </div>
            <div class="feature-item">
              <span class="feature-icon">🏷️</span>
              <span>Filter and search custom resources</span>
            </div>
            <div class="feature-item">
              <span class="feature-icon">📝</span>
              <span>Edit custom resource configurations</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  /* Import CSS variables */
  @import '../styles/variables.css';

  .custom-resources-tab {
    background: rgba(255, 255, 255, 0.03);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .tab-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-md);
    padding-bottom: var(--spacing-sm);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .tab-header h4 {
    margin: 0;
    color: white;
    font-size: 1.1rem;
    font-weight: 600;
  }

  .tab-controls {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .refresh-button {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-sm);
    color: white;
    cursor: pointer;
    font-size: 0.9rem;
    padding: 6px 12px;
    transition: var(--transition-normal);
  }

  .refresh-button:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.2);
    border-color: rgba(255, 255, 255, 0.3);
  }

  .refresh-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .last-update {
    font-size: 0.8rem;
    color: rgba(255, 255, 255, 0.6);
  }

  .error-message {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: var(--radius-md);
  }

  .error-icon {
    font-size: 1.2rem;
    flex-shrink: 0;
  }

  .error-content h5 {
    margin: 0 0 4px 0;
    color: white;
    font-size: 0.9rem;
    font-weight: 600;
  }

  .error-content p {
    margin: 0 0 8px 0;
    color: rgba(255, 255, 255, 0.8);
    font-size: 0.85rem;
  }

  .retry-button {
    background: var(--error-color);
    border: none;
    border-radius: var(--radius-sm);
    color: white;
    cursor: pointer;
    font-size: 0.8rem;
    padding: 4px 8px;
    transition: var(--transition-normal);
  }

  .retry-button:hover {
    background: #dc2626;
  }

  .loading-message {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-xl);
    color: rgba(255, 255, 255, 0.8);
  }

  .loading-spinner {
    font-size: 1.5rem;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .custom-resources-content {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  .crd-section, .custom-resources-section {
    background: rgba(255, 255, 255, 0.02);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .section-header {
    margin-bottom: var(--spacing-md);
  }

  .section-header h5 {
    margin: 0;
    color: white;
    font-size: 1rem;
    font-weight: 600;
  }

  .coming-soon {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-xl);
    color: rgba(255, 255, 255, 0.8);
    text-align: center;
  }

  .coming-soon-icon {
    font-size: 3rem;
    opacity: 0.7;
  }

  .coming-soon h6 {
    margin: 0;
    color: white;
    font-size: 1.1rem;
    font-weight: 600;
  }

  .coming-soon p {
    margin: 0;
    font-size: 0.9rem;
    line-height: 1.5;
    max-width: 500px;
  }

  .feature-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    margin-top: var(--spacing-md);
    text-align: left;
  }

  .feature-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm);
    background: rgba(255, 255, 255, 0.03);
    border-radius: var(--radius-sm);
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .feature-icon {
    font-size: 1.2rem;
    flex-shrink: 0;
  }

  .feature-item span:last-child {
    color: rgba(255, 255, 255, 0.9);
    font-size: 0.9rem;
  }

  /* Responsive Design */
  @media (max-width: 768px) {
    .custom-resources-content {
      gap: var(--spacing-md);
    }
    
    .coming-soon {
      padding: var(--spacing-lg);
    }
    
    .feature-list {
      gap: var(--spacing-xs);
    }
    
    .feature-item {
      padding: var(--spacing-xs);
    }
  }
</style>
