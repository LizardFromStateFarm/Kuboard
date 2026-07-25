import { writable, get } from 'svelte/store';

export interface LogTabItem {
  id: string;
  podName: string;
  namespace: string;
  containerName?: string;
  type?: 'log' | 'terminal';
}

export interface ProfileTabLogsState {
  isOpen: boolean;
  activeTab: string;
  tabs: LogTabItem[];
}

export const activeTabSessionIdStore = writable<string>('default-tab');
export const logsSessionStore = writable<Record<string, ProfileTabLogsState>>({});

export const activeLogsState = writable<ProfileTabLogsState>({
  isOpen: false,
  activeTab: '',
  tabs: []
});

/**
 * Open pod logs globally for current active tab session ID.
 */
export function openGlobalPodLogs(tabSessionId: string | undefined, podName: string, namespace: string, containerName?: string) {
  const currentActiveTabId = get(activeTabSessionIdStore);
  const sessionKey = (tabSessionId && tabSessionId.trim() !== '') ? tabSessionId : (currentActiveTabId || 'default-tab');
  
  activeTabSessionIdStore.set(sessionKey);

  logsSessionStore.update((sessions) => {
    const session = sessions[sessionKey] || { isOpen: true, activeTab: '', tabs: [] };
    const logTabId = containerName ? `${podName}:${containerName}` : podName;

    let updatedTabs = [...session.tabs];
    const existingIndex = updatedTabs.findIndex((t) => t.id === logTabId);

    if (existingIndex < 0) {
      updatedTabs.push({ id: logTabId, podName, namespace, containerName, type: 'log' });
    }

    const newSession = {
      isOpen: true,
      activeTab: logTabId,
      tabs: updatedTabs
    };

    sessions[sessionKey] = newSession;
    activeLogsState.set(newSession);
    return sessions;
  });
}

/**
 * Open pod exec terminal tab globally in logs window.
 */
export function openGlobalPodTerminal(tabSessionId: string | undefined, podName: string, namespace: string, containerName?: string) {
  const currentActiveTabId = get(activeTabSessionIdStore);
  const sessionKey = (tabSessionId && tabSessionId.trim() !== '') ? tabSessionId : (currentActiveTabId || 'default-tab');
  
  activeTabSessionIdStore.set(sessionKey);

  logsSessionStore.update((sessions) => {
    const session = sessions[sessionKey] || { isOpen: true, activeTab: '', tabs: [] };
    const termTabId = `exec:${containerName ? `${podName}:${containerName}` : podName}`;

    let updatedTabs = [...session.tabs];
    const existingIndex = updatedTabs.findIndex((t) => t.id === termTabId);

    if (existingIndex < 0) {
      updatedTabs.push({ id: termTabId, podName, namespace, containerName, type: 'terminal' });
    }

    const newSession = {
      isOpen: true,
      activeTab: termTabId,
      tabs: updatedTabs
    };

    sessions[sessionKey] = newSession;
    activeLogsState.set(newSession);
    return sessions;
  });
}

/**
 * Handle tab switch: Save old tab session state, restore new tab session state.
 */
export function switchLogsContext(oldTabSessionId: string | null, newTabSessionId: string) {
  if (oldTabSessionId && oldTabSessionId !== newTabSessionId) {
    // Save current active logs state into oldTabSessionId in session store
    const currentLogs = get(activeLogsState);
    logsSessionStore.update((sessions) => {
      sessions[oldTabSessionId] = { ...currentLogs };
      return sessions;
    });
  }

  const sessionKey = newTabSessionId || 'default-tab';
  activeTabSessionIdStore.set(sessionKey);

  // Restore newTabSessionId session if it exists, otherwise initialize closed & empty
  const sessions = get(logsSessionStore);
  const restoredSession = sessions[sessionKey] || {
    isOpen: false,
    activeTab: '',
    tabs: []
  };

  activeLogsState.set(restoredSession);
}

/**
 * Close a log tab for current active tab session ID.
 */
export function closeGlobalLogTab(tabSessionId: string | undefined, logTabId: string) {
  const currentActiveTabId = get(activeTabSessionIdStore);
  const sessionKey = (tabSessionId && tabSessionId.trim() !== '') ? tabSessionId : (currentActiveTabId || 'default-tab');

  logsSessionStore.update((sessions) => {
    const session = sessions[sessionKey];
    if (!session) return sessions;

    const remainingTabs = session.tabs.filter((t) => t.id !== logTabId);
    let newActiveTab = session.activeTab;

    if (session.activeTab === logTabId) {
      newActiveTab = remainingTabs.length > 0 ? remainingTabs[remainingTabs.length - 1].id : '';
    }

    const newSession = {
      isOpen: remainingTabs.length > 0 ? session.isOpen : false,
      activeTab: newActiveTab,
      tabs: remainingTabs
    };

    sessions[sessionKey] = newSession;
    activeLogsState.set(newSession);
    return sessions;
  });
}

/**
 * Toggle or set isOpen for active tab session ID.
 */
export function setGlobalLogsOpen(tabSessionId: string | undefined, isOpen: boolean) {
  const currentActiveTabId = get(activeTabSessionIdStore);
  const sessionKey = (tabSessionId && tabSessionId.trim() !== '') ? tabSessionId : (currentActiveTabId || 'default-tab');

  logsSessionStore.update((sessions) => {
    const session = sessions[sessionKey] || { isOpen: false, activeTab: '', tabs: [] };
    const newSession = { ...session, isOpen };
    sessions[sessionKey] = newSession;
    activeLogsState.set(newSession);
    return sessions;
  });
}
