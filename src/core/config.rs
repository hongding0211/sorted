use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, anyhow, bail};
use directories::{ProjectDirs, UserDirs};
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
        Ok(ArchiveSettings {
            destination_root: resolve_destination_root(&parsed.settings.destination_root)?,
            date_format: parsed.settings.date_format,
        })
    }

    pub fn save(&self, settings: &ArchiveSettings) -> Result<()> {
        let (settings, _) = validate_settings(settings)?;
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create config directory {}", parent.display())
            })?;
        }

        let document = PersistedConfig {
            version: CONFIG_VERSION,
            settings,
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

pub fn validate_settings(settings: &ArchiveSettings) -> Result<(ArchiveSettings, DatePreview)> {
    let destination_root = validate_destination_root(&settings.destination_root)?;
    let date_preview = validate_date_format(&settings.date_format)?;
    Ok((
        ArchiveSettings {
            destination_root,
            date_format: settings.date_format.clone(),
        },
        date_preview,
    ))
}

pub fn resolve_destination_root(path: &Path) -> Result<PathBuf> {
    let raw = path.to_string_lossy();
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        bail!("destination root cannot be empty");
    }

    if trimmed == "~" || trimmed.starts_with("~/") || trimmed.starts_with("~\\") {
        let user_dirs =
            UserDirs::new().ok_or_else(|| anyhow!("unable to resolve home directory"))?;
        let relative = trimmed
            .strip_prefix("~/")
            .or_else(|| trimmed.strip_prefix("~\\"))
            .unwrap_or("");
        let mut resolved = user_dirs.home_dir().to_path_buf();
        if !relative.is_empty() {
            resolved.push(relative);
        }
        return Ok(resolved);
    }

    Ok(PathBuf::from(trimmed))
}

pub fn validate_destination_root(path: &Path) -> Result<PathBuf> {
    let resolved = resolve_destination_root(path)?;

    if resolved.exists() {
        if !resolved.is_dir() {
            bail!("destination root {} is not a directory", resolved.display());
        }
        validate_directory_writable(&resolved)?;
        return Ok(resolved);
    }

    let ancestor = nearest_existing_ancestor(&resolved).ok_or_else(|| {
        anyhow!(
            "destination root {} cannot be created because no parent directory exists",
            resolved.display()
        )
    })?;

    if !ancestor.is_dir() {
        bail!(
            "destination root {} cannot be created because parent {} is not a directory",
            resolved.display(),
            ancestor.display()
        );
    }

    validate_directory_writable(&ancestor).with_context(|| {
        format!(
            "destination root {} cannot be created from parent {}",
            resolved.display(),
            ancestor.display()
        )
    })?;

    Ok(resolved)
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

fn nearest_existing_ancestor(path: &Path) -> Option<PathBuf> {
    let mut current = path.to_path_buf();
    while !current.exists() {
        if !current.pop() {
            return None;
        }
    }
    Some(current)
}

fn validate_directory_writable(path: &Path) -> Result<()> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let probe = path.join(format!(".sorted-write-test-{}-{nanos}", std::process::id()));
    fs::create_dir(&probe)
        .with_context(|| format!("destination root {} is not writable", path.display()))?;
    fs::remove_dir(&probe)
        .with_context(|| format!("failed to clean up temporary probe in {}", path.display()))?;
    Ok(())
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

    #[test]
    fn resolves_tilde_destination_root() {
        let resolved = resolve_destination_root(Path::new("~/Desktop/temp")).unwrap();
        let expected = UserDirs::new().unwrap().home_dir().join("Desktop/temp");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn accepts_missing_destination_root_when_parent_is_creatable() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("new-root");

        let resolved = validate_destination_root(&target).unwrap();

        assert_eq!(resolved, target);
        assert!(!resolved.exists());
    }

    #[test]
    fn rejects_destination_root_that_is_a_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("target-file");
        fs::write(&file_path, "not a directory").unwrap();

        let error = validate_destination_root(&file_path).unwrap_err();

        assert!(error.to_string().contains("is not a directory"));
    }

    #[test]
    fn saves_resolved_destination_root_in_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let store = ConfigStore::from_path(config_path.clone());
        let home = UserDirs::new().unwrap().home_dir().to_path_buf();
        let suffix = format!(
            "sorted-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        let settings = ArchiveSettings {
            destination_root: PathBuf::from(format!("~/{suffix}")),
            date_format: "%Y-%m-%d".to_string(),
        };

        store.save(&settings).unwrap();
        let raw = fs::read_to_string(config_path).unwrap();

        assert!(raw.contains(&home.join(&suffix).display().to_string()));
    }
}
