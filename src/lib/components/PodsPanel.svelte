<!-- Kuboard Pods Panel Component -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { createEventDispatcher } from 'svelte';
  import { formatMemory, formatCPU } from '$lib/utils/formatters';
  import MetricsGraph from './MetricsGraph.svelte';
  import LogsWindow from './LogsWindow.svelte';
  import PodDetails from './PodDetails.svelte';
  import QuickActionsMenu from './QuickActionsMenu.svelte';

  // Props
  export let currentContext: any = null;
  export let pods: any[] = [];

  // State
  let selectedPod: any = null;
  let selectedPodRaw: any = null;
  let showFullDetails: boolean = false;
  let refreshTimer: any;
  let logsWindowOpen = false;
  let logsWindowRef: LogsWindow;
  let isLoading: boolean = false;
  
  // Metrics state
  let podMetrics: any = null;
  let metricsLoading: boolean = false;
  let metricsError: string | null = null;
  let selectedResourceType: 'cpu' | 'memory' = 'cpu';
  let selectedTimeRange: number = 30; // Default to 30 minutes
  
  // Container metrics state
  let selectedContainer: any = null;
  let containerMetrics: any = null;
  let containerMetricsLoading: boolean = false;
  let containerMetricsError: string | null = null;
  let selectedContainerResourceType: 'cpu' | 'memory' = 'cpu';
  
  // Events state
  let podEvents: any[] = [];
  let eventsLoading: boolean = false;
  let eventsError: string | null = null;

  // Watch-based live update state
  let livePods: any[] | null = null;
  let watchError: string | null = null;
  let watchActive = false;
  let podsMap = new Map<string, any>(); // Track pods by key for efficient updates

  // Sorting state
  let sortColumn: string | null = null;
  let sortDirection: 'asc' | 'desc' | null = null;

  // Search state
  let searchQuery: string = '';

  // Quick Actions Menu state
  let contextMenuVisible = false;
  let contextMenuPosition = { x: 0, y: 0 };
  let contextMenuPod: any = null;
  let yamlViewerVisible = false;
  let yamlContent = '';

  function handleContextMenu(event: MouseEvent, pod: any) {
    event.preventDefault();
    event.stopPropagation();
    contextMenuPod = pod;
    contextMenuPosition = { x: event.clientX, y: event.clientY };
    contextMenuVisible = true;
  }

  function handleActionMenuClose() {
    contextMenuVisible = false;
    contextMenuPod = null;
  }

  function handleActionDeleted(event: CustomEvent) {
    // Watch will handle the update automatically, but we can optimistically remove
    if (contextMenuPod) {
      const keyToRemove = getPodKey(contextMenuPod);
      podsMap.delete(keyToRemove);
      livePods = [...Array.from(podsMap.values())]; // Force new array reference
      console.log(`🗑️ Optimistically deleted pod: ${keyToRemove}, total: ${livePods.length}`);
    }
    handleActionMenuClose();
  }

  function handleActionRestarted(event: CustomEvent) {
    // Optimistically remove (controller will recreate, watch will detect)
    if (contextMenuPod) {
      const keyToRemove = getPodKey(contextMenuPod);
      podsMap.delete(keyToRemove);
      livePods = [...Array.from(podsMap.values())]; // Force new array reference
      console.log(`🔄 Optimistically removed pod for restart: ${keyToRemove}, total: ${livePods.length}`);
    }
    handleActionMenuClose();
  }

  function handleViewYaml(event: CustomEvent) {
    yamlContent = event.detail.yaml;
    yamlViewerVisible = true;
    // Don't close menu for view-yaml, it opens a modal
  }

  function closeYamlViewer() {
    yamlViewerVisible = false;
    yamlContent = '';
    handleActionMenuClose();
  }

  function handleActionCopied(event: CustomEvent) {
    // Show a brief notification (could enhance later)
    console.log(`Copied ${event.detail.type}: ${event.detail.value}`);
    handleActionMenuClose();
  }

  function handleActionDescribe(event: CustomEvent) {
    // For now, just log - will implement describe view later
    console.log('Describe action for:', event.detail.resource);
    handleActionMenuClose();
  }

  function handleActionEdit(event: CustomEvent) {
    // For now, just log - will implement edit view later
    console.log('Edit action for:', event.detail.resource);
    handleActionMenuClose();
  }

  function getRenderPods(): any[] {
    return livePods ?? pods ?? [];
  }

  function getPodKey(pod: any): string {
    return pod?.metadata?.uid || `${pod?.metadata?.namespace || 'default'}/${pod?.metadata?.name || ''}`;
  }

  // Handle watch events
  function handleWatchEvent(event: any) {
    console.log('📡 Raw watch event received:', event);
    const { event_type, pod } = event;
    
    if (!pod || !pod.metadata) {
      console.error('⚠️ Invalid watch event: missing pod or metadata', event);
      return;
    }
    
    const key = getPodKey(pod);
    const eventTypeStr = String(event_type); // Ensure it's a string
    
    console.log(`📡 Watch event: ${eventTypeStr} for pod ${key}`);
    console.log(`📊 Current podsMap size: ${podsMap.size}, livePods length: ${livePods?.length ?? 0}`);
    
    switch (eventTypeStr) {
      case 'Added':
        // Add new pod
        if (!podsMap.has(key)) {
          podsMap.set(key, pod);
          livePods = [...Array.from(podsMap.values())]; // Force new array reference
          console.log(`✅ Added pod: ${key}, total: ${livePods.length}`);
        } else {
          console.log(`ℹ️ Pod ${key} already exists, skipping add`);
        }
        break;
        
      case 'Modified':
        // Update existing pod - this includes pods being terminated (have deletionTimestamp)
        const isTerminating = pod.metadata?.deletionTimestamp ? 'Terminating' : pod.status?.phase;
        podsMap.set(key, pod);
        livePods = [...Array.from(podsMap.values())]; // Force new array reference
        console.log(`🔄 Modified pod: ${key}, status: ${isTerminating}, total: ${livePods.length}`);
        break;
        
      case 'Deleted':
        // Remove pod
        if (podsMap.has(key)) {
          podsMap.delete(key);
          livePods = [...Array.from(podsMap.values())]; // Force new array reference
          console.log(`🗑️ Deleted pod: ${key}, total: ${livePods.length}`);
        } else {
          console.log(`ℹ️ Pod ${key} not in map, skipping delete`);
        }
        break;
        
      default:
        console.warn(`⚠️ Unknown event type: ${eventTypeStr}`, event);
    }
    
    console.log(`📊 After update - podsMap size: ${podsMap.size}, livePods length: ${livePods?.length ?? 0}`);
    watchError = null;
  }

  function handleWatchError(error: any) {
    console.error('Watch error:', error);
    watchError = error?.error || String(error) || 'Watch connection error';
  }

  // Start watch stream
  async function startWatch() {
    if (!currentContext) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('kuboard_stop_pod_watch'); // Stop any existing watch
      await invoke('kuboard_start_pod_watch');
      watchActive = true;
      watchError = null;
      console.log('✅ Watch started');
    } catch (e: any) {
      console.error('Failed to start watch:', e);
      watchError = String(e);
      watchActive = false;
    }
  }

  // Stop watch stream
  async function stopWatch() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('kuboard_stop_pod_watch');
      watchActive = false;
      console.log('🛑 Watch stopped');
    } catch (e: any) {
      console.error('Failed to stop watch:', e);
    }
  }

  // Initialize pods from initial list
  function initializePods() {
    if (pods && pods.length > 0) {
      podsMap.clear();
      for (const pod of pods) {
        podsMap.set(getPodKey(pod), pod);
      }
      livePods = [...Array.from(podsMap.values())]; // Force new array reference
      console.log(`📋 Initialized ${livePods.length} pods`);
    }
  }

  // Event dispatcher
  const dispatch = createEventDispatcher();

  // Debug logging
  console.log('🚀 PodsPanel component script loaded');
  
  // Set loading state when context changes but pods haven't loaded yet
  $: if (currentContext && (!pods || pods.length === 0)) {
    isLoading = true;
  } else {
    isLoading = false;
  }

  // Select pod (for dispatching to other components if needed)
  function selectPod(pod: any) {
    selectedPod = {
      name: pod.metadata?.name || 'Unknown',
      namespace: pod.metadata?.namespace || 'default',
      status: pod.status?.phase || 'Unknown',
      nodeName: pod.spec?.nodeName || 'Unknown',
      hostIP: pod.status?.hostIP || 'Unknown',
      podIP: pod.status?.podIP || 'Unknown',
      restartCount: pod.status?.containerStatuses?.[0]?.restartCount || 0,
      readyContainers: pod.status?.containerStatuses?.filter((c: any) => c.ready).length || 0,
      totalContainers: pod.status?.containerStatuses?.length || 0,
      creationTimestamp: pod.metadata?.creationTimestamp || 'Unknown',
      labels: pod.metadata?.labels || {},
      annotations: pod.metadata?.annotations || {},
      containers: pod.spec?.containers || [],
      containerStatuses: pod.status?.containerStatuses || []
    };
    dispatch('podSelect', selectedPod);
  }

  // Load metrics for a specific pod
  async function loadPodMetrics(pod: any) {
    if (!pod?.metadata?.name || !pod?.metadata?.namespace) return;
    
    metricsLoading = true;
    metricsError = null;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const metrics = await invoke('kuboard_get_pod_metrics_history', {
        podName: pod.metadata.name,
        namespace: pod.metadata.namespace,
        durationMinutes: selectedTimeRange
      });
      podMetrics = metrics;
    } catch (err) {
      metricsError = err as string;
      console.error('Failed to load pod metrics:', err);
    } finally {
      metricsLoading = false;
    }
  }

  // Show full details view
  function showFullPodDetails(pod: any) {
    selectedPodRaw = pod;
    selectedPod = {
      name: pod.metadata?.name || 'Unknown',
      namespace: pod.metadata?.namespace || 'default',
      status: pod.status?.phase || 'Unknown',
      nodeName: pod.spec?.nodeName || 'Unknown',
      hostIP: pod.status?.hostIP || 'Unknown',
      podIP: pod.status?.podIP || 'Unknown',
      restartCount: pod.status?.containerStatuses?.[0]?.restartCount || 0,
      readyContainers: pod.status?.containerStatuses?.filter((c: any) => c.ready).length || 0,
      totalContainers: pod.status?.containerStatuses?.length || 0,
      creationTimestamp: pod.metadata?.creationTimestamp || 'Unknown',
      labels: pod.metadata?.labels || {},
      annotations: pod.metadata?.annotations || {},
      containers: pod.spec?.containers || [],
      containerStatuses: pod.status?.containerStatuses || [],
      // Additional detailed information
      serviceAccount: pod.spec?.serviceAccountName || 'default',
      qosClass: pod.status?.qosClass || 'Unknown',
      conditions: pod.status?.conditions || [],
      tolerations: pod.spec?.tolerations || [],
      affinity: pod.spec?.affinity || {},
      volumes: pod.spec?.volumes || [],
      controller: getControllerInfo(pod),
      uid: pod.metadata?.uid || 'Unknown',
      resourceVersion: pod.metadata?.resourceVersion || 'Unknown',
      generation: pod.metadata?.generation || 'Unknown'
    };
    showFullDetails = true;
  }

  // Get controller information (Deployment, StatefulSet, etc.)
  function getControllerInfo(pod: any) {
    const ownerReferences = pod.metadata?.ownerReferences || [];
    if (ownerReferences.length === 0) {
      return { type: 'Pod', name: pod.metadata?.name || 'Unknown' };
    }
    
    const owner = ownerReferences[0];
    return {
      type: owner.kind || 'Unknown',
      name: owner.name || 'Unknown',
      uid: owner.uid || 'Unknown'
    };
  }

  // Load metrics for a specific container
  async function loadContainerMetrics(container: any) {
    if (!container?.name || !selectedPod?.metadata?.name || !selectedPod?.metadata?.namespace) return;
    
    containerMetricsLoading = true;
    containerMetricsError = null;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const metrics = await invoke('kuboard_get_pod_metrics_history', {
        podName: selectedPod.metadata.name,
        namespace: selectedPod.metadata.namespace,
        durationMinutes: selectedTimeRange
      });
      
      // Filter metrics for this specific container
      if (metrics && Array.isArray(metrics)) {
        containerMetrics = metrics.map((point: any) => ({
          ...point,
          // For now, we'll use the pod metrics and scale them down
          // In a real implementation, you'd want container-specific metrics
          cpu: (point.cpu || 0) * 0.1, // Simulate container using 10% of pod resources
          memory: (point.memory || 0) * 0.1
        }));
      } else {
        containerMetrics = [];
      }
    } catch (err) {
      containerMetricsError = err as string;
      console.error('Failed to load container metrics:', err);
    } finally {
      containerMetricsLoading = false;
    }
  }

  // Select container and load its metrics
  function selectContainer(container: any) {
    selectedContainer = container;
    loadContainerMetrics(container);
  }

  // Change container resource type
  function changeContainerResourceType(type: 'cpu' | 'memory') {
    selectedContainerResourceType = type;
  }

  // Load pod events
  async function loadPodEvents(pod: any) {
    if (!pod?.metadata?.name || !pod?.metadata?.namespace) return;
    
    eventsLoading = true;
    eventsError = null;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const events = await invoke('kuboard_get_pod_events', {
        podName: pod.metadata.name,
        namespace: pod.metadata.namespace
      });
      
      if (Array.isArray(events)) {
        // Sort events by timestamp (newest first)
        podEvents = events.sort((a, b) => {
          const timeA = new Date(a.firstTimestamp || a.eventTime || 0).getTime();
          const timeB = new Date(b.firstTimestamp || b.eventTime || 0).getTime();
          return timeB - timeA;
        });
      } else {
        podEvents = [];
      }
    } catch (err) {
      eventsError = err as string;
      console.error('Failed to load pod events:', err);
      // Generate mock events for demonstration
      podEvents = generateMockEvents(pod);
    } finally {
      eventsLoading = false;
    }
  }

  // Generate mock events for demonstration
  function generateMockEvents(pod: any) {
    const now = new Date();
    return [
      {
        type: 'Normal',
        reason: 'Created',
        message: `Created container ${pod.spec?.containers?.[0]?.name || 'main'}`,
        firstTimestamp: new Date(now.getTime() - 300000).toISOString(), // 5 minutes ago
        lastTimestamp: new Date(now.getTime() - 300000).toISOString(),
        count: 1,
        involvedObject: {
          kind: 'Pod',
          name: pod.metadata?.name || 'unknown',
          namespace: pod.metadata?.namespace || 'default'
        }
      },
      {
        type: 'Normal',
        reason: 'Scheduled',
        message: `Successfully assigned ${pod.metadata?.namespace || 'default'}/${pod.metadata?.name || 'unknown'} to ${pod.spec?.nodeName || 'node-1'}`,
        firstTimestamp: new Date(now.getTime() - 240000).toISOString(), // 4 minutes ago
        lastTimestamp: new Date(now.getTime() - 240000).toISOString(),
        count: 1,
        involvedObject: {
          kind: 'Pod',
          name: pod.metadata?.name || 'unknown',
          namespace: pod.metadata?.namespace || 'default'
        }
      },
      {
        type: 'Normal',
        reason: 'Pulling',
        message: `Pulling image "${pod.spec?.containers?.[0]?.image || 'nginx:latest'}"`,
        firstTimestamp: new Date(now.getTime() - 180000).toISOString(), // 3 minutes ago
        lastTimestamp: new Date(now.getTime() - 180000).toISOString(),
        count: 1,
        involvedObject: {
          kind: 'Pod',
          name: pod.metadata?.name || 'unknown',
          namespace: pod.metadata?.namespace || 'default'
        }
      },
      {
        type: 'Normal',
        reason: 'Pulled',
        message: `Successfully pulled image "${pod.spec?.containers?.[0]?.image || 'nginx:latest'}"`,
        firstTimestamp: new Date(now.getTime() - 120000).toISOString(), // 2 minutes ago
        lastTimestamp: new Date(now.getTime() - 120000).toISOString(),
        count: 1,
        involvedObject: {
          kind: 'Pod',
          name: pod.metadata?.name || 'unknown',
          namespace: pod.metadata?.namespace || 'default'
        }
      },
      {
        type: 'Normal',
        reason: 'Started',
        message: `Started container ${pod.spec?.containers?.[0]?.name || 'main'}`,
        firstTimestamp: new Date(now.getTime() - 60000).toISOString(), // 1 minute ago
        lastTimestamp: new Date(now.getTime() - 60000).toISOString(),
        count: 1,
        involvedObject: {
          kind: 'Pod',
          name: pod.metadata?.name || 'unknown',
          namespace: pod.metadata?.namespace || 'default'
        }
      }
    ];
  }

  // Back to pods list
  function backToPodsList() {
    showFullDetails = false;
    selectedPod = null;
    selectedPodRaw = null;
    podMetrics = null;
    selectedResourceType = 'cpu'; // Reset to CPU when going back
    selectedContainer = null;
    containerMetrics = null;
    containerMetricsLoading = false;
    containerMetricsError = null;
    podEvents = [];
    eventsLoading = false;
    eventsError = null;
  }
  
  function openPodLogs(pod: any, containerName?: string) {
    if (logsWindowRef) {
      // Handle both raw pod objects (with metadata) and processed pod objects (with direct properties)
      const podName = pod.metadata?.name || pod.name || 'Unknown';
      const podNamespace = pod.metadata?.namespace || pod.namespace || 'default';
      console.log('🔍 Pod data for logs:', { pod, podName, podNamespace, containerName });
      logsWindowRef.openPodLogs(podName, podNamespace, containerName);
      logsWindowOpen = true;
    }
  }
  
  function openContainerLogs(container: any) {
    if (selectedPod) {
      openPodLogs(selectedPod, container.name);
    }
  }

  // Change resource type for metrics graph
  function changeResourceType(type: 'cpu' | 'memory') {
    selectedResourceType = type;
  }

  // Change time range for metrics graph
  function changeTimeRange(minutes: number) {
    selectedTimeRange = minutes;
    // Reload metrics with new time range if a pod is selected
    if (selectedPod) {
      loadPodMetrics(selectedPod);
    }
  }

  // Get status class
  // Get effective pod status (checks for deletionTimestamp to show Terminating)
  function getEffectivePodStatus(pod: any): string {
    if (pod.metadata?.deletionTimestamp) {
      return 'Terminating';
    }
    return pod.status?.phase || 'Unknown';
  }

  function getStatusClass(status: string): string {
    switch (status?.toLowerCase()) {
      case 'running': return 'running';
      case 'pending': return 'pending';
      case 'succeeded': return 'ready';
      case 'failed': return 'failed';
      case 'terminating': return 'terminating';
      case 'unknown': return 'unknown';
      default: return 'unknown';
    }
  }

  // Format age
  function formatAge(creationTimestamp: string): string {
    if (!creationTimestamp) return 'Unknown';
    const created = new Date(creationTimestamp);
    const now = new Date();
    const diffMs = now.getTime() - created.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMins / 60);
    const diffDays = Math.floor(diffHours / 24);
    
    if (diffDays > 0) return `${diffDays}d`;
    if (diffHours > 0) return `${diffHours}h`;
    return `${diffMins}m`;
  }

  // Copy to clipboard function
  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
    } catch (err) {
      console.error('Failed to copy to clipboard:', err);
    }
  }

  // Format QoS class with appropriate styling
  function getQoSClassClass(qosClass: string): string {
    switch (qosClass?.toLowerCase()) {
      case 'guaranteed': return 'qos-guaranteed';
      case 'burstable': return 'qos-burstable';
      case 'besteffort': return 'qos-besteffort';
      default: return 'qos-unknown';
    }
  }

  // Format condition status
  function getConditionStatusClass(status: string): string {
    switch (status?.toLowerCase()) {
      case 'true': return 'condition-true';
      case 'false': return 'condition-false';
      case 'unknown': return 'condition-unknown';
      default: return 'condition-unknown';
    }
  }

  // Get controller name from owner references
  function getControllerName(pod: any): string {
    const ownerRefs = pod.metadata?.ownerReferences;
    if (!ownerRefs || ownerRefs.length === 0) return '-';
    const controller = ownerRefs.find((ref: any) => ref.controller);
    if (controller) {
      return `${controller.kind}/${controller.name}`;
    }
    return `${ownerRefs[0].kind}/${ownerRefs[0].name}`;
  }

  // Get pod error information (CrashLoopBackOff, OOMKilled, etc.)
  function getPodError(pod: any): { hasError: boolean; message: string } {
    const errors: string[] = [];
    
    // Check pod phase
    if (pod.status?.phase === 'Failed') {
      errors.push(`Pod Failed: ${pod.status.reason || 'Unknown reason'}`);
    }
    
    // Check container statuses
    const containerStatuses = pod.status?.containerStatuses || [];
    for (const containerStatus of containerStatuses) {
      const containerName = containerStatus.name || 'Unknown';
      
      // Check waiting state (CrashLoopBackOff, ImagePullBackOff, etc.)
      if (containerStatus.state?.waiting) {
        const reason = containerStatus.state.waiting.reason;
        const message = containerStatus.state.waiting.message || '';
        if (reason && reason !== 'ContainerCreating') {
          errors.push(`${containerName}: ${reason}${message ? ` - ${message}` : ''}`);
        }
      }
      
      // Check terminated state (OOMKilled, Error, etc.)
      if (containerStatus.state?.terminated) {
        const reason = containerStatus.state.terminated.reason;
        const exitCode = containerStatus.state.terminated.exitCode;
        const message = containerStatus.state.terminated.message || '';
        
        if (reason) {
          errors.push(`${containerName}: ${reason}${message ? ` - ${message}` : ''}`);
        } else if (exitCode !== undefined && exitCode !== 0) {
          errors.push(`${containerName}: Exited with code ${exitCode}${message ? ` - ${message}` : ''}`);
        }
      }
      
      // Check if container is not ready
      if (!containerStatus.ready && containerStatus.state?.running) {
        errors.push(`${containerName}: Container not ready`);
      }
    }
    
    // Check init container statuses
    const initContainerStatuses = pod.status?.initContainerStatuses || [];
    for (const initStatus of initContainerStatuses) {
      const containerName = initStatus.name || 'Unknown';
      
      if (initStatus.state?.waiting) {
        const reason = initStatus.state.waiting.reason;
        const message = initStatus.state.waiting.message || '';
        if (reason && reason !== 'PodInitializing') {
          errors.push(`Init ${containerName}: ${reason}${message ? ` - ${message}` : ''}`);
        }
      }
      
      if (initStatus.state?.terminated && initStatus.state.terminated.exitCode !== 0) {
        const reason = initStatus.state.terminated.reason;
        const message = initStatus.state.terminated.message || '';
        errors.push(`Init ${containerName}: ${reason || `Exited ${initStatus.state.terminated.exitCode}`}${message ? ` - ${message}` : ''}`);
      }
    }
    
    return {
      hasError: errors.length > 0,
      message: errors.join(' • ') || 'No errors'
    };
  }

  // Sorting functions
  function handleSort(column: string, event?: Event) {
    if (event) {
      event.stopPropagation();
      event.preventDefault();
    }
    
    if (sortColumn === column) {
      // Cycle: asc -> desc -> null
      if (sortDirection === 'asc') {
        sortDirection = 'desc';
      } else if (sortDirection === 'desc') {
        sortColumn = null;
        sortDirection = null;
      }
    } else {
      // New column, start with ascending
      sortColumn = column;
      sortDirection = 'asc';
    }
  }

  // Reactive sorted pods - automatically updates when sortColumn or sortDirection changes
  // Directly reference livePods and pods so Svelte tracks changes
  $: sortedPods = (() => {
    const podsToSort = livePods ?? pods ?? [];
    
    if (!sortColumn || !sortDirection) {
      return podsToSort;
    }
    
    const sorted = [...podsToSort];
    
    sorted.sort((a, b) => {
      let comparison = 0;
      
      switch (sortColumn) {
        case 'name':
          comparison = compareName(a, b);
          break;
        case 'status':
          comparison = compareStatus(a, b);
          break;
        case 'errors':
          comparison = compareErrors(a, b);
          break;
        case 'namespace':
          comparison = compareNamespace(a, b);
          break;
        case 'restarts':
          comparison = compareRestarts(a, b);
          break;
        case 'age':
          comparison = compareAge(a, b);
          break;
        case 'node':
          comparison = compareNode(a, b);
          break;
        default:
          return 0;
      }
      
      return sortDirection === 'asc' ? comparison : -comparison;
    });
    
    return sorted;
  })();

  // Reactive filtered pods - applies search to sorted pods
  $: filteredPods = (() => {
    if (!searchQuery || !searchQuery.trim()) {
      return sortedPods;
    }

    const parsed = parseSearchQuery(searchQuery);
    
    // For general search, relevance scoring is already applied in searchPods
    // For field-specific/label searches, maintain user sort preference
    const searched = searchPods(sortedPods, searchQuery);
    
    // If it's a general search, results are already sorted by relevance
    // If it's field-specific/label search and user has a sort active, maintain it
    if (parsed.type === 'general' || !sortColumn || !sortDirection) {
      return searched;
    }
    
    // For field/label searches with active sort, re-sort the filtered results
    const reSorted = [...searched];
    reSorted.sort((a, b) => {
      let comparison = 0;
      
      switch (sortColumn) {
        case 'name':
          comparison = compareName(a, b);
          break;
        case 'status':
          comparison = compareStatus(a, b);
          break;
        case 'errors':
          comparison = compareErrors(a, b);
          break;
        case 'namespace':
          comparison = compareNamespace(a, b);
          break;
        case 'restarts':
          comparison = compareRestarts(a, b);
          break;
        case 'age':
          comparison = compareAge(a, b);
          break;
        case 'node':
          comparison = compareNode(a, b);
          break;
        default:
          return 0;
      }
      
      return sortDirection === 'asc' ? comparison : -comparison;
    });
    
    return reSorted;
  })();

  function getSortedPods(): any[] {
    return sortedPods;
  }

  // Search parsing logic
  function parseSearchQuery(query: string): { type: 'label' | 'field' | 'ip' | 'general'; field?: string; value?: string; labelKey?: string; labelValue?: string } {
    if (!query || !query.trim()) {
      return { type: 'general' };
    }

    const trimmed = query.trim();

    // Detect IP address pattern (IPv4)
    const ipPattern = /^(\d{1,3}\.){3}\d{1,3}$/;
    if (ipPattern.test(trimmed)) {
      return { type: 'ip', value: trimmed };
    }

    // Detect label search: key:value or key=value
    const labelPattern = /^([^:=]+)[:=](.*)$/;
    const labelMatch = trimmed.match(labelPattern);
    if (labelMatch) {
      const key = labelMatch[1].trim();
      const value = labelMatch[2].trim();
      if (key || value) {
        return { type: 'label', labelKey: key || undefined, labelValue: value || undefined };
      }
    }

    // Detect field-specific search: field:value
    const fieldPattern = /^(name|namespace|node|ip|status):(.+)$/i;
    const fieldMatch = trimmed.match(fieldPattern);
    if (fieldMatch) {
      return { type: 'field', field: fieldMatch[1].toLowerCase(), value: fieldMatch[2].trim() };
    }

    // Default to general search
    return { type: 'general', value: trimmed };
  }

  // Match labels helper
  function matchLabels(pod: any, labelKey?: string, labelValue?: string): boolean {
    const labels = pod.metadata?.labels || {};
    
    if (labelKey && labelValue) {
      // Exact match: key:value
      return labels[labelKey] === labelValue;
    } else if (labelKey && !labelValue) {
      // Match any value for key: key:
      return labelKey in labels;
    } else if (!labelKey && labelValue) {
      // Match any key with value: :value
      return Object.values(labels).some((v: any) => 
        String(v).toLowerCase().includes(labelValue.toLowerCase())
      );
    }
    
    return false;
  }

  // Search pod fields and return match count
  function searchPodFields(pod: any, query: string): { count: number; locations: string[] } {
    const lowerQuery = query.toLowerCase();
    const locations: string[] = [];
    let count = 0;

    // Helper to check and count matches
    const checkMatch = (value: any, field: string) => {
      if (!value) return;
      const strValue = String(value).toLowerCase();
      if (strValue.includes(lowerQuery)) {
        locations.push(field);
        count++;
      }
    };

    // Search metadata
    checkMatch(pod.metadata?.name, 'name');
    checkMatch(pod.metadata?.namespace, 'namespace');
    checkMatch(pod.metadata?.uid, 'uid');

    // Search labels
    const labels = pod.metadata?.labels || {};
    for (const [key, value] of Object.entries(labels)) {
      checkMatch(key, `label:${key}`);
      checkMatch(value, `label:${key}`);
    }

    // Search annotations
    const annotations = pod.metadata?.annotations || {};
    for (const [key, value] of Object.entries(annotations)) {
      checkMatch(key, `annotation:${key}`);
      checkMatch(value, `annotation:${key}`);
    }

    // Search status fields
    checkMatch(pod.status?.podIP, 'podIP');
    checkMatch(pod.status?.hostIP, 'hostIP');
    checkMatch(pod.status?.phase, 'phase');
    checkMatch(pod.spec?.nodeName, 'node');

    // Search container statuses
    const containerStatuses = pod.status?.containerStatuses || [];
    for (const container of containerStatuses) {
      checkMatch(container.name, `container:${container.name}`);
      checkMatch(container.image, `image:${container.image}`);
    }

    return { count, locations };
  }

  // Score pod for relevance-based search
  function scorePod(pod: any, query: string): number {
    const lowerQuery = query.toLowerCase();
    let score = 0;
    const podName = (pod.metadata?.name || '').toLowerCase();
    const exactQuery = lowerQuery.trim();

    // Pod name exact match - highest priority (1000+ points)
    if (podName === exactQuery) {
      score += 1500;
    }
    // Pod name starts with query - high priority (800+ points)
    else if (podName.startsWith(exactQuery)) {
      score += 800;
    }
    // Pod name contains query - medium-high (300+ points)
    else if (podName.includes(exactQuery)) {
      score += 300;
    }

    // Search all fields and weight matches
    const fieldResults = searchPodFields(pod, query);
    
    // Apply field weights
    for (const location of fieldResults.locations) {
      if (location === 'name') {
        score += 10;
      } else if (location.startsWith('label:')) {
        score += 5;
      } else if (location === 'podIP' || location === 'hostIP') {
        score += 5;
      } else if (location === 'namespace') {
        score += 3;
      } else {
        score += 1;
      }
    }

    // Exact match bonus in any field
    const allText = JSON.stringify(pod).toLowerCase();
    if (allText.includes(`"${exactQuery}"`) || allText.includes(`'${exactQuery}'`)) {
      score += 500;
    }

    return score;
  }

  // Main search function
  function searchPods(pods: any[], query: string): any[] {
    if (!query || !query.trim()) {
      return pods;
    }

    const parsed = parseSearchQuery(query);
    
    switch (parsed.type) {
      case 'label':
        return pods.filter(pod => matchLabels(pod, parsed.labelKey, parsed.labelValue));
      
      case 'field':
        const field = parsed.field!;
        const value = parsed.value!.toLowerCase();
        return pods.filter(pod => {
          switch (field) {
            case 'name':
              return (pod.metadata?.name || '').toLowerCase().includes(value);
            case 'namespace':
              return (pod.metadata?.namespace || '').toLowerCase().includes(value);
            case 'node':
              return (pod.spec?.nodeName || '').toLowerCase().includes(value);
            case 'ip':
              return (pod.status?.podIP || '').includes(value) || 
                     (pod.status?.hostIP || '').includes(value);
            case 'status':
              return (pod.status?.phase || '').toLowerCase().includes(value);
            default:
              return false;
          }
        });
      
      case 'ip':
        const ipValue = parsed.value!;
        return pods.filter(pod => 
          pod.status?.podIP === ipValue || 
          pod.status?.hostIP === ipValue ||
          (pod.status?.podIP || '').includes(ipValue) ||
          (pod.status?.hostIP || '').includes(ipValue)
        );
      
      case 'general':
      default:
        // Relevance-scored search
        const scored = pods.map(pod => ({
          pod,
          score: scorePod(pod, parsed.value || query)
        })).filter(item => item.score > 0);
        
        // Sort by score (descending), then by original order
        scored.sort((a, b) => {
          if (b.score !== a.score) {
            return b.score - a.score;
          }
          return 0;
        });
        
        return scored.map(item => item.pod);
    }
  }

  function compareName(a: any, b: any): number {
    const nameA = (a.metadata?.name || '').toLowerCase();
    const nameB = (b.metadata?.name || '').toLowerCase();
    return nameA.localeCompare(nameB);
  }

  function compareStatus(a: any, b: any): number {
    const statusA = (a.status?.phase || '').toLowerCase();
    const statusB = (b.status?.phase || '').toLowerCase();
    return statusA.localeCompare(statusB);
  }

  function compareErrors(a: any, b: any): number {
    const errorA = getPodError(a).hasError ? 1 : 0;
    const errorB = getPodError(b).hasError ? 1 : 0;
    return errorA - errorB;
  }

  function compareNamespace(a: any, b: any): number {
    const nsA = (a.metadata?.namespace || '').toLowerCase();
    const nsB = (b.metadata?.namespace || '').toLowerCase();
    return nsA.localeCompare(nsB);
  }

  function compareRestarts(a: any, b: any): number {
    const restartsA = a.status?.containerStatuses?.[0]?.restartCount || 0;
    const restartsB = b.status?.containerStatuses?.[0]?.restartCount || 0;
    return restartsA - restartsB;
  }

  function compareAge(a: any, b: any): number {
    const timestampA = new Date(a.metadata?.creationTimestamp || 0).getTime();
    const timestampB = new Date(b.metadata?.creationTimestamp || 0).getTime();
    return timestampB - timestampA; // Higher timestamp (newer) first when ascending
  }

  function compareNode(a: any, b: any): number {
    const nodeA = (a.spec?.nodeName || '').toLowerCase();
    const nodeB = (b.spec?.nodeName || '').toLowerCase();
    return nodeA.localeCompare(nodeB);
  }

  // Reactive sort indicator - ensures it updates when sort state changes
  function getSortIndicator(column: string): string {
    // Force reactivity by referencing sortColumn and sortDirection
    const currentColumn = sortColumn;
    const currentDirection = sortDirection;
    
    if (currentColumn !== column) return '';
    if (currentDirection === 'asc') return '▲';  // Filled triangle up
    if (currentDirection === 'desc') return '▼';  // Filled triangle down
    return '';
  }

  // Format toleration effect
  function getTolerationEffectClass(effect: string): string {
    switch (effect?.toLowerCase()) {
      case 'noschedule': return 'toleration-noschedule';
      case 'prefernoschedule': return 'toleration-prefernoschedule';
      case 'noexecute': return 'toleration-noexecute';
      default: return 'toleration-unknown';
    }
  }

  // Format object to display string
  function formatObject(obj: any): string {
    if (typeof obj === 'string') return obj;
    if (typeof obj === 'number') return obj.toString();
    if (typeof obj === 'boolean') return obj.toString();
    if (Array.isArray(obj)) return obj.join(', ');
    if (obj && typeof obj === 'object') {
      return JSON.stringify(obj, null, 2);
    }
    return 'N/A';
  }

  // Format event timestamp
  function formatEventTime(timestamp: string): string {
    if (!timestamp) return 'Unknown';
    try {
      const date = new Date(timestamp);
      return date.toLocaleString();
    } catch {
      return timestamp;
    }
  }

  // Get event type class for styling
  function getEventTypeClass(type: string): string {
    switch (type?.toLowerCase()) {
      case 'normal': return 'event-normal';
      case 'warning': return 'event-warning';
      case 'error': return 'event-error';
      default: return 'event-unknown';
    }
  }

  // Get event reason class for styling
  function getEventReasonClass(reason: string): string {
    switch (reason?.toLowerCase()) {
      case 'created': return 'reason-created';
      case 'scheduled': return 'reason-scheduled';
      case 'pulling': return 'reason-pulling';
      case 'pulled': return 'reason-pulled';
      case 'started': return 'reason-started';
      case 'killing': return 'reason-killing';
      case 'killed': return 'reason-killed';
      case 'failed': return 'reason-failed';
      case 'backoff': return 'reason-backoff';
      case 'unhealthy': return 'reason-unhealthy';
      default: return 'reason-unknown';
    }
  }

  // Lifecycle
  let watchEventListenerUnsubscribe: (() => Promise<void>) | null = null;
  let watchErrorListenerUnsubscribe: (() => Promise<void>) | null = null;
  let lastContext: string | null = null;

  onMount(async () => {
    // Initialize pods from props
    initializePods();
    
    // Listen to watch events
    const { listen } = await import('@tauri-apps/api/event');
    
    watchEventListenerUnsubscribe = await listen('pod-watch-event', (event: any) => {
      handleWatchEvent(event.payload);
    });
    
    watchErrorListenerUnsubscribe = await listen('pod-watch-error', (event: any) => {
      handleWatchError(event.payload);
    });
    
    // Start watch when context is available
    if (currentContext && !watchActive) {
      startWatch();
    }
    
    return async () => {
      if (watchEventListenerUnsubscribe) await watchEventListenerUnsubscribe();
      if (watchErrorListenerUnsubscribe) await watchErrorListenerUnsubscribe();
      await stopWatch();
    };
  });

  onDestroy(async () => {
    if (watchEventListenerUnsubscribe) await watchEventListenerUnsubscribe();
    if (watchErrorListenerUnsubscribe) await watchErrorListenerUnsubscribe();
    await stopWatch();
  });

  // Restart watch only when context actually changes
  $: if (currentContext && currentContext !== lastContext) {
    lastContext = currentContext;
    if (watchActive) {
      stopWatch().then(() => {
        podsMap.clear();
        livePods = [];
        initializePods();
        startWatch();
      });
    } else {
      podsMap.clear();
      livePods = [];
      initializePods();
      startWatch();
    }
  }
</script>

<div class="pods-panel">
  <div class="panel-header">
    <h4>🟢 Pods ({getRenderPods().length})</h4>
    <div class="panel-controls">
      <span class="live-indicator {watchError ? 'error' : watchActive ? 'active' : ''}">
        {watchError ? 'Watch Error' : watchActive ? '🟢 Live' : '⏸️ Paused'}
      </span>
    </div>
  </div>

  <!-- Always show the basic UI structure -->
  <div class="pods-content">
    {#if showFullDetails && selectedPodRaw}
      <PodDetails pod={selectedPodRaw} onBack={backToPodsList} onOpenLogs={() => openPodLogs(selectedPodRaw)} />
    {:else}
      <!-- Pods List View (always show this) -->
      <div class="pods-list-view">
        
        {#if isLoading}
          <!-- Loading State -->
          <div class="loading-pods">
            <div class="loading-spinner">⏳</div>
            <h5>Loading Pods...</h5>
            <p>Please wait while we fetch the cluster pods information.</p>
          </div>
        {:else if getRenderPods() && getRenderPods().length > 0}
          <!-- Search Bar -->
          <div class="search-bar-container">
            <div class="search-input-wrapper">
              <span class="search-icon">🔍</span>
              <input
                type="text"
                class="search-input"
                placeholder="Search pods by name, namespace, labels, IP, or any field... (e.g., name:nginx, app:web, 192.168.1.1)"
                bind:value={searchQuery}
                autocomplete="off"
              />
              {#if searchQuery}
                <button
                  class="search-clear"
                  onclick={() => searchQuery = ''}
                  title="Clear search"
                >
                  ×
                </button>
              {/if}
            </div>
            <div class="search-results-count">
              {#if searchQuery}
                Showing {filteredPods.length} of {getRenderPods().length} pods
              {:else}
                {getRenderPods().length} pods
              {/if}
            </div>
          </div>

          <!-- Pods Table -->
          {#if filteredPods.length === 0 && searchQuery}
            <div class="no-search-results">
              <div class="no-results-icon">🔍</div>
              <h5>No Pods Found</h5>
              <p>No pods match your search query: <strong>"{searchQuery}"</strong></p>
              <button class="clear-search-button" onclick={() => searchQuery = ''}>
                Clear Search
              </button>
            </div>
          {:else}
            <div class="pods-table-container">
              <table class="pods-table">
                <thead>
                <tr>
                  <th 
                    class="sortable-header {sortColumn === 'name' ? 'sorted' : ''}"
                    onclick={(e) => handleSort('name', e)}
                    onkeydown={(e) => {
                      if (e.key === 'Enter' || e.key === ' ') {
                        e.preventDefault();
                        handleSort('name', e);
                      }
                    }}
                    role="button"
                    tabindex="0"
                    title="Click to sort by name"
                  >
                    Name <span class="sort-indicator">{sortColumn === 'name' ? (sortDirection === 'asc' ? '▲' : sortDirection === 'desc' ? '▼' : '') : ''}</span>
                  </th>
                  <th 
                    class="sortable-header {sortColumn === 'status' ? 'sorted' : ''}"
                    onclick={(e) => handleSort('status', e)}
                    onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && handleSort('status', e)}
                    role="button"
                    tabindex="0"
                    title="Click to sort by status"
                  >
                    Status <span class="sort-indicator">{sortColumn === 'status' ? (sortDirection === 'asc' ? '▲' : sortDirection === 'desc' ? '▼' : '') : ''}</span>
                  </th>
                  <th 
                    class="sortable-header {sortColumn === 'errors' ? 'sorted' : ''}"
                    style="width: 40px; text-align: center;"
                    onclick={(e) => handleSort('errors', e)}
                    onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && handleSort('errors', e)}
                    role="button"
                    tabindex="0"
                    title="Click to sort by errors"
                  >
                    ⚠ <span class="sort-indicator">{sortColumn === 'errors' ? (sortDirection === 'asc' ? '▲' : sortDirection === 'desc' ? '▼' : '') : ''}</span>
                  </th>
                  <th 
                    class="sortable-header {sortColumn === 'namespace' ? 'sorted' : ''}"
                    onclick={(e) => handleSort('namespace', e)}
                    onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && handleSort('namespace', e)}
                    role="button"
                    tabindex="0"
                    title="Click to sort by namespace"
                  >
                    Namespace <span class="sort-indicator">{sortColumn === 'namespace' ? (sortDirection === 'asc' ? '▲' : sortDirection === 'desc' ? '▼' : '') : ''}</span>
                  </th>
                  <th 
                    class="sortable-header {sortColumn === 'restarts' ? 'sorted' : ''}"
                    onclick={(e) => handleSort('restarts', e)}
                    onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && handleSort('restarts', e)}
                    role="button"
                    tabindex="0"
                    title="Click to sort by restarts"
                  >
                    Restarts <span class="sort-indicator">{sortColumn === 'restarts' ? (sortDirection === 'asc' ? '▲' : sortDirection === 'desc' ? '▼' : '') : ''}</span>
                  </th>
                  <th 
                    class="sortable-header {sortColumn === 'age' ? 'sorted' : ''}"
                    onclick={(e) => handleSort('age', e)}
                    onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && handleSort('age', e)}
                    role="button"
                    tabindex="0"
                    title="Click to sort by age"
                  >
                    Age <span class="sort-indicator">{sortColumn === 'age' ? (sortDirection === 'asc' ? '▲' : sortDirection === 'desc' ? '▼' : '') : ''}</span>
                  </th>
                  <th>CPU</th>
                  <th>Memory</th>
                  <th 
                    class="sortable-header {sortColumn === 'node' ? 'sorted' : ''}"
                    onclick={(e) => handleSort('node', e)}
                    onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && handleSort('node', e)}
                    role="button"
                    tabindex="0"
                    title="Click to sort by node"
                  >
                    Node <span class="sort-indicator">{sortColumn === 'node' ? (sortDirection === 'asc' ? '▲' : sortDirection === 'desc' ? '▼' : '') : ''}</span>
                  </th>
                  <th>Controlled By</th>
                </tr>
              </thead>
              <tbody>
                {#each filteredPods as pod (getPodKey(pod))}
                  {@const errorInfo = getPodError(pod)}
                  <tr 
                    class="pod-row" 
                    onclick={() => showFullPodDetails(pod)} 
                    role="button" 
                    tabindex="0" 
                    onkeydown={(e) => e.key === 'Enter' || e.key === ' ' ? showFullPodDetails(pod) : null}
                    oncontextmenu={(e) => handleContextMenu(e, pod)}
                  >
                    <td class="pod-name-cell">{pod.metadata?.name || 'Unknown'}</td>
                    <td><span class="status-badge status-{getStatusClass(getEffectivePodStatus(pod))}">{getEffectivePodStatus(pod)}</span></td>
                    <td class="error-cell">
                      {#if errorInfo.hasError}
                        <span class="error-indicator" title={errorInfo.message}>⚠️</span>
                      {:else}
                        <span class="no-error">-</span>
                      {/if}
                    </td>
                    <td>{pod.metadata?.namespace || 'default'}</td>
                    <td>{pod.status?.containerStatuses?.[0]?.restartCount || 0}</td>
                    <td>{formatAge(pod.metadata?.creationTimestamp)}</td>
                    <td class="metric-cell">-</td>
                    <td class="metric-cell">-</td>
                    <td>{pod.spec?.nodeName || 'Unknown'}</td>
                    <td>{getControllerName(pod)}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
          {/if}
        {:else}
          <!-- No Pods Available -->
          <div class="no-pods-message">
            <div class="no-pods-icon">🟢</div>
            <h5>No Pods Available</h5>
            <p>No pods are currently available in this cluster context.</p>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>

<!-- Logs Window -->
<LogsWindow 
  bind:this={logsWindowRef}
  bind:isOpen={logsWindowOpen}
  onClose={() => logsWindowOpen = false}
/>

<!-- Quick Actions Menu -->
{#if contextMenuPod}
  <QuickActionsMenu
    resource={contextMenuPod}
    resourceType="pod"
    position={contextMenuPosition}
    bind:visible={contextMenuVisible}
    on:close={handleActionMenuClose}
    on:deleted={handleActionDeleted}
    on:restarted={handleActionRestarted}
    on:view-yaml={handleViewYaml}
    on:copied={handleActionCopied}
    on:describe={handleActionDescribe}
    on:edit={handleActionEdit}
  />
{/if}

<!-- YAML Viewer Modal -->
{#if yamlViewerVisible}
  <div class="yaml-viewer-modal" onclick={closeYamlViewer}>
    <div class="yaml-viewer-content" onclick={(e) => e.stopPropagation()}>
      <div class="yaml-viewer-header">
        <h3>Pod YAML</h3>
        <button class="yaml-viewer-close" onclick={closeYamlViewer}>✕</button>
      </div>
      <div class="yaml-viewer-body">
        <pre><code>{yamlContent}</code></pre>
      </div>
    </div>
  </div>
{/if}

<style>
  /* Import CSS variables */
  @import '../styles/variables.css';
  @import '../styles/color-palette.css';

  .pods-panel {
    background: transparent;
    border-radius: 0;
    padding: 0;
    border: none;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-md);
    padding-bottom: var(--spacing-sm);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .panel-header h4 {
    margin: 0;
    color: white;
    font-size: 1rem;
    font-weight: 600;
  }

  .panel-controls {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .live-indicator {
    font-size: 0.85rem;
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid rgba(34, 197, 94, 0.3);
    background: rgba(34, 197, 94, 0.12);
    color: #22c55e;
  }
  .live-indicator.error {
    border-color: rgba(239, 68, 68, 0.3);
    background: rgba(239, 68, 68, 0.12);
    color: #ef4444;
  }

  .pods-content {
    min-height: 200px;
  }

  /* Loading State */
  .loading-pods {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-xl);
    color: var(--text-secondary);
    text-align: center;
  }

  .loading-spinner {
    font-size: 2rem;
    animation: spin 2s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .loading-pods h5 {
    margin: 0;
    color: var(--text-primary);
    font-size: 1.2rem;
  }

  .loading-pods p {
    margin: 0;
    font-size: 0.9rem;
  }

  /* No Pods Message */
  .no-pods-message {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-xl);
    color: var(--text-secondary);
    text-align: center;
  }

  .no-pods-icon {
    font-size: 3rem;
    opacity: 0.7;
  }

  .no-pods-message h5 {
    margin: 0;
    color: var(--text-primary);
    font-size: 1.2rem;
  }

  .no-pods-message p {
    margin: 0;
    font-size: 0.9rem;
  }

  /* Pods List View */
  .pods-list-view {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  /* Search Bar Styles */
  .search-bar-container {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-md);
  }

  .search-input-wrapper {
    position: relative;
    display: flex;
    align-items: center;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    padding: 0 var(--spacing-sm);
    transition: border-color 0.2s ease, background 0.2s ease;
  }

  .search-input-wrapper:focus-within {
    border-color: var(--primary-color);
    background: rgba(255, 255, 255, 0.08);
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
  }

  .search-icon {
    font-size: 1rem;
    color: var(--text-secondary);
    margin-right: var(--spacing-xs);
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: var(--text-primary);
    font-size: 0.9rem;
    padding: var(--spacing-sm) 0;
    width: 100%;
  }

  .search-input::placeholder {
    color: var(--text-muted);
    font-size: 0.85rem;
  }

  .search-clear {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 1.5rem;
    line-height: 1;
    padding: 0 var(--spacing-xs);
    transition: color 0.2s ease, transform 0.2s ease;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
  }

  .search-clear:hover {
    color: var(--text-primary);
    transform: scale(1.1);
  }

  .search-clear:active {
    transform: scale(0.95);
  }

  .search-results-count {
    color: var(--text-secondary);
    font-size: 0.85rem;
    padding-left: var(--spacing-xs);
    font-style: italic;
  }

  /* No Search Results */
  .no-search-results {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-xl);
    text-align: center;
    background: rgba(255, 255, 255, 0.03);
    border-radius: var(--radius-md);
    border: 1px dashed var(--border-primary);
    margin-top: var(--spacing-md);
  }

  .no-results-icon {
    font-size: 3rem;
    opacity: 0.6;
    margin-bottom: var(--spacing-md);
  }

  .no-search-results h5 {
    margin: 0 0 var(--spacing-sm) 0;
    color: var(--text-primary);
    font-size: 1.1rem;
  }

  .no-search-results p {
    margin: 0 0 var(--spacing-md) 0;
    color: var(--text-secondary);
    font-size: 0.9rem;
  }

  .no-search-results strong {
    color: var(--text-primary);
    font-weight: 600;
  }

  .clear-search-button {
    background: var(--primary-color);
    border: 1px solid var(--primary-color);
    border-radius: var(--radius-sm);
    color: white;
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 500;
    padding: var(--spacing-xs) var(--spacing-md);
    transition: all 0.2s ease;
  }

  .clear-search-button:hover {
    background: var(--accent-color);
    border-color: var(--accent-color);
    transform: translateY(-1px);
  }

  .pods-table-container {
    overflow-x: auto;
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    background: var(--background-card);
  }

  .pods-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.9rem;
  }

  .pods-table thead {
    background: rgba(255, 255, 255, 0.05);
    position: sticky;
    top: 0;
    z-index: 1;
  }

  .pods-table th {
    padding: 8px 12px;
    text-align: left;
    font-weight: 600;
    color: var(--text-secondary);
    border-bottom: 1px solid var(--border-primary);
    white-space: nowrap;
    font-size: 0.85rem;
  }

  .sortable-header {
    cursor: pointer;
    user-select: none;
    transition: background-color 0.2s ease, color 0.2s ease;
    position: relative;
  }

  .sortable-header:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-primary);
  }

  /* Active sorted column styling */
  .sortable-header.sorted {
    background: rgba(59, 130, 246, 0.12);
    color: var(--text-primary);
    border-bottom: 2px solid var(--primary-color);
  }

  .sortable-header.sorted:hover {
    background: rgba(59, 130, 246, 0.18);
  }

  .sort-indicator {
    display: inline-block;
    margin-left: 6px;
    color: var(--primary-color);
    font-size: 0.75em;
    font-weight: 700;
    min-width: 16px;
    opacity: 0.9;
    vertical-align: middle;
    line-height: 1;
  }

  .sortable-header:hover .sort-indicator {
    opacity: 1;
  }

  .pods-table tbody tr.pod-row {
    cursor: pointer;
    transition: background-color 0.2s ease;
    border-bottom: 1px solid var(--border-primary);
  }

  .pods-table tbody tr.pod-row:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .pods-table tbody tr.pod-row:last-child {
    border-bottom: none;
  }

  .pods-table td {
    padding: 6px 12px;
    color: var(--text-primary);
    vertical-align: middle;
  }

  .pod-name-cell {
    font-weight: 500;
    color: var(--primary-color);
    max-width: 250px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .metric-cell {
    font-family: monospace;
    color: var(--text-muted);
    font-size: 0.85rem;
  }

  .status-badge {
    padding: 3px 8px;
    border-radius: var(--radius-sm);
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    display: inline-block;
  }

  .status-running {
    background: var(--status-ready-bg);
    color: var(--status-ready-text);
    border: 1px solid var(--status-ready-border);
  }

  .status-pending {
    background: var(--status-pending-bg);
    color: var(--status-pending-text);
    border: 1px solid var(--status-pending-border);
  }

  .status-terminating {
    background: rgba(255, 165, 0, 0.15);
    color: #ffa500;
    border: 1px solid rgba(255, 165, 0, 0.4);
    animation: pulse-terminating 2s ease-in-out infinite;
  }

  @keyframes pulse-terminating {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.6;
    }
  }

  .status-failed {
    background: var(--status-error-bg);
    color: var(--status-error-text);
    border: 1px solid var(--status-error-border);
  }

  .status-ready {
    background: var(--status-ready-bg);
    color: var(--status-ready-text);
    border: 1px solid var(--status-ready-border);
  }

  /* Error Indicator Column */
  .error-cell {
    text-align: center;
    width: 40px;
    padding: 6px 8px;
  }

  .error-indicator {
    display: inline-block;
    font-size: 1.1rem;
    cursor: help;
    position: relative;
    animation: pulse-warning 2s ease-in-out infinite;
  }

  .error-indicator:hover {
    transform: scale(1.2);
    filter: brightness(1.3);
  }

  @keyframes pulse-warning {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.7;
    }
  }

  .no-error {
    color: var(--text-muted);
    font-size: 0.8rem;
  }

  .status-unknown {
    background: var(--status-pending-bg);
    color: var(--status-pending-text);
    border: 1px solid var(--status-pending-border);
  }


  /* Full Details View */
  .full-details-view {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  .details-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-bottom: var(--spacing-md);
    border-bottom: 1px solid var(--border-primary);
  }
  
  .header-left {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }
  
  .header-right {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
  }

  .back-button {
    background: var(--background-card);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    cursor: pointer;
    font-size: 0.9rem;
    padding: var(--spacing-sm) var(--spacing-md);
    transition: var(--transition-normal);
  }

  .back-button:hover {
    background: var(--accent-color);
    border-color: var(--accent-color);
    color: white;
  }
  
  .logs-button {
    background: var(--primary-color);
    border: 1px solid var(--primary-color);
    border-radius: var(--radius-sm);
    color: white;
    cursor: pointer;
    font-size: var(--font-size-sm);
    font-weight: 500;
    padding: var(--spacing-xs) var(--spacing-sm);
    transition: all 0.2s ease;
  }
  
  .logs-button:hover {
    background: var(--primary-dark);
    border-color: var(--primary-dark);
  }

  .details-header h3 {
    margin: 0;
    color: var(--text-primary);
    font-size: 1.5rem;
  }

  .pod-namespace {
    color: var(--text-secondary);
    font-size: 1rem;
    font-weight: 400;
  }

  .pod-details-content {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  .details-section {
    background: var(--background-card);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
  }

  .details-section h6 {
    margin: 0 0 var(--spacing-md) 0;
    color: var(--text-primary);
    font-size: 1.1rem;
    font-weight: 600;
  }

  .info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: var(--spacing-md);
  }

  .info-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-sm);
    background: rgba(255, 255, 255, 0.02);
    border-radius: var(--radius-sm);
  }

  .info-label {
    color: var(--text-secondary);
    font-weight: 500;
    font-size: 0.9rem;
  }

  .info-value-container {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .info-value {
    color: var(--text-primary);
    font-weight: 600;
    font-size: 0.95rem;
  }

  .copy-button {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    cursor: pointer;
    font-size: 0.8rem;
    padding: 2px 6px;
    transition: var(--transition-normal);
    opacity: 0.7;
  }

  .copy-button:hover {
    background: rgba(255, 255, 255, 0.2);
    border-color: rgba(255, 255, 255, 0.3);
    opacity: 1;
  }

  /* Container Information */
  .container-info {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .container-summary {
    display: flex;
    gap: var(--spacing-lg);
    padding: var(--spacing-sm);
    background: rgba(255, 255, 255, 0.02);
    border-radius: var(--radius-sm);
  }

  .containers-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .container-item {
    padding: var(--spacing-sm);
    background: rgba(255, 255, 255, 0.02);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-primary);
  }

  .container-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-xs);
  }
  
  .container-actions {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
  }

  .container-name {
    margin: 0;
    color: var(--text-primary);
    font-size: 1rem;
    font-weight: 600;
  }

  .container-image {
    color: var(--text-secondary);
    font-size: 0.8rem;
    font-family: monospace;
  }

  .container-status {
    display: flex;
    gap: var(--spacing-sm);
    align-items: center;
  }

  .restart-count {
    color: var(--text-secondary);
    font-size: 0.8rem;
  }

  /* Enhanced Container Item Styles */
  .container-item {
    cursor: pointer;
    transition: all 0.2s ease;
    position: relative;
  }

  .container-item:hover {
    background: rgba(255, 255, 255, 0.05);
    border-color: var(--accent-color);
    transform: translateY(-1px);
  }

  .container-item.selected {
    background: rgba(59, 130, 246, 0.1);
    border-color: var(--primary-color);
    box-shadow: 0 0 0 1px var(--primary-color);
  }

  .click-hint {
    color: var(--accent-color);
    font-size: 0.8rem;
    font-style: italic;
    opacity: 0.7;
  }
  
  .container-logs-button {
    background: var(--background-card);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-xs);
    color: var(--text-primary);
    cursor: pointer;
    font-size: 0.7rem;
    padding: 2px 6px;
    transition: all 0.2s ease;
  }
  
  .container-logs-button:hover {
    background: var(--primary-color);
    border-color: var(--primary-color);
    color: white;
  }

  /* Container Metrics Section */
  .container-metrics-section {
    margin-top: var(--spacing-lg);
    padding: var(--spacing-lg);
    background: rgba(0, 0, 0, 0.2);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-secondary);
  }

  .container-metrics-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-md);
  }

  .container-metrics-header h6 {
    margin: 0;
    color: var(--text-primary);
    font-size: 1.1rem;
    font-weight: 600;
  }

  .close-container-metrics {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 1.2rem;
    cursor: pointer;
    padding: var(--spacing-xs);
    border-radius: var(--radius-sm);
    transition: all 0.2s ease;
  }

  .close-container-metrics:hover {
    color: var(--error-color);
    background: rgba(239, 68, 68, 0.1);
  }

  .container-resource-tabs {
    display: flex;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-md);
  }

  .container-metrics-graph-container {
    background: rgba(0, 0, 0, 0.3);
    border-radius: var(--radius-sm);
    padding: var(--spacing-md);
    border: 1px solid var(--border-primary);
  }

  /* Pod Events */
  .events-loading, .events-error, .events-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-xl);
    color: var(--text-muted);
  }

  .events-loading .loading-spinner, .events-error .error-icon {
    font-size: 2rem;
    margin-bottom: var(--spacing-md);
  }

  .events-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .event-item {
    padding: var(--spacing-md);
    background: rgba(255, 255, 255, 0.02);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-primary);
    border-left: 4px solid var(--border-primary);
  }

  .event-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-sm);
    flex-wrap: wrap;
  }

  .event-type-badge {
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    font-size: 0.8rem;
    font-weight: 600;
    text-transform: uppercase;
    min-width: 60px;
    text-align: center;
  }

  .event-normal {
    background: var(--status-ready-bg);
    color: var(--status-ready-text);
    border: 1px solid var(--status-ready-border);
  }

  .event-warning {
    background: rgba(245, 158, 11, 0.2);
    color: #f59e0b;
    border: 1px solid #f59e0b;
  }

  .event-error {
    background: var(--status-error-bg);
    color: var(--status-error-text);
    border: 1px solid var(--status-error-border);
  }

  .event-unknown {
    background: var(--status-pending-bg);
    color: var(--status-pending-text);
    border: 1px solid var(--status-pending-border);
  }

  .event-reason {
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    font-size: 0.8rem;
    font-weight: 500;
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
  }

  .reason-created {
    background: rgba(34, 197, 94, 0.2);
    color: #22c55e;
  }

  .reason-scheduled {
    background: rgba(59, 130, 246, 0.2);
    color: #3b82f6;
  }

  .reason-pulling, .reason-pulled {
    background: rgba(168, 85, 247, 0.2);
    color: #a855f7;
  }

  .reason-started {
    background: rgba(34, 197, 94, 0.2);
    color: #22c55e;
  }

  .reason-killing, .reason-killed {
    background: rgba(245, 158, 11, 0.2);
    color: #f59e0b;
  }

  .reason-failed, .reason-backoff, .reason-unhealthy {
    background: rgba(239, 68, 68, 0.2);
    color: #ef4444;
  }

  .reason-unknown {
    background: rgba(107, 114, 128, 0.2);
    color: #6b7280;
  }

  .event-time {
    color: var(--text-secondary);
    font-size: 0.9rem;
    margin-left: auto;
  }

  .event-message {
    color: var(--text-primary);
    font-size: 0.95rem;
    line-height: 1.4;
    margin-bottom: var(--spacing-sm);
  }

  .event-details {
    display: flex;
    gap: var(--spacing-lg);
    flex-wrap: wrap;
  }

  .event-detail {
    display: flex;
    gap: var(--spacing-xs);
  }

  .event-detail-label {
    color: var(--text-secondary);
    font-size: 0.8rem;
    font-weight: 500;
  }

  .event-detail-value {
    color: var(--text-primary);
    font-size: 0.8rem;
  }

  /* Metrics Section */
  .metrics-loading, .metrics-error, .metrics-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-lg);
    text-align: center;
  }

  .metrics-loading {
    color: var(--text-secondary);
  }

  .metrics-error {
    color: var(--error-color);
  }

  .metrics-placeholder {
    color: var(--text-muted);
  }

  .error-icon {
    font-size: 1.5rem;
  }

  .retry-button {
    background: var(--accent-color);
    border: none;
    border-radius: var(--radius-sm);
    color: white;
    cursor: pointer;
    font-size: 0.9rem;
    padding: var(--spacing-sm) var(--spacing-md);
    transition: var(--transition-normal);
  }

  .retry-button:hover {
    background: var(--primary-color);
  }

  .metrics-graph-container {
    background: var(--background-card);
    border-radius: var(--radius-sm);
    padding: var(--spacing-sm);
    border: 1px solid var(--border-primary);
  }

  /* Resource Type Selector */
  .resource-type-selector {
    margin-bottom: var(--spacing-md);
  }

  .metrics-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-sm);
    flex-wrap: wrap;
    gap: var(--spacing-sm);
  }

  .resource-type-selector h6 {
    margin: 0;
    color: var(--text-primary);
    font-size: 1.1rem;
    font-weight: 600;
  }

  .time-range-selector {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .time-range-selector label {
    color: var(--text-secondary);
    font-size: 0.9rem;
    font-weight: 500;
    white-space: nowrap;
  }

  .time-range-select {
    background: var(--background-card);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 0.9rem;
    font-weight: 500;
    padding: var(--spacing-xs) var(--spacing-sm);
    cursor: pointer;
    transition: var(--transition-normal);
    min-width: 120px;
  }

  .time-range-select:hover {
    border-color: var(--accent-color);
    background: var(--background-tertiary);
  }

  .time-range-select:focus {
    outline: none;
    border-color: var(--primary-color);
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.3);
  }

  .resource-tabs {
    display: flex;
    gap: var(--spacing-xs);
    background: var(--background-secondary);
    border-radius: var(--radius-sm);
    padding: var(--spacing-xs);
    border: 1px solid var(--border-secondary);
  }

  .resource-tab {
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 0.9rem;
    font-weight: 500;
    padding: var(--spacing-xs) var(--spacing-sm);
    transition: var(--transition-normal);
    flex: 1;
    text-align: center;
  }

  .resource-tab:hover {
    background: var(--background-tertiary);
    border-color: var(--border-primary);
    color: var(--text-primary);
  }

  .resource-tab.active {
    background: var(--primary-color);
    border-color: var(--primary-color);
    color: white;
    font-weight: 600;
  }

  .resource-tab.active:hover {
    background: var(--accent-color);
    border-color: var(--accent-color);
  }

  /* QoS Class Styling */
  .qos-badge {
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    font-size: 0.8rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  .qos-guaranteed {
    background: var(--status-ready-bg);
    color: var(--status-ready-text);
    border: 1px solid var(--status-ready-border);
  }

  .qos-burstable {
    background: rgba(245, 158, 11, 0.2);
    color: #f59e0b;
    border: 1px solid #f59e0b;
  }

  .qos-besteffort {
    background: rgba(239, 68, 68, 0.2);
    color: var(--error-color);
    border: 1px solid var(--error-color);
  }

  .qos-unknown {
    background: var(--status-pending-bg);
    color: var(--status-pending-text);
    border: 1px solid var(--status-pending-border);
  }

  /* Labels and Annotations */
  .labels-section {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  .labels-subsection {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .labels-subsection h6 {
    margin: 0;
    color: var(--text-primary);
    font-size: 1rem;
    font-weight: 600;
  }

  .labels-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: var(--spacing-sm);
  }

  .label-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm);
    background: rgba(255, 255, 255, 0.02);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-primary);
  }

  .label-key {
    color: var(--text-secondary);
    font-weight: 600;
    font-size: 0.9rem;
    min-width: 120px;
  }

  .label-value {
    color: var(--text-primary);
    font-size: 0.9rem;
    flex: 1;
    word-break: break-all;
  }

  /* Conditions */
  .conditions-grid {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .condition-item {
    padding: var(--spacing-md);
    background: rgba(255, 255, 255, 0.02);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-primary);
  }

  .condition-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-sm);
  }

  .condition-type {
    color: var(--text-primary);
    font-weight: 600;
    font-size: 1rem;
  }

  .condition-status {
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    font-size: 0.8rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  .condition-true {
    background: var(--status-ready-bg);
    color: var(--status-ready-text);
    border: 1px solid var(--status-ready-border);
  }

  .condition-false {
    background: var(--status-error-bg);
    color: var(--status-error-text);
    border: 1px solid var(--status-error-border);
  }

  .condition-unknown {
    background: var(--status-pending-bg);
    color: var(--status-pending-text);
    border: 1px solid var(--status-pending-border);
  }

  .condition-details {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .condition-detail {
    display: flex;
    gap: var(--spacing-sm);
  }

  .condition-label {
    color: var(--text-secondary);
    font-weight: 500;
    font-size: 0.9rem;
    min-width: 120px;
  }

  .condition-value {
    color: var(--text-primary);
    font-size: 0.9rem;
    flex: 1;
  }

  /* Tolerations */
  .tolerations-grid {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .toleration-item {
    padding: var(--spacing-md);
    background: rgba(255, 255, 255, 0.02);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-primary);
  }

  .toleration-header {
    display: flex;
    gap: var(--spacing-sm);
    align-items: center;
    margin-bottom: var(--spacing-sm);
  }

  .toleration-key {
    color: var(--text-primary);
    font-weight: 600;
    font-size: 0.9rem;
  }

  .toleration-operator {
    color: var(--text-secondary);
    font-size: 0.8rem;
    padding: 2px 6px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: var(--radius-sm);
  }

  .toleration-effect {
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    font-size: 0.8rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  .toleration-noschedule {
    background: rgba(239, 68, 68, 0.2);
    color: var(--error-color);
    border: 1px solid var(--error-color);
  }

  .toleration-prefernoschedule {
    background: rgba(245, 158, 11, 0.2);
    color: #f59e0b;
    border: 1px solid #f59e0b;
  }

  .toleration-noexecute {
    background: rgba(220, 38, 38, 0.2);
    color: #dc2626;
    border: 1px solid #dc2626;
  }

  .toleration-unknown {
    background: var(--status-pending-bg);
    color: var(--status-pending-text);
    border: 1px solid var(--status-pending-border);
  }

  .toleration-detail {
    display: flex;
    gap: var(--spacing-sm);
    margin-top: var(--spacing-xs);
  }

  .toleration-label {
    color: var(--text-secondary);
    font-weight: 500;
    font-size: 0.9rem;
    min-width: 120px;
  }

  .toleration-value {
    color: var(--text-primary);
    font-size: 0.9rem;
  }

  /* Affinity Rules */
  .affinity-content {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .affinity-json {
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    padding: var(--spacing-md);
    color: var(--text-primary);
    font-family: 'Courier New', monospace;
    font-size: 0.8rem;
    line-height: 1.4;
    white-space: pre-wrap;
    word-break: break-all;
    max-height: 300px;
    overflow-y: auto;
  }

  /* Volumes */
  .volumes-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: var(--spacing-md);
  }

  .volume-item {
    padding: var(--spacing-md);
    background: rgba(255, 255, 255, 0.02);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-primary);
  }

  .volume-header {
    margin-bottom: var(--spacing-sm);
  }

  .volume-name {
    color: var(--text-primary);
    font-weight: 600;
    font-size: 1rem;
  }

  .volume-details {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .volume-detail {
    display: flex;
    gap: var(--spacing-sm);
  }

  .volume-type {
    color: var(--text-secondary);
    font-weight: 500;
    font-size: 0.9rem;
    min-width: 100px;
  }

  .volume-value {
    color: var(--text-primary);
    font-size: 0.9rem;
  }

  /* No Data Styling */
  .no-data {
    color: var(--text-muted);
    font-style: italic;
    text-align: center;
    padding: var(--spacing-lg);
    margin: 0;
  }

  /* Responsive Design */
  @media (max-width: 768px) {
    .pods-grid {
      grid-template-columns: 1fr;
    }
    
    .info-grid {
      grid-template-columns: 1fr;
    }
    
    .details-header {
      flex-direction: column;
      align-items: flex-start;
    }

    .container-summary {
      flex-direction: column;
      gap: var(--spacing-sm);
    }
  }

  /* YAML Viewer Modal */
  .yaml-viewer-modal {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 10001;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 20px;
  }

  .yaml-viewer-content {
    background: var(--bg-secondary);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    width: 90%;
    max-width: 900px;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .yaml-viewer-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.05);
  }

  .yaml-viewer-header h3 {
    margin: 0;
    color: var(--text-primary);
    font-size: 1.1rem;
    font-weight: 600;
  }

  .yaml-viewer-close {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 1.2rem;
    line-height: 1;
    transition: all 0.2s;
  }

  .yaml-viewer-close:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
  }

  .yaml-viewer-body {
    flex: 1;
    overflow: auto;
    padding: 20px;
  }

  .yaml-viewer-body pre {
    margin: 0;
    padding: 0;
    background: transparent;
    color: var(--text-primary);
    font-family: 'Courier New', Courier, monospace;
    font-size: 0.85rem;
    line-height: 1.6;
    white-space: pre-wrap;
    word-wrap: break-word;
  }

  .yaml-viewer-body code {
    color: var(--text-primary);
  }
</style>
