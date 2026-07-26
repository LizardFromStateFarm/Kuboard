<!-- Kuboard Settings & Theme Switcher Component -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { Settings } from 'lucide-svelte';

  // Theme state
  let currentTheme: string = 'dark';
  let isVisible: boolean = false;

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

  // Load saved theme on mount
  onMount(() => {
    const savedTheme = localStorage.getItem('kuboard-theme') || 'dark';
    applyTheme(savedTheme);
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
    <div class="theme-panel">
      <div class="panel-header">
        <h4><Settings size={16} class="inline-icon" /> Settings & Theme</h4>
        <button class="close-btn" onclick={toggleVisibility}>✕</button>
      </div>
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
</style>
