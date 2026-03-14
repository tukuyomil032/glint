use crate::cli::Language;
use crate::prism::CleanupTarget;
use crate::scanner::dir_size;
use anyhow::{Context, Result};
use dialoguer::{Confirm, theme::ColorfulTheme};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct CleanEntry {
    pub label: String,
    pub path: String,
    pub bytes: u64,
    pub action: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CleanSummary {
    pub dry_run: bool,
    pub total_candidates: usize,
    pub total_bytes: u64,
    pub cleaned_bytes: u64,
    pub entries: Vec<CleanEntry>,
}

pub fn run_clean(
    root: &Path,
    targets: &[CleanupTarget],
    dry_run: bool,
    yes: bool,
    lang: Language,
) -> Result<CleanSummary> {
    let theme = ColorfulTheme::default();

    if !dry_run && !yes {
        let approved = Confirm::with_theme(&theme)
            .with_prompt(match lang {
                Language::En => "Proceed with cleanup? (targets are moved to trash)",
                Language::Ja => "削除を実行しますか？(対象はゴミ箱へ移動)",
            })
            .default(false)
            .interact()
            .context(match lang {
                Language::En => "failed to read confirmation input",
                Language::Ja => "確認入力の読み取りに失敗しました",
            })?;

        if !approved {
            return Ok(CleanSummary {
                dry_run,
                total_candidates: 0,
                total_bytes: 0,
                cleaned_bytes: 0,
                entries: Vec::new(),
            });
        }
    }

    let mut entries = Vec::new();
    let mut total_bytes = 0_u64;
    let mut cleaned_bytes = 0_u64;

    for target in targets {
        if !target.path.exists() {
            continue;
        }

        let bytes = dir_size(&target.path);
        total_bytes += bytes;

        let mut entry = CleanEntry {
            label: target.label.clone(),
            path: target.path.display().to_string(),
            bytes,
            action: if dry_run {
                "dry-run".to_string()
            } else {
                "trash".to_string()
            },
            success: true,
            message: String::new(),
        };

        if !is_within_root(root, &target.path) {
            entry.success = false;
            entry.message = match lang {
                Language::En => "path is outside PrismLauncher root".to_string(),
                Language::Ja => "root外のパスは処理不可".to_string(),
            };
            entries.push(entry);
            continue;
        }

        if dry_run {
            entry.message = match lang {
                Language::En => "scheduled for deletion".to_string(),
                Language::Ja => "削除予定".to_string(),
            };
            cleaned_bytes += bytes;
            entries.push(entry);
            continue;
        }

        match trash::delete(&target.path) {
            Ok(_) => {
                entry.message = match lang {
                    Language::En => "moved to trash".to_string(),
                    Language::Ja => "ゴミ箱へ移動".to_string(),
                };
                cleaned_bytes += bytes;
            }
            Err(err) => {
                entry.success = false;
                entry.message = match lang {
                    Language::En => format!("failed: {err}"),
                    Language::Ja => format!("削除失敗: {err}"),
                };
            }
        }

        entries.push(entry);
    }

    Ok(CleanSummary {
        dry_run,
        total_candidates: entries.len(),
        total_bytes,
        cleaned_bytes,
        entries,
    })
}

fn is_within_root(root: &Path, path: &Path) -> bool {
    let root = match root.canonicalize() {
        Ok(path) => path,
        Err(_) => return false,
    };

    let path = match path.canonicalize() {
        Ok(path) => path,
        Err(_) => return false,
    };

    path.starts_with(root)
}
