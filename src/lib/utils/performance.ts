// Copyright 2025 Kuboard Contributors
// Licensed under the MIT License - see LICENSE file for details

// Performance optimization utilities for faster context switching and loading

import { writable, derived } from 'svelte/store';

// Cache store for frequently accessed data
export const clusterCache = writable<{
  overview: any | null;
  nodes: any[] | null;
  namespaces: any[] | null;
  pods: any[] | null;
  deployments: any[] | null;
  lastUpdated: number;
  contextName: string;
}>({
  overview: null,
  nodes: null,
  namespaces: null,
  pods: null,
  deployments: null,
  lastUpdated: 0,
  contextName: ''
});

// Cache validity check (30 second cache)
export const isCacheValid = derived(
  clusterCache,
  ($cache) => {
    const now = Date.now();
    const cacheAge = now - $cache.lastUpdated;
    return cacheAge < 30000; // 30 seconds
  }
);

// Optimized context switching with preloading
export async function optimizedContextSwitch(
  contextName: string,
  invoke: (command: string, args?: any) => Promise<any>
) {
  console.log(`🚀 Optimized context switch to: ${contextName}`);
  
  try {
    // Set context
    await invoke('kuboard_set_context_optimized', { contextName });
    
    // Preload cluster overview in parallel
    const overviewPromise = invoke('kuboard_get_cluster_overview_optimized');
    
    // Wait for overview
    const overview = await overviewPromise;
    
    // Update cache
    clusterCache.update(cache => ({
      ...cache,
      overview,
      contextName,
      lastUpdated: Date.now()
    }));
    
    console.log('✅ Context switch completed with preloading');
    return overview;
    
  } catch (error) {
    console.error('❌ Optimized context switch failed:', error);
    throw error;
  }
}

// Batch resource loading for better performance
export async function loadAllResourcesOptimized(
  invoke: (command: string, args?: any) => Promise<any>
) {
  console.log('🚀 Loading all resources (optimized)');
  
  try {
    // Check cache first
    let cache;
    clusterCache.subscribe(c => cache = c)();
    
    if (cache && cache.contextName && isCacheValid) {
      console.log('📦 Returning cached resources');
      return {
        nodes: cache.nodes || [],
        namespaces: cache.namespaces || [],
        pods: cache.pods || [],
        deployments: cache.deployments || []
      };
    }
    
    // Load all resources in one call
    const resources = await invoke('kuboard_get_all_resources_optimized');
    
    // Update cache
    clusterCache.update(cache => ({
      ...cache,
      nodes: resources.nodes,
      namespaces: resources.namespaces,
      pods: resources.pods,
      deployments: resources.deployments,
      lastUpdated: Date.now()
    }));
    
    console.log('✅ All resources loaded (optimized)');
    return resources;
    
  } catch (error) {
    console.error('❌ Failed to load resources:', error);
    throw error;
  }
}

// Debounced refresh to prevent excessive API calls
export function createDebouncedRefresh(
  callback: () => void,
  delay: number = 1000
) {
  let timeoutId: NodeJS.Timeout;
  
  return () => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(callback, delay);
  };
}

// Optimized metrics loading with caching
export async function loadMetricsOptimized(
  invoke: (command: string, args?: any) => Promise<any>,
  nodeName?: string
) {
  console.log('🚀 Loading metrics (optimized)');
  
  try {
    // Check if metrics server is available first
    const availability = await invoke('kuboard_check_metrics_availability');
    
    if (!availability.available) {
      throw new Error('Metrics server not available');
    }
    
    // Load metrics
    const metrics = nodeName 
      ? await invoke('kuboard_get_node_metrics', { nodeName })
      : await invoke('kuboard_get_node_metrics');
    
    console.log('✅ Metrics loaded (optimized)');
    return metrics;
    
  } catch (error) {
    console.error('❌ Failed to load metrics:', error);
    throw error;
  }
}

// Preload strategy for faster startup
export async function preloadCriticalData(
  invoke: (command: string, args?: any) => Promise<any>
) {
  console.log('🚀 Preloading critical data');
  
  try {
    // Load contexts first
    const contexts = await invoke('kuboard_list_contexts');
    
    // If there's a current context, preload overview
    if (contexts.current_context) {
      const overview = await invoke('kuboard_get_cluster_overview_optimized');
      
      // Update cache
      clusterCache.update(cache => ({
        ...cache,
        overview,
        contextName: contexts.current_context,
        lastUpdated: Date.now()
      }));
    }
    
    console.log('✅ Critical data preloaded');
    return contexts;
    
  } catch (error) {
    console.error('❌ Failed to preload data:', error);
    throw error;
  }
}

// Cache invalidation
export function invalidateCache() {
  console.log('🗑️ Invalidating cache');
  clusterCache.update(cache => ({
    ...cache,
    overview: null,
    nodes: null,
    namespaces: null,
    pods: null,
    deployments: null,
    lastUpdated: 0
  }));
}

// Performance monitoring
export function startPerformanceMonitoring() {
  const startTime = performance.now();
  
  return {
    end: () => {
      const endTime = performance.now();
      const duration = endTime - startTime;
      console.log(`⏱️ Operation completed in ${duration.toFixed(2)}ms`);
      return duration;
    }
  };
}
