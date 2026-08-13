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
    'remove-field-button', 'state-dynamics-mode', 'state-dynamics-rates',
    'state-semantic-qualities', 'metric-distribution',
    'dynamics-alignment', 'dynamics-alignment-value', 'dynamics-cohesion',
    'dynamics-cohesion-value', 'dynamics-separation', 'dynamics-separation-value',
    'dynamics-flow-title', 'flow-alignment-rate', 'flow-cohesion-rate',
    'flow-separation-rate', 'flow-distribution', 'semantic-dynamics-controls',
    'semantic-space', 'semantic-space-value', 'semantic-time', 'semantic-time-value',
    'semantic-weight', 'semantic-weight-value', 'semantic-flow', 'semantic-flow-value',
    'semantic-raw-inspector', 'resolved-control-mode', 'resolved-alignment',
    'resolved-cohesion', 'resolved-separation', 'resolved-speed-scale',
    'resolved-damping', 'resolved-jitter', 'state-group-count',
    'state-morphology-revision', 'metric-groups', 'metric-group-sizes',
    'metric-formation-extent', 'morphology-group-roster', 'morphology-trace-before',
    'morphology-trace-action', 'morphology-trace-policy', 'morphology-trace-receipt',
    'morphology-trace-after', 'morphology-controls', 'split-source-group',
    'split-partition-rule', 'split-new-group', 'split-group-button',
    'merge-first-group', 'merge-second-group', 'merge-survivor-group',
    'merge-groups-button', 'scale-group', 'formation-scale',
    'formation-scale-value', 'set-formation-scale-button', 'state-lease-count',
    'state-authority-revision', 'metric-leases', 'metric-pending-handoffs',
    'metric-lease-remaining', 'lease-roster', 'lease-trace-before',
    'lease-trace-action', 'lease-trace-policy', 'lease-trace-receipt',
    'lease-trace-after', 'lease-controls', 'lease-operator', 'lease-member',
    'lease-receiver', 'lease-lifetime', 'lease-lifetime-value',
    'request-lease-button', 'release-lease-button', 'offer-handoff-button',
    'accept-handoff-button', 'decline-handoff-button', 'leased-behavior',
    'use-lease-button', 'lease-command-reason'
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
if ($styles -notmatch '@media\s*\(prefers-reduced-motion:\s*reduce\)' -or
    $styles -notmatch '@media\s*\(forced-colors:\s*active\)' -or
    $styles -notmatch '(?s)@media\s*\(forced-colors:\s*active\).*\.resolved-inspector' -or
    $styles -notmatch '(?s)@media\s*\(forced-colors:\s*active\).*\.morphology-trace' -or
    $styles -notmatch '(?s)@media\s*\(forced-colors:\s*active\).*\.lease-trace') {
    throw 'Semantic dynamics, morphology, and leases must retain reduced-motion and forced-colors contracts.'
}
foreach ($functionName in @(
    'populateAtlasFilters', 'renderAtlasList', 'renderAtlasDetail',
    'updateActionTrace', 'outcomeMetrics', 'saveCheckpoint',
    'retrieveCheckpoint', 'replayCurrentRun', 'renderSessionHistory',
    'placePersonalField', 'moveSelectedField', 'setSelectedFieldPolarity',
    'removeSelectedField', 'renderFieldStateList', 'drawPersonalFields',
    'bindDynamicsSlider', 'updateDynamicsControls', 'updateDynamicsOutput',
    'bindSemanticSlider', 'updateSemanticControls', 'updateSemanticOutput',
    'renderResolvedDynamics', 'splitSelectedGroup', 'mergeSelectedGroups',
    'setSelectedFormationScale', 'nextCanonicalGroupId', 'updateMorphologyControls',
    'renderMorphologyRoster', 'updateMorphologyTrace', 'requestSelectedLease',
    'releaseSelectedLease', 'offerSelectedHandoff', 'resolveSelectedHandoff',
    'useSelectedLease', 'updateLeaseControls', 'renderLeaseRoster',
    'updateLeaseTrace', 'traceAnimatedLeaseExpiry'
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
foreach ($dynamicsAction in @('set_alignment', 'set_cohesion', 'set_separation')) {
    if ($script -notmatch [regex]::Escape($dynamicsAction)) {
        throw "The accessible raw dynamics adapter is missing semantic action: $dynamicsAction"
    }
}
foreach ($semanticAction in @('set_space_quality', 'set_time_quality', 'set_weight_quality', 'set_flow_quality')) {
    if ($script -notmatch [regex]::Escape($semanticAction)) {
        throw "The accessible semantic dynamics adapter is missing semantic action: $semanticAction"
    }
}
foreach ($morphologyAction in @('split_group', 'merge_groups', 'set_formation_scale')) {
    if ($script -notmatch [regex]::Escape($morphologyAction)) {
        throw "The accessible morphology adapter is missing semantic action: $morphologyAction"
    }
}
foreach ($leaseAction in @('request_lease', 'release_lease', 'offer_lease_handoff', 'resolve_lease_handoff', 'set_leased_behavior')) {
    if ($script -notmatch [regex]::Escape($leaseAction)) {
        throw "The accessible lease adapter is missing semantic action: $leaseAction"
    }
}
if ($script -notmatch 'const MAX_ACTIVE_LEASES = 8;' -or
    $script -notmatch 'expected_authority_revision: state\.authority_revision') {
    throw 'Lease controls must preserve the bounded authority-revision contract.'
}
if ($html -notmatch 'id=[`"'']lease-lifetime[`"''][^>]*min=[`"'']1[`"''][^>]*max=[`"'']600[`"''][^>]*step=[`"'']1[`"''][^>]*value=[`"'']120[`"'']') {
    throw 'Lease lifetime must preserve its accepted range, step, and default.'
}
if ($script -notmatch 'const MAX_MORPHOLOGY_GROUPS = 8;' -or
    $script -notmatch 'expected_morphology_revision: state\.morphology_revision') {
    throw 'Morphology controls must preserve the bounded group and stale-action contracts.'
}
if ($html -notmatch 'id=[`"'']formation-scale[`"''][^>]*min=[`"'']0\.5[`"''][^>]*max=[`"'']2[`"''][^>]*step=[`"'']0\.05[`"''][^>]*value=[`"'']1[`"'']') {
    throw 'Formation scale must preserve its accepted range, step, and default.'
}
if ($script -match 'set_randomness') {
    throw 'Raw dynamics must not expose an arbitrary randomness parameter.'
}
foreach ($parameter in @('alignment', 'cohesion', 'separation')) {
    $pattern = "id=[`"']dynamics-$parameter[`"'][^>]*min=[`"']0[`"'][^>]*max=[`"']1[`"'][^>]*step=[`"']0\.05[`"'][^>]*value=[`"']0[`"']"
    if ($html -notmatch $pattern) {
        throw "Raw dynamics slider $parameter must preserve the accepted range, step, and default."
    }
}
foreach ($quality in @('space', 'time', 'weight', 'flow')) {
    $pattern = "id=[`"']semantic-$quality[`"'][^>]*min=[`"']0[`"'][^>]*max=[`"']1[`"'][^>]*step=[`"']0\.05[`"'][^>]*value=[`"']0\.5[`"']"
    if ($html -notmatch $pattern) {
        throw "Semantic dynamics slider $quality must preserve the accepted range, step, and default."
    }
}
if ($script -match '(?:getUserMedia|MediaPipe|DeviceMotionEvent|requestPermission)') {
    throw 'The semantic dynamics slice must not request or imply a live camera or motion adapter.'
}
if ($script -notmatch 'state\.raw_dynamics_rates\[parameter\]' -or
    $script -notmatch 'state\.resolved_dynamics') {
    throw 'Raw controls and the effective resolved dynamics vector must remain separately inspectable.'
}
if ($script -notmatch 'event instanceof PointerEvent && event\.detail > 0') {
    throw 'The input adapter must distinguish keyboard-generated PointerEvent clicks from physical pointer input.'
}

$implemented = @($catalog.items | Where-Object { $_.reconstruction.status -eq 'implemented-reconstruction' })
$implementedIds = @($implemented.public_id | Sort-Object)
$expectedImplementedIds = @(
    'additive-personal-fields',
    'lease-expiry-and-handoff',
    'raw-dynamics-parameters',
    'save-retrieve-reset-replay',
    'scope-and-granularity',
    'semantic-laban-dynamics',
    'split-merge-and-rescale'
)
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

Write-Host 'Atlas shell contract passed: seven facets, evidence cards, action provenance, metrics, and seven bound reconstructions.'
