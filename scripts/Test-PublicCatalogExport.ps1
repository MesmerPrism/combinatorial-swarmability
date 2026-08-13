[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string]$InputPath
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$exportScript = Join-Path $PSScriptRoot 'Export-PublicCatalog.ps1'
$profilePath = Join-Path $repoRoot 'tools/catalog/export-profile.v1.json'
$schemaPath = Join-Path $repoRoot 'schemas/public-catalog.v1.schema.json'
$committedOutput = Join-Path $repoRoot 'web/data/catalog.v1.json'
$temporaryRoot = Join-Path $repoRoot 'target/catalog-export-test'

if (Test-Path -LiteralPath $temporaryRoot) {
    Remove-Item -LiteralPath $temporaryRoot -Recurse -Force
}
New-Item -ItemType Directory -Path $temporaryRoot | Out-Null

function Invoke-ExpectedFailure([scriptblock]$Operation, [string]$Pattern, [string]$Label) {
    $failed = $false
    try {
        & $Operation
    }
    catch {
        $failed = $true
        if ($_.Exception.Message -notmatch $Pattern) {
            throw "$Label failed for the wrong reason: $($_.Exception.Message)"
        }
    }
    if (-not $failed) { throw "$Label unexpectedly succeeded." }
}

try {
    $regeneratedOutput = Join-Path $temporaryRoot 'catalog.v1.json'
    $regeneratedManifest = Join-Path $temporaryRoot 'manifest.json'
    & $exportScript `
        -InputPath $InputPath `
        -ProfilePath $profilePath `
        -SchemaPath $schemaPath `
        -OutputPath $regeneratedOutput `
        -ManifestPath $regeneratedManifest

    $expectedBytes = [System.IO.File]::ReadAllBytes($committedOutput)
    $regeneratedBytes = [System.IO.File]::ReadAllBytes($regeneratedOutput)
    if ($expectedBytes.Length -ne $regeneratedBytes.Length -or
        [System.Convert]::ToBase64String($expectedBytes) -cne [System.Convert]::ToBase64String($regeneratedBytes)) {
        throw 'The locked export is not byte-for-byte reproducible.'
    }

    $damagedInput = Join-Path $temporaryRoot 'damaged-input.tsv'
    [System.IO.File]::WriteAllBytes($damagedInput, [System.IO.File]::ReadAllBytes($InputPath))
    [System.IO.File]::AppendAllText($damagedInput, "`n", [System.Text.UTF8Encoding]::new($false))
    Invoke-ExpectedFailure {
        & $exportScript -InputPath $damagedInput -OutputPath (Join-Path $temporaryRoot 'damage.json') -ManifestPath (Join-Path $temporaryRoot 'damage-manifest.json')
    } 'Authority input hash mismatch' 'Damaged-input rejection'

    $unknownProfilePath = Join-Path $temporaryRoot 'unknown-profile.json'
    $unknownProfile = Get-Content -LiteralPath $profilePath -Raw | ConvertFrom-Json
    $unknownProfile | Add-Member -NotePropertyName 'private_commentary' -NotePropertyValue 'must reject'
    [System.IO.File]::WriteAllText(
        $unknownProfilePath,
        ($unknownProfile | ConvertTo-Json -Depth 20) + "`n",
        [System.Text.UTF8Encoding]::new($false)
    )
    Invoke-ExpectedFailure {
        & $exportScript -InputPath $InputPath -ProfilePath $unknownProfilePath -OutputPath (Join-Path $temporaryRoot 'unknown.json') -ManifestPath (Join-Path $temporaryRoot 'unknown-manifest.json')
    } 'fields differ from the locked allowlist' 'Unknown-profile-field rejection'

    $duplicateProfilePath = Join-Path $temporaryRoot 'duplicate-profile.json'
    $duplicateProfile = Get-Content -LiteralPath $profilePath -Raw | ConvertFrom-Json
    $duplicateProfile.entries = @($duplicateProfile.entries) + @($duplicateProfile.entries[0])
    [System.IO.File]::WriteAllText(
        $duplicateProfilePath,
        ($duplicateProfile | ConvertTo-Json -Depth 20) + "`n",
        [System.Text.UTF8Encoding]::new($false)
    )
    Invoke-ExpectedFailure {
        & $exportScript -InputPath $InputPath -ProfilePath $duplicateProfilePath -OutputPath (Join-Path $temporaryRoot 'duplicate.json') -ManifestPath (Join-Path $temporaryRoot 'duplicate-manifest.json')
    } 'Duplicate export selector' 'Duplicate-selector rejection'

    Write-Host 'Locked export validation passed: byte-for-byte reproduction plus damaged-input, unknown-field, and duplicate-selector rejection.'
}
finally {
    if (Test-Path -LiteralPath $temporaryRoot) {
        Remove-Item -LiteralPath $temporaryRoot -Recurse -Force
    }
}
