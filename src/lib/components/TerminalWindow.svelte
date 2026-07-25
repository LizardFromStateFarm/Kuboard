<!--
  TerminalWindow.svelte - Terminal window for pod exec
  This component provides a terminal interface for executing commands in pods
-->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Terminal } from '@xterm/xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import '@xterm/xterm/css/xterm.css';

  // Props
  export let isOpen = false;
  export let podName: string = '';
  export let namespace: string = '';
  export let containerName: string = '';
  export let embedded: boolean = false;
  export let onClose: () => void = () => {};

  let terminalElement: HTMLDivElement;
  let terminal: Terminal | null = null;
  let fitAddon: FitAddon | null = null;
  let isConnected = false;
  let isMinimized = false;
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
        convertEol: true,
      });

      fitAddon = new FitAddon();
      terminal.loadAddon(fitAddon);
      terminal.open(terminalElement);
      fitAddon.fit();

      // Intercept Ctrl+C / Cmd+C when text is selected to copy to clipboard
      terminal.attachCustomKeyEventHandler((arg) => {
        if ((arg.ctrlKey || arg.metaKey) && arg.code === 'KeyC' && terminal?.hasSelection()) {
          const selectedText = terminal.getSelection();
          if (selectedText) {
            navigator.clipboard.writeText(selectedText);
          }
          return false;
        }
        return true;
      });

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

  let currentInputBuffer = '';

  async function startExecSession() {
    if (!terminal || !podName || !namespace) return;

    error = null;
    isConnected = true;

    try {
      terminal.writeln(`\x1b[32m✅ Connected to container shell: ${containerName || 'main'}\x1b[0m`);
      terminal.writeln(`Type commands and press Enter. (e.g. 'clear', 'ls', 'help')\r\n`);
      terminal.write('$ ');

      terminal.onData((data) => {
        if (data === '\r') {
          // Enter key pressed
          terminal?.writeln('');
          const cmd = currentInputBuffer.trim();
          currentInputBuffer = '';
          if (cmd) {
            executeInteractiveCommand(cmd);
          } else {
            terminal?.write('$ ');
          }
        } else if (data === '\u007F') {
          // Backspace
          if (currentInputBuffer.length > 0) {
            currentInputBuffer = currentInputBuffer.slice(0, -1);
            terminal?.write('\b \b');
          }
        } else if (data >= ' ' || data === '\t') {
          currentInputBuffer += data;
          terminal?.write(data);
        }
      });
    } catch (err: any) {
      error = err.toString();
      isConnected = false;
      if (terminal) {
        terminal.writeln(`\x1b[31mError: ${error}\x1b[0m`);
      }
    }
  }

  async function executeInteractiveCommand(cmd: string) {
    if (!terminal) return;
    try {
      if (cmd === 'clear') {
        terminal.clear();
        terminal.write('$ ');
        return;
      }
      const output = await invoke<string>('kuboard_exec_command', {
        podName,
        namespace,
        containerName: containerName || null,
        command: cmd
      });
      if (output && output.trim()) {
        terminal.writeln(output);
      }
    } catch (err: any) {
      terminal.writeln(`\x1b[31mError: ${err?.message || err}\x1b[0m`);
    } finally {
      terminal.write('$ ');
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
  <div class="terminal-window {embedded ? 'embedded' : ''} {isMinimized ? 'is-minimized' : ''}">
    {#if !embedded}
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
          <button class="minimize-button" onclick={() => isMinimized = !isMinimized} title={isMinimized ? "Expand" : "Minimize"}>
            {isMinimized ? '▲' : '▼'}
          </button>
          <button class="close-button" onclick={handleClose} title="Close Terminal">
            ✕
          </button>
        </div>
      </div>
    {/if}

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
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    z-index: 9000;
    display: flex;
    flex-direction: column;
    height: 360px;
    background: var(--background-card, #1a1a1a);
    border: 1px solid var(--border-color, #333);
    border-radius: 8px 8px 0 0;
    box-shadow: 0 -4px 20px rgba(0, 0, 0, 0.4);
    overflow: hidden;
  }

  .terminal-window.embedded {
    position: relative;
    height: 100%;
    border: none;
    border-radius: 0;
    box-shadow: none;
    z-index: 1;
  }

  .terminal-window.is-minimized {
    height: 42px !important;
  }

  .minimize-button {
    padding: 4px 8px;
    background: transparent;
    border: none;
    color: var(--text-primary, #fff);
    cursor: pointer;
    font-size: 12px;
    border-radius: 4px;
  }
  .minimize-button:hover {
    background: rgba(255, 255, 255, 0.1);
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

