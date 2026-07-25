<script lang="ts">
  export let volumes: any[] = [];
</script>

{#if volumes && volumes.length > 0}
  <div class="volumes-list">
    {#each volumes as v}
      <div class="volume-item">
        <span class="volume-name">{v.name}</span>
        <span class="volume-source">{Object.keys(v).find(k => k !== 'name')}</span>
      </div>
    {/each}
  </div>
{:else}
  <div class="events-placeholder"><p>No volumes</p></div>
{/if}

<style>
  .volumes-list { 
    display: grid; 
    gap: 8px; 
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); 
  }
  .volume-item { 
    display: flex; 
    justify-content: space-between; 
    align-items: center; 
    gap: 10px; 
    padding: 8px 10px; 
    border: 1px solid var(--border-primary); 
    border-radius: 8px; 
    background: rgba(255,255,255,0.03); 
  }
  .volume-name { 
    font-weight: 700; 
    color: var(--text-primary); 
  }
  .volume-source { 
    color: var(--text-secondary); 
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace; 
    font-size: 12px; 
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
