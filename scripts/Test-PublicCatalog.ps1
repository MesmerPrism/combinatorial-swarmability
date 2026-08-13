[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$catalogPath = Join-Path $repoRoot 'web/data/catalog.v1.json'
$manifestPath = Join-Path $repoRoot 'docs/catalog-export.v1.manifest.json'
$schemaPath = Join-Path $repoRoot 'schemas/public-catalog.v1.schema.json'
$profilePath = Join-Path $repoRoot 'tools/catalog/export-profile.v1.json'

foreach ($path in @($catalogPath, $manifestPath, $schemaPath, $profilePath)) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Missing public catalogue artifact: $path" }
}

function Get-Sha256([string]$Path) {
    (Get-FileHash -Algorithm SHA256 -LiteralPath $Path).Hash.ToLowerInvariant()
}

function Assert-ExactProperties([object]$Value, [string[]]$Expected, [string]$Label) {
    $actual = @($Value.PSObject.Properties.Name | Sort-Object)
    $wanted = @($Expected | Sort-Object)
    if (($actual -join "`n") -ne ($wanted -join "`n")) {
        throw "$Label fields differ from the public schema. Actual: $($actual -join ', ')"
    }
}

function Assert-StringArray([object[]]$Values, [string]$Label) {
    if (@($Values).Count -eq 0) { throw "$Label must not be empty." }
    $seen = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
    foreach ($value in @($Values)) {
        if ([string]::IsNullOrWhiteSpace([string]$value)) { throw "$Label contains an empty value." }
        if (-not $seen.Add([string]$value)) { throw "$Label contains a duplicate value: $value" }
    }
}

$catalog = Get-Content -LiteralPath $catalogPath -Raw | ConvertFrom-Json
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
$catalogBytes = [System.IO.File]::ReadAllBytes($catalogPath)
$manifestBytes = [System.IO.File]::ReadAllBytes($manifestPath)
foreach ($artifact in @(
    [pscustomobject]@{ Label = 'catalogue'; Bytes = $catalogBytes },
    [pscustomobject]@{ Label = 'manifest'; Bytes = $manifestBytes }
)) {
    if ($artifact.Bytes.Length -ge 3 -and $artifact.Bytes[0] -eq 0xEF -and $artifact.Bytes[1] -eq 0xBB -and $artifact.Bytes[2] -eq 0xBF) {
        throw "The public $($artifact.Label) must be UTF-8 without a byte-order mark."
    }
    if ([System.Text.Encoding]::UTF8.GetString($artifact.Bytes).Contains("`r")) {
        throw "The public $($artifact.Label) must use LF line endings."
    }
}
Assert-ExactProperties $catalog @('schema', 'export_version', 'source_binding', 'items') 'Catalogue'
if ($catalog.schema -ne 'combinatorial.swarmability.public.catalog.v1') { throw 'Unexpected public catalogue schema.' }
if ($catalog.export_version -ne $manifest.export_version) { throw 'Catalogue and manifest versions differ.' }
if ($catalog.source_binding.authority -ne 'research-memory') { throw 'Unexpected catalogue source authority.' }
if ($catalog.source_binding.sha256 -ne $manifest.source.sha256) { throw 'Source hash binding differs between catalogue and manifest.' }
if ($catalog.source_binding.schema_sha256 -ne (Get-Sha256 $schemaPath)) { throw 'Schema hash binding is stale.' }
if ($catalog.source_binding.profile_sha256 -ne (Get-Sha256 $profilePath)) { throw 'Profile hash binding is stale.' }
if ($manifest.output.sha256 -ne (Get-Sha256 $catalogPath)) { throw 'Generated public catalogue hash differs from its manifest.' }
if ([int]$manifest.output.item_count -ne @($catalog.items).Count) { throw 'Manifest item count differs from the catalogue.' }
if ([int]$manifest.transform.selected_rows + [int]$manifest.transform.rejected_rows -ne [int]$manifest.source.row_count) {
    throw 'Selected and rejected row counts do not cover the bound source input.'
}

$itemFields = @('public_id', 'display_order', 'title', 'source', 'reported', 'reconstruction', 'facets')
$sourceFields = @('source_id', 'system_or_study', 'year', 'paper_url', 'project_url', 'artifact_url', 'source_locus', 'evidence_kind', 'literature_status', 'catalog_projection_status', 'checked_on')
$reportedFields = @('input_expression', 'input_routes', 'user_action', 'semantic_action', 'control_family', 'controlled_quantity', 'parameter_exposure', 'target_scope', 'temporal_mode', 'human_configuration', 'multi_user_combination', 'substrate')
$reconstructionFields = @('status', 'summary', 'interaction', 'effect', 'semantic_actions', 'transfer_boundary', 'does_not_claim')
$facetFields = @('input_routes', 'control_families', 'scopes', 'temporal_modes', 'multi_user_policies', 'substrates', 'evidence_statuses')
$publicIds = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
$orders = [System.Collections.Generic.HashSet[int]]::new()

foreach ($item in @($catalog.items)) {
    Assert-ExactProperties $item $itemFields "Item $($item.public_id)"
    Assert-ExactProperties $item.source $sourceFields "Source $($item.public_id)"
    Assert-ExactProperties $item.reported $reportedFields "Reported fields $($item.public_id)"
    Assert-ExactProperties $item.reconstruction $reconstructionFields "Reconstruction $($item.public_id)"
    Assert-ExactProperties $item.facets $facetFields "Facets $($item.public_id)"
    if ($item.public_id -notmatch '^[a-z0-9]+(?:-[a-z0-9]+)*$' -or -not $publicIds.Add([string]$item.public_id)) {
        throw "Invalid or duplicate public ID: $($item.public_id)"
    }
    if (-not $orders.Add([int]$item.display_order)) { throw "Duplicate display order: $($item.display_order)" }
    if ($item.source.source_id -notmatch '^cs-source-[a-z0-9-]+$') { throw "Invalid public source ID: $($item.source.source_id)" }
    if ($item.source.catalog_projection_status -ne 'verified-projection') { throw "Unverified source projection: $($item.public_id)" }
    if ($item.reconstruction.status -notin @('implemented-reconstruction', 'planned-reconstruction')) {
        throw "Invalid reconstruction status: $($item.public_id)"
    }
    foreach ($urlField in @('paper_url', 'project_url', 'artifact_url')) {
        $value = $item.source.$urlField
        if ($null -ne $value -and $value -notmatch '^https?://') { throw "Non-public URL in $($item.public_id): $value" }
    }
    Assert-StringArray @($item.reconstruction.semantic_actions) "semantic actions for $($item.public_id)"
    foreach ($facet in $facetFields) {
        Assert-StringArray @($item.facets.$facet) "$facet for $($item.public_id)"
    }
}

$catalogText = Get-Content -LiteralPath $catalogPath -Raw
$blockedPatterns = [ordered]@{
    'absolute Windows path' = '[A-Za-z]:\\'
    'relative private note path' = '\.\./(?:source-notes|core-working-docs)/'
    'private note filename' = '(?:source_note|\.md")'
    'credential-like material' = '(?:ghp_|BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY)'
    'internal mapping identifier' = '(?:DEMO-MAP-|CLD-[0-9])'
}
foreach ($entry in $blockedPatterns.GetEnumerator()) {
    if ($catalogText -match $entry.Value) { throw "Public catalogue contains $($entry.Key)." }
}

Write-Host "Public catalogue validation passed: $(@($catalog.items).Count) allowlisted items, exact schema/profile/output hashes, no private path or internal identifier leakage."
