<!-- Kuboard Keyboard Manager Component -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { activeSelectionIndex, showKeyboardHelp } from '../stores/keyboard';
  import { openEditor } from '../stores/editor';
  import { openXRay } from '../stores/xray';
  import { navigationStore } from '../stores/nav';

  // State to track if we are in an input/textarea
  let isInputActive = false;
  let commandMode = false;
  let commandValue = '';
  let commandInput: HTMLInputElement;

  function handleCommand() {
    const [cmd, ...args] = commandValue.trim().split(' ');
    console.log('🚀 KeyboardManager: Executing command:', cmd, args);

    switch (cmd) {
      case 'q':
      case 'quit':
        // Close modals if any
        showKeyboardHelp.set(false);
        break;
      case 'search':
        if (args.length > 0) {
          // Trigger global search with query
        }
        break;
      case 'help':
        showKeyboardHelp.set(true);
        break;
      default:
        console.warn('Unknown command:', cmd);
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    // Check if user is typing in an input or modal is open
    const target = event.target as HTMLElement;
    isInputActive = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable;

    if (isInputActive) {
      if (event.key === 'Escape') {
        target.blur();
      }
      return;
    }

    // Global Shortcuts
    switch (event.key) {
      case '/':
        event.preventDefault();
        // Focus the first search input found in the DOM
        const searchInput = document.querySelector('.search-input') as HTMLInputElement;
        searchInput?.focus();
        break;

      case '?':
        event.preventDefault();
        showKeyboardHelp.update(v => !v);
        break;

      case ':':
        event.preventDefault();
        commandMode = true;
        commandValue = '';
        setTimeout(() => commandInput?.focus(), 10);
        break;

      case 'j':
        event.preventDefault();
        activeSelectionIndex.update(i => i + 1);
        break;

      case 'k':
        event.preventDefault();
        activeSelectionIndex.update(i => Math.max(-1, i - 1));
        break;

      case 'g':
        if (event.shiftKey) { // G - Bottom
            // TODO: implement
        } else { // g - check for gg
            // We'd need a multi-key buffer here
        }
        break;

      case 'Enter':
        // Trigger action on selected item
        break;

      case 'e':
        // Edit YAML on selected item
        break;

      case 'x':
        // X-Ray on selected item
        break;
    }
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeyDown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeyDown);
  });
</script>

{#if $showKeyboardHelp}
  <div class="kb-help-overlay" onclick={() => showKeyboardHelp.set(false)}>
    <div class="kb-help-modal" onclick={e => e.stopPropagation()}>
      <div class="kb-help-header">
        <h3>⌨️ Keyboard Shortcuts</h3>
        <button class="close-btn" onclick={() => showKeyboardHelp.set(false)}>×</button>
      </div>
      <div class="kb-help-content">
        <div class="kb-section">
          <h4>Navigation</h4>
          <div class="kb-row"><span class="key">j</span> Move selection down</div>
          <div class="kb-row"><span class="key">k</span> Move selection up</div>
          <div class="kb-row"><span class="key">g g</span> Go to top</div>
          <div class="kb-row"><span class="key">G</span> Go to bottom</div>
        </div>
        <div class="kb-section">
          <h4>Actions</h4>
          <div class="kb-row"><span class="key">/</span> Focus search</div>
          <div class="kb-row"><span class="key">Enter</span> View details</div>
          <div class="kb-row"><span class="key">e</span> Edit YAML</div>
          <div class="kb-row"><span class="key">x</span> Open X-Ray</div>
          <div class="kb-row"><span class="key">Ctrl+K</span> Global Search</div>
        </div>
        <div class="kb-section">
          <h4>Global</h4>
          <div class="kb-row"><span class="key">?</span> Show this help</div>
          <div class="kb-row"><span class="key">Esc</span> Close modal / Unfocus</div>
        </div>
      </div>
    </div>
  </div>
{/if}

{#if commandMode}
  <div class="command-bar" class:active={commandMode}>
    <span class="command-prefix">:</span>
    <input 
      bind:this={commandInput}
      bind:value={commandValue}
      onkeydown={(e) => {
        if (e.key === 'Enter') {
          handleCommand();
          commandMode = false;
        } else if (e.key === 'Escape') {
          commandMode = false;
        }
      }}
      placeholder="Enter command..."
    />
  </div>
{/if}

<style>
  .kb-help-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(4px);
    z-index: 5000;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .kb-help-modal {
    background: var(--bg-secondary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-lg);
    width: 100%;
    max-width: 500px;
    box-shadow: var(--shadow-xxl);
    overflow: hidden;
  }

  .kb-help-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-md) var(--spacing-lg);
    background: rgba(255, 255, 255, 0.05);
    border-bottom: 1px solid var(--border-primary);
  }

  .kb-help-header h3 {
    margin: 0;
    font-size: 1.1rem;
    color: white;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 1.5rem;
    cursor: pointer;
  }

  .kb-help-content {
    padding: var(--spacing-lg);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  .kb-section h4 {
    margin: 0 0 var(--spacing-sm) 0;
    font-size: 0.9rem;
    color: var(--primary-color);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .kb-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.9rem;
    color: var(--text-secondary);
    padding: 4px 0;
  }

  .key {
    background: rgba(255, 255, 255, 0.1);
    color: white;
    padding: 2px 8px;
    border-radius: 4px;
    font-family: monospace;
    font-weight: bold;
    min-width: 30px;
    text-align: center;
    box-shadow: 0 2px 0 rgba(0,0,0,0.3);
  }

  .command-bar {
    position: fixed;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    background: #1e1e1e;
    border: 1px solid var(--primary-color);
    border-radius: var(--radius-md);
    padding: 8px 16px;
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    max-width: 600px;
    box-shadow: var(--shadow-xxl);
    z-index: 6000;
  }

  .command-prefix {
    color: var(--primary-color);
    font-weight: bold;
    font-family: monospace;
    font-size: 1.2rem;
  }

  .command-bar input {
    flex: 1;
    background: transparent;
    border: none;
    color: white;
    font-family: monospace;
    font-size: 1rem;
    outline: none;
  }
</style>
