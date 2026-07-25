// Terminal session store managing pod exec terminal state per tab session
import { writable, get } from 'svelte/store';
import { activeTabSessionIdStore } from './logs';

export interface TerminalTabState {
  isOpen: boolean;
  podName: string;
  namespace: string;
  containerName?: string;
  isMinimized?: boolean;
}

export const terminalSessionStore = writable<Record<string, TerminalTabState>>({});

export const activeTerminalState = writable<TerminalTabState>({
  isOpen: false,
  podName: '',
  namespace: '',
  containerName: undefined,
  isMinimized: false,
});

// Switch terminal context when switching profile tabs
export function switchTerminalContext(oldTabId: string | null, newTabId: string) {
  const allSessions = get(terminalSessionStore);
  
  if (oldTabId) {
    const currentState = get(activeTerminalState);
    allSessions[oldTabId] = { ...currentState };
  }

  const restored = allSessions[newTabId] || {
    isOpen: false,
    podName: '',
    namespace: '',
    containerName: undefined,
    isMinimized: false,
  };

  terminalSessionStore.set(allSessions);
  activeTerminalState.set(restored);
}

// Open global pod terminal
export function openGlobalPodTerminal(tabId: string | undefined, podName: string, namespace: string, containerName?: string) {
  const currentTabId = tabId || get(activeTabSessionIdStore);
  if (!currentTabId) return;

  const newState: TerminalTabState = {
    isOpen: true,
    podName,
    namespace,
    containerName,
    isMinimized: false,
  };

  const allSessions = get(terminalSessionStore);
  allSessions[currentTabId] = newState;
  terminalSessionStore.set(allSessions);

  if (currentTabId === get(activeTabSessionIdStore)) {
    activeTerminalState.set(newState);
  }
}

// Close global pod terminal
export function closeGlobalPodTerminal(tabId?: string) {
  const currentTabId = tabId || get(activeTabSessionIdStore);
  if (!currentTabId) return;

  const allSessions = get(terminalSessionStore);
  if (allSessions[currentTabId]) {
    allSessions[currentTabId].isOpen = false;
    terminalSessionStore.set(allSessions);
  }

  if (currentTabId === get(activeTabSessionIdStore)) {
    activeTerminalState.update(s => ({ ...s, isOpen: false }));
  }
}
