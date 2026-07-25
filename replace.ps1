$file = "src/lib/components/PodsPanel.svelte"
$content = Get-Content -Path $file -Raw

# Since the line endings can be \r\n, we use string matching directly instead of array slicing
$startIndex = $content.IndexOf("        {#if getRenderPods() && getRenderPods().length > 0}")
if ($startIndex -eq -1) {
    Write-Output "Start index not found."
    exit 1
}

$endIndex = $content.IndexOf("      </div>`n    {/if}")
if ($endIndex -eq -1) {
    $endIndex = $content.IndexOf("      </div>`r`n    {/if}")
}

if ($endIndex -eq -1) {
    Write-Output "End index not found."
    exit 1
}

$targetContent = $content.Substring($startIndex, $endIndex - $startIndex)

# Extract headers and rows
$headerStart = $targetContent.IndexOf("                <tr>")
$headerEnd = $targetContent.IndexOf("              </thead>")
$headers = $targetContent.Substring($headerStart, $headerEnd - $headerStart)

$rowsStart = $targetContent.IndexOf("                {#each filteredPods as pod (getPodKey(pod))}")
$rowsEnd = $targetContent.IndexOf("              </tbody>")
$rows = $targetContent.Substring($rowsStart, $rowsEnd - $rowsStart)

$replacement = @"
        <ResourceTable
          items={getRenderPods()}
          filteredItems={filteredPods}
          bind:searchQuery
          searchPlaceholder="Search pods by name, namespace, labels, IP, or any field... (e.g., name:nginx, app:web, 192.168.1.1)"
          noItemsMessage="No pods are currently available in this cluster context."
          noSearchResultsMessage="No pods match your search query:"
        >
          <svelte:fragment slot="header">
$headers          </svelte:fragment>
          <svelte:fragment slot="rows">
$rows          </svelte:fragment>
        </ResourceTable>
"@

$newContent = $content.Replace($targetContent, $replacement)

# Add import
if ($newContent -notmatch "import ResourceTable") {
    $newContent = $newContent -replace "import \{ onMount, onDestroy \} from 'svelte';", "import { onMount, onDestroy } from 'svelte';`n  import ResourceTable from './ResourceTable.svelte';"
}

Set-Content -Path $file -Value $newContent -NoNewline
Write-Output "Success"
