$ErrorActionPreference = 'Stop'

$binDir = if ($env:LUMA_BIN_DIR) { $env:LUMA_BIN_DIR } else { Join-Path $env:LOCALAPPDATA 'luma-prism\bin' }
$target = Join-Path $binDir 'luma.exe'

if (Test-Path $target) {
    Remove-Item -Force $target
    Write-Host "Removed: $target"
}
else {
    Write-Host "Not found: $target"
}

Write-Host 'Uninstall complete.'
