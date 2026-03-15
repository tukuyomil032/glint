use super::{
    instance_allowed, CleanupStat, CleanupSummary, DuplicateModEntry, DuplicateModsSummary,
    InstanceUsage, UsageSummary, WorldBreakdownItem, WorldStat, WorldsSummary,
};
use rayon::prelude::*;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn scan_cleanup_targets(root: &Path, targets: &[crate::prism::CleanupTarget]) -> CleanupSummary {
    let mut entries: Vec<CleanupStat> = targets
        .par_iter()
        .map(|target| CleanupStat {
            kind: target.kind.clone(),
            label: target.label.clone(),
            path: target.path.clone(),
            bytes: dir_size(&target.path),
        })
        .collect();

    entries.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    let total_bytes = entries.iter().map(|entry| entry.bytes).sum();

    CleanupSummary {
        root: root.to_path_buf(),
        entries,
        total_bytes,
    }
}

pub fn scan_duplicate_mods(root: &Path) -> DuplicateModsSummary {
    scan_duplicate_mods_scoped(root, None)
}

pub fn scan_duplicate_mods_scoped(
    root: &Path,
    selected_instances: Option<&HashSet<String>>,
) -> DuplicateModsSummary {
    let mut jar_files: Vec<(String, PathBuf)> = Vec::new();
    let instances_dir = root.join("instances");

    if let Ok(entries) = fs::read_dir(&instances_dir) {
        for entry in entries.flatten() {
            let instance_name = entry.file_name().to_string_lossy().to_string();
            if !instance_allowed(&instance_name, selected_instances) {
                continue;
            }

            let mods_dir = entry.path().join(".minecraft/mods");
            if !mods_dir.exists() {
                continue;
            }

            for mod_entry in WalkDir::new(mods_dir)
                .min_depth(1)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
            {
                let path = mod_entry.path();
                if path.extension().is_some_and(|ext| ext == "jar") {
                    jar_files.push((instance_name.clone(), path.to_path_buf()));
                }
            }
        }
    }

    let hashed: Vec<(String, String, String, u64, PathBuf)> = jar_files
        .par_iter()
        .filter_map(|(instance, path)| {
            let bytes = fs::metadata(path).ok()?.len();
            let data = fs::read(path).ok()?;
            let hash = blake3::hash(&data).to_hex().to_string();
            let mod_name = path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown.jar".to_string());
            Some((hash, instance.clone(), mod_name, bytes, path.clone()))
        })
        .collect();

    let mut grouped: HashMap<String, Vec<(String, String, u64, PathBuf)>> = HashMap::new();
    for (hash, instance, mod_name, bytes, path) in hashed {
        grouped
            .entry(hash)
            .or_default()
            .push((instance, mod_name, bytes, path));
    }

    let mut duplicates = Vec::new();
    let mut potential_reclaim_bytes = 0_u64;

    for (hash, values) in grouped {
        if values.len() <= 1 {
            continue;
        }

        let bytes = values[0].2;
        let mod_name = values[0].1.clone();
        let mut instances: Vec<String> = values.iter().map(|v| v.0.clone()).collect();
        instances.sort();
        instances.dedup();

        let mut paths: Vec<PathBuf> = values.iter().map(|v| v.3.clone()).collect();
        paths.sort();

        potential_reclaim_bytes += bytes.saturating_mul((values.len() - 1) as u64);

        duplicates.push(DuplicateModEntry {
            hash,
            mod_name,
            bytes,
            instances,
            paths,
        });
    }

    duplicates.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    let duplicate_groups = duplicates.len();

    DuplicateModsSummary {
        root: root.to_path_buf(),
        duplicates,
        duplicate_groups,
        potential_reclaim_bytes,
    }
}

pub fn scan_world_sizes_scoped_with_breakdown(
    root: &Path,
    selected_instances: Option<&HashSet<String>>,
    include_breakdown: bool,
) -> WorldsSummary {
    let mut world_dirs: Vec<(String, String, PathBuf)> = Vec::new();
    let instances_dir = root.join("instances");

    if let Ok(entries) = fs::read_dir(&instances_dir) {
        for entry in entries.flatten() {
            let instance = entry.file_name().to_string_lossy().to_string();
            if !instance_allowed(&instance, selected_instances) {
                continue;
            }

            let saves_dir = entry.path().join(".minecraft/saves");
            if !saves_dir.exists() {
                continue;
            }

            if let Ok(worlds) = fs::read_dir(&saves_dir) {
                for world in worlds.flatten() {
                    let world_path = world.path();
                    let is_world_dir = world_path.is_dir()
                        || fs::metadata(&world_path)
                            .map(|meta| meta.is_dir())
                            .unwrap_or(false);
                    if is_world_dir {
                        let world_name = world.file_name().to_string_lossy().to_string();
                        world_dirs.push((instance.clone(), world_name, world_path));
                    }
                }
            }
        }
    }

    let mut worlds: Vec<WorldStat> = world_dirs
        .par_iter()
        .map(|(instance, world, path)| WorldStat {
            instance: instance.clone(),
            world: world.clone(),
            path: path.clone(),
            bytes: dir_size(path),
            breakdown: if include_breakdown {
                world_breakdown(path)
            } else {
                Vec::new()
            },
        })
        .collect();

    worlds.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    let total_world_bytes = worlds.iter().map(|world| world.bytes).sum();

    WorldsSummary {
        root: root.to_path_buf(),
        worlds,
        total_world_bytes,
    }
}

pub fn scan_instance_usage(root: &Path) -> UsageSummary {
    scan_instance_usage_scoped(root, None)
}

pub fn scan_instance_usage_scoped(
    root: &Path,
    selected_instances: Option<&HashSet<String>>,
) -> UsageSummary {
    let mut instances = Vec::new();
    let instances_dir = root.join("instances");

    if let Ok(entries) = fs::read_dir(&instances_dir) {
        instances = entries
            .flatten()
            .filter(|entry| entry.path().is_dir())
            .map(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                let path = entry.path();
                (name, path)
            })
            .filter(|(name, _)| instance_allowed(name, selected_instances))
            .collect();
    }

    let mut rows: Vec<InstanceUsage> = instances
        .par_iter()
        .map(|(instance, path)| InstanceUsage {
            instance: instance.clone(),
            path: path.clone(),
            bytes: dir_size(path),
        })
        .collect();

    rows.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    let total_bytes = rows.iter().map(|row| row.bytes).sum();

    UsageSummary {
        root: root.to_path_buf(),
        instances: rows,
        total_bytes,
    }
}

fn world_breakdown(world_path: &Path) -> Vec<WorldBreakdownItem> {
    let mut buckets: BTreeMap<String, u64> = BTreeMap::new();

    let Ok(entries) = fs::read_dir(world_path) else {
        return Vec::new();
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let bytes = if path.is_dir() {
            dir_size(&path)
        } else {
            fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
        };
        if bytes == 0 {
            continue;
        }

        let bucket = match name.as_str() {
            "region" | "playerdata" | "poi" | "data" | "entities" | "advancements"
            | "stats" => name.clone(),
            "DIM-1" | "DIM1" | "dimensions" => name.clone(),
            _ if name.starts_with("DIM") => name.clone(),
            _ => "other".to_string(),
        };

        *buckets.entry(bucket).or_insert(0) += bytes;
    }

    let mut items: Vec<WorldBreakdownItem> = buckets
        .into_iter()
        .map(|(bucket, bytes)| WorldBreakdownItem { bucket, bytes })
        .collect();
    items.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    items
}

pub fn dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }

    WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| entry.metadata().ok().map(|meta| meta.len()))
        .sum()
}
