<!-- Kuboard RoleBinding / ClusterRoleBinding Details View -->
<script lang="ts">
  import { ArrowLeft, Link2, ShieldCheck, ShieldAlert, FileText, User, Users, Box, Tag, ExternalLink } from 'lucide-svelte';
  import { navigationStore } from '../stores/nav';
  import YamlEditor from './YamlEditor.svelte';

  export let binding: any;
  export let isClusterScoped: boolean = false;
  export let onBack: () => void = () => {};

  let showYamlEditor: boolean = false;

  function formatAge(timestamp: string): string {
    if (!timestamp) return '-';
    const date = new Date(timestamp);
    const now = new Date();
    const diff = Math.floor((now.getTime() - date.getTime()) / 1000);
    if (diff < 60) return `${diff}s`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h`;
    return `${Math.floor(diff / 86400)}d`;
  }

  function navigateToTargetRole(kind: string, name: string) {
    if (!name) return;
    const isClusterRole = kind === 'ClusterRole';
    navigationStore.set({
      tab: 'security',
      resourceName: name
    });
  }
</script>

<div class="binding-details-container">
  <!-- Top Navigation Toolbar -->
  <div class="details-top-bar">
    <div class="left-actions">
      <button class="back-btn" onclick={onBack}>
        <ArrowLeft size={16} class="inline-icon" /> Back to {isClusterScoped ? 'Cluster Role Bindings' : 'Role Bindings'}
      </button>
      <div class="resource-title-badge">
        <Link2 size={18} class="inline-icon text-primary" />
        <span class="res-kind">{isClusterScoped ? 'ClusterRoleBinding' : 'RoleBinding'}</span>
        <span class="res-name">{binding?.metadata?.name || 'Unknown'}</span>
      </div>
    </div>

    <div class="right-actions">
      <button class="action-btn-direct primary" onclick={() => showYamlEditor = true}>
        <FileText size={14} class="inline-icon" /> Edit YAML
      </button>
    </div>
  </div>

  <!-- Specs Summary Strip -->
  <div class="specs-summary-strip">
    <div class="spec-cell">
      <span class="spec-label">Namespace</span>
      <span class="spec-val">{binding?.metadata?.namespace || (isClusterScoped ? 'Cluster-Scoped' : 'default')}</span>
    </div>
    <div class="spec-cell">
      <span class="spec-label">Bound Role</span>
      <span class="spec-val font-bold">
        {binding?.roleRef?.kind || 'Role'} / {binding?.roleRef?.name || '-'}
      </span>
    </div>
    <div class="spec-cell">
      <span class="spec-label">Subjects Count</span>
      <span class="spec-val font-bold">{binding?.subjects?.length || 0} bound subjects</span>
    </div>
    <div class="spec-cell">
      <span class="spec-label">Age</span>
      <span class="spec-val">{formatAge(binding?.metadata?.creationTimestamp)}</span>
    </div>
  </div>

  <!-- Bound Role Ref Section -->
  <div class="sheet-section">
    <div class="section-header">
      <h5><ShieldCheck size={16} class="inline-icon" /> Target Role Reference</h5>
    </div>

    <div class="role-ref-card">
      <div class="ref-info">
        <span class="ref-kind-badge">{binding?.roleRef?.kind || 'Role'}</span>
        <span class="ref-name">{binding?.roleRef?.name || '-'}</span>
        <span class="ref-group">API Group: {binding?.roleRef?.apiGroup || 'rbac.authorization.k8s.io'}</span>
      </div>
      <button 
        class="btn-inspect-role" 
        onclick={() => navigateToTargetRole(binding?.roleRef?.kind, binding?.roleRef?.name)}
      >
        <ExternalLink size={14} class="inline-icon" /> Inspect Role Details
      </button>
    </div>
  </div>

  <!-- Subjects Table Section -->
  <div class="sheet-section">
    <div class="section-header">
      <h5><Users size={16} class="inline-icon" /> Bound Subjects (Users, Groups & ServiceAccounts)</h5>
    </div>

    {#if binding?.subjects && binding.subjects.length > 0}
      <div class="table-container">
        <table class="subjects-table">
          <thead>
            <tr>
              <th>Subject Kind</th>
              <th>Name</th>
              <th>Namespace</th>
              <th>API Group</th>
            </tr>
          </thead>
          <tbody>
            {#each binding.subjects as subject}
              <tr>
                <td class="kind-cell">
                  <span class="kind-pill kind-{subject.kind?.toLowerCase()}">
                    {#if subject.kind === 'ServiceAccount'}
                      <Box size={12} class="inline-icon" />
                    {:else if subject.kind === 'User'}
                      <User size={12} class="inline-icon" />
                    {:else}
                      <Users size={12} class="inline-icon" />
                    {/if}
                    {subject.kind}
                  </span>
                </td>
                <td class="name-cell font-bold">{subject.name}</td>
                <td>{subject.namespace || binding?.metadata?.namespace || '-'}</td>
                <td class="font-mono text-muted">{subject.apiGroup || 'core'}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {:else}
      <div class="empty-state">
        <p class="muted-text">No subjects bound to this {isClusterScoped ? 'ClusterRoleBinding' : 'RoleBinding'}.</p>
      </div>
    {/if}
  </div>

  <!-- Labels & Annotations -->
  {#if binding?.metadata?.labels || binding?.metadata?.annotations}
    <div class="sheet-section">
      <div class="section-header">
        <h5><Tag size={16} class="inline-icon" /> Metadata & Labels</h5>
      </div>
      <div class="metadata-grid">
        {#if binding?.metadata?.labels}
          <div class="meta-block">
            <span class="meta-title">Labels</span>
            <div class="pill-cloud">
              {#each Object.entries(binding.metadata.labels) as [k, v]}
                <span class="label-pill"><strong>{k}:</strong> {v}</span>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

{#if showYamlEditor}
  <YamlEditor 
    resource={binding} 
    resourceType={isClusterScoped ? "clusterrolebinding" : "rolebinding"} 
    onSave={() => showYamlEditor = false} 
    onCancel={() => showYamlEditor = false} 
  />
{/if}

<style>
  .binding-details-container {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 16px;
    color: var(--text-primary);
  }

  .details-top-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: var(--background-secondary);
    padding: 10px 16px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-primary);
  }

  .left-actions {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .back-btn {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--border-primary);
    color: var(--text-primary);
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }

  .back-btn:hover {
    background: rgba(255, 255, 255, 0.12);
  }

  .resource-title-badge {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .res-kind {
    background: rgba(16, 185, 129, 0.2);
    color: #34d399;
    font-size: 11px;
    font-weight: 800;
    padding: 2px 8px;
    border-radius: 10px;
    text-transform: uppercase;
  }

  .res-name {
    font-size: 16px;
    font-weight: 700;
  }

  .action-btn-direct {
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid var(--border-primary);
    color: var(--text-primary);
    padding: 6px 14px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }
  .action-btn-direct.primary {
    background: var(--primary-color);
    color: white;
    border: none;
  }

  .specs-summary-strip {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 12px;
    background: rgba(0, 0, 0, 0.2);
    padding: 12px 16px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-primary);
  }

  .spec-cell {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .spec-label {
    font-size: 11px;
    color: var(--text-muted);
    font-weight: 600;
    text-transform: uppercase;
  }

  .spec-val {
    font-size: 13px;
    color: var(--text-primary);
  }

  .sheet-section {
    background: var(--background-secondary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    padding: 16px;
  }

  .section-header h5 {
    margin: 0 0 12px 0;
    font-size: 14px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .role-ref-card {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid rgba(255, 255, 255, 0.08);
    padding: 12px 16px;
    border-radius: var(--radius-sm);
  }

  .ref-info {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .ref-kind-badge {
    background: rgba(59, 130, 246, 0.2);
    color: #60a5fa;
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 700;
  }

  .ref-name {
    font-size: 14px;
    font-weight: 700;
  }

  .ref-group {
    font-size: 11px;
    color: var(--text-muted);
  }

  .btn-inspect-role {
    background: rgba(59, 130, 246, 0.15);
    border: 1px solid rgba(59, 130, 246, 0.3);
    color: #60a5fa;
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
  }

  .subjects-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
  }

  .subjects-table th, .subjects-table td {
    padding: 10px 12px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    text-align: left;
  }

  .subjects-table th {
    color: var(--text-muted);
    font-weight: 700;
    text-transform: uppercase;
    font-size: 11px;
  }

  .kind-pill {
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 700;
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .kind-serviceaccount {
    background: rgba(16, 185, 129, 0.2);
    color: #34d399;
  }

  .kind-user {
    background: rgba(59, 130, 246, 0.2);
    color: #60a5fa;
  }

  .kind-group {
    background: rgba(245, 158, 11, 0.2);
    color: #fbbf24;
  }

  .pill-cloud {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 6px;
  }

  .label-pill {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--border-primary);
    padding: 3px 8px;
    border-radius: 4px;
    font-size: 11px;
  }
</style>
