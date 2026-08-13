[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$textExtensions = @('.css', '.html', '.js', '.json', '.md', '.ps1', '.rs', '.toml', '.txt', '.yml', '.yaml')
$blockedPatterns = [ordered]@{
    'Windows user path' = 'C:\\Users\\'
    'private writing workspace path' = 'S:\\Work\\writing'
    'delegation thread metadata' = 'source_thread_id'
    'private key material' = 'BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY'
    'GitHub classic token' = 'ghp_[A-Za-z0-9]{30,}'
    'protected private catalog filename' = 'INTERACTIVE-DEMO-MAPPING-CATALOG'
    'protected literature showcase filename' = 'interactive-swarm-control-literature-showcase'
}

$violations = [System.Collections.Generic.List[string]]::new()
Get-ChildItem -LiteralPath $repoRoot -File -Recurse | Where-Object {
    $relative = [System.IO.Path]::GetRelativePath($repoRoot, $_.FullName)
    $_.FullName -ne $PSCommandPath -and
    $relative -notmatch '^(?:\.git|target|output|\.playwright-cli)[\\/]' -and
    $textExtensions -contains $_.Extension.ToLowerInvariant()
} | ForEach-Object {
    $relative = [System.IO.Path]::GetRelativePath($repoRoot, $_.FullName)
    $content = Get-Content -LiteralPath $_.FullName -Raw
    foreach ($entry in $blockedPatterns.GetEnumerator()) {
        if ($content -match $entry.Value) {
            $violations.Add("${relative}: $($entry.Key)")
        }
    }
}
if ($violations.Count -gt 0) {
    throw "Public-boundary scan failed:`n$($violations -join "`n")"
}

$publicDataRoot = Join-Path $repoRoot 'web/data'
$publicDataFiles = @(Get-ChildItem -LiteralPath $publicDataRoot -File -Recurse)
if ($publicDataFiles.Count -ne 1 -or $publicDataFiles[0].Name -ne 'catalog.v1.json') {
    throw 'The public data directory must contain exactly the versioned public catalogue.'
}
& (Join-Path $PSScriptRoot 'Test-PublicCatalog.ps1')

$forbiddenData = @(Get-ChildItem -LiteralPath $repoRoot -File -Recurse | Where-Object {
    $_.FullName -notlike "$(Join-Path $repoRoot 'target')*" -and
    $_.FullName -notlike "$(Join-Path $repoRoot '.git')*" -and
    $_.Extension.ToLowerInvariant() -in @('.csv', '.jsonl', '.parquet', '.sqlite', '.tsv')
})
if ($forbiddenData.Count -gt 0) {
    throw "Unexpected export-like data files found: $($forbiddenData.FullName -join ', ')"
}
if (Test-Path -LiteralPath (Join-Path $repoRoot 'morphospace')) {
    throw 'A morphospace layer is outside the approved direct package boundary.'
}

$html = Get-Content -LiteralPath (Join-Path $repoRoot 'web/index.html') -Raw
if ($html -match '(?:src|href)="/(?!/)') {
    throw 'Root-relative web assets would break the intended GitHub Pages project subpath.'
}
if (Test-Path -LiteralPath (Join-Path $repoRoot 'web/CNAME')) {
    throw 'This project repository must not contain a CNAME file.'
}
$pagesWorkflow = Get-Content -LiteralPath (Join-Path $repoRoot '.github/workflows/pages.yml') -Raw
if ($pagesWorkflow -notmatch '(?m)^\s{2}workflow_dispatch:\s*$' -or
    $pagesWorkflow -match '(?m)^\s{2}(?:push|pull_request):\s*$') {
    throw 'The Pages workflow must remain manual-only until the publication checkpoint is approved.'
}

Write-Host 'Public-boundary scan passed: versioned allowlisted catalogue only; no private paths, protected filenames, credentials, or unexpected export-like data files found.'
