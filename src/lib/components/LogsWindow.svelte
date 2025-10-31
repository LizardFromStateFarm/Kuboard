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
  type LogEntry = {
    id: string;
    timestamp: string;
    level?: string;
    message: string;
    isExpanded: boolean;
    isJson: boolean;
  };

  let logs: { [key: string]: string } = {}; // legacy; will be phased out
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
  let windowHeight = 30; // Default height as percentage of viewport
  
  // Incremental log tracking (append-only entries)
  let logLines: Record<string, string[]> = {}; // legacy; will be phased out
  let lastSeenLine: Record<string, string> = {};
  let newLogLines: Record<string, Set<number>> = {}; // legacy highlight

  let entriesByTab: Record<string, LogEntry[]> = {};
  let nextEntryIdByTab: Record<string, number> = {};
  const MAX_LINES = 5000;
  
  // Search functionality
  let searchQuery = '';
  let searchCaseSensitive = false;
  let currentMatchIndex = 0;
  let matchIndices: number[] = [];
  let filteredEntriesByTab: Record<string, LogEntry[]> = {};
  
  // Tab management
  function addLogTab(podName: string, namespace: string, containerName?: string) {
    const tabId = `${namespace}/${podName}${containerName ? `/${containerName}` : ''}`;
    console.log('🔍 Adding tab:', { tabId, podName, namespace, containerName });
    console.log('🔍 Current tabs before:', tabs.map(t => t.id));
    
    // Check if tab already exists
    if (tabs.find(tab => tab.id === tabId)) {
      console.log('🔍 Tab already exists, switching to it');
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
    console.log('✅ Added new tab, total tabs:', tabs.length);
    console.log('✅ Updated tabs array:', tabs.map(t => t.id));
    
    // Load logs for this tab (initial load)
    loadLogs(tabId, podName, namespace, containerName, true);
  }
  
  function closeTab(tabId: string) {
    console.log('🗑️ Closing tab:', tabId);
    console.log('🗑️ Tabs before close:', tabs.map(t => t.id));
    
    tabs = tabs.filter(tab => tab.id !== tabId);
    
    console.log('🗑️ Tabs after close:', tabs.map(t => t.id));
    
    // Clean up state
    delete logs[tabId];
    delete loading[tabId];
    delete errors[tabId];
    delete logLines[tabId];
    delete lastSeenLine[tabId];
    delete newLogLines[tabId];
    delete entriesByTab[tabId];
    delete nextEntryIdByTab[tabId];
    
    // Switch to another tab if this was active
    if (activeTab === tabId) {
      activeTab = tabs.length > 0 ? tabs[0].id : '';
      console.log('🗑️ Switched to new active tab:', activeTab);
    }
  }
  
  function setActiveTab(tabId: string) {
    activeTab = tabId;
  }
  
  // Log loading - incremental approach
  async function loadLogs(tabId: string, podName: string, namespace: string, containerName?: string, isInitial = false) {
    console.log(`📥 Loading logs for ${tabId}, isInitial: ${isInitial}, followMode: ${followMode}`);
    loading[tabId] = true;
    errors[tabId] = '';
    
    try {
      const rawLogData = await invoke('kuboard_get_pod_logs', {
        podName,
        namespace,
        containerName: containerName || null,
        tailLines: isInitial ? tailLines : 50,
        follow: followMode
      });

      const newLogData = String(rawLogData);
      console.log(`📊 Log data received for ${tabId}, length: ${newLogData.length}`);

      const incomingLines = newLogData.split('\n').filter((line) => line.trim());
      const existingLines = logLines[tabId] || [];
      const lastSeen =
        lastSeenLine[tabId] || (existingLines.length > 0 ? existingLines[existingLines.length - 1] : '');
      const idx = lastSeen ? incomingLines.lastIndexOf(lastSeen) : -1;
      const linesToAdd = isInitial
        ? incomingLines
        : idx >= 0
          ? incomingLines.slice(idx + 1)
          : existingLines.length === 0
            ? incomingLines
            : [];

      if (linesToAdd.length > 0) {
        // Update last seen line for delta calculation
        lastSeenLine[tabId] = incomingLines[incomingLines.length - 1] || lastSeenLine[tabId];

        // Build entries append-only
        if (!entriesByTab[tabId]) entriesByTab[tabId] = [];
        if (nextEntryIdByTab[tabId] === undefined) nextEntryIdByTab[tabId] = 0;

        const newEntries: LogEntry[] = linesToAdd.map((line) => {
          const info = formatLogLine(line);
          const idNum = nextEntryIdByTab[tabId]++;
          return {
            id: `${tabId}:${idNum}`,
            timestamp: info.timestamp || new Date().toLocaleString(),
            level: info.level,
            message: info.message,
            isExpanded: false,
            isJson: info.isJson
          };
        });

        entriesByTab[tabId] = [...entriesByTab[tabId], ...newEntries];

        // Trim buffer if exceeding MAX_LINES
        const overage = Math.max(0, entriesByTab[tabId].length - MAX_LINES);
        if (overage > 0) {
          entriesByTab[tabId].splice(0, overage);
        }

        // Trigger reactivity only for the specific tab
        entriesByTab = { ...entriesByTab };

        if (followMode) {
          setTimeout(() => {
            autoScrollToBottom(tabId);
          }, 50);
        }
      }
    } catch (error) {
      console.error('Failed to load logs:', error);
      errors[tabId] = `Failed to load logs: ${error}`;
    } finally {
      loading[tabId] = false;
    }
  }
  
  // Find new lines by comparing arrays
  function findNewLines(existing: string[], newLines: string[]): string[] {
    if (existing.length === 0) return newLines;
    
    // Find the last few lines of existing logs to match against new logs
    const lastExistingLines = existing.slice(-10); // Check last 10 lines
    const newLinesToAdd: string[] = [];
    
    // Find where the new logs start (skip overlapping lines)
    let startIndex = 0;
    for (let i = 0; i < newLines.length; i++) {
      const found = lastExistingLines.some(existingLine => 
        existingLine === newLines[i] || 
        existingLine.includes(newLines[i]) || 
        newLines[i].includes(existingLine)
      );
      if (!found) {
        startIndex = i;
        break;
      }
    }
    
    // Return only truly new lines
    return newLines.slice(startIndex);
  }
  
  function refreshCurrentLogs() {
    const activeTabData = tabs.find(tab => tab.id === activeTab);
    if (activeTabData) {
      loadLogs(activeTabData.id, activeTabData.podName, activeTabData.namespace, activeTabData.containerName, false);
    }
  }
  
  function refreshAllLogs() {
    console.log('🔄 Refreshing all logs, followMode:', followMode, 'tabs:', tabs.length);
    tabs.forEach(tab => {
      loadLogs(tab.id, tab.podName, tab.namespace, tab.containerName, false);
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
      
      // Immediately scroll to bottom when enabling follow mode
      if (activeTab) {
        // Reset user scrolling flag so auto-scroll works
        isUserScrolling = false;
        // Force scroll to bottom
        setTimeout(() => {
          scrollToBottom(activeTab);
        }, 100);
        // Also try again after a longer delay to ensure we get to the very bottom
        setTimeout(() => {
          scrollToBottom(activeTab);
        }, 300);
      }
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
      // Re-enable follow mode when user scrolls back to bottom
      if (!followMode) {
        followMode = true;
        // Restart refresh interval if it was stopped
        if (!refreshInterval) {
          refreshInterval = setInterval(() => {
            if (activeTab) {
              const tab = tabs.find(t => t.id === activeTab);
              if (tab) {
                loadLogs(activeTab, tab.podName, tab.namespace, tab.containerName);
              }
            }
          }, 2000); // 2 second refresh interval
        }
      }
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
      // Force scroll to absolute bottom
      container.scrollTop = container.scrollHeight;
      // Use requestAnimationFrame to ensure DOM is updated
      requestAnimationFrame(() => {
        if (container) {
          container.scrollTop = container.scrollHeight;
        }
      });
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
    console.log('🚀 Opening logs for:', { podName, namespace, containerName });
    console.log('🚀 Current state - isOpen:', isOpen, 'tabs count:', tabs.length);
    addLogTab(podName, namespace, containerName);
    isOpen = true;
    console.log('🚀 After opening - isOpen:', isOpen, 'tabs count:', tabs.length);
    
    // Start follow mode if not already running
    if (followMode && !refreshInterval) {
      refreshInterval = setInterval(() => {
        refreshAllLogs();
      }, 2000);
    }
  }
  
  // Format log line for display
  function formatLogLine(line: string): { timestamp: string, level: string, message: string, isError: boolean, isJson: boolean } {
    if (!line.trim()) {
      return { timestamp: '', level: 'INFO', message: '', isError: false, isJson: false };
    }

    // Simple approach: use the raw log line as-is
    // Most logs already have timestamps at the front, so we'll display them as-is
    const isJson = line.trim().startsWith('{') && line.trim().endsWith('}');
    const isError = line.toLowerCase().includes('error') || line.toLowerCase().includes('fatal') || line.toLowerCase().includes('panic');
    
    return { 
      timestamp: '', // No custom timestamp parsing - use raw log
      level: 'INFO', // Default level - logs will show their own levels
      message: line, // Use the full log line as-is
      isError, 
      isJson 
    };
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

  // Search functionality
  function performSearch() {
    if (!activeTab || !searchQuery.trim()) {
      filteredEntriesByTab[activeTab] = entriesByTab[activeTab] || [];
      matchIndices = [];
      currentMatchIndex = 0;
      return;
    }

    const entries = entriesByTab[activeTab] || [];
    const query = searchCaseSensitive ? searchQuery : searchQuery.toLowerCase();
    const filtered: LogEntry[] = [];
    const matches: number[] = [];

    entries.forEach((entry, index) => {
      const message = searchCaseSensitive ? entry.message : entry.message.toLowerCase();
      if (message.includes(query)) {
        filtered.push(entry);
        matches.push(index);
      }
    });

    filteredEntriesByTab[activeTab] = filtered;
    matchIndices = matches;
    currentMatchIndex = matches.length > 0 ? 0 : -1;

    // Scroll to first match
    if (matches.length > 0 && logContainers[activeTab]) {
      setTimeout(() => {
        const container = logContainers[activeTab];
        if (container) {
          const logLines = container.querySelectorAll('.log-line');
          // First match will be at index 0 in filtered view
          if (logLines.length > 0) {
            logLines[0].scrollIntoView({ behavior: 'smooth', block: 'center' });
          }
        }
      }, 150);
    }
  }

  function navigateMatch(direction: 'next' | 'prev') {
    if (matchIndices.length === 0) return;

    if (direction === 'next') {
      currentMatchIndex = (currentMatchIndex + 1) % matchIndices.length;
    } else {
      currentMatchIndex = currentMatchIndex <= 0 ? matchIndices.length - 1 : currentMatchIndex - 1;
    }

    // Scroll to match
    setTimeout(() => {
      const container = logContainers[activeTab];
      if (container && matchIndices.length > 0 && currentMatchIndex >= 0) {
        const logLines = container.querySelectorAll('.log-line');
        // Find the line that corresponds to the current match index
        const targetOriginalIndex = matchIndices[currentMatchIndex];
        const filteredEntries = filteredEntriesByTab[activeTab] || [];
        const filteredIndex = filteredEntries.findIndex((entry) => {
          const originalIndex = entriesByTab[activeTab]?.indexOf(entry) ?? -1;
          return originalIndex === targetOriginalIndex;
        });
        
        if (filteredIndex >= 0 && logLines[filteredIndex]) {
          logLines[filteredIndex].scrollIntoView({ behavior: 'smooth', block: 'center' });
        }
      }
    }, 50);
  }

  function clearSearch() {
    searchQuery = '';
    searchCaseSensitive = false;
    currentMatchIndex = 0;
    matchIndices = [];
    if (activeTab && entriesByTab[activeTab]) {
      filteredEntriesByTab[activeTab] = entriesByTab[activeTab];
    }
  }

  function highlightText(text: string, query: string): string {
    if (!query.trim()) return text;
    const queryRegex = new RegExp(`(${query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, searchCaseSensitive ? 'g' : 'gi');
    return text.replace(queryRegex, '<mark class="search-highlight">$1</mark>');
  }

  // Reactive: update filtered entries when search query or active tab changes
  $: {
    if (activeTab && entriesByTab[activeTab] !== undefined) {
      if (searchQuery.trim()) {
        performSearch();
      } else {
        filteredEntriesByTab[activeTab] = entriesByTab[activeTab] || [];
        matchIndices = [];
        currentMatchIndex = 0;
      }
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
        role="button"
        tabindex="0"
      ></div>
      
      <!-- Combined Header with Tabs and Controls -->
      <div class="logs-header">
        <!-- Tabs Section -->
        <div class="logs-tabs-container">
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
          {/if}
        </div>
        
        <!-- Controls Section -->
        <div class="logs-controls">
          <!-- Search Box -->
          <div class="search-container">
            <input
              type="text"
              class="search-input"
              placeholder="Search logs..."
              bind:value={searchQuery}
              oninput={performSearch}
              onkeydown={(e) => {
                if (e.key === 'Enter' && e.shiftKey) {
                  e.preventDefault();
                  navigateMatch('prev');
                } else if (e.key === 'Enter') {
                  e.preventDefault();
                  navigateMatch('next');
                } else if (e.key === 'Escape') {
                  e.stopPropagation();
                  clearSearch();
                }
              }}
            />
            {#if searchQuery}
              <span class="search-results">
                {matchIndices.length > 0 ? `${currentMatchIndex + 1}/${matchIndices.length}` : '0'}
              </span>
              <button
                class="control-button compact search-nav"
                onclick={() => navigateMatch('prev')}
                title="Previous match (Shift+Enter)"
                disabled={matchIndices.length === 0}
              >
                ↑
              </button>
              <button
                class="control-button compact search-nav"
                onclick={() => navigateMatch('next')}
                title="Next match (Enter)"
                disabled={matchIndices.length === 0}
              >
                ↓
              </button>
              <button
                class="control-button compact search-nav"
                onclick={clearSearch}
                title="Clear search (Esc)"
              >
                ✕
              </button>
            {/if}
            <button
              class="control-button compact {searchCaseSensitive ? 'active' : ''}"
              onclick={() => { searchCaseSensitive = !searchCaseSensitive; performSearch(); }}
              title="Case sensitive search"
            >
              Aa
            </button>
          </div>
          <button 
            class="control-button compact {followMode ? 'active' : ''}" 
            onclick={toggleFollowMode}
            title={followMode ? 'Stop following logs' : 'Follow logs (auto-refresh)'}
          >
            {followMode ? '⏸️' : '▶️'}
          </button>
          <button class="control-button compact" onclick={refreshCurrentLogs} title="Refresh current logs">
            🔄
          </button>
          <button class="control-button compact" onclick={onClose} title="Close logs window">
            ✕
          </button>
        </div>
      </div>
        
        <!-- Log Content -->
        <div class="logs-content" role="tabpanel" aria-labelledby="log-tab-{activeTab}">
          {#if loading[activeTab] && !(entriesByTab[activeTab] && entriesByTab[activeTab].length > 0)}
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
          {:else if entriesByTab[activeTab] && entriesByTab[activeTab].length > 0}
            <div 
              class="logs-viewer"
              bind:this={logContainers[activeTab]}
              onscroll={(e) => handleScroll(e, activeTab)}
            >
              {#each (searchQuery.trim() ? (filteredEntriesByTab[activeTab] || []) : (entriesByTab[activeTab] || [])) as entry, idx (entry.id)}
                {@const originalIndex = entriesByTab[activeTab]?.indexOf(entry) ?? -1}
                {@const isCurrentMatch = searchQuery.trim() && matchIndices.length > 0 && currentMatchIndex >= 0 && matchIndices[currentMatchIndex] === originalIndex}
                {@const displayMessage = entry.isExpanded ? entry.message : (entry.message.split('\n')[0].length > 300 ? entry.message.split('\n')[0].slice(0, 300) + '…' : entry.message.split('\n')[0])}
                <div class="log-line {getLevelClass(entry.level || 'INFO')} {entry.isJson ? 'log-json' : ''} {isCurrentMatch ? 'highlight-match' : ''}">
                  <span 
                    class="log-message" 
                    role="button" 
                    tabindex="0" 
                    onclick={() => entry.isExpanded = !entry.isExpanded} 
                    onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && (entry.isExpanded = !entry.isExpanded)}
                  >
                    {@html searchQuery.trim() ? highlightText(displayMessage, searchQuery) : displayMessage}
                  </span>
                </div>
              {/each}
              {#if searchQuery.trim() && (!filteredEntriesByTab[activeTab] || filteredEntriesByTab[activeTab].length === 0)}
                <div class="search-no-results">
                  <p>No results found for "{searchQuery}"</p>
                </div>
              {/if}
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
            {#if entriesByTab[activeTab]}
              <span>{entriesByTab[activeTab].length} lines</span>
              {#if followMode}
                <span class="follow-indicator">📡 Following</span>
              {:else}
                <span class="follow-indicator paused">⏸️ Paused</span>
              {/if}
            {/if}
          </div>
        </div>
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
    padding: 4px 12px;
    border-bottom: 1px solid var(--border-color);
    background: var(--background-secondary);
    min-height: 32px;
  }
  
  .logs-tabs-container {
    flex: 1;
    overflow-x: auto;
    margin-right: 12px;
    min-width: 0; /* Allow flex item to shrink below content size */
  }
  
  .logs-controls {
    display: flex;
    gap: 4px;
    align-items: center;
    flex-shrink: 0; /* Prevent controls from shrinking */
  }

  .search-container {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-right: 8px;
    background: var(--background-primary);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    padding: 2px 4px;
  }

  .search-input {
    background: transparent;
    border: none;
    color: var(--text-primary);
    padding: 4px 8px;
    font-size: 12px;
    min-width: 150px;
    outline: none;
  }

  .search-input::placeholder {
    color: var(--text-muted);
  }

  .search-input:focus {
    outline: none;
  }

  .search-results {
    color: var(--text-secondary);
    font-size: 11px;
    padding: 0 4px;
    white-space: nowrap;
  }

  .search-nav {
    padding: 2px 6px;
    min-width: 24px;
    height: 20px;
  }

  .search-nav:disabled {
    opacity: 0.5;
    cursor: not-allowed;
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
  
  .control-button.compact {
    padding: 4px 8px;
    font-size: 11px;
    min-width: 28px;
    text-align: center;
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
    background: transparent;
    border-bottom: none;
    overflow-x: auto;
    min-width: max-content; /* Ensure tabs don't compress too much */
  }
  
  .log-tab {
    display: flex;
    align-items: center;
    padding: 4px 12px;
    background: var(--background-secondary);
    border-right: 1px solid var(--border-color);
    cursor: pointer;
    transition: all 0.2s ease;
    min-width: 100px;
    white-space: nowrap;
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
    font-size: 11px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 80px;
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
    margin-left: 4px;
    padding: 1px 2px;
    border-radius: 2px;
    font-size: 9px;
    min-width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
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
  
  .logs-refreshing {
    position: sticky;
    top: 0;
    background: var(--background-card);
    border-bottom: 1px solid var(--border-primary);
    padding: 4px 8px;
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 11px;
    color: var(--text-muted);
    z-index: 10;
  }
  
  .refresh-indicator {
    animation: spin 1s linear infinite;
  }
  
  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
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
    opacity: 0;
    animation: fadeInUp 0.3s ease-out forwards;
    transition: background-color 0.2s ease, border-left 0.3s ease;
  }
  
  .log-line.new-log {
    border-left: 3px solid var(--info-color);
    background: rgba(59, 130, 246, 0.05);
  }
  
  .log-line:hover {
    background: var(--background-secondary);
  }

  .log-line.highlight-match {
    background: rgba(245, 158, 11, 0.2);
    border-left: 3px solid #f59e0b;
    animation: highlightPulse 1s ease-out;
  }

  @keyframes highlightPulse {
    0% {
      background: rgba(245, 158, 11, 0.4);
    }
    100% {
      background: rgba(245, 158, 11, 0.2);
    }
  }

  .search-highlight {
    background: rgba(245, 158, 11, 0.3);
    color: var(--text-primary);
    padding: 1px 2px;
    border-radius: 2px;
    font-weight: 600;
  }

  .search-no-results {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 40px;
    color: var(--text-muted);
    font-size: 14px;
  }
  
  .log-json {
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  }
  
  .log-json .log-message {
    color: var(--text-primary);
    white-space: pre-wrap;
    word-break: break-word;
  }
  
  @keyframes fadeInUp {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
  
  .log-message {
    color: var(--text-primary);
    flex: 1;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-size: 13px;
    line-height: 1.4;
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
    display: flex;
    align-items: center;
    gap: 12px;
  }
  
  .follow-indicator {
    font-size: 11px;
    padding: 2px 6px;
    border-radius: 4px;
    background: rgba(34, 197, 94, 0.1);
    color: #22c55e;
    border: 1px solid rgba(34, 197, 94, 0.2);
  }
  
  .follow-indicator.paused {
    background: rgba(156, 163, 175, 0.1);
    color: #9ca3af;
    border: 1px solid rgba(156, 163, 175, 0.2);
  }
  
  .empty-hint {
    font-size: 11px;
    margin-top: 4px;
  }
</style>

