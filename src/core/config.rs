use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow, bail};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::core::types::{ArchiveSettings, DatePreview};

const APP_QUALIFIER: &str = "com";
const APP_ORGANIZATION: &str = "sorted";
const APP_NAME: &str = "sorted";
const CONFIG_VERSION: u32 = 1;
const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Clone)]
pub struct ConfigStore {
    config_path: PathBuf,
}

impl ConfigStore {
    pub fn new() -> Result<Self> {
        let config_path = default_config_path()?;
        Ok(Self { config_path })
    }

    pub fn from_path(path: PathBuf) -> Self {
        Self { config_path: path }
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    pub fn load(&self) -> Result<ArchiveSettings> {
        if !self.config_path.exists() {
            return Ok(ArchiveSettings::default());
        }

        let raw = fs::read_to_string(&self.config_path)
            .with_context(|| format!("failed to read config at {}", self.config_path.display()))?;
        let parsed: PersistedConfig = toml::from_str(&raw)
            .with_context(|| format!("failed to parse config at {}", self.config_path.display()))?;
        Ok(parsed.settings)
    }

    pub fn save(&self, settings: &ArchiveSettings) -> Result<()> {
        validate_settings(settings)?;
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create config directory {}", parent.display())
            })?;
        }

        let document = PersistedConfig {
            version: CONFIG_VERSION,
            settings: settings.clone(),
        };
        let serialized = toml::to_string_pretty(&document).context("failed to serialize config")?;
        fs::write(&self.config_path, serialized)
            .with_context(|| format!("failed to write config at {}", self.config_path.display()))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedConfig {
    version: u32,
    settings: ArchiveSettings,
}

pub fn default_config_path() -> Result<PathBuf> {
    let project_dirs = ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, APP_NAME)
        .ok_or_else(|| anyhow!("unable to resolve a platform config directory"))?;
    Ok(project_dirs.config_dir().join(CONFIG_FILE))
}

pub fn validate_settings(settings: &ArchiveSettings) -> Result<DatePreview> {
    if settings.destination_root.as_os_str().is_empty() {
        bail!("destination root cannot be empty");
    }

    if !settings.destination_root.exists() {
        bail!(
            "destination root {} does not exist",
            settings.destination_root.display()
        );
    }

    if !settings.destination_root.is_dir() {
        bail!(
            "destination root {} is not a directory",
            settings.destination_root.display()
        );
    }

    validate_date_format(&settings.date_format)
}

pub fn validate_destination_root(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty() {
        bail!("destination root cannot be empty");
    }
    if !path.exists() {
        bail!("destination root {} does not exist", path.display());
    }
    if !path.is_dir() {
        bail!("destination root {} is not a directory", path.display());
    }
    Ok(())
}

pub fn validate_date_format(pattern: &str) -> Result<DatePreview> {
    if pattern.trim().is_empty() {
        bail!("date format cannot be empty");
    }

    let mut chars = pattern.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '%' {
            continue;
        }

        let Some(next) = chars.next() else {
            bail!("date format cannot end with a trailing %");
        };

        if next == '%' {
            continue;
        }

        if !is_supported_specifier(next) {
            bail!("unsupported date format specifier: %{next}");
        }
    }

    Ok(DatePreview::now(pattern))
}

fn is_supported_specifier(ch: char) -> bool {
    matches!(
        ch,
        'Y' | 'y' | 'm' | 'b' | 'B' | 'd' | 'e' | 'H' | 'M' | 'S' | 'F' | 'R' | 'T'
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn validates_supported_date_format() {
        let preview = validate_date_format("%Y-%m-%d").unwrap();
        assert_eq!(preview.pattern, "%Y-%m-%d");
        assert!(!preview.preview.is_empty());
    }

    #[test]
    fn rejects_unknown_date_format_specifier() {
        let error = validate_date_format("%Q").unwrap_err();
        assert!(error.to_string().contains("unsupported"));
    }

    #[test]
    fn saves_and_loads_archive_settings() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let store = ConfigStore::from_path(config_path);
        let settings = ArchiveSettings {
            destination_root: dir.path().to_path_buf(),
            date_format: "%Y-%m-%d".to_string(),
        };

        store.save(&settings).unwrap();
        let loaded = store.load().unwrap();

        assert_eq!(loaded, settings);
    }
}
