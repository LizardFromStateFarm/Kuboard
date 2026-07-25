<!-- Kuboard Ingress Details Component -->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { Lock, Globe } from 'lucide-svelte';

  export let ingress: any;
  export let onBack: () => void = () => {};

  const dispatch = createEventDispatcher();
  let copyNotice: string | null = null;

  $: rules = ingress?.spec?.rules || [];
  $: tls = ingress?.spec?.tls || [];
  $: ingressClass = ingress?.spec?.ingressClassName || '-';
  $: loadBalancerIPs = ingress?.status?.loadBalancer?.ingress || [];

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      copyNotice = '✓ Copied!';
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

<div class="ingress-details-container">
  <div class="details-top-bar">
    <button class="btn-back" onclick={() => { if (onBack) onBack(); dispatch('back'); }}>← Back</button>
    <div class="top-title">
      <span class="resource-icon">🌐</span>
      <h3>{ingress?.metadata?.name}</h3>
      <span class="namespace-pill">{ingress?.metadata?.namespace || 'default'}</span>
      <span class="class-pill">Class: {ingressClass}</span>
    </div>
    {#if copyNotice}
      <span class="copy-notice">{copyNotice}</span>
    {/if}
  </div>

  <div class="details-sheet">
    <div class="specs-row">
      <div class="spec-card">
        <span class="label">Name</span>
        <span class="val clickable" onclick={() => copyToClipboard(ingress?.metadata?.name)}>{ingress?.metadata?.name}</span>
      </div>
      <div class="spec-card">
        <span class="label">Namespace</span>
        <span class="val">{ingress?.metadata?.namespace || 'default'}</span>
      </div>
      <div class="spec-card">
        <span class="label">Ingress Class</span>
        <span class="val">{ingressClass}</span>
      </div>
      <div class="spec-card">
        <span class="label">Load Balancer Endpoints</span>
        <span class="val">{loadBalancerIPs.map(i => i.ip || i.hostname).join(', ') || '-'}</span>
      </div>
      <div class="spec-card">
        <span class="label">Age</span>
        <span class="val">{formatAge(ingress?.metadata?.creationTimestamp)}</span>
      </div>
    </div>

    <!-- TLS Certificates -->
    <div class="section-card">
      <h4><Lock size={16} /> TLS Certificates & Secret References ({tls.length})</h4>
      <div class="tls-list">
        {#each tls as t}
          <div class="tls-box">
            <div class="tls-info">
              <span class="label">Secret Name:</span>
              <code>{t.secretName || '-'}</code>
            </div>
            <div class="tls-info">
              <span class="label">Hosts:</span>
              <div class="badge-wrap">
                {#each (t.hosts || ['*']) as host}
                  <span class="host-badge">{host}</span>
                {/each}
              </div>
            </div>
          </div>
        {/each}

        {#if tls.length === 0}
          <div class="empty-state">No TLS termination configured (HTTP only).</div>
        {/if}
      </div>
    </div>

    <!-- Routing Rules & Backend Services -->
    <div class="section-card">
      <h4><Globe size={16} /> Host Rules & Path Routing ({rules.length})</h4>
      <div class="rules-list">
        {#each rules as r}
          <div class="rule-box">
            <div class="rule-header">
              <span class="host-name">🌐 <strong>Host:</strong> <code>{r.host || '*'}</code></span>
            </div>

            <div class="paths-table">
              {#each (r.http?.paths || []) as p}
                <div class="path-row">
                  <div class="path-info">
                    <span class="label">Path:</span>
                    <code>{p.path || '/'}</code>
                    <span class="path-type">({p.pathType || 'ImplementationSpecific'})</span>
                  </div>
                  <div class="backend-info">
                    <span class="label">Backend Service:</span>
                    <span class="svc-badge">🔌 {p.backend?.service?.name || p.backend?.serviceName || '-'}</span>
                    <span class="port-badge">Port {p.backend?.service?.port?.number || p.backend?.servicePort || '-'}</span>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/each}

        {#if rules.length === 0}
          <div class="empty-state">No explicit HTTP host rules defined.</div>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .ingress-details-container { display: flex; flex-direction: column; gap: 16px; }
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
  .namespace-pill, .class-pill {
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
  .tls-list, .rules-list { display: flex; flex-direction: column; gap: 10px; }
  .tls-box, .rule-box {
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    padding: 12px 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .tls-info { display: flex; align-items: center; gap: 8px; }
  .tls-info code { color: #4ade80; font-weight: 700; }
  .badge-wrap { display: flex; gap: 6px; }
  .host-badge { background: rgba(59, 130, 246, 0.15); color: #60a5fa; padding: 2px 8px; border-radius: 4px; font-size: 0.85rem; font-family: monospace; }
  .rule-header { display: flex; align-items: center; gap: 8px; border-bottom: 1px solid rgba(255, 255, 255, 0.05); padding-bottom: 8px; }
  .host-name code { color: #60a5fa; font-size: 1rem; }
  .paths-table { display: flex; flex-direction: column; gap: 8px; margin-top: 4px; }
  .path-row { display: flex; justify-content: space-between; align-items: center; background: rgba(0, 0, 0, 0.25); padding: 8px 12px; border-radius: 4px; }
  .path-info { display: flex; align-items: center; gap: 8px; }
  .path-info code { color: #4ade80; font-weight: 700; }
  .path-type { font-size: 0.75rem; color: var(--text-muted); }
  .backend-info { display: flex; align-items: center; gap: 8px; }
  .svc-badge { background: rgba(168, 85, 247, 0.15); color: #c084fc; padding: 2px 8px; border-radius: 4px; font-size: 0.85rem; font-weight: 600; }
  .port-badge { background: rgba(255, 255, 255, 0.08); color: var(--text-secondary); padding: 2px 6px; border-radius: 4px; font-size: 0.8rem; }
  .empty-state { text-align: center; color: var(--text-muted); padding: 16px; }
</style>
