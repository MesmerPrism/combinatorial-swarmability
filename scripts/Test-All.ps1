[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
Push-Location $repoRoot
try {
    & cargo +1.97.1 fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { throw 'rustfmt check failed.' }
    & cargo +1.97.1 test --workspace --locked
    if ($LASTEXITCODE -ne 0) { throw 'Rust 1.97.1 tests failed.' }
    & cargo +1.97.1 clippy --workspace --all-targets --locked -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'Clippy failed.' }
    & cargo +1.97.1 build --release --locked --target wasm32-unknown-unknown -p combinatorial-swarmability-demo-wasm
    if ($LASTEXITCODE -ne 0) { throw 'Wasm release build failed.' }
    & node --check web/app.js
    if ($LASTEXITCODE -ne 0) { throw 'Browser adapter syntax check failed.' }
    & (Join-Path $PSScriptRoot 'Test-AtlasShell.ps1')

    $previousTarget = $env:CARGO_TARGET_DIR
    try {
        $env:CARGO_TARGET_DIR = Join-Path $repoRoot 'target/msrv'
        & cargo +1.80.0 check --workspace --locked
        if ($LASTEXITCODE -ne 0) { throw 'Rust 1.80 check failed.' }
        & cargo +1.80.0 test --workspace --locked
        if ($LASTEXITCODE -ne 0) { throw 'Rust 1.80 tests failed.' }
    }
    finally {
        $env:CARGO_TARGET_DIR = $previousTarget
    }

    & (Join-Path $PSScriptRoot 'Test-DependencyPins.ps1')
    & (Join-Path $PSScriptRoot 'Build-Web.ps1') -Release
    & (Join-Path $PSScriptRoot 'Test-PublicBoundary.ps1')
    & (Join-Path $PSScriptRoot 'Artifact-Inventory.ps1')
    & git diff --check
    if ($LASTEXITCODE -ne 0) { throw 'Git whitespace validation failed.' }
    Write-Host 'All local non-browser gates passed.'
}
finally {
    Pop-Location
}
