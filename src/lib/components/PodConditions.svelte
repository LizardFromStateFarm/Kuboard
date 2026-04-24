<script lang="ts">
  export let conditions: any[] = [];

  function getConditionStatusClass(status: string): string {
    switch (status?.toLowerCase()) {
      case 'true': return 'condition-true';
      case 'false': return 'condition-false';
      case 'unknown': return 'condition-unknown';
      default: return 'condition-unknown';
    }
  }
</script>

{#if conditions && conditions.length > 0}
  <div class="conditions-list">
    {#each conditions as c}
      <div class="condition-item">
        <span class="condition-type">{c.type}</span>
        <span class="condition-status {getConditionStatusClass(c.status)}">{c.status}</span>
        <span class="condition-reason">{c.reason}</span>
        <span class="condition-message">{c.message}</span>
      </div>
    {/each}
  </div>
{:else}
  <div class="events-placeholder"><p>No conditions available</p></div>
{/if}

<style>
  .conditions-list { 
    display: grid; 
    gap: 8px; 
    max-width: 100%;
  }
  .condition-item { 
    display: grid; 
    grid-template-columns: 160px 90px 1fr; 
    align-items: start; 
    gap: 8px; 
    padding: 8px 10px; 
    border: 1px solid var(--border-primary); 
    border-radius: 8px; 
    background: rgba(255,255,255,0.03);
    min-height: 40px;
  }
  .condition-type { 
    font-weight: 700; 
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .condition-status { 
    padding: 2px 8px; 
    border-radius: 999px; 
    font-size: 12px; 
    font-weight: 700; 
    text-transform: uppercase; 
    text-align: center;
    flex-shrink: 0;
  }
  .condition-true { background: rgba(34, 197, 94, 0.12); color: #22c55e; border: 1px solid rgba(34, 197, 94, .24); }
  .condition-false { background: rgba(239, 68, 68, 0.12); color: #ef4444; border: 1px solid rgba(239, 68, 68, .24); }
  .condition-unknown { background: rgba(156, 163, 175, 0.12); color: #9ca3af; border: 1px solid rgba(156, 163, 175, .24); }
  .condition-reason { 
    color: var(--text-secondary); 
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .condition-message { 
    grid-column: 1 / -1; 
    color: var(--text-muted);
    word-break: break-word;
    overflow-wrap: break-word;
    margin-top: 4px;
  }
  .events-placeholder { 
    padding: 24px; 
    text-align: center; 
    color: var(--text-muted); 
    background: rgba(0,0,0,0.1); 
    border-radius: 8px; 
    border: 1px dashed var(--border-primary);
    margin-top: 8px;
  }
</style>
