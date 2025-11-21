<!--
  TerminalWindow.svelte - Terminal window for pod exec
  This component provides a terminal interface for executing commands in pods
-->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Terminal } from '@xterm/xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import '@xterm/xterm/css/xterm.css';

  // Props
  export let isOpen = false;
  export let podName: string = '';
  export let namespace: string = '';
  export let containerName: string = '';
  export let onClose: () => void = () => {};

  let terminalElement: HTMLDivElement;
  let terminal: Terminal | null = null;
  let fitAddon: FitAddon | null = null;
  let isConnected = false;
  let error: string | null = null;
  let availableContainers: string[] = [];
  let selectedContainer: string = '';

  // Initialize terminal
  onMount(() => {
    if (terminalElement) {
      terminal = new Terminal({
        theme: {
          background: '#1a1a1a',
          foreground: '#ffffff',
          cursor: '#ffffff',
          selection: '#264f78',
        },
        fontSize: 14,
        fontFamily: 'Consolas, "Courier New", monospace',
        cursorBlink: true,
        cursorStyle: 'block',
      });

      fitAddon = new FitAddon();
      terminal.loadAddon(fitAddon);
      terminal.open(terminalElement);
      fitAddon.fit();

      // Handle window resize
      const resizeObserver = new ResizeObserver(() => {
        if (fitAddon) {
          fitAddon.fit();
        }
      });
      resizeObserver.observe(terminalElement);

      // Write welcome message
      if (terminal) {
        terminal.writeln('\x1b[32mKuboard Terminal\x1b[0m');
        terminal.writeln(`Connecting to pod: ${podName} in namespace: ${namespace}`);
        terminal.writeln('');
      }

      // Start exec session
      if (podName && namespace) {
        startExecSession();
      }
    }

    return () => {
      if (terminal) {
        terminal.dispose();
      }
    };
  });

  async function startExecSession() {
    if (!terminal || !podName || !namespace) return;

    error = null;
    isConnected = false;

    try {
      // TODO: Implement exec command
      // For now, show a message that this feature is being implemented
      terminal.writeln('\x1b[33m⚠️  Exec functionality is being implemented...\x1b[0m');
      terminal.writeln('\x1b[36mThis will allow you to execute commands in the pod container.\x1b[0m');
      terminal.writeln('');
      terminal.writeln(`Pod: ${podName}`);
      terminal.writeln(`Namespace: ${namespace}`);
      terminal.writeln(`Container: ${containerName || 'default'}`);
      terminal.writeln('');
      terminal.writeln('\x1b[31mFeature coming soon!\x1b[0m');
      
      // Show a prompt
      terminal.write('\r\n$ ');
    } catch (err: any) {
      error = err.toString();
      if (terminal) {
        terminal.writeln(`\x1b[31mError: ${error}\x1b[0m`);
      }
    }
  }

  function handleClose() {
    if (terminal) {
      terminal.writeln('\r\n\x1b[33mTerminal session closed.\x1b[0m');
    }
    onClose();
  }

  function handleContainerChange() {
    if (selectedContainer && selectedContainer !== containerName) {
      containerName = selectedContainer;
      if (terminal) {
        terminal.clear();
        startExecSession();
      }
    }
  }
</script>

{#if isOpen}
  <div class="terminal-window">
    <div class="terminal-header">
      <div class="terminal-title">
        <span class="terminal-icon">💻</span>
        <span class="terminal-label">Terminal</span>
        <span class="terminal-pod-info">{namespace}/{podName}</span>
        {#if containerName}
          <span class="terminal-container">({containerName})</span>
        {/if}
      </div>
      <div class="terminal-controls">
        {#if availableContainers.length > 1}
          <select 
            class="container-selector" 
            bind:value={selectedContainer}
            onchange={handleContainerChange}
          >
            {#each availableContainers as container}
              <option value={container} selected={container === containerName}>
                {container}
              </option>
            {/each}
          </select>
        {/if}
        <button class="close-button" onclick={handleClose} title="Close Terminal">
          ✕
        </button>
      </div>
    </div>
    
    {#if error}
      <div class="terminal-error">
        <span>⚠️ {error}</span>
      </div>
    {/if}

    <div class="terminal-container-wrapper">
      <div class="terminal-status">
        {#if isConnected}
          <span class="status-connected">● Connected</span>
        {:else}
          <span class="status-disconnected">○ Disconnected</span>
        {/if}
      </div>
      <div class="terminal-content" bind:this={terminalElement}></div>
    </div>
  </div>
{/if}

<style>
  .terminal-window {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--background-card, #1a1a1a);
    border: 1px solid var(--border-color, #333);
    border-radius: 4px;
    overflow: hidden;
  }

  .terminal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    background: var(--background-secondary, #111);
    border-bottom: 1px solid var(--border-color, #333);
  }

  .terminal-title {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text-primary, #fff);
    font-size: 14px;
    font-weight: 500;
  }

  .terminal-icon {
    font-size: 16px;
  }

  .terminal-label {
    font-weight: 600;
  }

  .terminal-pod-info {
    color: var(--text-secondary, #aaa);
    font-size: 12px;
  }

  .terminal-container {
    color: var(--text-secondary, #aaa);
    font-size: 12px;
  }

  .terminal-controls {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .container-selector {
    padding: 4px 8px;
    background: var(--background-primary, #000);
    border: 1px solid var(--border-color, #333);
    border-radius: 4px;
    color: var(--text-primary, #fff);
    font-size: 12px;
    cursor: pointer;
  }

  .close-button {
    padding: 4px 8px;
    background: transparent;
    border: none;
    color: var(--text-primary, #fff);
    cursor: pointer;
    font-size: 16px;
    line-height: 1;
    border-radius: 4px;
  }

  .close-button:hover {
    background: var(--background-card, #1a1a1a);
  }

  .terminal-error {
    padding: 8px 12px;
    background: rgba(239, 68, 68, 0.1);
    border-bottom: 1px solid rgba(239, 68, 68, 0.3);
    color: var(--error-color, #ef4444);
    font-size: 12px;
  }

  .terminal-container-wrapper {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
  }

  .terminal-status {
    padding: 4px 12px;
    background: var(--background-secondary, #111);
    border-bottom: 1px solid var(--border-color, #333);
    font-size: 11px;
  }

  .status-connected {
    color: var(--success-color, #10b981);
  }

  .status-disconnected {
    color: var(--text-secondary, #aaa);
  }

  .terminal-content {
    flex: 1;
    padding: 8px;
    overflow: auto;
  }

  /* xterm.js styles are imported separately */
  :global(.xterm) {
    height: 100%;
  }

  :global(.xterm-viewport) {
    background: var(--background-card, #1a1a1a) !important;
  }
</style>

