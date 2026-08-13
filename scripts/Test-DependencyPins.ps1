[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$matterRevision = '54a18d85ee877f5cb6c8275013da8a76764b140e'
$expectedMatterSource = "git+https://github.com/MesmerPrism/rusty-matter.git?rev=$matterRevision#$matterRevision"

Push-Location $repoRoot
try {
    $metadata = (& cargo +1.97.1 metadata --locked --format-version 1 | ConvertFrom-Json)
    if ($LASTEXITCODE -ne 0) { throw 'cargo metadata failed.' }

    foreach ($packageName in @('rusty-matter-model', 'rusty-matter-particles')) {
        $package = @($metadata.packages | Where-Object name -eq $packageName)
        if ($package.Count -ne 1) {
            throw "Expected exactly one resolved $packageName package, found $($package.Count)."
        }
        if ($package[0].source -ne $expectedMatterSource) {
            throw "$packageName resolved to an unexpected source: $($package[0].source)"
        }
    }

    $manifest = Get-Content -LiteralPath (Join-Path $repoRoot 'Cargo.toml') -Raw
    if ($manifest -notmatch 'wasm-bindgen\s*=\s*"=0\.2\.105"') {
        throw 'The wasm-bindgen library version is not exact.'
    }
    $toolchainFile = Get-Content -LiteralPath (Join-Path $repoRoot 'rust-toolchain.toml') -Raw
    if ($toolchainFile -notmatch 'channel\s*=\s*"1\.97\.1"') {
        throw 'The ordinary Rust toolchain is not pinned to 1.97.1.'
    }
    if ($manifest -match '(?m)^\s*path\s*=') {
        throw 'Path dependencies are not permitted in the public package.'
    }

    Write-Host "Dependency pins verified: Rust 1.97.1, wasm-bindgen 0.2.105, Matter $matterRevision."
}
finally {
    Pop-Location
}
