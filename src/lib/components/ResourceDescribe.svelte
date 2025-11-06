<!-- Resource Describe Component -->
<script lang="ts">
  export let describeData: any = null;
  export let loading: boolean = false;
  export let error: string | null = null;

  let expandedSections: Set<string> = new Set(['status', 'metadata']);

  function toggleSection(section: string) {
    if (expandedSections.has(section)) {
      expandedSections.delete(section);
    } else {
      expandedSections.add(section);
    }
    expandedSections = expandedSections; // Trigger reactivity
  }

  function isExpanded(section: string): boolean {
    return expandedSections.has(section);
  }

  function formatObject(obj: any): string {
    if (obj === null || obj === undefined) return 'None';
    if (typeof obj === 'string') return obj;
    if (typeof obj === 'number' || typeof obj === 'boolean') return String(obj);
    if (Array.isArray(obj)) return obj.join(', ');
    if (typeof obj === 'object') {
      return Object.entries(obj)
        .map(([k, v]) => `${k}=${v}`)
        .join(', ');
    }
    return String(obj);
  }

  function formatTimestamp(timestamp: string): string {
    if (!timestamp || timestamp === 'None') return 'None';
    try {
      const date = new Date(timestamp);
      return date.toLocaleString();
    } catch {
      return timestamp;
    }
  }

  function getConditionClass(status: string): string {
    switch (status?.toLowerCase()) {
      case 'true': return 'condition-true';
      case 'false': return 'condition-false';
      default: return 'condition-unknown';
    }
  }

  function getEventTypeClass(type: string): string {
    switch (type?.toLowerCase()) {
      case 'normal': return 'event-normal';
      case 'warning': return 'event-warning';
      default: return 'event-unknown';
    }
  }
</script>

<div class="resource-describe">
  {#if loading}
    <div class="loading">Loading describe information...</div>
  {:else if error}
    <div class="error">Error: {error}</div>
  {:else if describeData}
    <!-- Name and Namespace -->
    <div class="section-header">
      <h2>Name: {describeData.name}</h2>
      <span class="namespace">Namespace: {describeData.namespace}</span>
    </div>

    <!-- Labels -->
    {#if describeData.labels && Object.keys(describeData.labels).length > 0}
      <div class="section">
        <div class="section-title" onclick={() => toggleSection('labels')}>
          <span class="toggle-icon">{isExpanded('labels') ? '▼' : '▶'}</span>
          <span>Labels</span>
        </div>
        {#if isExpanded('labels')}
          <div class="section-content">
            <div class="key-value-list">
              {#each Object.entries(describeData.labels) as [key, value]}
                <div class="key-value-item">
                  <span class="key">{key}:</span>
                  <span class="value">{value}</span>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/if}

    <!-- Annotations -->
    {#if describeData.annotations && Object.keys(describeData.annotations).length > 0}
      <div class="section">
        <div class="section-title" onclick={() => toggleSection('annotations')}>
          <span class="toggle-icon">{isExpanded('annotations') ? '▼' : '▶'}</span>
          <span>Annotations</span>
        </div>
        {#if isExpanded('annotations')}
          <div class="section-content">
            <div class="key-value-list">
              {#each Object.entries(describeData.annotations) as [key, value]}
                <div class="key-value-item">
                  <span class="key">{key}:</span>
                  <span class="value">{value}</span>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/if}

    <!-- Status -->
    {#if describeData.status}
      <div class="section">
        <div class="section-title" onclick={() => toggleSection('status')}>
          <span class="toggle-icon">{isExpanded('status') ? '▼' : '▶'}</span>
          <span>Status</span>
        </div>
        {#if isExpanded('status')}
          <div class="section-content">
            <div class="key-value-list">
              <div class="key-value-item">
                <span class="key">Phase:</span>
                <span class="value status-{describeData.status.phase?.toLowerCase()}">{describeData.status.phase || 'Unknown'}</span>
              </div>
              <div class="key-value-item">
                <span class="key">Pod IP:</span>
                <span class="value">{describeData.status.podIP || 'None'}</span>
              </div>
              <div class="key-value-item">
                <span class="key">Host IP:</span>
                <span class="value">{describeData.status.hostIP || 'None'}</span>
              </div>
              <div class="key-value-item">
                <span class="key">Node:</span>
                <span class="value">{describeData.status.nodeName || 'None'}</span>
              </div>
              <div class="key-value-item">
                <span class="key">QoS Class:</span>
                <span class="value qos-{describeData.status.qosClass?.toLowerCase()}">{describeData.status.qosClass || 'Unknown'}</span>
              </div>
              <div class="key-value-item">
                <span class="key">Start Time:</span>
                <span class="value">{formatTimestamp(describeData.status.startTime)}</span>
              </div>
            </div>
          </div>
        {/if}
      </div>
    {/if}

    <!-- Conditions -->
    {#if describeData.conditions && describeData.conditions.length > 0}
      <div class="section">
        <div class="section-title" onclick={() => toggleSection('conditions')}>
          <span class="toggle-icon">{isExpanded('conditions') ? '▼' : '▶'}</span>
          <span>Conditions</span>
        </div>
        {#if isExpanded('conditions')}
          <div class="section-content">
            <table class="conditions-table">
              <thead>
                <tr>
                  <th>Type</th>
                  <th>Status</th>
                  <th>Reason</th>
                  <th>Message</th>
                  <th>Last Transition</th>
                </tr>
              </thead>
              <tbody>
                {#each describeData.conditions as condition}
                  <tr>
                    <td>{condition.type}</td>
                    <td class="condition-{getConditionClass(condition.status)}">{condition.status}</td>
                    <td>{condition.reason || 'None'}</td>
                    <td>{condition.message || 'None'}</td>
                    <td>{formatTimestamp(condition.lastTransitionTime)}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>
    {/if}

    <!-- Containers -->
    {#if describeData.containers && describeData.containers.length > 0}
      <div class="section">
        <div class="section-title" onclick={() => toggleSection('containers')}>
          <span class="toggle-icon">{isExpanded('containers') ? '▼' : '▶'}</span>
          <span>Containers ({describeData.containers.length})</span>
        </div>
        {#if isExpanded('containers')}
          <div class="section-content">
            {#each describeData.containers as container, idx}
              <div class="container-section">
                <h3 class="container-name">{container.name}</h3>
                
                <div class="container-details">
                  <div class="key-value-item">
                    <span class="key">Image:</span>
                    <span class="value">{container.image}</span>
                  </div>
                  <div class="key-value-item">
                    <span class="key">Image Pull Policy:</span>
                    <span class="value">{container.imagePullPolicy}</span>
                  </div>
                  
                  {#if container.status}
                    <div class="container-status">
                      <div class="key-value-item">
                        <span class="key">Ready:</span>
                        <span class="value">{container.status.ready ? '✅' : '❌'}</span>
                      </div>
                      <div class="key-value-item">
                        <span class="key">Restart Count:</span>
                        <span class="value">{container.status.restartCount || 0}</span>
                      </div>
                      
                      {#if container.status.state}
                        <div class="container-state">
                          {#if container.status.state.running}
                            <div class="state-badge running">Running</div>
                          {:else if container.status.state.waiting}
                            <div class="state-badge waiting">
                              Waiting: {container.status.state.waiting.reason || 'Unknown'}
                              {#if container.status.state.waiting.message}
                                <span class="state-message">({container.status.state.waiting.message})</span>
                              {/if}
                            </div>
                          {:else if container.status.state.terminated}
                            <div class="state-badge terminated">
                              Terminated: {container.status.state.terminated.reason || 'Unknown'}
                              {#if container.status.state.terminated.exitCode !== undefined}
                                <span class="state-message">(Exit Code: {container.status.state.terminated.exitCode})</span>
                              {/if}
                            </div>
                          {/if}
                        </div>
                      {/if}
                    </div>
                  {/if}
                  
                  {#if container.resources}
                    <div class="container-resources">
                      <h4>Resources:</h4>
                      {#if container.resources.requests && Object.keys(container.resources.requests).length > 0}
                        <div class="key-value-item">
                          <span class="key">Requests:</span>
                          <span class="value">{formatObject(container.resources.requests)}</span>
                        </div>
                      {/if}
                      {#if container.resources.limits && Object.keys(container.resources.limits).length > 0}
                        <div class="key-value-item">
                          <span class="key">Limits:</span>
                          <span class="value">{formatObject(container.resources.limits)}</span>
                        </div>
                      {/if}
                    </div>
                  {/if}
                  
                  {#if container.ports && container.ports.length > 0}
                    <div class="container-ports">
                      <h4>Ports:</h4>
                      <div class="ports-list">
                        {#each container.ports as port}
                          <div class="port-item">
                            {port.name || 'unnamed'}: {port.containerPort}/{port.protocol}
                          </div>
                        {/each}
                      </div>
                    </div>
                  {/if}
                  
                  {#if container.env && container.env.length > 0}
                    <div class="container-env">
                      <h4>Environment Variables:</h4>
                      <div class="env-list">
                        {#each container.env as env}
                          <div class="env-item">
                            <span class="env-name">{env.name}</span>
                            <span class="env-value">= {env.value || (env.valueFrom ? '[from source]' : 'None')}</span>
                          </div>
                        {/each}
                      </div>
                    </div>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}

    <!-- Volumes -->
    {#if describeData.volumes && describeData.volumes.length > 0}
      <div class="section">
        <div class="section-title" onclick={() => toggleSection('volumes')}>
          <span class="toggle-icon">{isExpanded('volumes') ? '▼' : '▶'}</span>
          <span>Volumes ({describeData.volumes.length})</span>
        </div>
        {#if isExpanded('volumes')}
          <div class="section-content">
            <div class="volumes-list">
              {#each describeData.volumes as volume}
                <div class="volume-item">
                  <span class="volume-name">{volume.name}</span>
                  <span class="volume-type">({volume.type})</span>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/if}

    <!-- Tolerations -->
    {#if describeData.tolerations && describeData.tolerations.length > 0}
      <div class="section">
        <div class="section-title" onclick={() => toggleSection('tolerations')}>
          <span class="toggle-icon">{isExpanded('tolerations') ? '▼' : '▶'}</span>
          <span>Tolerations</span>
        </div>
        {#if isExpanded('tolerations')}
          <div class="section-content">
            <div class="tolerations-list">
              {#each describeData.tolerations as toleration}
                <div class="toleration-item">
                  {toleration.key || '*'}={toleration.operator || 'Equal'} {toleration.value || '*'}
                  {#if toleration.effect && toleration.effect !== 'None'}
                    <span class="toleration-effect">Effect: {toleration.effect}</span>
                  {/if}
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/if}

    <!-- Events -->
    {#if describeData.events && describeData.events.length > 0}
      <div class="section">
        <div class="section-title" onclick={() => toggleSection('events')}>
          <span class="toggle-icon">{isExpanded('events') ? '▼' : '▶'}</span>
          <span>Events ({describeData.events.length})</span>
        </div>
        {#if isExpanded('events')}
          <div class="section-content">
            <table class="events-table">
              <thead>
                <tr>
                  <th>Type</th>
                  <th>Reason</th>
                  <th>Message</th>
                  <th>Count</th>
                  <th>First Seen</th>
                  <th>Last Seen</th>
                </tr>
              </thead>
              <tbody>
                {#each describeData.events as event}
                  <tr class="event-{getEventTypeClass(event.type)}">
                    <td>{event.type}</td>
                    <td>{event.reason || 'None'}</td>
                    <td>{event.message || 'None'}</td>
                    <td>{event.count || 1}</td>
                    <td>{formatTimestamp(event.firstTimestamp)}</td>
                    <td>{formatTimestamp(event.lastTimestamp)}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>
    {/if}

    <!-- Metadata -->
    <div class="section">
      <div class="section-title" onclick={() => toggleSection('metadata')}>
        <span class="toggle-icon">{isExpanded('metadata') ? '▼' : '▶'}</span>
        <span>Metadata</span>
      </div>
      {#if isExpanded('metadata')}
        <div class="section-content">
          <div class="key-value-list">
            <div class="key-value-item">
              <span class="key">UID:</span>
              <span class="value">{describeData.metadata?.uid || 'None'}</span>
            </div>
            <div class="key-value-item">
              <span class="key">Resource Version:</span>
              <span class="value">{describeData.metadata?.resourceVersion || 'None'}</span>
            </div>
            <div class="key-value-item">
              <span class="key">Creation Timestamp:</span>
              <span class="value">{formatTimestamp(describeData.metadata?.creationTimestamp)}</span>
            </div>
            <div class="key-value-item">
              <span class="key">Generation:</span>
              <span class="value">{describeData.metadata?.generation || 0}</span>
            </div>
          </div>
        </div>
      {/if}
    </div>
  {:else}
    <div class="empty">No describe data available</div>
  {/if}
</div>

<style>
  @import '../styles/variables.css';

  .resource-describe {
    padding: 16px;
    color: var(--text-primary);
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
    font-size: 0.9rem;
    line-height: 1.6;
  }

  .loading, .error, .empty {
    padding: 20px;
    text-align: center;
    color: var(--text-secondary);
  }

  .error {
    color: #ff6b6b;
  }

  .section-header {
    margin-bottom: 24px;
    padding-bottom: 12px;
    border-bottom: 2px solid var(--border-primary);
  }

  .section-header h2 {
    margin: 0 0 8px 0;
    font-size: 1.5rem;
    color: var(--text-primary);
  }

  .namespace {
    color: var(--text-secondary);
    font-size: 0.9rem;
  }

  .section {
    margin-bottom: 16px;
    border: 1px solid var(--border-primary);
    border-radius: 6px;
    overflow: hidden;
    background: var(--bg-secondary);
  }

  .section-title {
    padding: 12px 16px;
    background: rgba(255, 255, 255, 0.05);
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 600;
    user-select: none;
    transition: background 0.2s;
  }

  .section-title:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .toggle-icon {
    font-size: 0.8rem;
    width: 16px;
    text-align: center;
  }

  .section-content {
    padding: 16px;
  }

  .key-value-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .key-value-item {
    display: flex;
    gap: 12px;
    padding: 4px 0;
  }

  .key-value-item .key {
    font-weight: 600;
    color: var(--text-secondary);
    min-width: 150px;
    flex-shrink: 0;
  }

  .key-value-item .value {
    color: var(--text-primary);
    word-break: break-word;
  }

  .conditions-table,
  .events-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.85rem;
  }

  .conditions-table th,
  .events-table th {
    text-align: left;
    padding: 8px 12px;
    background: rgba(255, 255, 255, 0.05);
    border-bottom: 1px solid var(--border-primary);
    font-weight: 600;
  }

  .conditions-table td,
  .events-table td {
    padding: 8px 12px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }

  .conditions-table tr:hover,
  .events-table tr:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .condition-true {
    color: #51cf66;
  }

  .condition-false {
    color: #ff6b6b;
  }

  .condition-unknown {
    color: var(--text-secondary);
  }

  .event-normal {
    color: #51cf66;
  }

  .event-warning {
    color: #ffd43b;
  }

  .status-running {
    color: #51cf66;
  }

  .status-pending {
    color: #ffd43b;
  }

  .status-failed {
    color: #ff6b6b;
  }

  .qos-guaranteed {
    color: #51cf66;
  }

  .qos-burstable {
    color: #ffd43b;
  }

  .qos-besteffort {
    color: #ff8787;
  }

  .container-section {
    margin-bottom: 24px;
    padding: 16px;
    background: rgba(255, 255, 255, 0.02);
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .container-name {
    margin: 0 0 12px 0;
    font-size: 1.1rem;
    color: var(--accent-color);
  }

  .container-status,
  .container-resources,
  .container-ports,
  .container-env {
    margin-top: 12px;
  }

  .container-status h4,
  .container-resources h4,
  .container-ports h4,
  .container-env h4 {
    margin: 0 0 8px 0;
    font-size: 0.95rem;
    color: var(--text-secondary);
  }

  .state-badge {
    display: inline-block;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 0.85rem;
    margin-top: 8px;
  }

  .state-badge.running {
    background: rgba(81, 207, 102, 0.2);
    color: #51cf66;
  }

  .state-badge.waiting {
    background: rgba(255, 212, 59, 0.2);
    color: #ffd43b;
  }

  .state-badge.terminated {
    background: rgba(255, 107, 107, 0.2);
    color: #ff6b6b;
  }

  .state-message {
    font-size: 0.85rem;
    opacity: 0.8;
  }

  .ports-list,
  .env-list,
  .volumes-list,
  .tolerations-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .port-item,
  .env-item,
  .volume-item,
  .toleration-item {
    padding: 6px 10px;
    background: rgba(255, 255, 255, 0.03);
    border-radius: 4px;
    font-family: 'Courier New', monospace;
    font-size: 0.85rem;
  }

  .env-name {
    font-weight: 600;
    color: var(--accent-color);
  }

  .env-value {
    color: var(--text-secondary);
  }

  .volume-name {
    font-weight: 600;
    color: var(--text-primary);
  }

  .volume-type {
    color: var(--text-secondary);
    margin-left: 8px;
  }

  .toleration-effect {
    margin-left: 12px;
    color: var(--text-secondary);
    font-size: 0.85rem;
  }
</style>
