<!-- Kuboard Settings & Grafana Integration Modal -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Settings, Palette, Activity, CheckCircle2, AlertTriangle, Loader2, Database, Key, Globe, Check } from 'lucide-svelte';

  // Active Tab State
  let activeTab: 'appearance' | 'grafana' = 'appearance';

  // Theme state
  let currentTheme: string = 'dark';
  let isVisible: boolean = false;

  // Grafana state
  let grafanaUrl: string = 'http://localhost:3000';
  let grafanaToken: string = '';
  let grafanaStatus: 'none' | 'testing' | 'success' | 'error' = 'none';
  let grafanaError: string | null = null;
  let datasources: any[] = [];
  let selectedDatasource: string = '';
  let isSavedNotice: boolean = false;

  // Available themes
  const themes = [
    { id: 'dark', name: 'Dark', description: 'Default dark theme' },
    { id: 'light', name: 'Light', description: 'Light theme' },
    { id: 'high-contrast', name: 'High Contrast', description: 'High contrast theme' }
  ];

  // Apply theme
  function applyTheme(themeId: string) {
    currentTheme = themeId;
    document.documentElement.setAttribute('data-theme', themeId);
    localStorage.setItem('kuboard-theme', themeId);
  }

  // Toggle visibility
  function toggleVisibility() {
    isVisible = !isVisible;
  }

  // Grafana IPC Handlers
  async function testGrafanaConnection() {
    if (!grafanaUrl.trim()) return;
    grafanaStatus = 'testing';
    grafanaError = null;
    try {
      const isConnected = await invoke('kuboard_grafana_test_connection', {
        url: grafanaUrl.trim(),
        apiToken: grafanaToken.trim() || null
      }) as boolean;

      if (isConnected) {
        grafanaStatus = 'success';
        await discoverDatasources();
      } else {
        grafanaStatus = 'error';
        grafanaError = 'Grafana health check returned unsuccessful status.';
      }
    } catch (err: any) {
      grafanaStatus = 'error';
      grafanaError = String(err);
    }
  }

  async function discoverDatasources() {
    try {
      const res = await invoke('kuboard_grafana_discover_datasources', {
        url: grafanaUrl.trim(),
        apiToken: grafanaToken.trim() || null
      }) as any[];
      if (Array.isArray(res)) {
        datasources = res;
      }
    } catch (err) {
      console.warn('Datasource discovery skipped:', err);
    }
  }

  function saveGrafanaSettings() {
    localStorage.setItem('kuboard-grafana-url', grafanaUrl.trim());
    localStorage.setItem('kuboard-grafana-token', grafanaToken.trim());
    localStorage.setItem('kuboard-grafana-ds', selectedDatasource);
    isSavedNotice = true;
    setTimeout(() => isSavedNotice = false, 2000);
  }

  // Load saved theme & Grafana settings on mount
  onMount(() => {
    const savedTheme = localStorage.getItem('kuboard-theme') || 'dark';
    applyTheme(savedTheme);

    const savedUrl = localStorage.getItem('kuboard-grafana-url');
    if (savedUrl) grafanaUrl = savedUrl;

    const savedToken = localStorage.getItem('kuboard-grafana-token');
    if (savedToken) grafanaToken = savedToken;

    const savedDs = localStorage.getItem('kuboard-grafana-ds');
    if (savedDs) selectedDatasource = savedDs;
  });
</script>

<div class="theme-switcher-container">
  <button 
    class="settings-btn" 
    onclick={toggleVisibility}
    title="Settings & Appearance"
  >
    <Settings size={16} />
  </button>
  
  {#if isVisible}
    <div class="settings-backdrop" onclick={toggleVisibility} role="button" tabindex="-1"></div>
    <div class="theme-panel settings-dialog">
      <div class="panel-header">
        <h4><Settings size={16} class="inline-icon" /> Kuboard Settings & Preferences</h4>
        <button class="close-btn" onclick={toggleVisibility}>✕</button>
      </div>

      <!-- Settings Sub-Tabs -->
      <div class="settings-nav-tabs">
        <button 
          class="nav-tab-btn" 
          class:active={activeTab === 'appearance'} 
          onclick={() => activeTab = 'appearance'}
        >
          <Palette size={14} class="inline-icon" /> Appearance & Theme
        </button>
        <button 
          class="nav-tab-btn" 
          class:active={activeTab === 'grafana'} 
          onclick={() => activeTab = 'grafana'}
        >
          <Activity size={14} class="inline-icon" /> Grafana Integration
        </button>
      </div>

      {#if activeTab === 'appearance'}
        <div class="settings-tab-content">
          <p class="panel-sub">Select color theme:</p>
          <div class="theme-options">
            {#each themes as theme}
              <button 
                class="theme-option"
                class:active={currentTheme === theme.id}
                onclick={() => applyTheme(theme.id)}
              >
                <div class="theme-preview" data-theme={theme.id}>
                  <div class="preview-primary"></div>
                  <div class="preview-success"></div>
                  <div class="preview-warning"></div>
                  <div class="preview-error"></div>
                </div>
                <div class="theme-info">
                  <div class="theme-name">{theme.name}</div>
                  <div class="theme-description">{theme.description}</div>
                </div>
              </button>
            {/each}
          </div>
        </div>
      {:else if activeTab === 'grafana'}
        <div class="settings-tab-content grafana-settings">
          <p class="panel-sub">Configure local Grafana or Grafana Cloud for PromQL time-series metrics:</p>
          
          <div class="form-group">
            <label for="grafana-url"><Globe size={13} class="inline-icon" /> Grafana Endpoint URL</label>
            <input 
              id="grafana-url"
              type="text" 
              bind:value={grafanaUrl} 
              placeholder="e.g. http://localhost:3000 or https://my-org.grafana.net" 
              class="settings-input"
            />
          </div>

          <div class="form-group">
            <label for="grafana-token"><Key size={13} class="inline-icon" /> API Bearer Token / Service Account Key</label>
            <input 
              id="grafana-token"
              type="password" 
              bind:value={grafanaToken} 
              placeholder="glsa_... or Bearer Token (optional for local unauthenticated Grafana)" 
              class="settings-input"
            />
          </div>

          {#if datasources.length > 0}
            <div class="form-group">
              <label for="grafana-ds"><Database size={13} class="inline-icon" /> Target Prometheus / Thanos Datasource</label>
              <select id="grafana-ds" bind:value={selectedDatasource} class="settings-select">
                <option value="">Default Prometheus Datasource</option>
                {#each datasources as ds}
                  <option value={ds.id}>{ds.name} ({ds.type})</option>
                {/each}
              </select>
            </div>
          {/if}

          {#if grafanaStatus === 'error' && grafanaError}
            <div class="status-box status-error">
              <AlertTriangle size={14} class="inline-icon" /> {grafanaError}
            </div>
          {:else if grafanaStatus === 'success'}
            <div class="status-box status-success">
              <CheckCircle2 size={14} class="inline-icon" /> Connected to Grafana successfully!
            </div>
          {/if}

          <div class="settings-actions">
            <button class="btn-secondary" onclick={testGrafanaConnection} disabled={grafanaStatus === 'testing'}>
              {#if grafanaStatus === 'testing'}
                <Loader2 size={14} class="spin inline-icon" /> Testing...
              {:else}
                <Activity size={14} class="inline-icon" /> Test Connection
              {/if}
            </button>
            <button class="btn-primary" onclick={saveGrafanaSettings}>
              {#if isSavedNotice}
                <Check size={14} class="inline-icon" /> Saved!
              {:else}
                Save Configuration
              {/if}
            </button>
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .theme-switcher-container {
    position: relative;
    display: inline-block;
  }

  .settings-btn {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-md);
    color: var(--text-color);
    font-size: 1.1em;
    padding: 8px 12px;
    cursor: pointer;
    transition: var(--transition-normal);
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
  }

  .settings-btn:hover {
    background: rgba(255, 255, 255, 0.2);
    transform: rotate(30deg);
  }

  .settings-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 999;
    background: transparent;
  }

  .theme-panel {
    position: absolute;
    top: 45px;
    right: 0;
    background: #1e1e2e;
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.15));
    border-radius: var(--radius-lg, 8px);
    padding: 16px;
    min-width: 280px;
    z-index: 1000;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 4px;
  }

  .panel-header h4 {
    margin: 0;
    color: white;
    font-size: 1rem;
    font-weight: 600;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: rgba(255, 255, 255, 0.6);
    cursor: pointer;
    font-size: 1rem;
  }

  .close-btn:hover {
    color: white;
  }

  .panel-sub {
    margin: 0 0 12px 0;
    color: rgba(255, 255, 255, 0.7);
    font-size: 0.85rem;
  }

  .theme-options {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .theme-option {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: left;
  }

  .theme-option:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.2);
  }

  .theme-option.active {
    background: var(--primary-color, #10b981);
    border-color: var(--primary-color, #10b981);
  }

  .theme-preview {
    display: flex;
    gap: 2px;
    width: 36px;
    height: 18px;
    border-radius: 4px;
    overflow: hidden;
  }

  .preview-primary, .preview-success, .preview-warning, .preview-error {
    flex: 1;
    height: 100%;
  }

  .preview-primary { background: #10b981; }
  .preview-success { background: #3b82f6; }
  .preview-warning { background: #f59e0b; }
  .preview-error { background: #ef4444; }

  .theme-info {
    flex: 1;
  }

  .theme-name {
    color: white;
    font-weight: 600;
    font-size: 0.88rem;
  }

  .theme-description {
    color: rgba(255, 255, 255, 0.7);
    font-size: 0.78rem;
  }

  .theme-option.active .theme-description {
    color: rgba(255, 255, 255, 0.9);
  }

  /* Settings Sub-Tabs & Grafana Form Styles */
  .settings-dialog {
    width: 440px;
    max-width: 90vw;
  }

  .settings-nav-tabs {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
    border-bottom: 1px solid var(--border-primary);
    padding-bottom: 8px;
  }

  .nav-tab-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    font-size: 0.82rem;
    font-weight: 600;
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all 0.15s ease;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .nav-tab-btn:hover {
    color: var(--text-primary);
    background: rgba(255, 255, 255, 0.05);
  }

  .nav-tab-btn.active {
    color: #60a5fa;
    background: rgba(59, 130, 246, 0.15);
  }

  .grafana-settings {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .form-group label {
    font-size: 0.78rem;
    font-weight: 600;
    color: var(--text-muted);
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .settings-input, .settings-select {
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--border-primary);
    color: var(--text-primary);
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
    outline: none;
    transition: border-color 0.15s ease;
  }

  .settings-input:focus, .settings-select:focus {
    border-color: var(--primary-color);
  }

  .status-box {
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    font-size: 0.82rem;
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .status-box.status-success {
    background: rgba(34, 197, 94, 0.15);
    color: #4ade80;
    border: 1px solid rgba(34, 197, 94, 0.3);
  }

  .status-box.status-error {
    background: rgba(239, 68, 68, 0.15);
    color: #f87171;
    border: 1px solid rgba(239, 68, 68, 0.3);
  }

  .settings-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 8px;
  }

  .btn-primary {
    background: var(--primary-color);
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s ease;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .btn-primary:hover {
    filter: brightness(1.1);
  }

  .btn-secondary {
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid var(--border-primary);
    color: var(--text-primary);
    padding: 8px 14px;
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s ease;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .btn-secondary:hover {
    background: rgba(255, 255, 255, 0.15);
  }
</style>
