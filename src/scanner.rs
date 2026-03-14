use crate::prism::CleanupTarget;
use log::warn;
use rayon::prelude::*;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize)]
pub struct CleanupStat {
    pub kind: String,
    pub label: String,
    pub path: PathBuf,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CleanupSummary {
    pub root: PathBuf,
    pub entries: Vec<CleanupStat>,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DuplicateModEntry {
    pub hash: String,
    pub mod_name: String,
    pub bytes: u64,
    pub instances: Vec<String>,
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DuplicateModsSummary {
    pub root: PathBuf,
    pub duplicates: Vec<DuplicateModEntry>,
    pub duplicate_groups: usize,
    pub potential_reclaim_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorldStat {
    pub instance: String,
    pub world: String,
    pub path: PathBuf,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorldsSummary {
    pub root: PathBuf,
    pub worlds: Vec<WorldStat>,
    pub total_world_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstanceUsage {
    pub instance: String,
    pub path: PathBuf,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageSummary {
    pub root: PathBuf,
    pub instances: Vec<InstanceUsage>,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnusedLibrary {
    pub relative_path: String,
    pub path: PathBuf,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnusedLibrariesSummary {
    pub root: PathBuf,
    pub candidates: Vec<UnusedLibrary>,
    pub total_bytes: u64,
    pub referenced_files: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnusedAsset {
    pub hash: String,
    pub path: PathBuf,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnusedAssetsSummary {
    pub root: PathBuf,
    pub candidates: Vec<UnusedAsset>,
    pub total_bytes: u64,
    pub referenced_hashes: usize,
}

pub fn scan_cleanup_targets(root: &Path, targets: &[CleanupTarget]) -> CleanupSummary {
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
    let mut jar_files: Vec<(String, PathBuf)> = Vec::new();
    let instances_dir = root.join("instances");

    if let Ok(entries) = fs::read_dir(&instances_dir) {
        for entry in entries.flatten() {
            let instance_name = entry.file_name().to_string_lossy().to_string();
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

pub fn scan_world_sizes(root: &Path) -> WorldsSummary {
    let mut world_dirs: Vec<(String, String, PathBuf)> = Vec::new();
    let instances_dir = root.join("instances");

    if let Ok(entries) = fs::read_dir(&instances_dir) {
        for entry in entries.flatten() {
            let instance = entry.file_name().to_string_lossy().to_string();
            let saves_dir = entry.path().join(".minecraft/saves");
            if !saves_dir.exists() {
                continue;
            }

            if let Ok(worlds) = fs::read_dir(&saves_dir) {
                for world in worlds.flatten() {
                    let world_path = world.path();
                    if world_path.is_dir() {
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

pub fn dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }

    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| entry.metadata().ok().map(|meta| meta.len()))
        .sum()
}

pub fn scan_unused_libraries(root: &Path) -> UnusedLibrariesSummary {
    let libraries_root = root.join("libraries");
    let mut referenced_rel_paths: HashSet<String> = HashSet::new();

    let json_roots = [root.join("meta"), root.join("instances")];
    for scan_root in json_roots {
        if !scan_root.exists() {
            continue;
        }

        for entry in WalkDir::new(scan_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
        {
            let Ok(content) = fs::read_to_string(entry.path()) else {
                continue;
            };
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
                continue;
            };
            extract_library_paths_from_json(&value, &mut referenced_rel_paths);
        }
    }

    if referenced_rel_paths.is_empty() {
        warn!("no library references were discovered; skipping unused-library candidates");
        return UnusedLibrariesSummary {
            root: root.to_path_buf(),
            candidates: Vec::new(),
            total_bytes: 0,
            referenced_files: 0,
        };
    }

    let mut candidates: Vec<UnusedLibrary> = Vec::new();
    if libraries_root.exists() {
        candidates = WalkDir::new(&libraries_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter_map(|entry| {
                let path = entry.path();
                let rel = path.strip_prefix(&libraries_root).ok()?;
                let rel_norm = rel.to_string_lossy().replace('\\', "/");
                if referenced_rel_paths.contains(&rel_norm) {
                    return None;
                }
                let bytes = entry.metadata().ok()?.len();
                Some(UnusedLibrary {
                    relative_path: rel_norm,
                    path: path.to_path_buf(),
                    bytes,
                })
            })
            .collect();
    }

    candidates.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    let total_bytes = candidates.iter().map(|entry| entry.bytes).sum();

    UnusedLibrariesSummary {
        root: root.to_path_buf(),
        candidates,
        total_bytes,
        referenced_files: referenced_rel_paths.len(),
    }
}

pub fn scan_unused_assets(root: &Path) -> UnusedAssetsSummary {
    let indexes_dir = root.join("assets/indexes");
    let objects_dir = root.join("assets/objects");
    let mut used_hashes = HashSet::new();

    if indexes_dir.exists() {
        for entry in WalkDir::new(&indexes_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
        {
            let Ok(content) = fs::read_to_string(entry.path()) else {
                continue;
            };
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
                continue;
            };
            extract_asset_hashes(&value, &mut used_hashes);
        }
    }

    if used_hashes.is_empty() {
        warn!("no asset hashes were discovered; skipping unused-asset candidates");
        return UnusedAssetsSummary {
            root: root.to_path_buf(),
            candidates: Vec::new(),
            total_bytes: 0,
            referenced_hashes: 0,
        };
    }

    let mut candidates: Vec<UnusedAsset> = Vec::new();
    if objects_dir.exists() {
        candidates = WalkDir::new(&objects_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter_map(|entry| {
                let path = entry.path();
                let hash = path.file_name()?.to_string_lossy().to_string();
                if used_hashes.contains(&hash) {
                    return None;
                }
                let bytes = entry.metadata().ok()?.len();
                Some(UnusedAsset {
                    hash,
                    path: path.to_path_buf(),
                    bytes,
                })
            })
            .collect();
    }

    candidates.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    let total_bytes = candidates.iter().map(|entry| entry.bytes).sum();

    UnusedAssetsSummary {
        root: root.to_path_buf(),
        candidates,
        total_bytes,
        referenced_hashes: used_hashes.len(),
    }
}

pub fn cleanup_targets_from_unused_libraries(
    summary: &UnusedLibrariesSummary,
    max_candidates: usize,
) -> Vec<CleanupTarget> {
    summary
        .candidates
        .iter()
        .take(max_candidates)
        .map(|entry| CleanupTarget {
            kind: "advanced".to_string(),
            label: format!("unused-library/{}", entry.relative_path),
            path: entry.path.clone(),
        })
        .collect()
}

pub fn cleanup_targets_from_unused_assets(
    summary: &UnusedAssetsSummary,
    max_candidates: usize,
) -> Vec<CleanupTarget> {
    summary
        .candidates
        .iter()
        .take(max_candidates)
        .map(|entry| CleanupTarget {
            kind: "advanced".to_string(),
            label: format!("unused-asset/{}", entry.hash),
            path: entry.path.clone(),
        })
        .collect()
}

fn extract_library_paths_from_json(value: &serde_json::Value, out: &mut HashSet<String>) {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(artifact_path) = map
                .get("downloads")
                .and_then(|downloads| downloads.get("artifact"))
                .and_then(|artifact| artifact.get("path"))
                .and_then(serde_json::Value::as_str)
            {
                out.insert(artifact_path.replace('\\', "/"));
            }

            if let Some(path) = map.get("path").and_then(serde_json::Value::as_str)
                && path.ends_with(".jar")
                && path.contains('/')
            {
                out.insert(path.replace('\\', "/"));
            }

            for child in map.values() {
                extract_library_paths_from_json(child, out);
            }
        }
        serde_json::Value::Array(arr) => {
            for child in arr {
                extract_library_paths_from_json(child, out);
            }
        }
        _ => {}
    }
}

fn extract_asset_hashes(value: &serde_json::Value, out: &mut HashSet<String>) {
    if let Some(objects) = value.get("objects").and_then(serde_json::Value::as_object) {
        for object in objects.values() {
            if let Some(hash) = object.get("hash").and_then(serde_json::Value::as_str) {
                out.insert(hash.to_string());
            }
        }
    }
}
