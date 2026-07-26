<!-- Kuboard Cluster Linter Component -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { Stethoscope, RefreshCw, Play, AlertTriangle, Sparkles, CheckCircle2 } from 'lucide-svelte';

  // Props
  export let currentContext: any = null;

  // State
  let report: { findings: any[], health_score: number } | null = null;
  let loading = false;
  let error: string | null = null;
  let lastRun = '';

  async function runLinter() {
    if (!currentContext || loading) return;
    
    loading = true;
    error = null;
    
    try {
      report = await invoke('kuboard_run_linter', { namespace: null });
      lastRun = new Date().toLocaleTimeString();
    } catch (err: any) {
      error = err;
      console.error('Failed to run linter:', err);
    } finally {
      loading = false;
    }
  }

  function getSeverityClass(severity: string) {
    switch (severity.toLowerCase()) {
      case 'critical': return 'sev-critical';
      case 'warning': return 'sev-warning';
      default: return 'sev-info';
    }
  }

  function getScoreColor(score: number) {
    if (score >= 90) return '#10b981'; // Green
    if (score >= 70) return '#f59e0b'; // Amber
    return '#ef4444'; // Red
  }

  onMount(() => {
    runLinter();
  });

  $: if (currentContext) {
    runLinter();
  }
</script>

<div class="linter-tab">
  <div class="tab-header">
    <div class="header-left">
      <h4><Stethoscope size={18} /> Cluster Health (Popeye-style)</h4>
      {#if report}
        <div class="health-gauge">
          <span class="score" style="color: {getScoreColor(report.health_score)}">{report.health_score}%</span>
          <span class="label">Health Score</span>
        </div>
      {/if}
    </div>
    <div class="tab-controls">
      <button 
        class="run-button" 
        onclick={runLinter}
        disabled={loading}
      >
        {#if loading}
          <RefreshCw size={15} class="spin" /> Analyzing...
        {:else}
          <Play size={15} /> Run Analysis
        {/if}
      </button>
      {#if lastRun}
        <span class="last-update">Last run: {lastRun}</span>
      {/if}
    </div>
  </div>

  {#if error}
    <div class="error-banner">
      <span class="error-icon"><AlertTriangle size={18} /></span>
      <p>{error}</p>
    </div>
  {:else if loading && !report}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Performing deep cluster analysis...</p>
      <small>Checking pods, services, and deployments for misconfigurations.</small>
    </div>
  {:else if report}
    <div class="report-content">
      <div class="findings-summary">
        <div class="summary-card critical">
          <span class="count">{report.findings.filter(f => f.severity === 'Critical').length}</span>
          <span class="label">Critical Issues</span>
        </div>
        <div class="summary-card warning">
          <span class="count">{report.findings.filter(f => f.severity === 'Warning').length}</span>
          <span class="label">Warnings</span>
        </div>
        <div class="summary-card info">
          <span class="count">{report.findings.filter(f => f.severity === 'Info').length}</span>
          <span class="label">Optimizations</span>
        </div>
      </div>

      <div class="findings-list">
        {#if report.findings.length === 0}
          <div class="clean-state">
            <div class="clean-icon"><Sparkles size={36} /></div>
            <h5>Cluster is Healthy!</h5>
            <p>No issues found in the current namespace analysis.</p>
          </div>
        {:else}
          <table>
            <thead>
              <tr>
                <th>Severity</th>
                <th>Resource</th>
                <th>Issue</th>
                <th>Code</th>
              </tr>
            </thead>
            <tbody>
              {#each report.findings as finding}
                <tr>
                  <td>
                    <span class="sev-badge {getSeverityClass(finding.severity)}">
                      {finding.severity}
                    </span>
                  </td>
                  <td>
                    <div class="resource-cell">
                      <span class="kind">{finding.resource_kind}</span>
                      <span class="name">{finding.resource_name}</span>
                    </div>
                  </td>
                  <td class="message">{finding.message}</td>
                  <td class="code">{finding.code}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  @import '../styles/variables.css';

  .linter-tab {
    padding: 0;
  }

  .tab-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-xl);
    padding-bottom: var(--spacing-md);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: var(--spacing-xl);
  }

  .tab-header h4 {
    margin: 0;
    color: white;
    font-size: 1.2rem;
  }

  .health-gauge {
    display: flex;
    flex-direction: column;
    align-items: center;
    background: rgba(255, 255, 255, 0.05);
    padding: 4px 16px;
    border-radius: var(--radius-md);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .health-gauge .score {
    font-size: 1.4rem;
    font-weight: 800;
  }

  .health-gauge .label {
    font-size: 0.7rem;
    text-transform: uppercase;
    color: var(--text-muted);
    letter-spacing: 0.05em;
  }

  .tab-controls {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .run-button {
    background: var(--primary-color);
    border: none;
    border-radius: var(--radius-md);
    color: white;
    padding: 8px 20px;
    font-weight: 600;
    cursor: pointer;
    transition: var(--transition-normal);
  }

  .run-button:hover:not(:disabled) {
    background: var(--accent-color);
    transform: translateY(-1px);
    box-shadow: var(--shadow-lg);
  }

  .last-update {
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .report-content {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xl);
  }

  .findings-summary {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--spacing-lg);
  }

  .summary-card {
    background: rgba(255, 255, 255, 0.03);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
    display: flex;
    flex-direction: column;
    align-items: center;
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .summary-card.critical { border-left: 4px solid #ef4444; }
  .summary-card.warning { border-left: 4px solid #f59e0b; }
  .summary-card.info { border-left: 4px solid #3b82f6; }

  .summary-card .count {
    font-size: 2rem;
    font-weight: 800;
    color: white;
  }

  .summary-card .label {
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .findings-list {
    background: var(--card-background);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th {
    text-align: left;
    padding: var(--spacing-md);
    background: rgba(255, 255, 255, 0.05);
    font-size: 0.8rem;
    text-transform: uppercase;
    color: var(--text-muted);
  }

  td {
    padding: var(--spacing-md);
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    font-size: 0.9rem;
  }

  .sev-badge {
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 0.7rem;
    font-weight: 700;
    text-transform: uppercase;
  }

  .sev-critical { background: rgba(239, 68, 68, 0.1); color: #ef4444; border: 1px solid rgba(239, 68, 68, 0.3); }
  .sev-warning { background: rgba(245, 158, 11, 0.1); color: #f59e0b; border: 1px solid rgba(245, 158, 11, 0.3); }
  .sev-info { background: rgba(59, 130, 246, 0.1); color: #3b82f6; border: 1px solid rgba(59, 130, 246, 0.3); }

  .resource-cell {
    display: flex;
    flex-direction: column;
  }

  .resource-cell .kind {
    font-size: 0.7rem;
    color: var(--text-muted);
    text-transform: uppercase;
  }

  .resource-cell .name {
    font-weight: 600;
    color: white;
  }

  .message {
    color: var(--text-primary);
  }

  .code {
    font-family: monospace;
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .loading-state, .clean-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-xxl);
    text-align: center;
    color: var(--text-secondary);
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid rgba(255, 255, 255, 0.1);
    border-top-color: var(--primary-color);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: var(--spacing-lg);
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .clean-icon {
    font-size: 3rem;
    margin-bottom: var(--spacing-lg);
  }
</style>
