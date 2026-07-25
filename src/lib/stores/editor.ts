import { writable } from 'svelte/store';

export type EditorRequest = {
  resource: any;
  resourceType: string;
};

export const editorStore = writable<EditorRequest | null>(null);

export function openEditor(resource: any, resourceType: string) {
  editorStore.set({ resource, resourceType });
}

export function closeEditor() {
  editorStore.set(null);
}
