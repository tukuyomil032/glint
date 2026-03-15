$ErrorActionPreference = 'Stop'

$repo = if ($env:LUMA_REPO) { $env:LUMA_REPO } else { 'tukuyomil032/luma-prism' }
$versionInput = if ($env:LUMA_VERSION) { $env:LUMA_VERSION } else { 'latest' }
$binDir = if ($env:LUMA_BIN_DIR) { $env:LUMA_BIN_DIR } else { Join-Path $env:LOCALAPPDATA 'luma-prism\bin' }
$assetUrlOverride = if ($env:LUMA_ASSET_URL) { $env:LUMA_ASSET_URL } else { '' }

$target = 'x86_64-pc-windows-msvc'
$ext = 'zip'

if ($assetUrlOverride) {
    $url = $assetUrlOverride
    $asset = [System.IO.Path]::GetFileName($url)
}
else {
    if ($versionInput -eq 'latest') {
        $tag = $null
        try {
            $releaseApi = "https://api.github.com/repos/$repo/releases/latest"
            $release = Invoke-RestMethod -Uri $releaseApi
            $tag = $release.tag_name
        }
        catch {
            $tag = $null
        }

        if (-not $tag) {
            $tagsApi = "https://api.github.com/repos/$repo/tags"
            $tags = Invoke-RestMethod -Uri $tagsApi
            if ($tags -and $tags.Count -gt 0) {
                $tag = $tags[0].name
            }
        }

        if (-not $tag) {
            throw 'Failed to resolve latest release/tag.'
        }
    }
    elseif ($versionInput.StartsWith('v')) {
        $tag = $versionInput
    } else {
        $tag = "v$versionInput"
    }

    $asset = "luma-prism-$tag-$target.$ext"
    $url = "https://github.com/$repo/releases/download/$tag/$asset"
}

$tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ("luma-prism-install-" + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force -Path $tmpDir | Out-Null

try {
    New-Item -ItemType Directory -Force -Path $binDir | Out-Null

    $archivePath = Join-Path $tmpDir $asset
    Write-Host "Downloading $url"
    Invoke-WebRequest -Uri $url -OutFile $archivePath

    Expand-Archive -Path $archivePath -DestinationPath $tmpDir -Force

    $sourceExe = Join-Path $tmpDir 'luma.exe'
    if (-not (Test-Path $sourceExe)) {
        throw 'luma.exe was not found in downloaded archive.'
    }

    $destExe = Join-Path $binDir 'luma.exe'
    Copy-Item -Force $sourceExe $destExe

    Write-Host "Installed luma to $destExe"

    $pathItems = $env:Path -split ';'
    if ($pathItems -notcontains $binDir) {
        Write-Host "Add this directory to your PATH if needed: $binDir"
    }

    Write-Host 'Run: luma --help'
}
finally {
    Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
}
