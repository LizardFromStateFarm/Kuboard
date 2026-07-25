import { writable } from 'svelte/store';

export type NavTarget = {
  tab: string;
  resourceType?: string;
  resourceName?: string;
  namespace?: string;
};

export const navigationStore = writable<NavTarget | null>(null);

export function navigateTo(target: NavTarget) {
  navigationStore.set(target);
}
