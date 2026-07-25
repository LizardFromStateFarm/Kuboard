<!-- Kuboard RBAC Role Details Component -->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let role: any;
  export let onBack: () => void = () => {};

  const dispatch = createEventDispatcher();
  let copyNotice: string | null = null;

  $: rules = role?.rules || [];

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      copyNotice = '✓ Copied!';
      setTimeout(() => copyNotice = null, 1500);
    } catch (e) {
      console.error(e);
    }
  }

  function getVerbClass(verb: string): string {
    const v = verb.toLowerCase();
    if (['get', 'list', 'watch'].includes(v)) return 'verb-read';
    if (['create', 'update', 'patch'].includes(v)) return 'verb-write';
    if (['delete', 'deletecollection', '*'].includes(v)) return 'verb-delete';
    return 'verb-default';
  }

  function formatAge(timestamp: string): string {
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
</script>

<div class="role-details-container">
  <div class="details-top-bar">
    <button class="btn-back" onclick={() => { if (onBack) onBack(); dispatch('back'); }}>← Back</button>
    <div class="top-title">
      <span class="resource-icon">🛡️</span>
      <h3>{role?.metadata?.name}</h3>
      <span class="kind-pill">{role?.kind || 'Role'}</span>
      {#if role?.metadata?.namespace}
        <span class="namespace-pill">{role.metadata.namespace}</span>
      {/if}
    </div>
    {#if copyNotice}
      <span class="copy-notice">{copyNotice}</span>
    {/if}
  </div>

  <div class="details-sheet">
    <div class="specs-row">
      <div class="spec-card">
        <span class="label">Name</span>
        <span class="val clickable" onclick={() => copyToClipboard(role?.metadata?.name)}>{role?.metadata?.name}</span>
      </div>
      <div class="spec-card">
        <span class="label">Namespace</span>
        <span class="val">{role?.metadata?.namespace || 'Cluster-wide'}</span>
      </div>
      <div class="spec-card">
        <span class="label">Rule Count</span>
        <span class="val">{rules.length} rules</span>
      </div>
      <div class="spec-card">
        <span class="label">Age</span>
        <span class="val">{formatAge(role?.metadata?.creationTimestamp)}</span>
      </div>
    </div>

    <!-- Rules Table -->
    <div class="section-card">
      <h4>🛡️ RBAC Permissions & Policy Rules ({rules.length})</h4>
      <div class="rules-list">
        {#each rules as r, i}
          <div class="rule-box">
            <div class="rule-header">
              <span class="rule-index">Rule #{i + 1}</span>
              <div class="api-groups">
                <span class="sub-label">API Groups:</span>
                {#each (r.apiGroups || ['""']) as group}
                  <code class="api-badge">{group === '' ? 'core ("")' : group}</code>
                {/each}
              </div>
            </div>

            <div class="rule-body">
              <div class="resources-col">
                <span class="sub-label">Resources:</span>
                <div class="badge-wrap">
                  {#each (r.resources || ['*']) as res}
                    <span class="res-badge">📄 {res}</span>
                  {/each}
                </div>
                {#if r.resourceNames && r.resourceNames.length > 0}
                  <span class="sub-label">Resource Names:</span>
                  <div class="badge-wrap">
                    {#each r.resourceNames as name}
                      <span class="res-name-badge">🏷️ {name}</span>
                    {/each}
                  </div>
                {/if}
              </div>

              <div class="verbs-col">
                <span class="sub-label">Allowed Verbs:</span>
                <div class="badge-wrap">
                  {#each (r.verbs || ['*']) as verb}
                    <span class="verb-badge {getVerbClass(verb)}">{verb}</span>
                  {/each}
                </div>
              </div>
            </div>
          </div>
        {/each}

        {#if rules.length === 0}
          <div class="empty-state">No explicit policy rules defined.</div>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .role-details-container { display: flex; flex-direction: column; gap: 16px; }
  .details-top-bar {
    display: flex;
    align-items: center;
    gap: 12px;
    background: var(--background-secondary);
    padding: 10px 16px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-primary);
  }
  .btn-back {
    background: var(--primary-color);
    border: none;
    color: white;
    padding: 6px 14px;
    border-radius: var(--radius-sm);
    font-weight: 600;
    cursor: pointer;
  }
  .top-title { display: flex; align-items: center; gap: 10px; }
  .top-title h3 { margin: 0; color: var(--text-primary); font-size: 1.2rem; }
  .kind-pill, .namespace-pill {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-secondary);
    padding: 2px 10px;
    border-radius: 12px;
    font-size: 0.8rem;
  }
  .copy-notice { color: #4ade80; font-weight: 700; font-size: 0.85rem; margin-left: auto; }
  .details-sheet { display: flex; flex-direction: column; gap: 16px; }
  .specs-row {
    display: flex;
    gap: 12px;
    background: var(--background-secondary);
    padding: 14px 18px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-primary);
  }
  .spec-card { display: flex; flex-direction: column; gap: 4px; flex: 1; }
  .spec-card .label { font-size: 0.75rem; color: var(--text-muted); text-transform: uppercase; }
  .spec-card .val { font-size: 0.95rem; color: var(--text-primary); font-weight: 600; }
  .val.clickable { cursor: pointer; color: #60a5fa; }
  .val.clickable:hover { text-decoration: underline; }
  .section-card {
    background: var(--background-secondary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    padding: 18px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .section-card h4 { margin: 0; color: var(--text-primary); }
  .rules-list { display: flex; flex-direction: column; gap: 12px; }
  .rule-box {
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .rule-header { display: flex; justify-content: space-between; align-items: center; }
  .rule-index { font-weight: 700; color: var(--primary-color); font-size: 0.85rem; }
  .api-groups { display: flex; align-items: center; gap: 6px; }
  .api-badge { background: rgba(255, 255, 255, 0.08); padding: 2px 6px; border-radius: 4px; color: #9ca3af; }
  .sub-label { font-size: 0.75rem; color: var(--text-muted); text-transform: uppercase; margin-right: 4px; }
  .rule-body { display: flex; justify-content: space-between; gap: 20px; }
  .resources-col, .verbs-col { flex: 1; display: flex; flex-direction: column; gap: 6px; }
  .badge-wrap { display: flex; flex-wrap: wrap; gap: 6px; }
  .res-badge { background: rgba(59, 130, 246, 0.12); color: #60a5fa; padding: 3px 8px; border-radius: 4px; font-size: 0.85rem; font-family: monospace; }
  .res-name-badge { background: rgba(168, 85, 247, 0.12); color: #c084fc; padding: 3px 8px; border-radius: 4px; font-size: 0.85rem; font-family: monospace; }
  .verb-badge { padding: 3px 8px; border-radius: 4px; font-size: 0.75rem; font-weight: 700; text-transform: uppercase; }
  .verb-read { background: rgba(34, 197, 94, 0.15); color: #4ade80; }
  .verb-write { background: rgba(245, 158, 11, 0.15); color: #fbbf24; }
  .verb-delete { background: rgba(239, 68, 68, 0.15); color: #f87171; }
  .verb-default { background: rgba(156, 163, 175, 0.15); color: #9ca3af; }
  .empty-state { text-align: center; color: var(--text-muted); padding: 20px; }
</style>
