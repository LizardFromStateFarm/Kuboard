<!--
  LogsWindow.svelte - Semi-detached logs window with tabbed interface
  This component provides a floating window for viewing pod logs that stays open
  while browsing other resources. Supports multiple log tabs for different pods.
-->

<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, onDestroy } from 'svelte';
  
  // Props
  export let isOpen = false;
  export let onClose: () => void = () => {};
  
  // State
  let logs: { [key: string]: string } = {};
  let loading: { [key: string]: boolean } = {};
  let errors: { [key: string]: string } = {};
  let activeTab = '';
  let tabs: Array<{id: string, podName: string, namespace: string, containerName?: string}> = [];
  let tailLines = 100;
  let followMode = true; // Enable follow mode by default
  let refreshInterval: number | null = null;
  let logContainers: Record<string, HTMLElement> = {};
  let isUserScrolling = false;
  let isResizing = false;
  let windowHeight = 70; // Default height as percentage of viewport
  
  // Tab management
  function addLogTab(podName: string, namespace: string, containerName?: string) {
    const tabId = `${namespace}/${podName}${containerName ? `/${containerName}` : ''}`;
    console.log('Adding tab:', { tabId, podName, namespace, containerName });
    console.log('Current tabs:', tabs.map(t => t.id));
    
    // Check if tab already exists
    if (tabs.find(tab => tab.id === tabId)) {
      console.log('Tab already exists, switching to it');
      activeTab = tabId;
      return;
    }
    
    // Add new tab
    const newTab = {
      id: tabId,
      podName,
      namespace,
      containerName
    };
    
    tabs = [...tabs, newTab];
    activeTab = tabId;
    console.log('Added new tab, total tabs:', tabs.length);
    
    // Load logs for this tab
    loadLogs(tabId, podName, namespace, containerName);
  }
  
  function closeTab(tabId: string) {
    tabs = tabs.filter(tab => tab.id !== tabId);
    
    // Clean up state
    delete logs[tabId];
    delete loading[tabId];
    delete errors[tabId];
    
    // Switch to another tab if this was active
    if (activeTab === tabId) {
      activeTab = tabs.length > 0 ? tabs[0].id : '';
    }
  }
  
  function setActiveTab(tabId: string) {
    activeTab = tabId;
  }
  
  // Log loading
  async function loadLogs(tabId: string, podName: string, namespace: string, containerName?: string) {
    loading[tabId] = true;
    errors[tabId] = '';
    
    try {
      const logData = await invoke('kuboard_get_pod_logs', {
        podName,
        namespace,
        containerName: containerName || null,
        tailLines: tailLines,
        follow: followMode
      });
      
      logs[tabId] = logData as string;
      
      // Auto-scroll to bottom after loading
      setTimeout(() => {
        autoScrollToBottom(tabId);
      }, 100);
    } catch (error) {
      console.error('Failed to load logs:', error);
      errors[tabId] = `Failed to load logs: ${error}`;
    } finally {
      loading[tabId] = false;
    }
  }
  
  function refreshCurrentLogs() {
    const activeTabData = tabs.find(tab => tab.id === activeTab);
    if (activeTabData) {
      loadLogs(activeTabData.id, activeTabData.podName, activeTabData.namespace, activeTabData.containerName);
    }
  }
  
  function refreshAllLogs() {
    tabs.forEach(tab => {
      loadLogs(tab.id, tab.podName, tab.namespace, tab.containerName);
    });
  }
  
  // Follow mode management
  function toggleFollowMode() {
    followMode = !followMode;
    
    if (followMode) {
      // Start refresh interval
      refreshInterval = setInterval(() => {
        refreshAllLogs();
      }, 2000); // Refresh every 2 seconds
    } else {
      // Stop refresh interval
      if (refreshInterval) {
        clearInterval(refreshInterval);
        refreshInterval = null;
      }
    }
  }
  
  // Scroll detection and auto-scroll
  function handleScroll(event: Event, tabId: string) {
    const container = event.target as HTMLElement;
    const isAtBottom = container.scrollTop + container.clientHeight >= container.scrollHeight - 10;
    
    if (isAtBottom) {
      isUserScrolling = false;
    } else {
      isUserScrolling = true;
      // Exit follow mode when user scrolls up
      if (followMode) {
        followMode = false;
        if (refreshInterval) {
          clearInterval(refreshInterval);
          refreshInterval = null;
        }
      }
    }
  }
  
  function scrollToBottom(tabId: string) {
    const container = logContainers[tabId];
    if (container) {
      container.scrollTop = container.scrollHeight;
    }
  }
  
  function autoScrollToBottom(tabId: string) {
    if (!isUserScrolling) {
      scrollToBottom(tabId);
    }
  }
  
  // Resize functionality
  function startResize(event: MouseEvent) {
    isResizing = true;
    event.preventDefault();
    
    function handleMouseMove(e: MouseEvent) {
      if (!isResizing) return;
      
      const newHeight = ((window.innerHeight - e.clientY) / window.innerHeight) * 100;
      windowHeight = Math.max(20, Math.min(90, newHeight)); // Clamp between 20% and 90%
    }
    
    function handleMouseUp() {
      isResizing = false;
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    }
    
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  }
  
  function adjustHeight(delta: number) {
    windowHeight = Math.max(20, Math.min(90, windowHeight + delta));
  }
  
  // Cleanup on destroy
  onDestroy(() => {
    if (refreshInterval) {
      clearInterval(refreshInterval);
    }
  });
  
  // Expose function for external use
  export function openPodLogs(podName: string, namespace: string, containerName?: string) {
    console.log('Opening logs for:', { podName, namespace, containerName });
    addLogTab(podName, namespace, containerName);
    isOpen = true;
    
    // Start follow mode if not already running
    if (followMode && !refreshInterval) {
      refreshInterval = setInterval(() => {
        refreshAllLogs();
      }, 2000);
    }
  }
  
  // Format log line for display
  function formatLogLine(line: string): { timestamp: string, level: string, message: string, isError: boolean } {
    // Parse log line format: "2025-01-28 10:30:45 INFO [container] pod-name: message"
    const parts = line.split(' ');
    if (parts.length >= 4) {
      const timestamp = `${parts[0]} ${parts[1]}`;
      const level = parts[2];
      const message = parts.slice(3).join(' ');
      const isError = level === 'ERROR' || level === 'FATAL';
      
      return { timestamp, level, message, isError };
    }
    
    return { timestamp: '', level: 'INFO', message: line, isError: false };
  }
  
  // Get level color class
  function getLevelClass(level: string): string {
    switch (level) {
      case 'ERROR':
      case 'FATAL':
        return 'log-error';
      case 'WARN':
        return 'log-warn';
      case 'DEBUG':
        return 'log-debug';
      case 'INFO':
      default:
        return 'log-info';
    }
  }
</script>

{#if isOpen}
  <div 
    class="logs-window" 
    onkeydown={(e) => e.key === 'Escape' && onClose()}
    role="dialog"
    tabindex="-1"
    style="height: {windowHeight}vh;"
  >
      <!-- Resize Handle -->
      <div 
        class="resize-handle" 
        onmousedown={startResize}
        title="Drag to resize window height"
      ></div>
      
      <!-- Header -->
      <div class="logs-header">
        <div class="logs-title">
          <h3>📋 Pod Logs</h3>
          <span class="logs-subtitle">{tabs.length} tab{tabs.length !== 1 ? 's' : ''} open</span>
        </div>
        <div class="logs-controls">
          <div class="height-controls">
            <button class="control-button" onclick={() => adjustHeight(-10)} title="Decrease height">
              📉
            </button>
            <span class="height-indicator">{Math.round(windowHeight)}%</span>
            <button class="control-button" onclick={() => adjustHeight(10)} title="Increase height">
              📈
            </button>
          </div>
          <button 
            class="control-button {followMode ? 'active' : ''}" 
            onclick={toggleFollowMode}
            title={followMode ? 'Stop following logs' : 'Follow logs (auto-refresh)'}
          >
            {followMode ? '⏸️' : '▶️'} {followMode ? 'Following' : 'Follow'}
          </button>
          <button class="control-button" onclick={refreshCurrentLogs} title="Refresh current logs">
            🔄 Refresh
          </button>
          <button class="control-button" onclick={onClose} title="Close logs window">
            ✕ Close
          </button>
        </div>
      </div>
      
      <!-- Tabs -->
      {#if tabs.length > 0}
        <div class="logs-tabs" role="tablist">
          {#each tabs as tab}
            <div 
              class="log-tab {activeTab === tab.id ? 'active' : ''}"
              onclick={() => setActiveTab(tab.id)}
              onkeydown={(e) => e.key === 'Enter' && setActiveTab(tab.id)}
              role="tab"
              aria-selected={activeTab === tab.id}
              tabindex="0"
              id="log-tab-{tab.id}"
            >
              <span class="tab-name">
                {tab.podName}
                {#if tab.containerName}
                  <span class="container-name">/{tab.containerName}</span>
                {/if}
              </span>
              <button 
                class="tab-close" 
                onclick={(e) => { e.stopPropagation(); closeTab(tab.id); }}
                title="Close tab"
              >
                ✕
              </button>
            </div>
          {/each}
        </div>
        
        <!-- Log Content -->
        <div class="logs-content" role="tabpanel" aria-labelledby="log-tab-{activeTab}">
          {#if loading[activeTab]}
            <div class="logs-loading">
              <div class="loading-spinner">⏳</div>
              <p>Loading logs...</p>
            </div>
          {:else if errors[activeTab]}
            <div class="logs-error">
              <div class="error-icon">⚠️</div>
              <p>{errors[activeTab]}</p>
              <button class="retry-button" onclick={refreshCurrentLogs}>
                Retry
              </button>
            </div>
          {:else if logs[activeTab]}
            <div 
              class="logs-viewer"
              bind:this={logContainers[activeTab]}
              onscroll={(e) => handleScroll(e, activeTab)}
            >
              {#each logs[activeTab].split('\n').reverse() as line, index}
                {@const logData = formatLogLine(line)}
                <div class="log-line {getLevelClass(logData.level)}">
                  <span class="log-timestamp">{logData.timestamp}</span>
                  <span class="log-level">{logData.level}</span>
                  <span class="log-message">{logData.message}</span>
                </div>
              {/each}
            </div>
          {:else}
            <div class="logs-placeholder">
              <p>No logs available</p>
            </div>
          {/if}
        </div>
        
        <!-- Footer Controls -->
        <div class="logs-footer">
          <div class="logs-settings">
            <label for="tail-lines">Tail Lines:</label>
            <select bind:value={tailLines} onchange={refreshCurrentLogs}>
              <option value={50}>50</option>
              <option value={100}>100</option>
              <option value={500}>500</option>
              <option value={1000}>1000</option>
            </select>
          </div>
          <div class="logs-info">
            {#if logs[activeTab]}
              {@const lineCount = logs[activeTab].split('\n').length}
              <span>{lineCount} lines</span>
            {/if}
          </div>
        </div>
      {:else}
        <div class="logs-empty">
          <div class="empty-icon">📋</div>
          <p>No log tabs open</p>
          <p class="empty-hint">Click "View Logs" on a pod to open its logs</p>
        </div>
      {/if}
  </div>
{/if}

<style>
  .logs-window {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    background: var(--background-card);
    border: 1px solid var(--border-color);
    border-radius: 8px 8px 0 0;
    width: 100%;
    height: 70vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 -4px 20px rgba(0, 0, 0, 0.3);
    z-index: 1000;
  }
  
  .resize-handle {
    height: 4px;
    background: var(--border-color);
    cursor: ns-resize;
    position: relative;
    transition: background-color 0.2s ease;
  }
  
  .resize-handle:hover {
    background: var(--primary-color);
  }
  
  .resize-handle::before {
    content: '';
    position: absolute;
    top: -2px;
    left: 50%;
    transform: translateX(-50%);
    width: 40px;
    height: 8px;
    background: var(--border-color);
    border-radius: 2px;
  }
  
  .resize-handle:hover::before {
    background: var(--primary-color);
  }
  
  .logs-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-color);
    background: var(--background-secondary);
  }
  
  .logs-title h3 {
    margin: 0;
    color: var(--text-primary);
    font-size: 18px;
  }
  
  .logs-subtitle {
    color: var(--text-muted);
    font-size: 12px;
    margin-left: 8px;
  }
  
  .logs-controls {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  
  .height-controls {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-right: 16px;
    padding-right: 16px;
    border-right: 1px solid var(--border-color);
  }
  
  .height-indicator {
    font-size: 11px;
    color: var(--text-muted);
    min-width: 30px;
    text-align: center;
    font-weight: 500;
  }
  
  .control-button {
    background: var(--background-primary);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    padding: 6px 12px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    transition: all 0.2s ease;
  }
  
  .control-button:hover {
    background: var(--background-secondary);
    border-color: var(--primary-color);
  }
  
  .control-button.active {
    background: var(--primary-color);
    color: white;
    border-color: var(--primary-color);
  }
  
  .logs-tabs {
    display: flex;
    background: var(--background-secondary);
    border-bottom: 1px solid var(--border-color);
    overflow-x: auto;
  }
  
  .log-tab {
    display: flex;
    align-items: center;
    padding: 8px 16px;
    background: var(--background-secondary);
    border-right: 1px solid var(--border-color);
    cursor: pointer;
    transition: all 0.2s ease;
    min-width: 120px;
  }
  
  .log-tab:hover {
    background: var(--background-primary);
  }
  
  .log-tab.active {
    background: var(--background-card);
    border-bottom: 2px solid var(--primary-color);
  }
  
  .tab-name {
    color: var(--text-primary);
    font-size: 12px;
    white-space: nowrap;
  }
  
  .container-name {
    color: var(--text-muted);
    font-size: 10px;
  }
  
  .tab-close {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    margin-left: 8px;
    padding: 2px;
    border-radius: 2px;
    font-size: 10px;
  }
  
  .tab-close:hover {
    background: var(--error-color);
    color: white;
  }
  
  .logs-content {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
  
  .logs-loading, .logs-error, .logs-placeholder, .logs-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted);
  }
  
  .loading-spinner, .error-icon, .empty-icon {
    font-size: 24px;
    margin-bottom: 8px;
  }
  
  .retry-button {
    background: var(--primary-color);
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
    margin-top: 8px;
  }
  
  .logs-viewer {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
    font-family: 'Courier New', monospace;
    font-size: 12px;
    line-height: 1.4;
  }
  
  .log-line {
    display: flex;
    margin-bottom: 2px;
    padding: 2px 4px;
    border-radius: 2px;
  }
  
  .log-line:hover {
    background: var(--background-secondary);
  }
  
  .log-timestamp {
    color: var(--text-muted);
    margin-right: 8px;
    min-width: 150px;
  }
  
  .log-level {
    margin-right: 8px;
    min-width: 60px;
    font-weight: bold;
  }
  
  .log-message {
    color: var(--text-primary);
    flex: 1;
  }
  
  .log-info .log-level {
    color: var(--info-color);
  }
  
  .log-debug .log-level {
    color: var(--text-muted);
  }
  
  .log-warn .log-level {
    color: var(--warning-color);
  }
  
  .log-error .log-level {
    color: var(--error-color);
  }
  
  .log-error {
    background: rgba(239, 68, 68, 0.1);
  }
  
  .logs-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 16px;
    border-top: 1px solid var(--border-color);
    background: var(--background-secondary);
    font-size: 12px;
  }
  
  .logs-settings {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  
  .logs-settings select {
    background: var(--background-primary);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 12px;
  }
  
  .logs-info {
    color: var(--text-muted);
  }
  
  .empty-hint {
    font-size: 11px;
    margin-top: 4px;
  }
</style>
