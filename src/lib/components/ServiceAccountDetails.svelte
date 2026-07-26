<!-- Kuboard ServiceAccount Details Component -->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { UserCheck, ArrowLeft, Check, Key, Lock, Copy, Package, Shield } from 'lucide-svelte';

  export let sa: any;
  export let onBack: () => void = () => {};

  const dispatch = createEventDispatcher();
  let copyNotice: string | null = null;

  $: secrets = sa?.secrets || [];
  $: imagePullSecrets = sa?.imagePullSecrets || [];

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      copyNotice = 'Copied!';
      setTimeout(() => copyNotice = null, 1500);
    } catch (e) {
      console.error(e);
    }
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

<div class="sa-details-container">
  <div class="details-top-bar">
    <button class="btn-back" onclick={() => { if (onBack) onBack(); dispatch('back'); }}><ArrowLeft size={14} class="inline-icon" /> Back</button>
    <div class="top-title">
      <span class="resource-icon"><UserCheck size={18} /></span>
      <h3>{sa?.metadata?.name}</h3>
      <span class="namespace-pill">{sa?.metadata?.namespace || 'default'}</span>
    </div>
    {#if copyNotice}
      <span class="copy-notice"><Check size={14} class="inline-icon" /> {copyNotice}</span>
    {/if}
  </div>

  <div class="details-sheet">
    <div class="specs-row">
      <div class="spec-card">
        <span class="label">Name</span>
        <span class="val clickable" onclick={() => copyToClipboard(sa?.metadata?.name)}>{sa?.metadata?.name}</span>
      </div>
      <div class="spec-card">
        <span class="label">Namespace</span>
        <span class="val">{sa?.metadata?.namespace || 'default'}</span>
      </div>
      <div class="spec-card">
        <span class="label">Automount Token</span>
        <span class="val">{sa?.automountServiceAccountToken !== false ? 'Enabled (True)' : 'Disabled (False)'}</span>
      </div>
      <div class="spec-card">
        <span class="label">Bound Secrets</span>
        <span class="val">{secrets.length} secrets</span>
      </div>
      <div class="spec-card">
        <span class="label">Age</span>
        <span class="val">{formatAge(sa?.metadata?.creationTimestamp)}</span>
      </div>
    </div>

    <!-- Bound Secrets & Tokens Section -->
    <div class="section-card">
      <h4><Key size={16} /> Bound Tokens & Secrets ({secrets.length})</h4>
      <div class="item-list">
        {#each secrets as sec}
          <div class="item-box">
            <span class="item-icon"><Lock size={14} /></span>
            <span class="item-name"><code>{sec.name}</code></span>
            <button class="btn-sm" onclick={() => copyToClipboard(sec.name)}><Copy size={13} class="inline-icon" /> Copy Secret Name</button>
          </div>
        {/each}

        {#if secrets.length === 0}
          <div class="empty-state">No explicit tokens or secrets attached. (Auto-token projected)</div>
        {/if}
      </div>
    </div>

    <!-- Image Pull Secrets Section -->
    <div class="section-card">
      <h4><Package size={16} /> Image Pull Secrets ({imagePullSecrets.length})</h4>
      <div class="item-list">
        {#each imagePullSecrets as ips}
          <div class="item-box">
            <span class="item-icon"><Shield size={14} /></span>
            <span class="item-name"><code>{ips.name}</code></span>
            <button class="btn-sm" onclick={() => copyToClipboard(ips.name)}><Copy size={13} class="inline-icon" /> Copy Secret Name</button>
          </div>
        {/each}

        {#if imagePullSecrets.length === 0}
          <div class="empty-state">No image pull secrets configured for private registry auth.</div>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .sa-details-container { display: flex; flex-direction: column; gap: 16px; }
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
  .namespace-pill {
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
  .item-list { display: flex; flex-direction: column; gap: 8px; }
  .item-box {
    display: flex;
    align-items: center;
    gap: 12px;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    padding: 10px 14px;
  }
  .item-name code { color: #60a5fa; font-weight: 700; font-size: 0.95rem; }
  .btn-sm {
    margin-left: auto;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid var(--border-primary);
    color: white;
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: 0.8rem;
  }
  .empty-state { text-align: center; color: var(--text-muted); padding: 16px; }
</style>
