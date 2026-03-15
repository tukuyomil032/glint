use crate::cli::Language;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub language: Language,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: Language::En,
        }
    }
}

pub fn load_config() -> Result<AppConfig> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read config: {}", path.display()))?;
    let cfg = serde_json::from_str::<AppConfig>(&content)
        .with_context(|| format!("failed to parse config: {}", path.display()))?;
    Ok(cfg)
}

pub fn save_config(cfg: &AppConfig) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config dir: {}", parent.display()))?;
    }

    let body = serde_json::to_string_pretty(cfg)?;
    fs::write(&path, body).with_context(|| format!("failed to write config: {}", path.display()))
}

pub fn config_path() -> Result<PathBuf> {
    let base = dirs::config_dir().context("failed to resolve config dir")?;
    Ok(base.join("glint").join("config.json"))
}
