[CmdletBinding()]
param(
    [switch]$Release
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$toolchain = '1.97.1'
$wasmBindgenVersion = '0.2.105'
$profile = if ($Release) { 'release' } else { 'debug' }
$buildArguments = @(
    "+$toolchain",
    'build',
    '--locked',
    '--target',
    'wasm32-unknown-unknown',
    '-p',
    'combinatorial-swarmability-demo-wasm'
)
if ($Release) { $buildArguments += '--release' }
$toolRoot = Join-Path $repoRoot 'target/wasm-tools'
$executableName = if ($IsWindows) { 'wasm-bindgen.exe' } else { 'wasm-bindgen' }
$wasmBindgen = Join-Path $toolRoot "bin/$executableName"
$wasmArtifact = Join-Path $repoRoot "target/wasm32-unknown-unknown/$profile/combinatorial_swarmability_demo_wasm.wasm"
$distRoot = Join-Path $repoRoot 'dist'

function Assert-RepoChild([string]$Path) {
    $fullPath = [System.IO.Path]::GetFullPath($Path)
    $prefix = $repoRoot.TrimEnd([System.IO.Path]::DirectorySeparatorChar) + [System.IO.Path]::DirectorySeparatorChar
    if (-not $fullPath.StartsWith($prefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to modify a path outside the repository: $fullPath"
    }
}

Push-Location $repoRoot
try {
    & cargo @buildArguments
    if ($LASTEXITCODE -ne 0) { throw 'The Wasm crate build failed.' }

    $installedVersion = if (Test-Path -LiteralPath $wasmBindgen) {
        (& $wasmBindgen --version 2>$null)
    } else {
        ''
    }
    if ($installedVersion -ne "wasm-bindgen $wasmBindgenVersion") {
        & cargo "+$toolchain" install wasm-bindgen-cli --version $wasmBindgenVersion --root $toolRoot --locked --force
        if ($LASTEXITCODE -ne 0) { throw 'The repository-local wasm-bindgen CLI install failed.' }
    }

    Assert-RepoChild $distRoot
    if (Test-Path -LiteralPath $distRoot) {
        Remove-Item -LiteralPath $distRoot -Recurse -Force
    }
    New-Item -ItemType Directory -Path $distRoot | Out-Null
    Copy-Item -Path (Join-Path $repoRoot 'web/*') -Destination $distRoot -Recurse -Force

    $packageRoot = Join-Path $distRoot 'pkg'
    New-Item -ItemType Directory -Path $packageRoot | Out-Null
    & $wasmBindgen $wasmArtifact --target web --out-dir $packageRoot --out-name demo_wasm --no-typescript
    if ($LASTEXITCODE -ne 0) { throw 'wasm-bindgen packaging failed.' }

    Write-Host "Built $distRoot with wasm-bindgen $wasmBindgenVersion ($profile)."
}
finally {
    Pop-Location
}
