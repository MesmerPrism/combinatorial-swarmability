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
    'metric-relations', 'state-replay-events', 'state-checkpoint-count',
    'checkpoint-name', 'checkpoint-select', 'save-checkpoint-button',
    'retrieve-checkpoint-button', 'replay-button', 'reset-button',
    'history-events', 'state-field-count', 'state-contributor-count',
    'metric-fields', 'field-state-list', 'field-contributor', 'field-polarity',
    'field-lifetime', 'field-x', 'field-y', 'field-select',
    'place-field-button', 'move-field-button', 'polarity-field-button',
    'remove-field-button'
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
foreach ($functionName in @(
    'populateAtlasFilters', 'renderAtlasList', 'renderAtlasDetail',
    'updateActionTrace', 'outcomeMetrics', 'saveCheckpoint',
    'retrieveCheckpoint', 'replayCurrentRun', 'renderSessionHistory',
    'placePersonalField', 'moveSelectedField', 'setSelectedFieldPolarity',
    'removeSelectedField', 'renderFieldStateList', 'drawPersonalFields'
)) {
    if ($script -notmatch "function\s+$functionName\s*\(") {
        throw "Atlas adapter is missing $functionName."
    }
}
foreach ($adapterMethod in @('engine.replay_json()', 'engine.load_replay_json(')) {
    if (-not $script.Contains($adapterMethod)) {
        throw "Atlas history infrastructure is missing the Wasm adapter call: $adapterMethod"
    }
}
if ($script -notmatch 'const MAX_CHECKPOINTS = 5;' -or
    $script -notmatch 'const MAX_SESSION_HISTORY = 50;' -or
    $script -notmatch 'const savedCheckpoints = new Map\(\);') {
    throw 'Atlas history must remain bounded to five tab-local checkpoints and 50 visible operations.'
}
if ($script -match '(?:localStorage|sessionStorage|indexedDB)') {
    throw 'The first history slice must remain session-local and in memory.'
}
if ($script -notmatch 'const MAX_PERSONAL_FIELDS = 8;' -or
    $script -notmatch 'const CONTRIBUTOR_LABELS = \["A", "B", "C", "D"\];') {
    throw 'Personal fields must remain bounded to eight sources and four synthetic contributor channels.'
}
foreach ($fieldAction in @('place_field', 'move_field', 'set_field_polarity', 'remove_field')) {
    if ($script -notmatch [regex]::Escape($fieldAction)) {
        throw "The accessible field adapter is missing semantic action: $fieldAction"
    }
}
if ($script -notmatch 'event instanceof PointerEvent && event\.detail > 0') {
    throw 'The input adapter must distinguish keyboard-generated PointerEvent clicks from physical pointer input.'
}

$implemented = @($catalog.items | Where-Object { $_.reconstruction.status -eq 'implemented-reconstruction' })
$implementedIds = @($implemented.public_id | Sort-Object)
$expectedImplementedIds = @('save-retrieve-reset-replay', 'scope-and-granularity')
if (($implementedIds -join "`n") -ne ($expectedImplementedIds -join "`n")) {
    throw "The atlas implemented set differs from the accepted mechanism slices: $($implementedIds -join ', ')"
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

Write-Host 'Atlas shell contract passed: seven facets, evidence cards, action provenance, metrics, two bound reconstructions, and bounded additive-field controls.'
