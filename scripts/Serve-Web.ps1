[CmdletBinding()]
param(
    [ValidateRange(1024, 65535)]
    [int]$Port = 4173,
    [switch]$SkipBuild,
    [switch]$StageOnly,
    [switch]$Background
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$siteRoot = Join-Path $repoRoot 'target/local-site'
$projectRoot = Join-Path $siteRoot 'combinatorial-swarmability'

if (-not $SkipBuild) {
    & (Join-Path $PSScriptRoot 'Build-Web.ps1') -Release
    if ($LASTEXITCODE -ne 0) { throw 'Web build failed.' }
}

$sitePrefix = $repoRoot.TrimEnd([System.IO.Path]::DirectorySeparatorChar) + [System.IO.Path]::DirectorySeparatorChar
$resolvedSiteRoot = [System.IO.Path]::GetFullPath($siteRoot)
if (-not $resolvedSiteRoot.StartsWith($sitePrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "Refusing to stage outside the repository: $resolvedSiteRoot"
}

if (Test-Path -LiteralPath $siteRoot) {
    Remove-Item -LiteralPath $siteRoot -Recurse -Force
}
New-Item -ItemType Directory -Path $projectRoot | Out-Null
Copy-Item -Path (Join-Path $repoRoot 'dist/*') -Destination $projectRoot -Recurse -Force

if ($StageOnly) {
    Write-Host "Staged $projectRoot"
    return
}

Write-Host "Serving http://127.0.0.1:$Port/combinatorial-swarmability/"
if ($Background) {
    $pidPath = Join-Path $repoRoot 'target/local-server.pid'
    $stdoutPath = Join-Path $repoRoot 'target/local-server.stdout.log'
    $stderrPath = Join-Path $repoRoot 'target/local-server.stderr.log'
    $server = Start-Process -FilePath 'python' `
        -ArgumentList @('-m', 'http.server', $Port, '--bind', '127.0.0.1', '--directory', $siteRoot) `
        -WorkingDirectory $repoRoot `
        -WindowStyle Hidden `
        -RedirectStandardOutput $stdoutPath `
        -RedirectStandardError $stderrPath `
        -PassThru
    Set-Content -LiteralPath $pidPath -Value $server.Id -Encoding ascii
    Write-Host "Background server PID: $($server.Id)"
    return
}

& python -m http.server $Port --bind 127.0.0.1 --directory $siteRoot
