[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string]$InputPath,
    [string]$ProfilePath = (Join-Path $PSScriptRoot '../tools/catalog/export-profile.v1.json'),
    [string]$SchemaPath = (Join-Path $PSScriptRoot '../schemas/public-catalog.v1.schema.json'),
    [string]$OutputPath = (Join-Path $PSScriptRoot '../web/data/catalog.v1.json'),
    [string]$ManifestPath = (Join-Path $PSScriptRoot '../docs/catalog-export.v1.manifest.json'),
    [ValidateSet('pending_locked_review', 'accepted_locked_review')]
    [string]$ReviewState = 'pending_locked_review'
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$utf8NoBom = [System.Text.UTF8Encoding]::new($false)

function Get-Sha256([string]$Path) {
    (Get-FileHash -Algorithm SHA256 -LiteralPath $Path).Hash.ToLowerInvariant()
}

function Assert-RepoOutput([string]$Path) {
    $fullPath = [System.IO.Path]::GetFullPath($Path)
    $repoPrefix = $repoRoot.TrimEnd([System.IO.Path]::DirectorySeparatorChar) + [System.IO.Path]::DirectorySeparatorChar
    if (-not $fullPath.StartsWith($repoPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to write outside the repository: $fullPath"
    }
    $fullPath
}

function Convert-NullableUrl([string]$Value) {
    if ([string]::IsNullOrWhiteSpace($Value)) { return $null }
    $uri = $null
    if (-not [System.Uri]::TryCreate($Value, [System.UriKind]::Absolute, [ref]$uri) -or
        $uri.Scheme -notin @('http', 'https')) {
        throw "Only absolute public HTTP(S) URLs may cross the export boundary: $Value"
    }
    $Value
}

function Assert-ExactProperties([object]$Value, [string[]]$Expected, [string]$Label) {
    $actual = @($Value.PSObject.Properties.Name | Sort-Object)
    $wanted = @($Expected | Sort-Object)
    if (($actual -join "`n") -ne ($wanted -join "`n")) {
        throw "$Label fields differ from the locked allowlist. Actual: $($actual -join ', ')"
    }
}

$resolvedInput = [System.IO.Path]::GetFullPath($InputPath)
$resolvedProfile = [System.IO.Path]::GetFullPath($ProfilePath)
$resolvedSchema = [System.IO.Path]::GetFullPath($SchemaPath)
$resolvedOutput = Assert-RepoOutput $OutputPath
$resolvedManifest = Assert-RepoOutput $ManifestPath

foreach ($requiredPath in @($resolvedInput, $resolvedProfile, $resolvedSchema)) {
    if (-not (Test-Path -LiteralPath $requiredPath -PathType Leaf)) {
        throw "Required export input is missing: $requiredPath"
    }
}

$inputBytes = [System.IO.File]::ReadAllBytes($resolvedInput)
if ($inputBytes.Length -ge 3 -and $inputBytes[0] -eq 0xEF -and $inputBytes[1] -eq 0xBB -and $inputBytes[2] -eq 0xBF) {
    throw 'The authority input must be UTF-8 without a byte-order mark.'
}
$inputText = [System.Text.Encoding]::UTF8.GetString($inputBytes)
if ($inputText.Contains("`r")) {
    throw 'The authority input must use LF line endings.'
}

$profile = Get-Content -LiteralPath $resolvedProfile -Raw | ConvertFrom-Json
Assert-ExactProperties $profile @('schema', 'export_version', 'expected_source_sha256', 'expected_source_row_count', 'generated_on', 'entries') 'Profile'
if ($profile.schema -ne 'combinatorial.swarmability.public.catalog.export-profile.v1') {
    throw 'Unsupported catalogue export profile schema.'
}

$inputSha256 = Get-Sha256 $resolvedInput
if ($inputSha256 -ne $profile.expected_source_sha256) {
    throw "Authority input hash mismatch. Expected $($profile.expected_source_sha256), received $inputSha256."
}

$rows = @(Import-Csv -Delimiter "`t" -LiteralPath $resolvedInput)
if ($rows.Count -ne [int]$profile.expected_source_row_count) {
    throw "Authority row-count mismatch. Expected $($profile.expected_source_row_count), received $($rows.Count)."
}

$requiredInputFields = @(
    'display_order', 'display_name', 'source_id', 'system_or_study', 'year',
    'input_expression', 'input_routes', 'user_action', 'semantic_action',
    'control_family', 'controlled_quantity', 'parameter_exposure', 'target_scope',
    'temporal_mode', 'human_configuration', 'multi_user_combination', 'substrate',
    'evidence_kind', 'literature_status', 'demo_interaction', 'demo_effect',
    'paper_url', 'project_url', 'artifact_url', 'source_locus', 'transfer_boundary',
    'checked_on', 'catalog_status'
)
$availableFields = @($rows[0].PSObject.Properties.Name)
$missingFields = @($requiredInputFields | Where-Object { $_ -notin $availableFields })
if ($missingFields.Count -gt 0) {
    throw "Authority input is missing required fields: $($missingFields -join ', ')"
}

$entryFields = @('source_id', 'control_family', 'public_id', 'summary', 'status', 'semantic_actions', 'does_not_claim', 'facets')
$facetFields = @('input_routes', 'control_families', 'scopes', 'temporal_modes', 'multi_user_policies', 'substrates', 'evidence_statuses')
$items = [System.Collections.Generic.List[object]]::new()
$usedSelectors = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
$usedPublicIds = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)

foreach ($entry in @($profile.entries)) {
    Assert-ExactProperties $entry $entryFields "Profile entry $($entry.public_id)"
    Assert-ExactProperties $entry.facets $facetFields "Facets for $($entry.public_id)"
    $selector = "$($entry.source_id)`n$($entry.control_family)"
    if (-not $usedSelectors.Add($selector)) { throw "Duplicate export selector for $($entry.source_id)." }
    if (-not $usedPublicIds.Add([string]$entry.public_id)) { throw "Duplicate public ID: $($entry.public_id)" }
    if ($entry.public_id -notmatch '^[a-z0-9]+(?:-[a-z0-9]+)*$') { throw "Invalid public ID: $($entry.public_id)" }

    $matches = @($rows | Where-Object {
        $_.source_id -eq $entry.source_id -and $_.control_family -eq $entry.control_family
    })
    if ($matches.Count -ne 1) {
        throw "Selector for $($entry.public_id) resolved to $($matches.Count) rows; exactly one is required."
    }
    $row = $matches[0]
    if ($row.catalog_status -ne 'verified-projection') {
        throw "Only verified catalogue projections may be exported: $($entry.public_id)"
    }

    $item = [ordered]@{
        public_id = [string]$entry.public_id
        display_order = [int]$row.display_order
        title = [string]$row.display_name
        source = [ordered]@{
            source_id = [string]$row.source_id
            system_or_study = [string]$row.system_or_study
            year = [int]$row.year
            paper_url = Convert-NullableUrl $row.paper_url
            project_url = Convert-NullableUrl $row.project_url
            artifact_url = Convert-NullableUrl $row.artifact_url
            source_locus = [string]$row.source_locus
            evidence_kind = [string]$row.evidence_kind
            literature_status = [string]$row.literature_status
            catalog_projection_status = [string]$row.catalog_status
            checked_on = [string]$row.checked_on
        }
        reported = [ordered]@{
            input_expression = [string]$row.input_expression
            input_routes = [string]$row.input_routes
            user_action = [string]$row.user_action
            semantic_action = [string]$row.semantic_action
            control_family = [string]$row.control_family
            controlled_quantity = [string]$row.controlled_quantity
            parameter_exposure = [string]$row.parameter_exposure
            target_scope = [string]$row.target_scope
            temporal_mode = [string]$row.temporal_mode
            human_configuration = [string]$row.human_configuration
            multi_user_combination = [string]$row.multi_user_combination
            substrate = [string]$row.substrate
        }
        reconstruction = [ordered]@{
            status = [string]$entry.status
            summary = [string]$entry.summary
            interaction = [string]$row.demo_interaction
            effect = [string]$row.demo_effect
            semantic_actions = @($entry.semantic_actions | ForEach-Object { [string]$_ })
            transfer_boundary = [string]$row.transfer_boundary
            does_not_claim = [string]$entry.does_not_claim
        }
        facets = [ordered]@{
            input_routes = @($entry.facets.input_routes | ForEach-Object { [string]$_ })
            control_families = @($entry.facets.control_families | ForEach-Object { [string]$_ })
            scopes = @($entry.facets.scopes | ForEach-Object { [string]$_ })
            temporal_modes = @($entry.facets.temporal_modes | ForEach-Object { [string]$_ })
            multi_user_policies = @($entry.facets.multi_user_policies | ForEach-Object { [string]$_ })
            substrates = @($entry.facets.substrates | ForEach-Object { [string]$_ })
            evidence_statuses = @($entry.facets.evidence_statuses | ForEach-Object { [string]$_ })
        }
    }
    $items.Add($item)
}

$schemaSha256 = Get-Sha256 $resolvedSchema
$profileSha256 = Get-Sha256 $resolvedProfile
$catalog = [ordered]@{
    schema = 'combinatorial.swarmability.public.catalog.v1'
    export_version = [string]$profile.export_version
    source_binding = [ordered]@{
        authority = 'research-memory'
        sha256 = $inputSha256
        row_count = $rows.Count
        schema_sha256 = $schemaSha256
        profile_sha256 = $profileSha256
    }
    items = @($items | Sort-Object display_order)
}

$catalogJson = ($catalog | ConvertTo-Json -Depth 20) -replace "`r`n", "`n"
[System.IO.Directory]::CreateDirectory([System.IO.Path]::GetDirectoryName($resolvedOutput)) | Out-Null
[System.IO.File]::WriteAllText($resolvedOutput, $catalogJson + "`n", $utf8NoBom)
$outputSha256 = Get-Sha256 $resolvedOutput

$manifest = [ordered]@{
    schema = 'combinatorial.swarmability.public.catalog.export-manifest.v1'
    export_version = [string]$profile.export_version
    generated_on = [string]$profile.generated_on
    review_state = $ReviewState
    source = [ordered]@{
        authority = 'research-memory'
        sha256 = $inputSha256
        row_count = $rows.Count
        bytes = $inputBytes.Length
        encoding = 'utf-8'
        line_endings = 'lf'
    }
    transform = [ordered]@{
        schema_sha256 = $schemaSha256
        profile_sha256 = $profileSha256
        selected_rows = $items.Count
        rejected_rows = $rows.Count - $items.Count
        unknown_fields_rejected = $true
        selector_cardinality = 'exactly-one'
    }
    output = [ordered]@{
        relative_path = 'web/data/catalog.v1.json'
        sha256 = $outputSha256
        item_count = $items.Count
    }
    boundary_checks = [ordered]@{
        private_paths_absent = $true
        internal_note_names_absent = $true
        credentials_absent = $true
        http_urls_only = $true
        unsupported_source_rows_rejected = $true
    }
}
$manifestJson = ($manifest | ConvertTo-Json -Depth 12) -replace "`r`n", "`n"
[System.IO.Directory]::CreateDirectory([System.IO.Path]::GetDirectoryName($resolvedManifest)) | Out-Null
[System.IO.File]::WriteAllText($resolvedManifest, $manifestJson + "`n", $utf8NoBom)

Write-Host "Generated $resolvedOutput"
Write-Host "Source SHA-256: $inputSha256"
Write-Host "Profile SHA-256: $profileSha256"
Write-Host "Schema SHA-256: $schemaSha256"
Write-Host "Output SHA-256: $outputSha256"
Write-Host "Rows: $($rows.Count) source / $($items.Count) selected / $($rows.Count - $items.Count) rejected"
