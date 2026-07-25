const fs = require('fs');
const file = 'src/lib/components/PodsPanel.svelte';
let content = fs.readFileSync(file, 'utf8');

// Normalize line endings to \n for easier manipulation
const originalEnding = content.includes('\r\n') ? '\r\n' : '\n';
const lines = content.split(/\r?\n/);

const headers = lines.slice(1224, 1305).join('\n');
const rows = lines.slice(1307, 1335).join('\n');
const target = lines.slice(1179, 1348).join('\n'); // 1179 to 1347 inclusive

const replacement = `        <ResourceTable
          items={getRenderPods()}
          filteredItems={filteredPods}
          bind:searchQuery
          searchPlaceholder="Search pods by name, namespace, labels, IP, or any field... (e.g., name:nginx, app:web, 192.168.1.1)"
          noItemsMessage="No pods are currently available in this cluster context."
          noSearchResultsMessage="No pods match your search query:"
        >
          <svelte:fragment slot="header">
${headers}
          </svelte:fragment>
          <svelte:fragment slot="rows">
${rows}
          </svelte:fragment>
        </ResourceTable>`;

let newContent = lines.join('\n');
if (newContent.includes(target)) {
    newContent = newContent.replace(target, replacement);
    
    // Add import if not exists
    if (!newContent.includes('import ResourceTable')) {
        newContent = newContent.replace("import { onMount, onDestroy } from 'svelte';", "import { onMount, onDestroy } from 'svelte';\n  import ResourceTable from './ResourceTable.svelte';");
    }
    
    // Restore original line endings
    newContent = newContent.split('\n').join(originalEnding);
    
    fs.writeFileSync(file, newContent);
    console.log('Success');
} else {
    console.log('Target not found');
}
