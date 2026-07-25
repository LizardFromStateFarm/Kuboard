import { writable } from 'svelte/store';

export type KeyAction = {
  key: string;
  action: () => void;
  description: string;
  category: string;
};

export const activeSelectionIndex = writable<number>(-1);
export const showKeyboardHelp = writable<boolean>(false);

// Global shortcut registration could be done here if needed
