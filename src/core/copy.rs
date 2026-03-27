use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use walkdir::WalkDir;

use crate::core::{
    archive::{build_archive_plan, ensure_archive_root},
    types::{ArchiveSettings, CopyPlan, DeviceInfo, MediaFile},
};

const MEDIA_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "bmp", "tif", "tiff", "heic", "raw", "cr2", "cr3", "nef", "arw",
    "dng", "raf", "mp4", "mov", "avi", "mkv", "mts", "m2ts",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyProgress {
    pub copied_files: usize,
    pub total_files: usize,
    pub current_file: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyFailure {
    pub file: PathBuf,
    pub error: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopySummary {
    pub destination: PathBuf,
    pub copied_files: usize,
    pub failures: Vec<CopyFailure>,
    pub was_cancelled: bool,
}

pub fn discover_media_files(root: &Path) -> Result<Vec<MediaFile>> {
    if !root.exists() {
        bail!("device mount path {} does not exist", root.display());
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        let extension = entry
            .path()
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase());

        let Some(extension) = extension else {
            continue;
        };
        if !MEDIA_EXTENSIONS.contains(&extension.as_str()) {
            continue;
        }

        let metadata = entry.metadata()?;
        let relative_path = entry
            .path()
            .strip_prefix(root)
            .with_context(|| {
                format!(
                    "failed to compute relative path for {}",
                    entry.path().display()
                )
            })?
            .to_path_buf();

        files.push(MediaFile {
            source_path: entry.path().to_path_buf(),
            relative_path,
            size_bytes: metadata.len(),
        });
    }

    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(files)
}

pub fn plan_copy(
    settings: &ArchiveSettings,
    theme: &str,
    device: &DeviceInfo,
    source_root: &Path,
    now: chrono::DateTime<chrono::Local>,
) -> Result<CopyPlan> {
    let archive_plan = build_archive_plan(settings, theme, device, now)?;
    validate_archive_destination_available(&archive_plan.archive_root)?;
    let files = discover_media_files(source_root)?;
    Ok(CopyPlan {
        source_device: device.clone(),
        source_root: source_root.to_path_buf(),
        archive_plan,
        files,
    })
}

pub fn archive_destination_exists(archive_root: &Path) -> bool {
    archive_root.exists()
}

fn validate_archive_destination_available(archive_root: &Path) -> Result<()> {
    if archive_destination_exists(archive_root) {
        bail!(
            "archive destination {} already exists",
            archive_root.display()
        );
    }

    Ok(())
}

pub fn execute_copy<F, C>(
    plan: &CopyPlan,
    mut on_progress: F,
    should_cancel: C,
) -> Result<CopySummary>
where
    F: FnMut(CopyProgress),
    C: Fn() -> bool,
{
    fs::create_dir_all(&plan.archive_plan.destination_root)?;
    ensure_archive_root(&plan.archive_plan.archive_root)?;

    let total_files = plan.files.len();
    let mut copied_files = 0usize;
    let mut failures = Vec::new();

    for file in &plan.files {
        if should_cancel() {
            return Ok(CopySummary {
                destination: plan.archive_plan.archive_root.clone(),
                copied_files,
                failures,
                was_cancelled: true,
            });
        }

        let destination = plan.archive_plan.archive_root.join(&file.relative_path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        match fs::copy(&file.source_path, &destination) {
            Ok(_) => {
                copied_files += 1;
                on_progress(CopyProgress {
                    copied_files,
                    total_files,
                    current_file: Some(destination),
                });
            }
            Err(error) => failures.push(CopyFailure {
                file: file.source_path.clone(),
                error: error.to_string(),
            }),
        }
    }

    Ok(CopySummary {
        destination: plan.archive_plan.archive_root.clone(),
        copied_files,
        failures,
        was_cancelled: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Local, TimeZone};
    use tempfile::tempdir;

    use crate::core::types::{ArchiveSettings, DeviceAvailability, DeviceInfo};

    #[test]
    fn discovers_supported_media_files() {
        let root = tempdir().unwrap();
        fs::write(root.path().join("frame.jpg"), "a").unwrap();
        fs::write(root.path().join("notes.txt"), "b").unwrap();

        let files = discover_media_files(root.path()).unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].relative_path, PathBuf::from("frame.jpg"));
    }

    #[test]
    fn copies_media_files_into_archive_root() {
        let device_root = tempdir().unwrap();
        let destination_root = tempdir().unwrap();
        let nested = device_root.path().join("DCIM");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("frame.jpg"), "image").unwrap();

        let settings = ArchiveSettings {
            destination_root: destination_root.path().to_path_buf(),
            date_format: "%Y-%m-%d".to_string(),
        };
        let device = DeviceInfo {
            id: "cam".to_string(),
            display_name: "EOS R6".to_string(),
            mount_path: device_root.path().to_path_buf(),
            availability: DeviceAvailability::Available,
        };

        let plan = plan_copy(
            &settings,
            "shoot",
            &device,
            device_root.path(),
            Local.with_ymd_and_hms(2026, 3, 27, 10, 0, 0).unwrap(),
        )
        .unwrap();
        let summary = execute_copy(&plan, |_| {}, || false).unwrap();

        assert_eq!(summary.copied_files, 1);
        assert!(summary.failures.is_empty());
        assert!(!summary.was_cancelled);
        assert!(summary.destination.join("DCIM/frame.jpg").exists());
    }

    #[test]
    fn plans_copy_from_selected_subdirectory_only() {
        let device_root = tempdir().unwrap();
        let destination_root = tempdir().unwrap();
        let dcim = device_root.path().join("DCIM");
        let misc = device_root.path().join("MISC");
        fs::create_dir_all(&dcim).unwrap();
        fs::create_dir_all(&misc).unwrap();
        fs::write(dcim.join("frame.jpg"), "image").unwrap();
        fs::write(misc.join("clip.mov"), "video").unwrap();

        let settings = ArchiveSettings {
            destination_root: destination_root.path().to_path_buf(),
            date_format: "%Y-%m-%d".to_string(),
        };
        let device = DeviceInfo {
            id: "cam".to_string(),
            display_name: "EOS R6".to_string(),
            mount_path: device_root.path().to_path_buf(),
            availability: DeviceAvailability::Available,
        };

        let plan = plan_copy(
            &settings,
            "shoot",
            &device,
            &dcim,
            Local.with_ymd_and_hms(2026, 3, 27, 10, 0, 0).unwrap(),
        )
        .unwrap();

        assert_eq!(plan.files.len(), 1);
        assert_eq!(plan.source_root, dcim);
        assert_eq!(plan.files[0].relative_path, PathBuf::from("frame.jpg"));
    }

    #[test]
    fn creates_missing_destination_root_before_copying() {
        let device_root = tempdir().unwrap();
        let base_root = tempdir().unwrap();
        let destination_root = base_root.path().join("missing-root");
        let nested = device_root.path().join("DCIM");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("frame.jpg"), "image").unwrap();

        let settings = ArchiveSettings {
            destination_root: destination_root.clone(),
            date_format: "%Y-%m-%d".to_string(),
        };
        let device = DeviceInfo {
            id: "cam".to_string(),
            display_name: "EOS R6".to_string(),
            mount_path: device_root.path().to_path_buf(),
            availability: DeviceAvailability::Available,
        };

        let plan = plan_copy(
            &settings,
            "shoot",
            &device,
            device_root.path(),
            Local.with_ymd_and_hms(2026, 3, 27, 10, 0, 0).unwrap(),
        )
        .unwrap();
        assert!(!destination_root.exists());

        let summary = execute_copy(&plan, |_| {}, || false).unwrap();

        assert!(destination_root.exists());
        assert!(!summary.was_cancelled);
        assert!(summary.destination.join("DCIM/frame.jpg").exists());
    }

    #[test]
    fn stops_copying_when_cancellation_is_requested() {
        let device_root = tempdir().unwrap();
        let destination_root = tempdir().unwrap();
        fs::write(device_root.path().join("a.jpg"), "a").unwrap();
        fs::write(device_root.path().join("b.jpg"), "b").unwrap();

        let settings = ArchiveSettings {
            destination_root: destination_root.path().to_path_buf(),
            date_format: "%Y-%m-%d".to_string(),
        };
        let device = DeviceInfo {
            id: "cam".to_string(),
            display_name: "EOS R6".to_string(),
            mount_path: device_root.path().to_path_buf(),
            availability: DeviceAvailability::Available,
        };

        let plan = plan_copy(
            &settings,
            "shoot",
            &device,
            device_root.path(),
            Local.with_ymd_and_hms(2026, 3, 27, 10, 0, 0).unwrap(),
        )
        .unwrap();

        let cancel_requested = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let cancel_for_progress = cancel_requested.clone();
        let cancel_for_check = cancel_requested.clone();
        let summary = execute_copy(
            &plan,
            move |_| {
                cancel_for_progress.store(true, std::sync::atomic::Ordering::SeqCst);
            },
            move || cancel_for_check.load(std::sync::atomic::Ordering::SeqCst),
        )
        .unwrap();

        assert_eq!(summary.copied_files, 1);
        assert!(summary.was_cancelled);
    }

    #[test]
    fn overwrites_existing_destination_file() {
        let device_root = tempdir().unwrap();
        let destination_root = tempdir().unwrap();
        let nested = device_root.path().join("DCIM");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("frame.jpg"), "new-image").unwrap();

        let settings = ArchiveSettings {
            destination_root: destination_root.path().to_path_buf(),
            date_format: "%Y-%m-%d".to_string(),
        };
        let device = DeviceInfo {
            id: "cam".to_string(),
            display_name: "EOS R6".to_string(),
            mount_path: device_root.path().to_path_buf(),
            availability: DeviceAvailability::Available,
        };

        let plan = plan_copy(
            &settings,
            "shoot",
            &device,
            device_root.path(),
            Local.with_ymd_and_hms(2026, 3, 27, 10, 0, 0).unwrap(),
        )
        .unwrap();
        fs::create_dir_all(summary_destination_root(&plan).join("DCIM")).unwrap();
        fs::write(
            summary_destination_root(&plan).join("DCIM/frame.jpg"),
            "old-image",
        )
        .unwrap();

        let summary = execute_copy(&plan, |_| {}, || false).unwrap();

        assert_eq!(summary.copied_files, 1);
        assert!(summary.failures.is_empty());
        assert_eq!(
            fs::read_to_string(summary.destination.join("DCIM/frame.jpg")).unwrap(),
            "new-image"
        );
    }

    #[test]
    fn rejects_existing_archive_destination() {
        let device_root = tempdir().unwrap();
        let destination_root = tempdir().unwrap();
        let source_root = device_root.path().join("DCIM");
        fs::create_dir_all(&source_root).unwrap();
        fs::write(source_root.join("frame.jpg"), "image").unwrap();

        let settings = ArchiveSettings {
            destination_root: destination_root.path().to_path_buf(),
            date_format: "%Y-%m-%d".to_string(),
        };
        let device = DeviceInfo {
            id: "cam".to_string(),
            display_name: "EOS R6".to_string(),
            mount_path: device_root.path().to_path_buf(),
            availability: DeviceAvailability::Available,
        };

        let archive_root = destination_root
            .path()
            .join("shoot_2026-03-27")
            .join("EOS_R6");
        fs::create_dir_all(&archive_root).unwrap();

        let plan = plan_copy(
            &settings,
            "shoot",
            &device,
            &source_root,
            Local.with_ymd_and_hms(2026, 3, 27, 10, 0, 0).unwrap(),
        );

        let error = plan.unwrap_err();

        assert!(error.to_string().contains("already exists"));
    }

    fn summary_destination_root(plan: &CopyPlan) -> PathBuf {
        plan.archive_plan.archive_root.clone()
    }
}
