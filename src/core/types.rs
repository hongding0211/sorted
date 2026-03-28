use std::path::PathBuf;

use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceAvailability {
    Available,
    Unavailable(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: String,
    pub display_name: String,
    pub mount_path: PathBuf,
    pub availability: DeviceAvailability,
}

impl DeviceInfo {
    pub fn is_available(&self) -> bool {
        matches!(self.availability, DeviceAvailability::Available)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchiveSettings {
    pub destination_root: PathBuf,
    pub date_format: String,
}

impl Default for ArchiveSettings {
    fn default() -> Self {
        Self {
            destination_root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            date_format: "%Y-%m-%d".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportSession {
    pub selected_device: Option<DeviceInfo>,
    pub selected_source: Option<PathBuf>,
    pub theme: String,
    pub device_directory_override: String,
    pub resolved_destination: Option<PathBuf>,
}

impl Default for ImportSession {
    fn default() -> Self {
        Self {
            selected_device: None,
            selected_source: None,
            theme: String::new(),
            device_directory_override: String::new(),
            resolved_destination: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchivePlan {
    pub theme_segment: String,
    pub date_segment: String,
    pub device_segment: String,
    pub destination_root: PathBuf,
    pub archive_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyPlan {
    pub source_device: DeviceInfo,
    pub source_root: PathBuf,
    pub archive_plan: ArchivePlan,
    pub files: Vec<MediaFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaFile {
    pub source_path: PathBuf,
    pub relative_path: PathBuf,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatePreview {
    pub pattern: String,
    pub preview: String,
}

impl DatePreview {
    pub fn now(pattern: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
            preview: Local::now().format(pattern).to_string(),
        }
    }
}
