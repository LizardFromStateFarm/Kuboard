<!-- Kuboard Live YAML Editor Component -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { FileText, AlertTriangle } from 'lucide-svelte';

  // Props
  export let resource: any;
  export let resourceType: string;
  export let onSave: () => void = () => {};
  export let onCancel: () => void = () => {};

  // State
  let editorContainer: HTMLElement;
  let editor: any;
  let monaco: any;
  let loading = true;
  let saving = false;
  let error: string | null = null;
  let originalYaml = '';
  let currentYaml = '';

  async function initMonaco() {
    try {
      // In a real SvelteKit app, we might need a more complex loader
      // for workers. For now, let's try a dynamic import.
      const monacoModule = await import('monaco-editor');
      monaco = monacoModule;

      // Fetch the YAML
      originalYaml = await invoke('kuboard_get_resource_yaml', {
        kind: resourceType,
        name: resource.metadata.name,
        namespace: resource.metadata.namespace
      });
      currentYaml = originalYaml;

      // Create editor
      editor = monaco.editor.create(editorContainer, {
        value: currentYaml,
        language: 'yaml',
        theme: 'vs-dark',
        automaticLayout: true,
        minimap: { enabled: true },
        fontSize: 14,
        lineNumbers: 'on',
        scrollBeyondLastLine: false,
        roundedSelection: false,
        readOnly: false,
        cursorStyle: 'line',
        padding: { top: 10, bottom: 10 }
      });

      // Listen for changes
      editor.onDidChangeModelContent(() => {
        currentYaml = editor.getValue();
      });

      // Keyboard shortcut for save (Ctrl+S)
      editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
        handleSave();
      });

      loading = false;
    } catch (e: any) {
      console.error('Failed to initialize Monaco:', e);
      error = `Failed to initialize editor: ${e}`;
      loading = false;
    }
  }

  async function handleSave() {
    if (saving) return;
    try {
      saving = true;
      error = null;
      await invoke('kuboard_apply_resource_yaml', { yamlContent: currentYaml });
      onSave();
    } catch (e: any) {
      console.error('Failed to save YAML:', e);
      error = String(e);
    } finally {
      saving = false;
    }
  }

  onMount(() => {
    initMonaco();
  });

  onDestroy(() => {
    if (editor) {
      editor.dispose();
    }
  });

  function getResourceTitle() {
    return `${resourceType}: ${resource.metadata.name}`;
  }
</script>

<div class="yaml-editor-overlay">
  <div class="yaml-editor-modal">
    <div class="editor-header">
      <div class="header-left">
        <h3><FileText size={18} class="inline-icon" /> Edit YAML</h3>
        <span class="resource-title">{getResourceTitle()}</span>
      </div>
      <div class="header-actions">
        {#if saving}
          <div class="saving-indicator">
            <span class="spinner"></span>
            Saving...
          </div>
        {/if}
        <button class="btn-cancel" onclick={onCancel} disabled={saving}>Cancel</button>
        <button class="btn-save" onclick={handleSave} disabled={saving || loading}>
          Save Changes
        </button>
      </div>
    </div>

    {#if error}
      <div class="error-banner">
        <span class="error-icon"><AlertTriangle size={18} /></span>
        <p>{error}</p>
        <button class="error-close" onclick={() => error = null}>×</button>
      </div>
    {/if}

    <div class="editor-body">
      {#if loading}
        <div class="loading-state">
          <div class="spinner large"></div>
          <p>Initializing Monaco Editor...</p>
        </div>
      {/if}
      <div bind:this={editorContainer} class="monaco-container" class:hidden={loading}></div>
    </div>

    <div class="editor-footer">
      <div class="footer-hint">
        Tip: Press <span>Ctrl+S</span> to quickly apply changes.
      </div>
    </div>
  </div>
</div>

<style>
  .yaml-editor-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.8);
    backdrop-filter: blur(8px);
    z-index: 2000;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-xl);
  }

  .yaml-editor-modal {
    width: 100%;
    max-width: 1200px;
    height: 90vh;
    background: var(--bg-secondary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .editor-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-md) var(--spacing-lg);
    background: rgba(255, 255, 255, 0.03);
    border-bottom: 1px solid var(--border-primary);
  }

  .header-left h3 {
    margin: 0;
    font-size: 1.2rem;
    color: var(--text-primary);
  }

  .resource-title {
    font-size: 0.85rem;
    color: var(--text-secondary);
    font-family: monospace;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .btn-cancel {
    background: transparent;
    border: 1px solid var(--border-primary);
    color: var(--text-secondary);
    padding: 6px 16px;
    border-radius: var(--radius-md);
    cursor: pointer;
  }

  .btn-save {
    background: var(--primary-color);
    border: none;
    color: white;
    padding: 6px 20px;
    border-radius: var(--radius-md);
    font-weight: 600;
    cursor: pointer;
  }

  .btn-save:hover:not(:disabled) {
    background: #2563eb;
  }

  .btn-save:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .error-banner {
    background: rgba(239, 68, 68, 0.1);
    border-bottom: 1px solid rgba(239, 68, 68, 0.2);
    padding: var(--spacing-sm) var(--spacing-lg);
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    color: #f87171;
  }

  .error-banner p {
    margin: 0;
    flex: 1;
    font-size: 0.9rem;
  }

  .error-close {
    background: transparent;
    border: none;
    color: inherit;
    font-size: 1.2rem;
    cursor: pointer;
  }

  .editor-body {
    flex: 1;
    position: relative;
    background: #1e1e1e;
  }

  .monaco-container {
    width: 100%;
    height: 100%;
  }

  .monaco-container.hidden {
    visibility: hidden;
  }

  .loading-state {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
  }

  .spinner {
    width: 20px;
    height: 20px;
    border: 2px solid rgba(255, 255, 255, 0.1);
    border-top-color: var(--primary-color);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  .spinner.large {
    width: 40px;
    height: 40px;
    border-width: 3px;
    margin-bottom: var(--spacing-md);
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .editor-footer {
    padding: var(--spacing-xs) var(--spacing-lg);
    background: rgba(0, 0, 0, 0.2);
    border-top: 1px solid var(--border-primary);
  }

  .footer-hint {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .footer-hint span {
    background: rgba(255, 255, 255, 0.1);
    padding: 1px 4px;
    border-radius: 3px;
    color: var(--text-secondary);
  }

  .saving-indicator {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.9rem;
    color: var(--primary-color);
    font-weight: 500;
  }
</style>
