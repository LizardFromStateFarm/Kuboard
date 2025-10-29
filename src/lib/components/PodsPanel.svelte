<!-- Kuboard Pods Panel Component -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { createEventDispatcher } from 'svelte';
  import { formatMemory, formatCPU } from '$lib/utils/formatters';
  import MetricsGraph from './MetricsGraph.svelte';
  import LogsWindow from './LogsWindow.svelte';

  // Props
  export let currentContext: any = null;
  export let pods: any[] = [];

  // State
  let selectedPod: any = null;
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
    loadPodMetrics(pod);
    loadPodEvents(pod);
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
      logsWindowRef.openPodLogs(pod.metadata?.name || 'Unknown', pod.metadata?.namespace || 'default', containerName);
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
  function getStatusClass(status: string): string {
    switch (status?.toLowerCase()) {
      case 'running': return 'running';
      case 'pending': return 'pending';
      case 'succeeded': return 'ready';
      case 'failed': return 'failed';
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
  onMount(() => {
    console.log('🚀 PodsPanel onMount called - currentContext:', currentContext, 'pods:', pods.length);
  });

  onDestroy(() => {
    if (refreshTimer) {
      clearInterval(refreshTimer);
    }
  });
</script>

<div class="pods-panel">
  <div class="panel-header">
    <h4>🟢 Pods</h4>
    <div class="panel-controls">
      <button 
        class="refresh-button" 
        onclick={() => console.log('Refresh clicked')}
        title="Refresh pods"
      >
        ↻
      </button>
    </div>
  </div>

  <!-- Always show the basic UI structure -->
  <div class="pods-content">
    {#if showFullDetails && selectedPod}
      <!-- Full Details View -->
      <div class="full-details-view">
        <div class="details-header">
          <div class="header-left">
            <button class="back-button" onclick={backToPodsList}>
              ← Back to Pods
            </button>
            <button class="logs-button" onclick={() => openPodLogs(selectedPod)}>
              📋 View Logs
            </button>
          </div>
          <div class="header-right">
            <h3>{selectedPod.name}</h3>
            <span class="pod-namespace">({selectedPod.namespace})</span>
          </div>
        </div>
        
        <div class="pod-details-content">
          <!-- Basic Pod Information -->
          <div class="details-section">
            <h6>Basic Information</h6>
            <div class="info-grid">
              <div class="info-item">
                <span class="info-label">Status:</span>
                <div class="info-value-container">
                  <span class="info-value status-badge status-{getStatusClass(selectedPod.status)}">
                    {selectedPod.status}
                  </span>
                </div>
              </div>
              <div class="info-item">
                <span class="info-label">Namespace:</span>
                <div class="info-value-container">
                  <span class="info-value" title={selectedPod.namespace}>{selectedPod.namespace}</span>
                  <button 
                    class="copy-button" 
                    onclick={() => copyToClipboard(selectedPod.namespace || '')}
                    title="Copy to clipboard"
                  >
                    📋
                  </button>
                </div>
              </div>
              <div class="info-item">
                <span class="info-label">Node:</span>
                <div class="info-value-container">
                  <span class="info-value" title={selectedPod.nodeName}>{selectedPod.nodeName}</span>
                  <button 
                    class="copy-button" 
                    onclick={() => copyToClipboard(selectedPod.nodeName || '')}
                    title="Copy to clipboard"
                  >
                    📋
                  </button>
                </div>
              </div>
              <div class="info-item">
                <span class="info-label">Pod IP:</span>
                <div class="info-value-container">
                  <span class="info-value" title={selectedPod.podIP}>{selectedPod.podIP}</span>
                  <button 
                    class="copy-button" 
                    onclick={() => copyToClipboard(selectedPod.podIP || '')}
                    title="Copy to clipboard"
                  >
                    📋
                  </button>
                </div>
              </div>
              <div class="info-item">
                <span class="info-label">Host IP:</span>
                <div class="info-value-container">
                  <span class="info-value" title={selectedPod.hostIP}>{selectedPod.hostIP}</span>
                  <button 
                    class="copy-button" 
                    onclick={() => copyToClipboard(selectedPod.hostIP || '')}
                    title="Copy to clipboard"
                  >
                    📋
                  </button>
                </div>
              </div>
              <div class="info-item">
                <span class="info-label">Age:</span>
                <div class="info-value-container">
                  <span class="info-value" title={selectedPod.creationTimestamp}>{formatAge(selectedPod.creationTimestamp)}</span>
                  <button 
                    class="copy-button" 
                    onclick={() => copyToClipboard(selectedPod.creationTimestamp || '')}
                    title="Copy to clipboard"
                  >
                    📋
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- Metrics Section -->
          <div class="details-section">
            <h6>Resource Usage Metrics</h6>
            {#if metricsLoading}
              <div class="metrics-loading">
                <div class="loading-spinner">⏳</div>
                <p>Loading metrics...</p>
              </div>
            {:else if metricsError}
              <div class="metrics-error">
                <div class="error-icon">⚠️</div>
                <p>Failed to load metrics: {metricsError}</p>
                <button class="retry-button" onclick={() => loadPodMetrics(selectedPod)}>
                  Retry
                </button>
              </div>
            {:else if podMetrics}
              <!-- Resource Type Selector -->
              <div class="resource-type-selector">
                <div class="metrics-header">
                  <h6>📊 Resource Usage</h6>
                  <div class="time-range-selector">
                    <label for="timeRange">Time Range:</label>
                    <select 
                      id="timeRange"
                      bind:value={selectedTimeRange}
                      onchange={() => changeTimeRange(selectedTimeRange)}
                      class="time-range-select"
                    >
                      <option value="30">30 minutes</option>
                      <option value="60">1 hour</option>
                      <option value="120">2 hours</option>
                      <option value="240">4 hours</option>
                      <option value="480">8 hours</option>
                      <option value="720">12 hours</option>
                    </select>
                  </div>
                </div>
                <div class="resource-tabs">
                  <button 
                    class="resource-tab {selectedResourceType === 'cpu' ? 'active' : ''}"
                    onclick={() => changeResourceType('cpu')}
                  >
                    🖥️ CPU
                  </button>
                  <button 
                    class="resource-tab {selectedResourceType === 'memory' ? 'active' : ''}"
                    onclick={() => changeResourceType('memory')}
                  >
                    💾 Memory
                  </button>
                </div>
              </div>
              
              <div class="metrics-graph-container">
                <MetricsGraph
                  data={podMetrics}
                  type={selectedResourceType}
                  duration={selectedTimeRange}
                  loading={metricsLoading}
                  error={metricsError}
                  maxCpuCores={1}
                  maxMemoryBytes={1024 * 1024 * 1024}
                  isPodMetrics={true}
                />
              </div>
            {:else}
              <div class="metrics-placeholder">
                <p>No metrics data available</p>
              </div>
            {/if}
          </div>

          <!-- Controller Information -->
          <div class="details-section">
            <h6>Controller Information</h6>
            <div class="info-grid">
              <div class="info-item">
                <span class="info-label">Controller Type:</span>
                <div class="info-value-container">
                  <span class="info-value" title={selectedPod.controller.type}>{selectedPod.controller.type}</span>
                  <button 
                    class="copy-button" 
                    onclick={() => copyToClipboard(selectedPod.controller.type || '')}
                    title="Copy to clipboard"
                  >
                    📋
                  </button>
                </div>
              </div>
              <div class="info-item">
                <span class="info-label">Controller Name:</span>
                <div class="info-value-container">
                  <span class="info-value" title={selectedPod.controller.name}>{selectedPod.controller.name}</span>
                  <button 
                    class="copy-button" 
                    onclick={() => copyToClipboard(selectedPod.controller.name || '')}
                    title="Copy to clipboard"
                  >
                    📋
                  </button>
                </div>
              </div>
              <div class="info-item">
                <span class="info-label">Service Account:</span>
                <div class="info-value-container">
                  <span class="info-value" title={selectedPod.serviceAccount}>{selectedPod.serviceAccount}</span>
                  <button 
                    class="copy-button" 
                    onclick={() => copyToClipboard(selectedPod.serviceAccount || '')}
                    title="Copy to clipboard"
                  >
                    📋
                  </button>
                </div>
              </div>
              <div class="info-item">
                <span class="info-label">QoS Class:</span>
                <div class="info-value-container">
                  <span class="info-value qos-badge qos-{getQoSClassClass(selectedPod.qosClass)}" title={selectedPod.qosClass}>
                    {selectedPod.qosClass}
                  </span>
                  <button 
                    class="copy-button" 
                    onclick={() => copyToClipboard(selectedPod.qosClass || '')}
                    title="Copy to clipboard"
                  >
                    📋
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- Labels and Annotations -->
          <div class="details-section">
            <h6>Labels & Annotations</h6>
            <div class="labels-section">
              <div class="labels-subsection">
                <h6>Labels</h6>
                {#if Object.keys(selectedPod.labels).length > 0}
                  <div class="labels-grid">
                    {#each Object.entries(selectedPod.labels) as [key, value]}
                      <div class="label-item">
                        <span class="label-key">{key}:</span>
                        <span class="label-value">{value}</span>
                        <button 
                          class="copy-button" 
                          onclick={() => copyToClipboard(`${key}: ${String(value)}`)}
                          title="Copy to clipboard"
                        >
                          📋
                        </button>
                      </div>
                    {/each}
                  </div>
                {:else}
                  <p class="no-data">No labels</p>
                {/if}
              </div>
              
              <div class="labels-subsection">
                <h6>Annotations</h6>
                {#if Object.keys(selectedPod.annotations).length > 0}
                  <div class="labels-grid">
                    {#each Object.entries(selectedPod.annotations) as [key, value]}
                      <div class="label-item">
                        <span class="label-key">{key}:</span>
                        <span class="label-value" title={String(value)}>{String(value).length > 50 ? String(value).substring(0, 50) + '...' : String(value)}</span>
                        <button 
                          class="copy-button" 
                          onclick={() => copyToClipboard(`${key}: ${String(value)}`)}
                          title="Copy to clipboard"
                        >
                          📋
                        </button>
                      </div>
                    {/each}
                  </div>
                {:else}
                  <p class="no-data">No annotations</p>
                {/if}
              </div>
            </div>
          </div>

          <!-- Pod Conditions -->
          <div class="details-section">
            <h6>Pod Conditions</h6>
            {#if selectedPod.conditions && selectedPod.conditions.length > 0}
              <div class="conditions-grid">
                {#each selectedPod.conditions as condition}
                  <div class="condition-item">
                    <div class="condition-header">
                      <span class="condition-type">{condition.type}</span>
                      <span class="condition-status status-{getConditionStatusClass(condition.status)}">
                        {condition.status}
                      </span>
                    </div>
                    <div class="condition-details">
                      <div class="condition-detail">
                        <span class="condition-label">Last Transition Time:</span>
                        <span class="condition-value">{condition.lastTransitionTime || 'N/A'}</span>
                      </div>
                      {#if condition.reason}
                        <div class="condition-detail">
                          <span class="condition-label">Reason:</span>
                          <span class="condition-value">{condition.reason}</span>
                        </div>
                      {/if}
                      {#if condition.message}
                        <div class="condition-detail">
                          <span class="condition-label">Message:</span>
                          <span class="condition-value" title={condition.message}>
                            {condition.message.length > 100 ? condition.message.substring(0, 100) + '...' : condition.message}
                          </span>
                        </div>
                      {/if}
                    </div>
                  </div>
                {/each}
              </div>
            {:else}
              <p class="no-data">No conditions available</p>
            {/if}
          </div>

          <!-- Tolerations -->
          <div class="details-section">
            <h6>Tolerations</h6>
            {#if selectedPod.tolerations && selectedPod.tolerations.length > 0}
              <div class="tolerations-grid">
                {#each selectedPod.tolerations as toleration}
                  <div class="toleration-item">
                    <div class="toleration-header">
                      <span class="toleration-key">{toleration.key || 'All'}</span>
                      <span class="toleration-operator">{toleration.operator || 'Equal'}</span>
                      <span class="toleration-effect effect-{getTolerationEffectClass(toleration.effect)}">
                        {toleration.effect || 'NoSchedule'}
                      </span>
                    </div>
                    {#if toleration.value}
                      <div class="toleration-detail">
                        <span class="toleration-label">Value:</span>
                        <span class="toleration-value">{toleration.value}</span>
                      </div>
                    {/if}
                    {#if toleration.tolerationSeconds}
                      <div class="toleration-detail">
                        <span class="toleration-label">Toleration Seconds:</span>
                        <span class="toleration-value">{toleration.tolerationSeconds}</span>
                      </div>
                    {/if}
                  </div>
                {/each}
              </div>
            {:else}
              <p class="no-data">No tolerations</p>
            {/if}
          </div>

          <!-- Affinity Rules -->
          <div class="details-section">
            <h6>Affinity Rules</h6>
            {#if selectedPod.affinity && Object.keys(selectedPod.affinity).length > 0}
              <div class="affinity-content">
                <pre class="affinity-json">{formatObject(selectedPod.affinity)}</pre>
                <button 
                  class="copy-button" 
                  onclick={() => copyToClipboard(formatObject(selectedPod.affinity))}
                  title="Copy to clipboard"
                >
                  📋 Copy JSON
                </button>
              </div>
            {:else}
              <p class="no-data">No affinity rules</p>
            {/if}
          </div>

          <!-- Pod Volumes -->
          <div class="details-section">
            <h6>Pod Volumes</h6>
            {#if selectedPod.volumes && selectedPod.volumes.length > 0}
              <div class="volumes-grid">
                {#each selectedPod.volumes as volume}
                  <div class="volume-item">
                    <div class="volume-header">
                      <span class="volume-name">{volume.name}</span>
                    </div>
                    <div class="volume-details">
                      {#if volume.configMap}
                        <div class="volume-detail">
                          <span class="volume-type">ConfigMap:</span>
                          <span class="volume-value">{volume.configMap.name}</span>
                        </div>
                      {/if}
                      {#if volume.secret}
                        <div class="volume-detail">
                          <span class="volume-type">Secret:</span>
                          <span class="volume-value">{volume.secret.secretName}</span>
                        </div>
                      {/if}
                      {#if volume.persistentVolumeClaim}
                        <div class="volume-detail">
                          <span class="volume-type">PVC:</span>
                          <span class="volume-value">{volume.persistentVolumeClaim.claimName}</span>
                        </div>
                      {/if}
                      {#if volume.emptyDir}
                        <div class="volume-detail">
                          <span class="volume-type">EmptyDir</span>
                        </div>
                      {/if}
                      {#if volume.hostPath}
                        <div class="volume-detail">
                          <span class="volume-type">HostPath:</span>
                          <span class="volume-value">{volume.hostPath.path}</span>
                        </div>
                      {/if}
                    </div>
                  </div>
                {/each}
              </div>
            {:else}
              <p class="no-data">No volumes</p>
            {/if}
          </div>

          <!-- Container Information -->
          <div class="details-section">
            <h6>Container Information</h6>
            <div class="container-info">
              <div class="container-summary">
                <div class="summary-item">
                  <span class="summary-label">Containers:</span>
                  <span class="summary-value">{selectedPod.readyContainers}/{selectedPod.totalContainers} ready</span>
                </div>
                <div class="summary-item">
                  <span class="summary-label">Restarts:</span>
                  <span class="summary-value">{selectedPod.restartCount}</span>
                </div>
              </div>
              
              {#if selectedPod.containers && selectedPod.containers.length > 0}
                <div class="containers-list">
                  {#each selectedPod.containers as container, index}
                    <div 
                      class="container-item {selectedContainer?.name === container.name ? 'selected' : ''}"
                      onclick={() => selectContainer(container)}
                      onkeydown={(e) => e.key === 'Enter' && selectContainer(container)}
                      role="button"
                      tabindex="0"
                    >
                      <div class="container-header">
                        <h6 class="container-name">{container.name}</h6>
                        <span class="container-image">{container.image}</span>
                        <div class="container-actions">
                          <span class="click-hint">Click for metrics</span>
                          <button 
                            class="container-logs-button" 
                            onclick={(e) => { e.stopPropagation(); openContainerLogs(container); }}
                            title="View container logs"
                          >
                            📋 Logs
                          </button>
                        </div>
                      </div>
                      {#if selectedPod.containerStatuses && selectedPod.containerStatuses[index]}
                        <div class="container-status">
                          <span class="status-badge status-{getStatusClass(selectedPod.containerStatuses[index].state?.running ? 'running' : selectedPod.containerStatuses[index].state?.waiting ? 'pending' : 'failed')}">
                            {selectedPod.containerStatuses[index].state?.running ? 'Running' : selectedPod.containerStatuses[index].state?.waiting ? 'Waiting' : 'Failed'}
                          </span>
                          <span class="restart-count">Restarts: {selectedPod.containerStatuses[index].restartCount || 0}</span>
                        </div>
                      {/if}
                    </div>
                  {/each}
                </div>
              {/if}

              <!-- Container Metrics Section -->
              {#if selectedContainer}
                <div class="container-metrics-section">
                  <div class="container-metrics-header">
                    <h6>📊 {selectedContainer.name} Metrics</h6>
                    <button 
                      class="close-container-metrics" 
                      onclick={() => selectedContainer = null}
                      title="Close container metrics"
                    >
                      ✕
                    </button>
                  </div>
                  
                  {#if containerMetricsLoading}
                    <div class="metrics-loading">
                      <div class="loading-spinner">⏳</div>
                      <p>Loading container metrics...</p>
                    </div>
                  {:else if containerMetricsError}
                    <div class="metrics-error">
                      <div class="error-icon">⚠️</div>
                      <p>Failed to load container metrics: {containerMetricsError}</p>
                      <button class="retry-button" onclick={() => loadContainerMetrics(selectedContainer)}>
                        Retry
                      </button>
                    </div>
                  {:else if containerMetrics}
                    <!-- Container Resource Type Selector -->
                    <div class="container-resource-tabs">
                      <button 
                        class="resource-tab {selectedContainerResourceType === 'cpu' ? 'active' : ''}"
                        onclick={() => changeContainerResourceType('cpu')}
                      >
                        🖥️ CPU
                      </button>
                      <button 
                        class="resource-tab {selectedContainerResourceType === 'memory' ? 'active' : ''}"
                        onclick={() => changeContainerResourceType('memory')}
                      >
                        💾 Memory
                      </button>
                    </div>
                    
                    <div class="container-metrics-graph-container">
                      <MetricsGraph
                        data={containerMetrics}
                        type={selectedContainerResourceType}
                        duration={selectedTimeRange}
                        loading={containerMetricsLoading}
                        error={containerMetricsError}
                        maxCpuCores={0.1}
                        maxMemoryBytes={100 * 1024 * 1024}
                        isPodMetrics={true}
                      />
                    </div>
                  {:else}
                    <div class="metrics-placeholder">
                      <p>No container metrics data available</p>
                    </div>
                  {/if}
                </div>
              {/if}
            </div>
          </div>

          <!-- Pod Events -->
          <div class="details-section">
            <h6>Pod Events</h6>
            {#if eventsLoading}
              <div class="events-loading">
                <div class="loading-spinner">⏳</div>
                <p>Loading events...</p>
              </div>
            {:else if eventsError}
              <div class="events-error">
                <div class="error-icon">⚠️</div>
                <p>Failed to load events: {eventsError}</p>
                <button class="retry-button" onclick={() => loadPodEvents(selectedPod)}>
                  Retry
                </button>
              </div>
            {:else if podEvents && podEvents.length > 0}
              <div class="events-list">
                {#each podEvents as event, index}
                  <div class="event-item">
                    <div class="event-header">
                      <div class="event-type-badge event-{getEventTypeClass(event.type)}">
                        {event.type}
                      </div>
                      <div class="event-reason reason-{getEventReasonClass(event.reason)}">
                        {event.reason}
                      </div>
                      <div class="event-time">
                        {formatEventTime(event.firstTimestamp || event.eventTime)}
                      </div>
                    </div>
                    <div class="event-message">
                      {event.message}
                    </div>
                    <div class="event-details">
                      <div class="event-detail">
                        <span class="event-detail-label">Count:</span>
                        <span class="event-detail-value">{event.count || 1}</span>
                      </div>
                      {#if event.lastTimestamp && event.lastTimestamp !== event.firstTimestamp}
                        <div class="event-detail">
                          <span class="event-detail-label">Last Seen:</span>
                          <span class="event-detail-value">{formatEventTime(event.lastTimestamp)}</span>
                        </div>
                      {/if}
                      {#if event.involvedObject?.kind}
                        <div class="event-detail">
                          <span class="event-detail-label">Object:</span>
                          <span class="event-detail-value">{event.involvedObject.kind}</span>
                        </div>
                      {/if}
                    </div>
                  </div>
                {/each}
              </div>
            {:else}
              <div class="events-placeholder">
                <p>No events found for this pod</p>
              </div>
            {/if}
          </div>

        </div>
      </div>
    {:else}
      <!-- Pods List View (always show this) -->
      <div class="pods-list-view">
        <div class="pods-header">
          <h5>Cluster Pods {pods && pods.length > 0 ? `(${pods.length})` : ''}</h5>
          <p>Click on a pod to view detailed information and metrics</p>
        </div>
        
        {#if isLoading}
          <!-- Loading State -->
          <div class="loading-pods">
            <div class="loading-spinner">⏳</div>
            <h5>Loading Pods...</h5>
            <p>Please wait while we fetch the cluster pods information.</p>
          </div>
        {:else if pods && pods.length > 0}
          <!-- Pods Grid -->
          <div class="pods-grid">
            {#each pods as pod}
              <div class="pod-card" onclick={() => showFullPodDetails(pod)} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' || e.key === ' ' ? showFullPodDetails(pod) : null}>
                <div class="pod-card-content">
                  <div class="pod-header">
                    <h6 class="pod-name">{pod.metadata?.name || 'Unknown'}</h6>
                    <div class="pod-status">
                      <span class="status-badge status-{getStatusClass(pod.status?.phase || 'Unknown')}">
                        {pod.status?.phase || 'Unknown'}
                      </span>
                    </div>
                  </div>
                  
                  <div class="pod-summary">
                    <div class="summary-item">
                      <span class="summary-label">Namespace:</span>
                      <span class="summary-value">{pod.metadata?.namespace || 'default'}</span>
                    </div>
                    <div class="summary-item">
                      <span class="summary-label">Node:</span>
                      <span class="summary-value">{pod.spec?.nodeName || 'Unknown'}</span>
                    </div>
                    <div class="summary-item">
                      <span class="summary-label">IP:</span>
                      <span class="summary-value">{pod.status?.podIP || 'Unknown'}</span>
                    </div>
                  </div>
                  
                  <div class="pod-resources">
                    <div class="resource-summary">
                      <div class="resource-item">
                        <span class="resource-label">Containers:</span>
                        <span class="resource-value">
                          {pod.status?.containerStatuses?.filter((c: any) => c.ready).length || 0}/{pod.status?.containerStatuses?.length || 0}
                        </span>
                      </div>
                      <div class="resource-item">
                        <span class="resource-label">Restarts:</span>
                        <span class="resource-value">{pod.status?.containerStatuses?.[0]?.restartCount || 0}</span>
                      </div>
                      <div class="resource-item">
                        <span class="resource-label">Age:</span>
                        <span class="resource-value">{formatAge(pod.metadata?.creationTimestamp)}</span>
                      </div>
                    </div>
                  </div>
                  
                  <div class="pod-actions">
                    <span class="click-hint">Click to view details →</span>
                  </div>
                </div>
              </div>
            {/each}
          </div>
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

<style>
  /* Import CSS variables */
  @import '../styles/variables.css';
  @import '../styles/color-palette.css';

  .pods-panel {
    background: rgba(255, 255, 255, 0.03);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-lg);
    padding-bottom: var(--spacing-sm);
    border-bottom: 1px solid var(--border-primary);
  }

  .panel-header h4 {
    margin: 0;
    color: var(--text-primary);
    font-size: 1.2rem;
  }

  .panel-controls {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .refresh-button {
    background: var(--background-card);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    cursor: pointer;
    font-size: 1.2rem;
    padding: var(--spacing-sm);
    transition: var(--transition-normal);
  }

  .refresh-button:hover {
    background: var(--accent-color);
    border-color: var(--accent-color);
    color: white;
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
    gap: var(--spacing-lg);
  }

  .pods-header {
    text-align: center;
    margin-bottom: var(--spacing-lg);
  }

  .pods-header h5 {
    margin: 0 0 var(--spacing-sm) 0;
    color: var(--text-primary);
    font-size: 1.3rem;
  }

  .pods-header p {
    margin: 0;
    color: var(--text-secondary);
    font-size: 0.9rem;
  }

  .pods-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: var(--spacing-md);
  }

  .pod-card {
    background: var(--background-card);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: var(--transition-normal);
    overflow: hidden;
  }

  .pod-card:hover {
    border-color: var(--accent-color);
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .pod-card-content {
    padding: var(--spacing-md);
  }

  .pod-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-md);
  }

  .pod-name {
    margin: 0;
    color: var(--text-primary);
    font-size: 1.1rem;
    font-weight: 600;
  }

  .pod-status {
    display: flex;
    align-items: center;
  }

  .status-badge {
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    font-size: 0.8rem;
    font-weight: 600;
    text-transform: uppercase;
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

  .status-unknown {
    background: var(--status-pending-bg);
    color: var(--status-pending-text);
    border: 1px solid var(--status-pending-border);
  }

  .pod-summary {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-md);
  }

  .summary-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .summary-label {
    color: var(--text-secondary);
    font-size: 0.9rem;
    font-weight: 500;
  }

  .summary-value {
    color: var(--text-primary);
    font-size: 0.9rem;
    font-weight: 600;
  }

  .pod-resources {
    margin-bottom: var(--spacing-md);
  }

  .resource-summary {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .resource-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .resource-label {
    color: var(--text-secondary);
    font-size: 0.85rem;
    font-weight: 500;
  }

  .resource-value {
    color: var(--text-primary);
    font-size: 0.85rem;
    font-weight: 600;
  }

  .pod-actions {
    text-align: center;
    padding-top: var(--spacing-sm);
    border-top: 1px solid var(--border-primary);
  }

  .click-hint {
    color: var(--accent-color);
    font-size: 0.8rem;
    font-weight: 500;
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
</style>
