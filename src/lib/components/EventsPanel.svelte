<!-- Kuboard Events Panel Component -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import ResourceTable from './ResourceTable.svelte';
  import { Activity, AlertTriangle, CheckCircle2, Radio, Pause, RefreshCw } from 'lucide-svelte';

  // Props
  export let currentContext: any = null;
  export let namespace: string = 'all';

  // State
  let events: any[] = [];
  let loading: boolean = true;
  let error: string | null = null;
  let searchQuery: string = '';
  let typeFilter: 'all' | 'warning' | 'normal' = 'all';
  let sortColumn: string = 'timestamp';
  let sortDirection: 'asc' | 'desc' = 'desc';



  function handleSort(column: string) {
    if (sortColumn === column) {
      sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
    } else {
      sortColumn = column;
      sortDirection = 'asc';
    }
  }

  function formatTime(timestamp: string | undefined): string {
    if (!timestamp) return '-';
    try {
      const d = new Date(timestamp);
      return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
    } catch {
      return timestamp;
    }
  }

  function formatAge(timestamp: string | undefined): string {
    if (!timestamp) return '-';
    const created = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - created.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMins / 60);
    const diffDays = Math.floor(diffHours / 24);
    if (diffDays > 0) return `${diffDays}d`;
    if (diffHours > 0) return `${diffHours}h`;
    return `${diffMins}m`;
  }

  $: filteredEvents = events.filter(evt => {
    const matchesNamespace = namespace === 'all' || evt.involved_object?.namespace === namespace;
    const matchesType = typeFilter === 'all' || (evt.type_ || '').toLowerCase() === typeFilter;
    const q = searchQuery.toLowerCase();
    const matchesSearch = !q ||
      (evt.reason || '').toLowerCase().includes(q) ||
      (evt.message || '').toLowerCase().includes(q) ||
      (evt.involved_object?.name || '').toLowerCase().includes(q) ||
      (evt.involved_object?.kind || '').toLowerCase().includes(q) ||
      (evt.involved_object?.namespace || '').toLowerCase().includes(q);
    return matchesNamespace && matchesType && matchesSearch;
  });

  $: sortedEvents = [...filteredEvents].sort((a, b) => {
    let comp = 0;
    if (sortColumn === 'timestamp') {
      const timeA = new Date(a.last_timestamp || a.first_timestamp || 0).getTime();
      const timeB = new Date(b.last_timestamp || b.first_timestamp || 0).getTime();
      comp = timeA - timeB;
    } else if (sortColumn === 'type') {
      comp = (a.type_ || '').localeCompare(b.type_ || '');
    } else if (sortColumn === 'reason') {
      comp = (a.reason || '').localeCompare(b.reason || '');
    } else if (sortColumn === 'object') {
      comp = (a.involved_object?.name || '').localeCompare(b.involved_object?.name || '');
    }
    return sortDirection === 'asc' ? comp : -comp;
  });

  let liveStream: boolean = true;
  let pollInterval: any = null;
  let lastEventCount = 0;
  let alertNotice: string | null = null;

  async function fetchEvents() {
    if (!currentContext) return;
    try {
      loading = true;
      const data: any[] = await invoke('kuboard_get_cluster_events', {
        namespace: namespace === 'all' ? null : namespace
      });
      
      const warnings = data.filter(e => (e.type_ || '').toLowerCase() === 'warning');
      if (lastEventCount > 0 && warnings.length > lastEventCount) {
        alertNotice = `🚨 New Warning Event: ${warnings[0]?.reason || 'Anomaly Detected'}`;
        setTimeout(() => alertNotice = null, 4000);
      }
      lastEventCount = warnings.length;
      events = data;
      error = null;
    } catch (e: any) {
      console.error('Failed to fetch cluster events:', e);
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function toggleLiveStream() {
    liveStream = !liveStream;
    if (liveStream) {
      pollInterval = setInterval(fetchEvents, 4000);
    } else if (pollInterval) {
      clearInterval(pollInterval);
      pollInterval = null;
    }
  }



  $: warningCount = events.filter(e => (e.type_ || '').toLowerCase() === 'warning').length;
  $: normalCount = events.filter(e => (e.type_ || '').toLowerCase() === 'normal').length;

  onMount(() => {
    fetchEvents();
    pollInterval = setInterval(fetchEvents, 4000);
    return () => { if (pollInterval) clearInterval(pollInterval); };
  });

  $: if (currentContext || namespace) {
    fetchEvents();
  }
</script>

<div class="events-panel">
  <!-- Top Anomaly Bar -->
  <div class="anomaly-summary-bar">
    <div class="stat-pill">
      <span class="lbl"><Activity size={13} /> Total Events</span>
      <span class="val">{events.length}</span>
    </div>
    <div class="stat-pill warn">
      <span class="lbl"><AlertTriangle size={13} /> Warnings / Anomalies</span>
      <span class="val">{warningCount}</span>
    </div>
    <div class="stat-pill ok">
      <span class="lbl"><CheckCircle2 size={13} /> Normal Events</span>
      <span class="val">{normalCount}</span>
    </div>
    {#if alertNotice}
      <div class="event-toast-alert">{alertNotice}</div>
    {/if}
  </div>

  <div class="panel-header">
    <div class="header-left">
      <h4><Activity size={17} /> Live Cluster Events Stream ({filteredEvents.length})</h4>
      <div class="type-filter-group">
        <button 
          class="filter-btn" 
          class:active={typeFilter === 'all'} 
          onclick={() => typeFilter = 'all'}
        >All ({events.length})</button>
        <button 
          class="filter-btn warning" 
          class:active={typeFilter === 'warning'} 
          onclick={() => typeFilter = 'warning'}
        ><AlertTriangle size={13} /> Warnings ({warningCount})</button>
        <button 
          class="filter-btn normal" 
          class:active={typeFilter === 'normal'} 
          onclick={() => typeFilter = 'normal'}
        ><CheckCircle2 size={13} /> Normal ({normalCount})</button>
      </div>
    </div>
    <div class="header-actions">
      <button class="stream-btn {liveStream ? 'active' : ''}" onclick={toggleLiveStream}>
        {#if liveStream}
          <Radio size={14} class="spin-slow" /> Live Stream (ON)
        {:else}
          <Pause size={14} /> Stream Paused
        {/if}
      </button>
      <button class="refresh-btn" onclick={fetchEvents} disabled={loading}>
        <RefreshCw size={14} class={loading ? 'spin' : ''} /> Refresh
      </button>
    </div>
  </div>

  <ResourceTable
    items={events}
    filteredItems={sortedEvents}
    bind:searchQuery
    searchPlaceholder="Search cluster events by reason, object, message, or namespace..."
    noItemsMessage="No events recorded in this cluster/namespace."
    noSearchResultsMessage="No events match your search:"
  >
    <svelte:fragment slot="header">
      <tr>
        <th class="sortable" onclick={() => handleSort('type')}>
          Type {sortColumn === 'type' ? (sortDirection === 'asc' ? '▲' : '▼') : ''}
        </th>
        <th class="sortable" onclick={() => handleSort('reason')}>
          Reason {sortColumn === 'reason' ? (sortDirection === 'asc' ? '▲' : '▼') : ''}
        </th>
        <th class="sortable" onclick={() => handleSort('object')}>
          Object {sortColumn === 'object' ? (sortDirection === 'asc' ? '▲' : '▼') : ''}
        </th>
        <th>Namespace</th>
        <th>Message</th>
        <th>Count</th>
        <th class="sortable" onclick={() => handleSort('timestamp')}>
          Age {sortColumn === 'timestamp' ? (sortDirection === 'asc' ? '▲' : '▼') : ''}
        </th>
      </tr>
    </svelte:fragment>

    <svelte:fragment slot="rows">
      {#each sortedEvents as evt}
        {@const isWarning = (evt.type_ || '').toLowerCase() === 'warning'}
        <tr class:warning-row={isWarning}>
          <td>
            <span class="type-badge {isWarning ? 'warning' : 'normal'}">
              {evt.type_ || 'Normal'}
            </span>
          </td>
          <td class="reason-cell">{evt.reason}</td>
          <td class="object-cell">
            <span class="kind-tag">{evt.involved_object?.kind}</span> {evt.involved_object?.name}
          </td>
          <td>{evt.involved_object?.namespace || 'default'}</td>
          <td class="message-cell" title={evt.message}>{evt.message}</td>
          <td>{evt.count || 1}</td>
          <td>{formatAge(evt.last_timestamp || evt.first_timestamp)}</td>
        </tr>
      {/each}
    </svelte:fragment>
  </ResourceTable>
</div>

<style>
  .events-panel {
    padding: 12px;
  }
  .anomaly-summary-bar {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 14px;
    background: rgba(0, 0, 0, 0.25);
    padding: 10px 14px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-primary);
  }
  .stat-pill { display: flex; flex-direction: column; gap: 2px; }
  .stat-pill .lbl { font-size: 0.72rem; color: var(--text-muted); text-transform: uppercase; font-weight: 600; }
  .stat-pill .val { font-size: 1rem; color: var(--text-primary); font-weight: 700; }
  .stat-pill.warn .val { color: #fbbf24; }
  .stat-pill.ok .val { color: #4ade80; }
  .event-toast-alert {
    margin-left: auto;
    background: rgba(239, 68, 68, 0.2);
    border: 1px solid #ef4444;
    color: #f87171;
    padding: 4px 12px;
    border-radius: var(--radius-sm);
    font-weight: 700;
    font-size: 0.85rem;
    animation: fadeIn 0.2s ease-in;
  }
  .header-actions { display: flex; align-items: center; gap: 8px; }
  .stream-btn {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-primary);
    color: var(--text-secondary);
    font-size: 0.8rem;
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    cursor: pointer;
  }
  .stream-btn.active {
    background: rgba(239, 68, 68, 0.15);
    color: #f87171;
    border-color: rgba(239, 68, 68, 0.4);
  }
  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }
  .header-left {
    display: flex;
    align-items: center;
    gap: 16px;
  }
  .panel-header h4 {
    margin: 0;
    color: white;
  }
  .type-filter-group {
    display: flex;
    gap: 6px;
  }
  .filter-btn {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-primary);
    color: var(--text-secondary);
    font-size: 0.8rem;
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all 0.2s;
  }
  .filter-btn:hover {
    color: white;
    background: rgba(255, 255, 255, 0.1);
  }
  .filter-btn.active {
    background: var(--primary-color);
    color: white;
    font-weight: 600;
  }
  .filter-btn.warning.active {
    background: #d97706;
  }
  .filter-btn.normal.active {
    background: #059669;
  }
  .refresh-btn {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-sm);
    color: white;
    padding: 4px 12px;
    cursor: pointer;
  }
  .type-badge {
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
  }
  .type-badge.normal {
    background: rgba(16, 185, 129, 0.15);
    color: #34d399;
    border: 1px solid rgba(16, 185, 129, 0.3);
  }
  .type-badge.warning {
    background: rgba(245, 158, 11, 0.15);
    color: #fbbf24;
    border: 1px solid rgba(245, 158, 11, 0.3);
  }
  .warning-row {
    background: rgba(245, 158, 11, 0.03);
  }
  .reason-cell {
    font-weight: 600;
    color: white;
  }
  .object-cell {
    font-family: monospace;
    font-size: 0.85rem;
  }
  .kind-tag {
    color: var(--text-muted);
    font-size: 0.75rem;
    background: rgba(255, 255, 255, 0.08);
    padding: 1px 4px;
    border-radius: 3px;
    margin-right: 4px;
  }
  .message-cell {
    max-width: 450px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-secondary);
  }
</style>
