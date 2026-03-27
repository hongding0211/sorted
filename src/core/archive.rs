use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use chrono::{DateTime, Local};

use crate::core::{
    config::{validate_date_format, validate_destination_root},
    types::{ArchivePlan, ArchiveSettings, DeviceInfo},
};

pub fn build_archive_plan(
    settings: &ArchiveSettings,
    theme: &str,
    device: &DeviceInfo,
    now: DateTime<Local>,
) -> Result<ArchivePlan> {
    let destination_root = validate_destination_root(&settings.destination_root)?;
    validate_date_format(&settings.date_format)?;

    if !device.is_available() {
        bail!("selected device is unavailable");
    }
    if !device.mount_path.exists() {
        bail!(
            "selected device mount path {} is missing",
            device.mount_path.display()
        );
    }

    let normalized_theme = normalize_path_component(theme);
    if normalized_theme.is_empty() {
        bail!("theme must contain at least one valid path character");
    }

    let normalized_device = normalize_path_component(&device.display_name);
    if normalized_device.is_empty() {
        bail!("device name must contain at least one valid path character");
    }

    let date_segment = now.format(&settings.date_format).to_string();
    let archive_root = destination_root
        .join(format!("{normalized_theme}_{date_segment}"))
        .join(&normalized_device);

    Ok(ArchivePlan {
        theme_segment: normalized_theme,
        date_segment,
        device_segment: normalized_device,
        destination_root,
        archive_root,
    })
}

pub fn normalize_path_component(input: &str) -> String {
    let mut normalized = String::with_capacity(input.len());

    for ch in input.trim().chars() {
        let safe = match ch {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            c if c.is_control() => '_',
            c => c,
        };
        normalized.push(safe);
    }

    normalized
        .split_whitespace()
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("_")
        .trim_matches('.')
        .trim()
        .to_string()
}

pub fn destination_preview(plan: &ArchivePlan) -> String {
    plan.archive_root.display().to_string()
}

pub fn is_destination_writable(path: &Path) -> bool {
    path.exists() && path.is_dir()
}

pub fn ensure_archive_root(path: &PathBuf) -> Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Local, TimeZone};
    use tempfile::tempdir;

    use crate::core::types::{ArchiveSettings, DeviceAvailability, DeviceInfo};

    #[test]
    fn normalizes_unsafe_path_characters() {
        assert_eq!(normalize_path_component("EOS:R6/Primary"), "EOS_R6_Primary");
    }

    #[test]
    fn replaces_spaces_with_underscores() {
        assert_eq!(
            normalize_path_component("EOS R6 Main Card"),
            "EOS_R6_Main_Card"
        );
    }

    #[test]
    fn builds_expected_archive_path() {
        let root = tempdir().unwrap();
        let settings = ArchiveSettings {
            destination_root: root.path().to_path_buf(),
            date_format: "%Y-%m-%d".to_string(),
        };
        let device = DeviceInfo {
            id: "cam".to_string(),
            display_name: "EOS R6".to_string(),
            mount_path: root.path().to_path_buf(),
            availability: DeviceAvailability::Available,
        };

        let plan = build_archive_plan(
            &settings,
            "xxx travel",
            &device,
            Local.with_ymd_and_hms(2026, 3, 27, 10, 0, 0).unwrap(),
        )
        .unwrap();

        assert_eq!(
            plan.archive_root,
            root.path().join("xxx_travel_2026-03-27").join("EOS_R6")
        );
    }

    #[test]
    fn resolves_tilde_destination_root_in_archive_plan() {
        let home = directories::UserDirs::new()
            .unwrap()
            .home_dir()
            .to_path_buf();
        let mount_root = tempdir().unwrap();
        let suffix = format!(
            "sorted-archive-plan-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        let settings = ArchiveSettings {
            destination_root: PathBuf::from(format!("~/{suffix}")),
            date_format: "%Y-%m-%d".to_string(),
        };
        let device = DeviceInfo {
            id: "cam".to_string(),
            display_name: "EOS R6".to_string(),
            mount_path: mount_root.path().to_path_buf(),
            availability: DeviceAvailability::Available,
        };

        let plan = build_archive_plan(
            &settings,
            "xxx travel",
            &device,
            Local.with_ymd_and_hms(2026, 3, 27, 10, 0, 0).unwrap(),
        )
        .unwrap();

        assert_eq!(plan.destination_root, home.join(&suffix));
        assert_eq!(
            plan.archive_root,
            home.join(&suffix)
                .join("xxx_travel_2026-03-27")
                .join("EOS_R6")
        );
    }
}
