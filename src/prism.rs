use anyhow::{Context, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct CleanupTarget {
    pub kind: String,
    pub label: String,
    pub path: PathBuf,
}

pub fn resolve_root(explicit: Option<PathBuf>) -> Result<PathBuf> {
    let root = match explicit {
        Some(path) => path,
        None => default_prism_root().context("failed to resolve default PrismLauncher root")?,
    };

    let normalized = root
        .canonicalize()
        .with_context(|| format!("failed to resolve path: {}", root.display()))?;

    Ok(normalized)
}

pub fn default_prism_root() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir()?;
        return Some(home.join("Library/Application Support/PrismLauncher"));
    }

    #[cfg(target_os = "windows")]
    {
        let roaming = dirs::config_dir()?;
        return Some(roaming.join("PrismLauncher"));
    }

    #[allow(unreachable_code)]
    None
}

pub fn collect_cleanup_targets(root: &Path) -> Vec<CleanupTarget> {
    let mut targets = Vec::new();

    let push_if_exists =
        |targets: &mut Vec<CleanupTarget>, kind: &str, label: &str, path: PathBuf| {
            if path.exists() {
                targets.push(CleanupTarget {
                    kind: kind.to_string(),
                    label: label.to_string(),
                    path,
                });
            }
        };

    push_if_exists(&mut targets, "global", "cache", root.join("cache"));
    push_if_exists(&mut targets, "global", "logs", root.join("logs"));
    push_if_exists(&mut targets, "global", "meta", root.join("meta"));
    push_if_exists(&mut targets, "global", "catpacks", root.join("catpacks"));

    let instances_dir = root.join("instances");
    if let Ok(entries) = std::fs::read_dir(instances_dir) {
        for entry in entries.flatten() {
            let instance_path = entry.path();
            if !instance_path.is_dir() {
                continue;
            }

            let instance_name = entry.file_name().to_string_lossy().to_string();
            let mc = instance_path.join(".minecraft");
            push_if_exists(
                &mut targets,
                "instance",
                &format!("{instance_name}/logs"),
                mc.join("logs"),
            );
            push_if_exists(
                &mut targets,
                "instance",
                &format!("{instance_name}/crash-reports"),
                mc.join("crash-reports"),
            );
        }
    }

    targets
}

pub fn list_instances(root: &Path) -> Vec<String> {
    let instances_dir = root.join("instances");
    let Ok(entries) = std::fs::read_dir(instances_dir) else {
        return Vec::new();
    };

    let mut names: Vec<String> = entries
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    names
}
