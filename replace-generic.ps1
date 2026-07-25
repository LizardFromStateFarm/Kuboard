param (
    [Parameter(Mandatory=$true)]
    [string]$file,
    [Parameter(Mandatory=$true)]
    [string]$resourceType
)

$content = Get-Content -Path $file -Raw

# Start of header controls and search input
$startIndex = $content.IndexOf("      <div class=""header-controls"">")
if ($startIndex -eq -1) { Write-Output "Start index not found in $file."; exit 1 }

# End of empty states, just before the table
$endIndex = $content.IndexOf("      <div class=""${resourceType}-table"">")
if ($endIndex -eq -1) { Write-Output "End index not found in $file."; exit 1 }

$targetContent = $content.Substring($startIndex, $endIndex - $startIndex)

# Figure out the variable names
$capResourceType = (Get-Culture).TextInfo.ToTitleCase($resourceType)
$capResourceType = $capResourceType -replace 'set', 'Set'
$capResourceType = $capResourceType -replace 'job', 'Job'
$renderFunc = "getRender" + $capResourceType + "s()"
$filteredVar = "filtered" + $capResourceType + "s"

$replacement = @"
      <ResourceTable
        items={$renderFunc}
        filteredItems={$filteredVar}
        bind:searchQuery
        searchPlaceholder="Search $capResourceType..."
        noItemsMessage="No $capResourceType are currently in this cluster"
        noSearchResultsMessage="No $capResourceType match your search query:"
      >
        <svelte:fragment slot="table">
"@

$newContent = $content.Replace($targetContent, $replacement)

# Close the ResourceTable tags after the table block.
$tableEndStr = "      </div>`n    {/if}`n  </div>`n`n  <!-- Quick Actions Menu -->"
$tableEndIndex = $newContent.IndexOf($tableEndStr)
if ($tableEndIndex -eq -1) {
    $tableEndStr = "      </div>`r`n    {/if}`r`n  </div>`r`n`r`n  <!-- Quick Actions Menu -->"
    $tableEndIndex = $newContent.IndexOf($tableEndStr)
}

if ($tableEndIndex -ne -1) {
    $newContent = $newContent -replace '      </div>\r?\n    \{/if\}', "      </div>`n        </svelte:fragment>`n      </ResourceTable>"
} else {
    Write-Output "Table end index not found in $file."
    exit 1
}

# Add import
if ($newContent -notmatch "import ResourceTable") {
    $newContent = $newContent -replace "import \{ onMount, onDestroy \} from 'svelte';", "import { onMount, onDestroy } from 'svelte';`n  import ResourceTable from './ResourceTable.svelte';"
}

Set-Content -Path $file -Value $newContent -NoNewline

# Clean up CSS
$content = $newContent
$cssStartIndex = $content.IndexOf("  .header-controls {")
$cssEndIndex = $content.IndexOf("  .${resourceType}-table {")
if ($cssStartIndex -ne -1 -and $cssEndIndex -ne -1) {
    $toRemove = $content.Substring($cssStartIndex, $cssEndIndex - $cssStartIndex)
    $content = $content.Replace($toRemove, "")
    Set-Content -Path $file -Value $content -NoNewline
    Write-Output "Cleaned up CSS for $file"
}

Write-Output "Done $file"
