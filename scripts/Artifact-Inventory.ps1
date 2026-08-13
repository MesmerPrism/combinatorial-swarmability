[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$distRoot = Join-Path $repoRoot 'dist'
$outputRoot = Join-Path $repoRoot 'output'
if (-not (Test-Path -LiteralPath $distRoot)) {
    throw 'dist/ does not exist. Run Build-Web.ps1 first.'
}

New-Item -ItemType Directory -Path $outputRoot -Force | Out-Null
$items = @(Get-ChildItem -LiteralPath $distRoot -File -Recurse | Sort-Object FullName | ForEach-Object {
    [pscustomobject][ordered]@{
        path = [System.IO.Path]::GetRelativePath($distRoot, $_.FullName).Replace('\', '/')
        bytes = $_.Length
        sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    }
})
$totalBytes = [long](($items | Measure-Object -Property bytes -Sum).Sum)
$inventory = [ordered]@{
    schema = 'combinatorial.swarmability.artifact-inventory.v1'
    file_count = $items.Count
    total_bytes = $totalBytes
    files = $items
}
$inventoryPath = Join-Path $outputRoot 'artifact-inventory.json'
$inventory | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $inventoryPath -Encoding utf8
$inventoryHash = (Get-FileHash -LiteralPath $inventoryPath -Algorithm SHA256).Hash.ToLowerInvariant()
Set-Content -LiteralPath (Join-Path $outputRoot 'artifact-inventory.sha256') -Value "$inventoryHash  artifact-inventory.json" -Encoding ascii

Write-Host "Artifact inventory: $($items.Count) files, $($inventory.total_bytes) bytes."
Write-Host "Inventory SHA-256: $inventoryHash"
$items | ForEach-Object { Write-Host "$($_.sha256)  $($_.path)" }
