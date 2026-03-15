# AGENTS.md

## 1. Purpose

This document provides instructions specifically for AI coding agents (e.g., Devin, Cursor, Copilot agents).

Before performing any work in this repository, the agent MUST read this file to understand:

- project architecture
- development rules
- safety constraints
- filesystem assumptions
- workflow expectations

The goal is to ensure agents make safe and correct changes to the project.

This document is written to be readable by both humans and AI agents.

---

# 2. Project Overview

**Project Name**

luma

**Description**

luma is a high-performance Rust CLI tool that analyzes and optimizes PrismLauncher storage usage.

It scans the PrismLauncher filesystem and identifies unnecessary or duplicate data.

### Primary Features

| Feature | Description |
|------|-------------|
| Storage Scan | Analyze disk usage |
| Cache Cleanup | Remove launcher cache |
| Log Cleanup | Remove logs |
| Unused Library Detection | Detect unused Minecraft libraries |
| Duplicate Mod Detection | Find identical mods across instances |
| World Size Analyzer | Detect large worlds |
| Asset Deduplication | Remove unused assets |
| Modpack Cache Cleanup | Remove cached downloads |

luma must **never modify PrismLauncher configuration files**.

It only analyzes and cleans filesystem data.

---

# 3. Supported Platforms

| OS | Support |
|----|--------|
| Windows | Supported |
| macOS | Supported |

Linux support may be added later.

---

# 4. PrismLauncher Filesystem Layout

luma operates on the PrismLauncher data directory.

### Windows

```
%APPDATA%/PrismLauncher
```

### macOS

```
~/Library/Application Support/PrismLauncher
```

### Important Directories

| Directory | Description |
|-----------|-------------|
| instances/ | Minecraft instances |
| libraries/ | Minecraft runtime libraries |
| assets/ | Minecraft asset storage |
| cache/ | Download cache |
| meta/ | Metadata cache |
| logs/ | Launcher logs |
| java/ | Managed Java installations |
| icons/ | Instance icons |

Agents must **never assume directories outside this root**.

---

# 5. Instance Structure

Instances exist under:

```
instances/<instance_name>/
```

Important files:

```
instance.cfg
mmc-pack.json
.minecraft/
```

Inside `.minecraft`:

```
mods/
config/
saves/
logs/
resourcepacks/
shaderpacks/
crash-reports/
```

### Protected Directories

The following directories must **never be deleted automatically**:

| Directory | Reason |
|----------|--------|
| mods | Installed mods |
| config | Mod configs |
| saves | World saves |
| resourcepacks | User assets |

---

# 6. Safe Cleanup Targets

The following directories are safe to clean:

| Path | Description |
|-----|-------------|
| cache/ | Download cache |
| logs/ | Launcher logs |
| meta/ | Metadata cache |
| instances/*/.minecraft/logs | Game logs |
| instances/*/.minecraft/crash-reports | Crash reports |

---

# 7. Duplicate Mod Detection

Goal: detect identical mods across instances.

Algorithm:

1. scan all `mods/` directories
2. hash each `.jar`
3. group identical hashes
4. report duplicates

Example:

```
Mod: Sodium.jar

Instances:
FabricPack
VanillaPlus
TechPack
```

---

# 8. World Size Analyzer

Scan:

```
instances/*/.minecraft/saves
```

Compute directory sizes.

Example output:

```
Instance: TechPack
World: survival_world
Size: 3.4GB
```

---

# 9. Asset Deduplication

Assets stored in:

```
assets/objects/
```

Algorithm:

1. read asset index files
2. collect referenced hashes
3. detect orphan files
4. mark unused assets

Agents must verify asset usage before deletion.

---

# 10. CLI Design

Main command:

```
luma
```

### Subcommands

| Command | Description |
|--------|-------------|
| scan | analyze disk usage |
| clean | remove unnecessary files |
| mods | analyze duplicate mods |
| worlds | analyze world sizes |
| usage | show instance usage |

Example usage:

```
luma scan
luma clean --dry-run
luma mods
luma worlds
```

---

# 11. CLI User Experience

luma should provide clear terminal feedback.

Example spinner output:

```
Scanning instances...
⠋ hashing mods
⠋ scanning worlds
⠋ scanning libraries
```

Recommended crates:

| Crate | Purpose |
|------|---------|
| indicatif | spinners / progress bars |
| console | terminal styling |
| colored | colored text |
| tabled | table output |

---

# 12. Performance Requirements

Large installations may contain:

- 300k+ files
- 50GB+ data

Use parallel scanning.

Recommended crates:

| Crate | Usage |
|------|------|
| rayon | parallel processing |
| walkdir | filesystem traversal |

Parallel tasks:

- instance scanning
- mod hashing
- library scanning
- world size calculation

---

# 13. Safety Rules

luma must never cause data loss.

Mandatory safety features:

| Rule | Description |
|-----|-------------|
| dry-run | default mode |
| confirmation | required before deletion |
| trash support | move to system trash |

Rust crate:

```
trash
```

---

# 14. Project Structure

```
src/

main.rs

cli/
  commands.rs
  args.rs

scanner/
  instances.rs
  mods.rs
  libraries.rs
  assets.rs
  worlds.rs

analysis/
  duplicates.rs
  sizes.rs

cleaner/
  delete.rs

ui/
  spinners.rs
  tables.rs
```

---

# 15. Development Workflow

1. create feature branch from `main`
2. implement feature
3. update documentation
4. open Draft PR

Agents must **never push directly to main**.

---

# 16. Pull Request Rules

PR title format:

```
[luma] <feature description>
```

Example:

```
[luma] add duplicate mod detection
```

Requirements:

- must be Draft PR
- assign repository owner
- do not mark ready for review automatically

Agents must wait for instructions.

---

# 17. Code Style

Language: Rust

Requirements:

- Rust 2021 edition
- idiomatic Rust
- avoid unsafe
- modular architecture

Tools:

```
cargo fmt
cargo clippy
```

---

# 18. Testing

Tests should cover:

- duplicate mod detection
- world size calculation
- library scanning

Run tests with:

```
cargo test
```

Target coverage:

```
>80%
```

---

# 19. Related Agent Config Files

Agents may also reference:

```
.rules
.cursorrules
.windsurf
.mdc
```

AGENTS.md remains the primary instruction source.
