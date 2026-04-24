<script lang="ts">
  export let events: any[] = [];
  export let loading: boolean = false;
  export let error: string | null = null;
  export let onRetry: () => void;

  function formatEventTime(timestamp: string): string {
    if (!timestamp) return 'Unknown';
    try { return new Date(timestamp).toLocaleString(); } catch { return timestamp; }
  }

  function getEventTypeClass(type: string): string {
    switch (type?.toLowerCase()) {
      case 'normal': return 'event-normal';
      case 'warning': return 'event-warning';
      case 'error': return 'event-error';
      default: return 'event-unknown';
    }
  }

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
</script>

{#if loading}
  <div class="events-loading"><div class="loading-spinner">⏳</div><p>Loading events...</p></div>
{:else if error}
  <div class="events-error"><div class="error-icon">⚠️</div><p>Failed to load events: {error}</p><button class="retry-button" onclick={onRetry}>Retry</button></div>
{:else if events && events.length > 0}
  <div class="events-list">
    {#each events as event}
      <div class="event-item">
        <div class="event-header">
          <div class="event-type-badge event-{getEventTypeClass(event.type)}">{event.type}</div>
          <div class="event-reason reason-{getEventReasonClass(event.reason)}">{event.reason}</div>
          <div class="event-time">{formatEventTime(event.firstTimestamp || event.eventTime)}</div>
        </div>
        <div class="event-message">{event.message}</div>
      </div>
    {/each}
  </div>
{:else}
  <div class="events-placeholder"><p>No events found for this pod</p></div>
{/if}

<style>
  .events-loading, .events-error, .events-placeholder {
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--text-secondary);
  }

  .events-list {
    display: grid;
    gap: 8px;
    max-width: 100%;
  }

  .event-item {
    padding: 8px 10px;
    border: 1px solid var(--border-primary);
    border-radius: 8px;
    background: rgba(255,255,255,0.02);
    max-width: 100%;
  }

  .event-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
    flex-wrap: wrap;
  }

  .event-type-badge {
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    flex-shrink: 0;
  }

  .event-normal { background: rgba(34, 197, 94, 0.12); color: #22c55e; }
  .event-warning { background: rgba(245, 158, 11, 0.12); color: #f59e0b; }
  .event-error { background: rgba(239, 68, 68, 0.12); color: #ef4444; }
  .event-unknown { background: rgba(156, 163, 175, 0.12); color: #9ca3af; }

  .event-reason {
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 10px;
    font-weight: 600;
    flex-shrink: 0;
  }

  .reason-created, .reason-scheduled, .reason-pulled, .reason-started { background: rgba(34, 197, 94, 0.12); color: #22c55e; }
  .reason-pulling, .reason-killing, .reason-backoff { background: rgba(245, 158, 11, 0.12); color: #f59e0b; }
  .reason-failed, .reason-killed, .reason-unhealthy { background: rgba(239, 68, 68, 0.12); color: #ef4444; }
  .reason-unknown { background: rgba(156, 163, 175, 0.12); color: #9ca3af; }

  .event-time {
    color: var(--text-muted);
    font-size: 11px;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
    margin-left: auto;
    flex-shrink: 0;
  }

  .event-message {
    color: var(--text-primary);
    font-size: 12px;
    word-break: break-word;
    overflow-wrap: break-word;
    line-height: 1.4;
  }

  .retry-button {
    padding: 4px 8px;
    background: var(--primary-color);
    border: 1px solid var(--primary-color);
    color: white;
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
    font-weight: 600;
  }

  .retry-button:hover {
    background: var(--accent-color);
  }
</style>
