$file = "src/lib/components/DeploymentsPanel.svelte"
$content = Get-Content -Path $file -Raw

# Start of header controls and search input
$startIndex = $content.IndexOf("      <div class=""header-controls"">")
if ($startIndex -eq -1) { Write-Output "Start index not found."; exit 1 }

# End of empty states, just before the deployments-table
$endIndex = $content.IndexOf("      <div class=""deployments-table"">")
if ($endIndex -eq -1) { Write-Output "End index not found."; exit 1 }

$targetContent = $content.Substring($startIndex, $endIndex - $startIndex)

$replacement = @"
      <ResourceTable
        items={getRenderDeployments()}
        filteredItems={filteredDeployments}
        bind:searchQuery
        searchPlaceholder="Search Deployments..."
        noItemsMessage="No Deployments are currently in this cluster"
        noSearchResultsMessage="No Deployments match your search query:"
      >
        <svelte:fragment slot="table">
"@

$newContent = $content.Replace($targetContent, $replacement)

# Now we need to close the ResourceTable tags after the deployments-table block.
# The table block ends with "      </div>`n    {/if}`n  </div>`n`n  <!-- Quick Actions Menu -->"
$tableEndIndex = $newContent.IndexOf("      </div>`n    {/if}`n  </div>`n`n  <!-- Quick Actions Menu -->")
if ($tableEndIndex -eq -1) {
    $tableEndIndex = $newContent.IndexOf("      </div>`r`n    {/if}`r`n  </div>`r`n`r`n  <!-- Quick Actions Menu -->")
}

if ($tableEndIndex -ne -1) {
    $tableEndTarget = $newContent.Substring($tableEndIndex, ("      </div>`n    {/if}".Length))
    # Note: `tableEndTarget` matches `      </div>\n    {/if}` exactly if we substring just that length.
    # Actually, let's just replace `      </div>\n    {/if}` that precedes `  </div>\n\n  <!-- Quick Actions Menu -->`
    $newContent = $newContent -replace '      </div>\r?\n    \{/if\}', "      </div>`n        </svelte:fragment>`n      </ResourceTable>"
} else {
    Write-Output "Table end index not found."
    exit 1
}

# Add import
if ($newContent -notmatch "import ResourceTable") {
    $newContent = $newContent -replace "import \{ onMount, onDestroy \} from 'svelte';", "import { onMount, onDestroy } from 'svelte';`n  import ResourceTable from './ResourceTable.svelte';"
}

Set-Content -Path $file -Value $newContent -NoNewline
Write-Output "DeploymentsPanel.svelte updated"
