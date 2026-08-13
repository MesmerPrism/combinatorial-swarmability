[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$html = Get-Content -LiteralPath (Join-Path $repoRoot 'web/index.html') -Raw
$script = Get-Content -LiteralPath (Join-Path $repoRoot 'web/app.js') -Raw
$styles = Get-Content -LiteralPath (Join-Path $repoRoot 'web/styles.css') -Raw
$catalog = Get-Content -LiteralPath (Join-Path $repoRoot 'web/data/catalog.v1.json') -Raw | ConvertFrom-Json

$requiredIds = @(
    'atlas-filters', 'atlas-list', 'atlas-detail', 'atlas-detail-title',
    'detail-input-expression', 'detail-semantic-action', 'detail-controlled-quantity',
    'detail-scope-timing', 'detail-combination', 'detail-source', 'detail-evidence',
    'detail-locus', 'detail-transfer', 'detail-nonclaim', 'entry-trace-input',
    'entry-trace-normalized', 'entry-trace-action', 'entry-trace-policy',
    'entry-trace-effect', 'trace-input-route', 'trace-normalized-input',
    'trace-semantic-action', 'trace-policy', 'trace-receipt', 'metric-cohesion',
    'metric-polarization', 'metric-spacing', 'metric-speed', 'metric-subgroup',
    'metric-relations'
)
foreach ($id in $requiredIds) {
    if ($html -notmatch "id=[`"']$([regex]::Escape($id))[`"']") {
        throw "Atlas shell is missing required semantic surface: $id"
    }
}

$expectedFacets = @(
    'input_routes', 'control_families', 'scopes', 'temporal_modes',
    'multi_user_policies', 'substrates', 'evidence_statuses'
)
$actualFacets = @(
    [regex]::Matches($html, 'data-facet="([a-z_]+)"') |
        ForEach-Object { $_.Groups[1].Value } |
        Sort-Object
)
if (($actualFacets -join "`n") -ne (($expectedFacets | Sort-Object) -join "`n")) {
    throw "Atlas filter facets differ from the public catalogue contract: $($actualFacets -join ', ')"
}

if ($script -notmatch 'fetch\("\./data/catalog\.v1\.json"' -or $script -match 'catalog\.synthetic') {
    throw 'The atlas adapter must load only the versioned public catalogue.'
}
if ($styles -notmatch '\[hidden\]\s*\{[^}]*display:\s*none\s*!important') {
    throw 'Author styles must preserve the native hidden state for planned entry actions.'
}
foreach ($functionName in @('populateAtlasFilters', 'renderAtlasList', 'renderAtlasDetail', 'updateActionTrace', 'outcomeMetrics')) {
    if ($script -notmatch "function\s+$functionName\s*\(") {
        throw "Atlas adapter is missing $functionName."
    }
}
if ($script -notmatch 'event instanceof PointerEvent && event\.detail > 0') {
    throw 'The input adapter must distinguish keyboard-generated PointerEvent clicks from physical pointer input.'
}

$implemented = @($catalog.items | Where-Object { $_.reconstruction.status -eq 'implemented-reconstruction' })
if ($implemented.Count -ne 1 -or $implemented[0].public_id -ne 'scope-and-granularity') {
    throw 'The first shell must enable only the accepted scope-and-granularity reconstruction.'
}
foreach ($item in @($catalog.items)) {
    if ([string]::IsNullOrWhiteSpace($item.source.paper_url)) {
        throw "Every first-shell entry requires a public paper link: $($item.public_id)"
    }
    if ([string]::IsNullOrWhiteSpace($item.source.literature_status) -or
        [string]::IsNullOrWhiteSpace($item.reconstruction.transfer_boundary)) {
        throw "Every first-shell entry requires an evidence label and transfer boundary: $($item.public_id)"
    }
}

Write-Host 'Atlas shell contract passed: seven filter facets, source/evidence/transfer cards, action provenance, metrics, and one enabled reconstruction.'
