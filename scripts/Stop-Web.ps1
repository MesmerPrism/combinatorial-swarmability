[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$pidPath = Join-Path $repoRoot 'target/local-server.pid'
if (-not (Test-Path -LiteralPath $pidPath)) {
    Write-Host 'No recorded local server is running.'
    return
}

$serverPid = [int](Get-Content -LiteralPath $pidPath -Raw)
$process = Get-CimInstance Win32_Process -Filter "ProcessId = $serverPid"
if ($null -eq $process) {
    Remove-Item -LiteralPath $pidPath -Force
    Write-Host 'Removed a stale local-server PID file.'
    return
}
if ($process.Name -notmatch '^python(?:\.exe)?$' -or $process.CommandLine -notmatch 'http\.server') {
    throw "PID $serverPid is not the expected Python HTTP server; refusing to stop it."
}

Stop-Process -Id $serverPid -Force
Remove-Item -LiteralPath $pidPath -Force
Write-Host "Stopped local server PID $serverPid."
