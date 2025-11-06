<!-- Quick Actions Menu Component -->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let resource: any; // Pod, Deployment, Service, etc.
  export let resourceType: 'pod' | 'deployment' | 'service' | 'replicaset' | 'node' | 'configmap' | 'secret' = 'pod';
  export let position: { x: number; y: number } = { x: 0, y: 0 };
  export let visible: boolean = false;

  const dispatch = createEventDispatcher();

  let menuElement: HTMLElement;
  let actionExecuting: string | null = null;
  let errorMessage: string | null = null;
  let confirmAction: { action: string; message: string } | null = null;

  $: {
    if (visible && menuElement) {
      // Position menu near cursor
      const rect = menuElement.getBoundingClientRect();
      const windowWidth = window.innerWidth;
      const windowHeight = window.innerHeight;
      
      let x = position.x;
      let y = position.y;

      // Adjust if menu would go off-screen
      if (x + rect.width > windowWidth) {
        x = windowWidth - rect.width - 10;
      }
      if (y + rect.height > windowHeight) {
        y = windowHeight - rect.height - 10;
      }
      if (x < 0) x = 10;
      if (y < 0) y = 10;

      menuElement.style.left = `${x}px`;
      menuElement.style.top = `${y}px`;
    }
  }

  function getResourceName(): string {
    return resource?.metadata?.name || 'Unknown';
  }

  function getResourceNamespace(): string {
    return resource?.metadata?.namespace || 'default';
  }

  function getResourceIP(): string {
    if (resourceType === 'pod') {
      return resource?.status?.podIP || '';
    }
    return '';
  }

  function handleAction(action: string) {
    if (confirmAction && confirmAction.action === action) {
      // Second click confirms
      executeAction(action);
      confirmAction = null;
    } else if (isDestructiveAction(action)) {
      // First click shows confirmation
      confirmAction = { action, message: getConfirmationMessage(action) };
    } else {
      executeAction(action);
    }
  }

  function isDestructiveAction(action: string): boolean {
    return ['delete', 'restart'].includes(action);
  }

  function getConfirmationMessage(action: string): string {
    const name = getResourceName();
    const namespace = getResourceNamespace();
    
    switch (action) {
      case 'delete':
        return `Are you sure you want to delete ${resourceType} "${name}" in namespace "${namespace}"?`;
      case 'restart':
        return `Are you sure you want to restart ${resourceType} "${name}" in namespace "${namespace}"?`;
      default:
        return `Are you sure?`;
    }
  }

  async function executeAction(action: string) {
    if (actionExecuting) return; // Prevent double-click

    actionExecuting = action;
    errorMessage = null;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      switch (action) {
        case 'delete':
          if (resourceType === 'pod') {
            await invoke('kuboard_delete_pod', {
              podName: getResourceName(),
              namespace: getResourceNamespace()
            });
            dispatch('deleted', { resource, resourceType });
          }
          break;
        
        case 'restart':
          if (resourceType === 'pod') {
            await invoke('kuboard_restart_pod', {
              podName: getResourceName(),
              namespace: getResourceNamespace()
            });
            dispatch('restarted', { resource, resourceType });
          }
          break;
        
        case 'view-yaml':
          if (resourceType === 'pod') {
            const yaml = await invoke('kuboard_get_pod_yaml', {
              podName: getResourceName(),
              namespace: getResourceNamespace()
            });
            dispatch('view-yaml', { yaml, resource, resourceType });
          } else {
            // For non-pod resources, use JSON as fallback
            const json = JSON.stringify(resource, null, 2);
            dispatch('view-yaml', { yaml: json, resource, resourceType });
          }
          break;
        
        case 'copy-name':
          await navigator.clipboard.writeText(getResourceName());
          dispatch('copied', { type: 'name', value: getResourceName() });
          break;
        
        case 'copy-ip':
          const ip = getResourceIP();
          if (ip) {
            await navigator.clipboard.writeText(ip);
            dispatch('copied', { type: 'ip', value: ip });
          }
          break;
        
        case 'edit':
          // Fetch YAML for editing
          if (resourceType === 'pod') {
            const yaml = await invoke('kuboard_get_pod_yaml', {
              podName: getResourceName(),
              namespace: getResourceNamespace()
            });
            dispatch('edit', { yaml, resource, resourceType });
          } else {
            // For non-pod resources, use JSON as fallback
            const json = JSON.stringify(resource, null, 2);
            dispatch('edit', { yaml: json, resource, resourceType });
          }
          break;
        
        default:
          dispatch('action', { action, resource, resourceType });
      }
      
      // Close menu after successful action (except for view-yaml which opens a modal)
      if (action !== 'view-yaml' && action !== 'edit') {
        setTimeout(() => {
          visible = false;
          dispatch('close');
        }, 100);
      }
    } catch (error: any) {
      errorMessage = error.toString() || 'Unknown error occurred';
      console.error(`Error executing action ${action}:`, error);
    } finally {
      actionExecuting = null;
    }
  }

  function closeMenu() {
    visible = false;
    confirmAction = null;
    errorMessage = null;
    dispatch('close');
  }

  // Close menu on outside click
  function handleClickOutside(event: MouseEvent) {
    if (visible && menuElement && !menuElement.contains(event.target as Node)) {
      closeMenu();
    }
  }

  // Close menu on Escape key
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && visible) {
      closeMenu();
    }
  }

  // Get available actions for resource type
  function getAvailableActions(): Array<{ id: string; label: string; icon: string; disabled?: boolean }> {
    const baseActions = [
      { id: 'view-yaml', label: 'View YAML', icon: '📄' },
      { id: 'edit', label: 'Edit', icon: '✏️' },
    ];

    if (resourceType === 'pod') {
      return [
        { id: 'copy-name', label: 'Copy Name', icon: '📋' },
        { id: 'copy-ip', label: 'Copy IP', icon: '🌐', disabled: !getResourceIP() },
        ...baseActions,
        { id: 'restart', label: 'Restart', icon: '🔄' },
        { id: 'delete', label: 'Delete', icon: '🗑️' },
      ];
    }

    if (resourceType === 'service') {
      return [
        { id: 'copy-name', label: 'Copy Name', icon: '📋' },
        { id: 'copy-namespace', label: 'Copy Namespace', icon: '📋' },
        ...baseActions,
      ];
    }

    if (resourceType === 'replicaset') {
      return [
        { id: 'copy-name', label: 'Copy Name', icon: '📋' },
        { id: 'copy-namespace', label: 'Copy Namespace', icon: '📋' },
        ...baseActions,
      ];
    }

    return baseActions;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if visible}
  <div
    bind:this={menuElement}
    class="quick-actions-menu"
    role="menu"
    onclick={(e) => e.stopPropagation()}
  >
    {#if confirmAction}
      <!-- Confirmation Dialog -->
      <div class="confirmation-dialog">
        <div class="confirmation-message">{confirmAction.message}</div>
        <div class="confirmation-buttons">
          <button
            class="btn-confirm"
            onclick={() => handleAction(confirmAction!.action)}
            disabled={actionExecuting === confirmAction.action}
          >
            {actionExecuting === confirmAction.action ? '⏳' : '✓'} Confirm
          </button>
          <button
            class="btn-cancel"
            onclick={() => { confirmAction = null; }}
          >
            ✕ Cancel
          </button>
        </div>
      </div>
    {:else}
      <!-- Action Menu -->
      <div class="menu-header">
        <span class="resource-name">{getResourceName()}</span>
        <button class="btn-close" onclick={closeMenu} title="Close">✕</button>
      </div>
      
      <div class="menu-actions">
        {#each getAvailableActions() as action}
          <button
            class="menu-action"
            class:disabled={action.disabled || actionExecuting === action.id}
            onclick={() => handleAction(action.id)}
            role="menuitem"
            disabled={action.disabled || actionExecuting === action.id}
          >
            <span class="action-icon">{action.icon}</span>
            <span class="action-label">{action.label}</span>
            {#if actionExecuting === action.id}
              <span class="action-spinner">⏳</span>
            {/if}
          </button>
        {/each}
      </div>

      {#if errorMessage}
        <div class="error-message">
          <span class="error-icon">⚠️</span>
          <span class="error-text">{errorMessage}</span>
        </div>
      {/if}
    {/if}
  </div>
{/if}

{#if visible}
  <!-- Backdrop -->
  <div class="menu-backdrop" onclick={closeMenu}></div>
{/if}

<style>
  @import '../styles/variables.css';

  .quick-actions-menu {
    position: fixed;
    z-index: 10000;
    background: var(--bg-secondary);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
    min-width: 200px;
    max-width: 300px;
    overflow: hidden;
    backdrop-filter: blur(10px);
  }

  .menu-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 9999;
    background: rgba(0, 0, 0, 0.1);
    backdrop-filter: blur(2px);
  }

  .menu-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.05);
  }

  .resource-name {
    font-weight: 600;
    font-size: 0.9rem;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  .btn-close {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 1.2rem;
    line-height: 1;
    transition: all 0.2s;
  }

  .btn-close:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
  }

  .menu-actions {
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .menu-action {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 16px;
    background: transparent;
    border: none;
    border-radius: 6px;
    color: var(--text-primary);
    cursor: pointer;
    text-align: left;
    font-size: 0.9rem;
    transition: all 0.2s;
    position: relative;
  }

  .menu-action:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
  }

  .menu-action:disabled,
  .menu-action.disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .action-icon {
    font-size: 1.1rem;
    width: 20px;
    text-align: center;
    flex-shrink: 0;
  }

  .action-label {
    flex: 1;
  }

  .action-spinner {
    font-size: 0.9rem;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .confirmation-dialog {
    padding: 16px;
  }

  .confirmation-message {
    color: var(--text-primary);
    margin-bottom: 16px;
    font-size: 0.9rem;
    line-height: 1.5;
  }

  .confirmation-buttons {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }

  .btn-confirm,
  .btn-cancel {
    padding: 8px 16px;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.9rem;
    font-weight: 500;
    transition: all 0.2s;
  }

  .btn-confirm {
    background: var(--accent-color);
    color: white;
  }

  .btn-confirm:hover:not(:disabled) {
    background: var(--accent-color-hover);
    transform: translateY(-1px);
  }

  .btn-confirm:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .btn-cancel {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
  }

  .btn-cancel:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .error-message {
    padding: 12px 16px;
    background: rgba(255, 0, 0, 0.1);
    border-top: 1px solid rgba(255, 0, 0, 0.2);
    display: flex;
    align-items: center;
    gap: 8px;
    color: #ff6b6b;
    font-size: 0.85rem;
  }

  .error-icon {
    font-size: 1.1rem;
  }

  .error-text {
    flex: 1;
  }
</style>

