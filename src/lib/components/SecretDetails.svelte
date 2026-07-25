<!-- Kuboard Secret Details Component -->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  export let secret: any;
  export let onBack: () => void = () => {};

  const dispatch = createEventDispatcher();

  let showDecoded: Record<string, boolean> = {};
  let editingKey: string | null = null;
  let editValue: string = '';
  let editMode: 'plain' | 'base64' = 'plain';
  let isAddingKey = false;
  let newKeyName = '';
  let newKeyValue = '';
  let isSaving = false;
  let saveError: string | null = null;
  let saveSuccess: string | null = null;
  let copyFeedback: string | null = null;

  $: secretData = secret?.data || secret?.stringData || {};

  function toggleDecode(key: string) {
    showDecoded[key] = !showDecoded[key];
    showDecoded = { ...showDecoded };
  }

  function decodeBase64(str: string): string {
    try {
      return atob(str);
    } catch {
      return str;
    }
  }

  function encodeBase64(str: string): string {
    try {
      return btoa(str);
    } catch {
      return str;
    }
  }

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
    editMode = 'plain';
    editValue = typeof val === 'string' ? decodeBase64(val) : JSON.stringify(val);
  }

  function cancelEdit() {
    editingKey = null;
    editValue = '';
  }

  async function saveSecretUpdate(updatedDataMap: Record<string, string>) {
    try {
      isSaving = true;
      saveError = null;
      saveSuccess = null;

      // Construct payload manifest
      const updatedSecret = {
        apiVersion: secret.apiVersion || 'v1',
        kind: secret.kind || 'Secret',
        metadata: {
          name: secret.metadata?.name,
          namespace: secret.metadata?.namespace,
          labels: secret.metadata?.labels,
          annotations: secret.metadata?.annotations
        },
        type: secret.type || 'Opaque',
        data: updatedDataMap
      };

      const yamlStr = JSON.stringify(updatedSecret);
      await invoke('kuboard_apply_resource_yaml', { yamlContent: yamlStr });
      
      // Update local object
      secret.data = updatedDataMap;
      secret = { ...secret };
      
      saveSuccess = 'Secret updated successfully!';
      editingKey = null;
      isAddingKey = false;
      newKeyName = '';
      newKeyValue = '';
      setTimeout(() => saveSuccess = null, 2500);
    } catch (err: any) {
      console.error('Failed to save Secret:', err);
      saveError = String(err);
    } finally {
      isSaving = false;
    }
  }

  function handleSaveEditKey(keyToSave: string) {
    const nextData = { ...secretData };
    const finalB64 = editMode === 'plain' ? encodeBase64(editValue) : editValue;
    nextData[keyToSave] = finalB64;
    saveSecretUpdate(nextData);
  }

  function handleDeleteKey(keyToDelete: string) {
    if (!confirm(`Are you sure you want to delete secret key "${keyToDelete}"?`)) return;
    const nextData = { ...secretData };
    delete nextData[keyToDelete];
    saveSecretUpdate(nextData);
  }

  function handleAddNewKey() {
    if (!newKeyName.trim()) return;
    const nextData = { ...secretData };
    nextData[newKeyName.trim()] = encodeBase64(newKeyValue);
    saveSecretUpdate(nextData);
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

<div class="secret-details-container">
  <!-- Top Navigation & Action Header -->
  <div class="details-top-bar">
    <button class="btn-back" onclick={() => { if (onBack) onBack(); dispatch('back'); }}>← Back</button>
    <div class="top-title">
      <span class="resource-icon">🔒</span>
      <h3>{secret?.metadata?.name}</h3>
      <span class="namespace-pill">{secret?.metadata?.namespace || 'default'}</span>
      <span class="type-pill">{secret?.type || 'Opaque'}</span>
    </div>
    {#if copyFeedback}
      <span class="copy-notice">✓ {copyFeedback}</span>
    {/if}
  </div>

  {#if saveError}
    <div class="alert-box error">⚠️ {saveError}</div>
  {/if}

  {#if saveSuccess}
    <div class="alert-box success">✓ {saveSuccess}</div>
  {/if}

  <!-- Master Secret Specs Sheet -->
  <div class="details-sheet">
    <div class="specs-row">
      <div class="spec-card">
        <span class="label">Name</span>
        <span class="val clickable" onclick={() => copyToClipboard(secret?.metadata?.name, 'Name')}>{secret?.metadata?.name}</span>
      </div>
      <div class="spec-card">
        <span class="label">Namespace</span>
        <span class="val">{secret?.metadata?.namespace || 'default'}</span>
      </div>
      <div class="spec-card">
        <span class="label">Type</span>
        <span class="val">{secret?.type || 'Opaque'}</span>
      </div>
      <div class="spec-card">
        <span class="label">Data Keys</span>
        <span class="val">{Object.keys(secretData).length} keys</span>
      </div>
      <div class="spec-card">
        <span class="label">Age</span>
        <span class="val">{formatAge(secret?.metadata?.creationTimestamp)}</span>
      </div>
    </div>

    <!-- Interactive Data Key Values Editor -->
    <div class="section-card">
      <div class="card-header-bar">
        <h4>🔒 Secret Data Entries & Values ({Object.keys(secretData).length})</h4>
        <button class="btn-primary-sm" onclick={() => isAddingKey = !isAddingKey}>
          {isAddingKey ? 'Cancel' : '+ Add Secret Key'}
        </button>
      </div>

      {#if isAddingKey}
        <div class="add-key-box">
          <h5>Add New Secret Key</h5>
          <div class="input-group">
            <input type="text" placeholder="Key Name (e.g. DB_PASSWORD)" bind:value={newKeyName} />
            <textarea placeholder="Plain text value..." bind:value={newKeyValue}></textarea>
          </div>
          <div class="box-actions">
            <button class="btn-save" onclick={handleAddNewKey} disabled={isSaving || !newKeyName.trim()}>
              {isSaving ? 'Saving...' : 'Save New Key'}
            </button>
            <button class="btn-cancel" onclick={() => isAddingKey = false}>Cancel</button>
          </div>
        </div>
      {/if}

      <div class="keys-table">
        {#each Object.entries(secretData) as [key, val]}
          <div class="key-row">
            <div class="key-info">
              <span class="key-name"><code>{key}</code></span>
              <button class="btn-icon" onclick={() => copyToClipboard(key, 'Key Name')} title="Copy Key Name">📋</button>
            </div>

            <div class="val-container">
              {#if editingKey === key}
                <div class="inline-editor">
                  <div class="mode-toggle">
                    <button class={editMode === 'plain' ? 'active' : ''} onclick={() => editMode = 'plain'}>Plaintext</button>
                    <button class={editMode === 'base64' ? 'active' : ''} onclick={() => editMode = 'base64'}>Base64</button>
                  </div>
                  <textarea bind:value={editValue} rows="3"></textarea>
                  <div class="editor-actions">
                    <button class="btn-save" onclick={() => handleSaveEditKey(key)} disabled={isSaving}>
                      {isSaving ? 'Saving...' : '💾 Save'}
                    </button>
                    <button class="btn-cancel" onclick={cancelEdit}>Cancel</button>
                  </div>
                </div>
              {:else}
                <div class="value-display">
                  {#if showDecoded[key]}
                    <pre class="decoded-text">{typeof val === 'string' ? decodeBase64(val) : JSON.stringify(val)}</pre>
                  {:else}
                    <span class="masked-text">••••••••••••••••••••••••</span>
                  {/if}
                </div>

                <div class="row-actions">
                  <button class="btn-sm" onclick={() => toggleDecode(key)}>
                    {showDecoded[key] ? '👁️ Hide' : '🔓 Reveal'}
                  </button>
                  {#if showDecoded[key]}
                    <button class="btn-sm" onclick={() => copyToClipboard(decodeBase64(val), 'Decoded Value')}>
                      📋 Copy Plain
                    </button>
                  {/if}
                  <button class="btn-sm" onclick={() => startEdit(key, val)}>✏️ Edit</button>
                  <button class="btn-sm danger" onclick={() => handleDeleteKey(key)}>🗑️</button>
                </div>
              {/if}
            </div>
          </div>
        {/each}

        {#if Object.keys(secretData).length === 0}
          <div class="empty-state">No data keys stored in this secret.</div>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .secret-details-container {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
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
  .top-title {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .top-title h3 {
    margin: 0;
    color: var(--text-primary);
    font-size: 1.2rem;
  }
  .namespace-pill, .type-pill {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-secondary);
    padding: 2px 10px;
    border-radius: 12px;
    font-size: 0.8rem;
  }
  .copy-notice {
    color: #4ade80;
    font-weight: 700;
    font-size: 0.85rem;
    margin-left: auto;
  }
  .alert-box {
    padding: 10px 14px;
    border-radius: var(--radius-sm);
    font-size: 0.9rem;
  }
  .alert-box.error { background: rgba(239, 68, 68, 0.15); color: #f87171; border: 1px solid rgba(239, 68, 68, 0.3); }
  .alert-box.success { background: rgba(34, 197, 94, 0.15); color: #4ade80; border: 1px solid rgba(34, 197, 94, 0.3); }
  .details-sheet {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .specs-row {
    display: flex;
    gap: 12px;
    background: var(--background-secondary);
    padding: 14px 18px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-primary);
  }
  .spec-card {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 1;
  }
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
  .card-header-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
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
  .keys-table { display: flex; flex-direction: column; gap: 10px; }
  .key-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    padding: 12px 16px;
    gap: 16px;
  }
  .key-info { display: flex; align-items: center; gap: 8px; min-width: 180px; }
  .key-name code { color: #60a5fa; font-weight: 700; font-size: 0.95rem; }
  .btn-icon { background: none; border: none; cursor: pointer; font-size: 0.85rem; }
  .val-container { flex: 1; display: flex; align-items: center; justify-content: space-between; gap: 12px; }
  .masked-text { color: var(--text-muted); font-family: monospace; }
  .decoded-text {
    margin: 0;
    font-family: monospace;
    color: #4ade80;
    background: rgba(0, 0, 0, 0.4);
    padding: 6px 10px;
    border-radius: 4px;
    max-width: 500px;
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-all;
  }
  .row-actions { display: flex; gap: 6px; }
  .btn-sm {
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid var(--border-primary);
    color: white;
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: 0.8rem;
  }
  .btn-sm.danger { color: #f87171; }
  .btn-sm:hover { background: rgba(255, 255, 255, 0.15); }
  .inline-editor { width: 100%; display: flex; flex-direction: column; gap: 8px; }
  .mode-toggle { display: flex; gap: 6px; }
  .mode-toggle button {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-primary);
    color: var(--text-muted);
    padding: 3px 8px;
    border-radius: 4px;
    font-size: 0.75rem;
    cursor: pointer;
  }
  .mode-toggle button.active { background: var(--primary-color); color: white; border-color: var(--primary-color); }
  .inline-editor textarea {
    background: rgba(0, 0, 0, 0.35);
    border: 1px solid var(--border-primary);
    color: #4ade80;
    font-family: monospace;
    padding: 8px;
    border-radius: 4px;
  }
  .editor-actions { display: flex; gap: 8px; }
  .btn-save {
    background: #22c55e;
    border: none;
    color: white;
    padding: 5px 12px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-weight: 600;
  }
  .btn-cancel {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid var(--border-primary);
    color: white;
    padding: 5px 12px;
    border-radius: var(--radius-sm);
    cursor: pointer;
  }
  .empty-state { text-align: center; color: var(--text-muted); padding: 20px; }
</style>
