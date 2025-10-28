<!-- Kuboard Theme Switcher Component -->
<!-- Use this in dev mode to quickly test different color schemes -->
<script lang="ts">
  import { onMount } from 'svelte';

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

<!-- Theme Switcher Button (only visible in dev mode) -->
{#if import.meta.env.DEV}
  <div class="theme-switcher">
    <button 
      class="theme-toggle-btn" 
      onclick={toggleVisibility}
      title="Toggle Theme Switcher"
    >
      🎨
    </button>
    
    {#if isVisible}
      <div class="theme-panel">
        <h4>🎨 Theme Switcher</h4>
        <p>Quickly test different color schemes:</p>
        
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
        
        <div class="theme-actions">
          <button 
            class="action-btn"
            onclick={() => window.open('/src/lib/styles/color-palette.css', '_blank')}
            title="Open color-palette.css in editor"
          >
            📝 Edit Colors
          </button>
          <button 
            class="action-btn"
            onclick={() => location.reload()}
            title="Reload to see changes"
          >
            🔄 Reload
          </button>
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .theme-switcher {
    position: fixed;
    top: 20px;
    right: 20px;
    z-index: 1000;
  }

  .theme-toggle-btn {
    background: var(--primary-color);
    border: none;
    border-radius: 50%;
    width: 50px;
    height: 50px;
    font-size: 1.5rem;
    cursor: pointer;
    box-shadow: 0 4px 12px var(--shadow-primary);
    transition: var(--transition-normal);
  }

  .theme-toggle-btn:hover {
    transform: scale(1.1);
    box-shadow: 0 6px 16px var(--shadow-primary);
  }

  .theme-panel {
    position: absolute;
    top: 60px;
    right: 0;
    background: var(--background-card);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
    min-width: 300px;
    box-shadow: 0 10px 25px var(--shadow-dark);
  }

  .theme-panel h4 {
    margin: 0 0 var(--spacing-sm) 0;
    color: var(--text-primary);
    font-size: 1.1rem;
  }

  .theme-panel p {
    margin: 0 0 var(--spacing-md) 0;
    color: var(--text-secondary);
    font-size: 0.9rem;
  }

  .theme-options {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-md);
  }

  .theme-option {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-sm);
    background: var(--background-secondary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: var(--transition-normal);
  }

  .theme-option:hover {
    background: var(--background-hover);
    border-color: var(--border-secondary);
  }

  .theme-option.active {
    background: var(--primary-color);
    border-color: var(--primary-color);
  }

  .theme-preview {
    display: flex;
    gap: 2px;
    width: 40px;
    height: 20px;
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .preview-primary, .preview-success, .preview-warning, .preview-error {
    flex: 1;
    height: 100%;
  }

  .preview-primary { background: var(--primary-color); }
  .preview-success { background: var(--success-color); }
  .preview-warning { background: var(--warning-color); }
  .preview-error { background: var(--error-color); }

  .theme-info {
    flex: 1;
  }

  .theme-name {
    color: var(--text-primary);
    font-weight: 600;
    font-size: 0.9rem;
  }

  .theme-description {
    color: var(--text-secondary);
    font-size: 0.8rem;
  }

  .theme-option.active .theme-name,
  .theme-option.active .theme-description {
    color: white;
  }

  .theme-actions {
    display: flex;
    gap: var(--spacing-sm);
  }

  .action-btn {
    flex: 1;
    padding: var(--spacing-sm);
    background: var(--background-hover);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    cursor: pointer;
    font-size: 0.8rem;
    transition: var(--transition-normal);
  }

  .action-btn:hover {
    background: var(--primary-color);
    border-color: var(--primary-color);
    color: white;
  }
</style>
