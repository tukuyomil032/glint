$ErrorActionPreference = 'Stop'

$binDir = if ($env:GLINT_BIN_DIR) { $env:GLINT_BIN_DIR } else { Join-Path $env:LOCALAPPDATA 'glint\bin' }
$target = Join-Path $binDir 'glint.exe'

if (Test-Path $target) {
    Remove-Item -Force $target
    Write-Host "Removed: $target"
}
else {
    Write-Host "Not found: $target"
}

Write-Host 'Uninstall complete.'
