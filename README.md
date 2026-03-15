<div align="center">
  <h1>glint</h1>
  <p><strong>Fast PrismLauncher storage analysis and safe cleanup CLI</strong></p>

  <p>
    <a href="https://github.com/tukuyomil032/glint/actions/workflows/ci.yml"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/tukuyomil032/glint/ci.yml?branch=main&label=CI"></a>
    <a href="https://github.com/tukuyomil032/glint/actions/workflows/release.yml"><img alt="Release" src="https://img.shields.io/github/actions/workflow/status/tukuyomil032/glint/release.yml?branch=main&label=release"></a>
    <a href="https://github.com/tukuyomil032/glint/releases"><img alt="Downloads" src="https://img.shields.io/github/downloads/tukuyomil032/glint/total"></a>
    <a href="./LICENSE"><img alt="License" src="https://img.shields.io/badge/license-MIT-green"></a>
    <a href="https://www.rust-lang.org/"><img alt="Rust" src="https://img.shields.io/badge/rust-2024-orange"></a>
  </p>
</div>

glint scans PrismLauncher data and helps you reclaim space safely.

It focuses on:
- safe cleanup targets (`cache`, `logs`, `meta`, instance logs/crash reports)
- duplicate mod detection
- world size analysis
- per-instance usage summaries
- optional candidates for unused libraries/assets

By default, cleanup runs in dry-run mode and deletion uses the system trash.

## Features

- Fast parallel scanning (`rayon` + `walkdir`)
- English/Japanese output switch via `glint config`
- Interactive instance selection for `scan`
- Paged scan report viewer
- World breakdown mode (`region`, `playerdata`, `poi`, etc.)
- Clean preview filtering by kind/size/age and optional interactive candidate selection

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Commands](#commands)
- [Release Automation](#release-automation)
- [Safety](#safety)

## Installation

### 1) One-command installer (macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/tukuyomil032/glint/main/install.sh | sh
```

Options:

- Pin a version: `GLINT_VERSION=0.1.0 ...`
- Change install directory: `GLINT_BIN_DIR=$HOME/bin ...`

Local test before push:

```bash
cargo build --release
tar -C target/release -czf /tmp/glint-local-macos.tar.gz glint
cat install.sh | GLINT_BIN_DIR=/tmp/glint-test/bin GLINT_ASSET_URL=file:///tmp/glint-local-macos.tar.gz sh
```

### 2) One-command installer (Windows PowerShell)

```powershell
iwr -useb https://raw.githubusercontent.com/tukuyomil032/glint/main/install.ps1 | iex
```

Options:

- Pin a version: `$env:GLINT_VERSION='0.1.0'`
- Change install directory: `$env:GLINT_BIN_DIR='C:\\tools\\glint\\bin'`

Local test before push:

```powershell
cargo build --release
Compress-Archive -Path .\target\release\glint.exe -DestinationPath $env:TEMP\glint-local-win.zip -Force
$env:GLINT_ASSET_URL = "file:///$($env:TEMP -replace '\\','/')/glint-local-win.zip"
Get-Content .\install.ps1 -Raw | Invoke-Expression
```

### 3) GitHub Releases binaries

Download prebuilt binaries from:

- https://github.com/tukuyomil032/glint/releases

### 4) cargo install

From git (works now):

```bash
cargo install --git https://github.com/tukuyomil032/glint glint
```

From crates.io (after publish):

```bash
cargo install glint
```

### 5) Build from source

```bash
git clone https://github.com/tukuyomil032/glint
cd glint
cargo build --release
```

## Uninstall

### macOS / Linux shell

```bash
cat scripts/uninstall.sh | GLINT_BIN_DIR=$HOME/.local/bin sh
```

### Windows PowerShell

```powershell
Get-Content .\scripts\uninstall.ps1 -Raw | Invoke-Expression
```

## Quick Start

```bash
# Analyze reclaimable storage
./target/release/glint scan

# Show worlds with breakdown of large buckets
./target/release/glint worlds --breakdown

# Dry-run clean with preview filters and interactive selection
./target/release/glint clean --dry-run --kind global --min-size 200MB --older-than-days 30 --select

# Apply cleanup (moves files to trash)
./target/release/glint clean --apply -y
```

## Commands

- `glint scan`
- `glint clean`
- `glint mods`
- `glint worlds`
- `glint usage`
- `glint config`

Useful clean options:

- `--kind <kind>` (repeatable: `global`, `instance`, `advanced`)
- `--min-size <size>` (e.g. `500MB`, `2GB`)
- `--older-than-days <days>`
- `--select` (interactive candidate selection)

## Release Automation

This repository includes automated release flow in:

- [.github/workflows/release.yml](.github/workflows/release.yml)

Behavior:

1. Push to `main`
2. Workflow reads `version` from `Cargo.toml`
3. If tag `v<version>` does not exist, it creates and pushes it
4. Builds binaries for macOS (x86_64/aarch64) and Windows (x86_64)
5. Publishes GitHub Release with attached archives and `SHA256SUMS.txt`

That means your release workflow is driven by `Cargo.toml` version only.

## Safety

glint is designed to avoid accidental data loss.

- Dry-run is the default cleanup mode
- Cleanup confirmation is required unless `-y` is set
- Deletions are sent to system trash, not hard-deleted
- PrismLauncher root bounds are checked before cleanup
