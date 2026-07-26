<!-- Kuboard ConfigMap Details Component -->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { FileText, ArrowLeft, Copy, FileEdit, Trash2, Save, AlertTriangle, Check } from 'lucide-svelte';

  export let configMap: any;
  export let onBack: () => void = () => {};

  const dispatch = createEventDispatcher();

  let editingKey: string | null = null;
  let editValue: string = '';
  let isAddingKey = false;
  let newKeyName = '';
  let newKeyValue = '';
  let isSaving = false;
  let saveError: string | null = null;
  let saveSuccess: string | null = null;
  let copyFeedback: string | null = null;

  $: dataMap = configMap?.data || {};

  async function copyToClipboard(text: string, label: string) {
    try {
      await navigator.clipboard.writeText(text);
      copyFeedback = `Copied ${label}!`;
      setTimeout(() => copyFeedback = null, 1500);
    } catch (e) {
      console.error('Failed to copy:', e);
    }
  }

  function startEdit(key: string, val: string) {
    editingKey = key;
    editValue = val;
  }

  function cancelEdit() {
    editingKey = null;
    editValue = '';
  }

  async function saveConfigMapUpdate(updatedDataMap: Record<string, string>) {
    try {
      isSaving = true;
      saveError = null;
      saveSuccess = null;

      const updatedCm = {
        apiVersion: configMap.apiVersion || 'v1',
        kind: configMap.kind || 'ConfigMap',
        metadata: {
          name: configMap.metadata?.name,
          namespace: configMap.metadata?.namespace,
          labels: configMap.metadata?.labels,
          annotations: configMap.metadata?.annotations
        },
        data: updatedDataMap
      };

      const yamlStr = JSON.stringify(updatedCm);
      await invoke('kuboard_apply_resource_yaml', { yamlContent: yamlStr });
      
      configMap.data = updatedDataMap;
      configMap = { ...configMap };
      
      saveSuccess = 'ConfigMap updated successfully!';
      editingKey = null;
      isAddingKey = false;
      newKeyName = '';
      newKeyValue = '';
      setTimeout(() => saveSuccess = null, 2500);
    } catch (err: any) {
      console.error('Failed to save ConfigMap:', err);
      saveError = String(err);
    } finally {
      isSaving = false;
    }
  }

  function handleSaveEditKey(keyToSave: string) {
    const nextData = { ...dataMap };
    nextData[keyToSave] = editValue;
    saveConfigMapUpdate(nextData);
  }

  function handleDeleteKey(keyToDelete: string) {
    if (!confirm(`Are you sure you want to delete ConfigMap key "${keyToDelete}"?`)) return;
    const nextData = { ...dataMap };
    delete nextData[keyToDelete];
    saveConfigMapUpdate(nextData);
  }

  function handleAddNewKey() {
    if (!newKeyName.trim()) return;
    const nextData = { ...dataMap };
    nextData[newKeyName.trim()] = newKeyValue;
    saveConfigMapUpdate(nextData);
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

<div class="cm-details-container">
  <div class="details-top-bar">
    <button class="btn-back" onclick={() => { if (onBack) onBack(); dispatch('back'); }}><ArrowLeft size={14} class="inline-icon" /> Back</button>
    <div class="top-title">
      <span class="resource-icon"><FileText size={18} /></span>
      <h3>{configMap?.metadata?.name}</h3>
      <span class="namespace-pill">{configMap?.metadata?.namespace || 'default'}</span>
    </div>
    {#if copyFeedback}
      <span class="copy-notice"><Check size={14} class="inline-icon" /> {copyFeedback}</span>
    {/if}
  </div>

  {#if saveError}
    <div class="alert-box error"><AlertTriangle size={15} class="inline-icon" /> {saveError}</div>
  {/if}

  {#if saveSuccess}
    <div class="alert-box success"><Check size={15} class="inline-icon" /> {saveSuccess}</div>
  {/if}

  <div class="details-sheet">
    <div class="specs-row">
      <div class="spec-card">
        <span class="label">Name</span>
        <span class="val clickable" onclick={() => copyToClipboard(configMap?.metadata?.name, 'Name')}>{configMap?.metadata?.name}</span>
      </div>
      <div class="spec-card">
        <span class="label">Namespace</span>
        <span class="val">{configMap?.metadata?.namespace || 'default'}</span>
      </div>
      <div class="spec-card">
        <span class="label">Data Keys</span>
        <span class="val">{Object.keys(dataMap).length} keys</span>
      </div>
      <div class="spec-card">
        <span class="label">Age</span>
        <span class="val">{formatAge(configMap?.metadata?.creationTimestamp)}</span>
      </div>
    </div>

    <!-- Data Keys & Payloads -->
    <div class="section-card">
      <div class="card-header-bar">
        <h4><FileText size={16} /> Config Data Payloads ({Object.keys(dataMap).length})</h4>
        <button class="btn-primary-sm" onclick={() => isAddingKey = !isAddingKey}>
          {isAddingKey ? 'Cancel' : '+ Add Config Key'}
        </button>
      </div>

      {#if isAddingKey}
        <div class="add-key-box">
          <h5>Add New Config Key</h5>
          <div class="input-group">
            <input type="text" placeholder="Key Name (e.g. app.config)" bind:value={newKeyName} />
            <textarea placeholder="Config value payload..." bind:value={newKeyValue} rows="4"></textarea>
          </div>
          <div class="box-actions">
            <button class="btn-save" onclick={handleAddNewKey} disabled={isSaving || !newKeyName.trim()}>
              {isSaving ? 'Saving...' : 'Save Key'}
            </button>
            <button class="btn-cancel" onclick={() => isAddingKey = false}>Cancel</button>
          </div>
        </div>
      {/if}

      <div class="keys-list">
        {#each Object.entries(dataMap) as [key, val]}
          <div class="key-card">
            <div class="key-header">
              <span class="key-title"><code>{key}</code></span>
              <div class="key-actions">
                <button class="btn-sm" onclick={() => copyToClipboard(String(val), 'Value')}><Copy size={13} class="inline-icon" /> Copy Value</button>
                <button class="btn-sm" onclick={() => startEdit(key, String(val))}><FileEdit size={13} class="inline-icon" /> Edit</button>
                <button class="btn-sm danger" onclick={() => handleDeleteKey(key)} title="Delete Key"><Trash2 size={13} /></button>
              </div>
            </div>

            {#if editingKey === key}
              <div class="inline-editor">
                <textarea bind:value={editValue} rows="6"></textarea>
                <div class="editor-actions">
                  <button class="btn-save" onclick={() => handleSaveEditKey(key)} disabled={isSaving}>
                    <Save size={13} class="inline-icon" /> {isSaving ? 'Saving...' : 'Save'}
                  </button>
                  <button class="btn-cancel" onclick={cancelEdit}>Cancel</button>
                </div>
              </div>
            {:else}
              <pre class="payload-preview">{val}</pre>
            {/if}
          </div>
        {/each}

        {#if Object.keys(dataMap).length === 0}
          <div class="empty-state">No config data stored in this ConfigMap.</div>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .cm-details-container { display: flex; flex-direction: column; gap: 16px; }
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
  .alert-box { padding: 10px 14px; border-radius: var(--radius-sm); font-size: 0.9rem; }
  .alert-box.error { background: rgba(239, 68, 68, 0.15); color: #f87171; border: 1px solid rgba(239, 68, 68, 0.3); }
  .alert-box.success { background: rgba(34, 197, 94, 0.15); color: #4ade80; border: 1px solid rgba(34, 197, 94, 0.3); }
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
  .card-header-bar { display: flex; justify-content: space-between; align-items: center; }
  .card-header-bar h4 { margin: 0; color: var(--text-primary); }
  .btn-primary-sm {
    background: var(--primary-color);
    border: none;
    color: white;
    padding: 5px 12px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-weight: 600;
  }
  .add-key-box {
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .add-key-box h5 { margin: 0; color: var(--text-primary); }
  .input-group { display: flex; flex-direction: column; gap: 8px; }
  .input-group input, .input-group textarea {
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--border-primary);
    color: white;
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    font-family: monospace;
  }
  .box-actions { display: flex; gap: 8px; }
  .keys-list { display: flex; flex-direction: column; gap: 12px; }
  .key-card {
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .key-header { display: flex; justify-content: space-between; align-items: center; }
  .key-title code { color: #60a5fa; font-weight: 700; font-size: 1rem; }
  .key-actions { display: flex; gap: 6px; }
  .payload-preview {
    margin: 0;
    font-family: monospace;
    color: #4ade80;
    background: rgba(0, 0, 0, 0.4);
    padding: 10px;
    border-radius: 4px;
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-all;
    max-height: 250px;
  }
  .inline-editor { display: flex; flex-direction: column; gap: 8px; }
  .inline-editor textarea {
    background: rgba(0, 0, 0, 0.35);
    border: 1px solid var(--border-primary);
    color: #4ade80;
    font-family: monospace;
    padding: 10px;
    border-radius: 4px;
  }
  .editor-actions { display: flex; gap: 8px; }
  .btn-save { background: #22c55e; border: none; color: white; padding: 5px 12px; border-radius: var(--radius-sm); cursor: pointer; font-weight: 600; }
  .btn-cancel { background: rgba(255, 255, 255, 0.1); border: 1px solid var(--border-primary); color: white; padding: 5px 12px; border-radius: var(--radius-sm); cursor: pointer; }
  .btn-sm { background: rgba(255, 255, 255, 0.08); border: 1px solid var(--border-primary); color: white; padding: 4px 10px; border-radius: var(--radius-sm); cursor: pointer; font-size: 0.8rem; }
  .btn-sm.danger { color: #f87171; }
  .empty-state { text-align: center; color: var(--text-muted); padding: 16px; }
</style>
