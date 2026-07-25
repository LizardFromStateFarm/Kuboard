<!-- Kuboard Cluster Health & Diagnostic Report Generator Modal -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { createEventDispatcher } from 'svelte';
  import { FileText, Copy, Download } from 'lucide-svelte';

  export let currentContext: any;
  export let isOpen = false;
  export let onClose = () => {};

  const dispatch = createEventDispatcher();

  let generating = false;
  let reportMarkdown = '';
  let copyNotice: string | null = null;

  async function generateClusterReport() {
    if (!currentContext) return;
    generating = true;
    try {
      const [nodes, pods, events, pvcs]: [any[], any[], any[], any[]] = await Promise.all([
        invoke('kuboard_get_nodes').catch(() => []),
        invoke('kuboard_get_pods').catch(() => []),
        invoke('kuboard_get_cluster_events', { namespace: null }).catch(() => []),
        invoke('kuboard_list_persistent_volume_claims', { namespace: 'all' }).catch(() => [])
      ]);

      const nowStr = new Date().toISOString();
      const ctxName = currentContext.name || 'default';
      const warningEvents = events.filter((e: any) => (e.type_ || '').toLowerCase() === 'warning');
      const failingPods = pods.filter((p: any) => {
        const ph = (p.status?.phase || '').toLowerCase();
        return ph !== 'running' && ph !== 'succeeded';
      });

      let md = `# 🛡️ Kuboard Cluster Diagnostic & Health Report\n`;
      md += `**Generated At:** ${nowStr}\n`;
      md += `**Target Context:** \`${ctxName}\`\n\n`;
      md += `---\n\n`;

      md += `### 📊 1. Executive Summary\n`;
      md += `- **Nodes Count:** ${nodes.length}\n`;
      md += `- **Total Pods:** ${pods.length}\n`;
      md += `- **Unhealthy / Crashing Pods:** ${failingPods.length}\n`;
      md += `- **Active Warning Events:** ${warningEvents.length}\n`;
      md += `- **Persistent Volume Claims:** ${pvcs.length}\n\n`;

      md += `### 🖥️ 2. Node Status Breakdown (${nodes.length})\n`;
      md += `| Node Name | Status | OS / Kernel | Kubelet Version |\n`;
      md += `|---|---|---|---|\n`;
      nodes.forEach((n: any) => {
        const name = n.metadata?.name || '-';
        const readyCond = n.status?.conditions?.find((c: any) => c.type === 'Ready')?.status;
        const status = readyCond === 'True' ? '🟢 Ready' : '🔴 NotReady';
        const os = n.status?.nodeInfo?.operatingSystem || '-';
        const ver = n.status?.nodeInfo?.kubeletVersion || '-';
        md += `| \`${name}\` | ${status} | ${os} | ${ver} |\n`;
      });
      md += `\n`;

      md += `### 🚨 3. Unhealthy / Non-Running Pods (${failingPods.length})\n`;
      if (failingPods.length === 0) {
        md += `*All pods are currently in Running or Succeeded state. No anomalies detected.*\n\n`;
      } else {
        md += `| Pod Name | Namespace | Node | Phase |\n`;
        md += `|---|---|---|---|\n`;
        failingPods.forEach((p: any) => {
          md += `| \`${p.metadata?.name}\` | \`${p.metadata?.namespace}\` | ${p.spec?.nodeName || '-'} | **${p.status?.phase || 'Unknown'}** |\n`;
        });
        md += `\n`;
      }

      md += `### ⚡ 4. Recent Cluster Warning Events (${warningEvents.length})\n`;
      if (warningEvents.length === 0) {
        md += `*No warning events found in cluster audit log.*\n\n`;
      } else {
        md += `| Reason | Target Object | Namespace | Message |\n`;
        md += `|---|---|---|---|\n`;
        warningEvents.slice(0, 15).forEach((e: any) => {
          md += `| **${e.reason}** | \`${e.involved_object?.kind}/${e.involved_object?.name}\` | \`${e.involved_object?.namespace || 'default'}\` | ${e.message} |\n`;
        });
        md += `\n`;
      }

      md += `### 💾 5. Persistent Volume Claims (${pvcs.length})\n`;
      if (pvcs.length === 0) {
        md += `*No Persistent Volume Claims bound in context.*\n\n`;
      } else {
        md += `| PVC Name | Namespace | Status | Storage Class | Capacity |\n`;
        md += `|---|---|---|---|---|\n`;
        pvcs.forEach((pvc: any) => {
          const cap = pvc.status?.capacity?.storage || pvc.spec?.resources?.requests?.storage || '-';
          md += `| \`${pvc.metadata?.name}\` | \`${pvc.metadata?.namespace}\` | ${pvc.status?.phase || '-'} | ${pvc.spec?.storageClassName || '-'} | ${cap} |\n`;
        });
        md += `\n`;
      }

      md += `---\n`;
      md += `*Report generated automatically by Kuboard desktop cluster manager.*\n`;

      reportMarkdown = md;
    } catch (err: any) {
      console.error('Failed to generate cluster report:', err);
      reportMarkdown = `Error generating report: ${err}`;
    } finally {
      generating = false;
    }
  }

  async function copyReport() {
    try {
      await navigator.clipboard.writeText(reportMarkdown);
      copyNotice = '✓ Report Copied!';
      setTimeout(() => copyNotice = null, 2000);
    } catch (e) {
      console.error(e);
    }
  }

  function downloadMarkdown() {
    const blob = new Blob([reportMarkdown], { type: 'text/markdown' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `kuboard-health-report-${currentContext?.name || 'cluster'}.md`;
    a.click();
    URL.revokeObjectURL(url);
  }

  $: if (isOpen && currentContext) {
    generateClusterReport();
  }
</script>

{#if isOpen}
  <div class="modal-backdrop" onclick={onClose}>
    <div class="modal-card" onclick={(e) => e.stopPropagation()}>
      <div class="modal-header">
        <div class="header-title">
          <FileText size={18} class="text-primary" />
          <h4>Cluster Health & Diagnostic Report</h4>
        </div>
        <button class="btn-close" onclick={onClose}>✕</button>
      </div>

      <div class="modal-body">
        {#if generating}
          <div class="generating-state">
            <div class="spinner"></div>
            <p>Gathering node metrics, crashing pods, warnings, and PVC capacity...</p>
          </div>
        {:else}
          <div class="toolbar">
            <button class="btn-action" onclick={copyReport}><Copy size={14} /> Copy Markdown</button>
            <button class="btn-action primary" onclick={downloadMarkdown}><Download size={14} /> Export .MD Report</button>
            {#if copyNotice}
              <span class="copy-toast">{copyNotice}</span>
            {/if}
          </div>
          <textarea class="report-preview" readonly value={reportMarkdown}></textarea>
        {/if}
      </div>

      <div class="modal-footer">
        <button class="btn-close-modal" onclick={onClose}>Close</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed; top: 0; left: 0; right: 0; bottom: 0;
    background: rgba(0, 0, 0, 0.75); backdrop-filter: blur(4px);
    display: flex; align-items: center; justify-content: center; z-index: 11000; padding: 20px;
  }
  .modal-card {
    background: var(--background-secondary); border: 1px solid var(--border-primary);
    border-radius: var(--radius-lg); width: 720px; max-width: 92vw; max-height: 85vh;
    display: flex; flex-direction: column; overflow: hidden;
  }
  .modal-header {
    display: flex; justify-content: space-between; align-items: center;
    padding: 14px 18px; border-bottom: 1px solid var(--border-primary);
  }
  .header-title { display: flex; align-items: center; gap: 10px; }
  .header-title h4 { margin: 0; color: var(--text-primary); font-size: 1.1rem; }
  .btn-close { background: transparent; border: none; color: var(--text-muted); font-size: 1.2rem; cursor: pointer; }
  .modal-body { padding: 18px; display: flex; flex-direction: column; gap: 14px; overflow: hidden; flex: 1; }
  .toolbar { display: flex; align-items: center; gap: 10px; }
  .btn-action {
    background: rgba(255, 255, 255, 0.08); border: 1px solid var(--border-primary);
    color: var(--text-primary); padding: 6px 14px; border-radius: var(--radius-sm);
    font-weight: 600; font-size: 0.85rem; cursor: pointer;
  }
  .btn-action.primary { background: var(--primary-color); color: white; border: none; }
  .copy-toast { color: #4ade80; font-weight: 700; font-size: 0.85rem; margin-left: auto; }
  .generating-state { display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 60px; color: var(--text-secondary); }
  .spinner { width: 32px; height: 32px; border: 3px solid rgba(255, 255, 255, 0.1); border-top-color: var(--primary-color); border-radius: 50%; animation: spin 1s linear infinite; margin-bottom: 14px; }
  @keyframes spin { to { transform: rotate(360deg); } }
  .report-preview {
    flex: 1; min-height: 350px; background: #0d0d14; color: #a7f3d0; border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm); padding: 14px; font-family: monospace; font-size: 0.82rem;
    line-height: 1.45; resize: none; outline: none;
  }
  .modal-footer { display: flex; justify-content: flex-end; padding: 12px 18px; border-top: 1px solid var(--border-primary); }
  .btn-close-modal { background: rgba(255, 255, 255, 0.1); color: white; border: none; padding: 6px 16px; border-radius: var(--radius-sm); font-weight: 600; cursor: pointer; }
</style>
