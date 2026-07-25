import { writable } from 'svelte/store';

export type XRayRequest = {
  resource: any;
  resourceType: string;
};

export const xrayStore = writable<XRayRequest | null>(null);

export function openXRay(resource: any, resourceType: string) {
  xrayStore.set({ resource, resourceType });
}

export function closeXRay() {
  xrayStore.set(null);
}
