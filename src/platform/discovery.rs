use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
#[cfg(not(target_os = "macos"))]
use sysinfo::DiskKind;
use sysinfo::Disks;

use crate::core::types::{DeviceAvailability, DeviceInfo};

pub trait DeviceDiscovery {
    fn discover(&self) -> Result<Vec<DeviceInfo>>;
}

#[derive(Debug, Default, Clone)]
pub struct SystemDeviceDiscovery;

impl DeviceDiscovery for SystemDeviceDiscovery {
    fn discover(&self) -> Result<Vec<DeviceInfo>> {
        Ok(discover_devices())
    }
}

pub fn discover_devices() -> Vec<DeviceInfo> {
    let mut devices = BTreeMap::new();
    let disks = Disks::new_with_refreshed_list();
    for disk in disks.iter().filter(|disk| should_include_disk(disk)) {
        let device = build_device_info(
            disk.name().to_string_lossy().as_ref(),
            disk.mount_point().to_path_buf(),
        );
        devices.insert(device.id.clone(), device);
    }

    #[cfg(target_os = "macos")]
    {
        for device in discover_macos_volume_devices() {
            devices.insert(device.id.clone(), device);
        }
    }

    devices.into_values().collect()
}

pub fn validate_selected_device(
    device: &DeviceInfo,
    visible_devices: &[DeviceInfo],
) -> Result<DeviceInfo> {
    let fresh = visible_devices
        .iter()
        .find(|candidate| candidate.id == device.id);
    match fresh {
        Some(candidate) if candidate.is_available() => Ok(candidate.clone()),
        Some(candidate) => Ok(candidate.clone()),
        None => Ok(DeviceInfo {
            availability: DeviceAvailability::Unavailable(
                "device is no longer connected or readable".to_string(),
            ),
            ..device.clone()
        }),
    }
}

fn normalize_device_name(raw_name: &str, mount_path: &Path) -> String {
    let trimmed = raw_name.trim();
    if !trimmed.is_empty() {
        return trimmed.to_string();
    }

    mount_path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(platform_device_fallback_name)
}

fn build_device_info(raw_name: &str, mount_path: PathBuf) -> DeviceInfo {
    let name = normalize_device_name(raw_name, &mount_path);
    let availability = if mount_path.exists() && mount_path.is_dir() {
        DeviceAvailability::Available
    } else {
        DeviceAvailability::Unavailable("device mount path is unavailable".to_string())
    };

    DeviceInfo {
        id: mount_path.display().to_string(),
        display_name: name,
        mount_path,
        availability,
    }
}

fn should_include_disk(disk: &sysinfo::Disk) -> bool {
    #[cfg(target_os = "macos")]
    {
        disk.mount_point().starts_with("/Volumes")
            && disk.mount_point() != Path::new("/Volumes")
            && disk.mount_point().is_dir()
    }

    #[cfg(not(target_os = "macos"))]
    {
        disk.kind() != DiskKind::Unknown(-1) && disk.is_removable()
    }
}

#[cfg(target_os = "macos")]
fn discover_macos_volume_devices() -> Vec<DeviceInfo> {
    discover_macos_volume_devices_from_root(Path::new("/Volumes"))
}

#[cfg(target_os = "macos")]
fn discover_macos_volume_devices_from_root(root: &Path) -> Vec<DeviceInfo> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };

    let mut devices = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .map(|path| {
            let raw_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_string();
            build_device_info(&raw_name, path)
        })
        .collect::<Vec<_>>();

    devices.sort_by(|left, right| left.display_name.cmp(&right.display_name));
    devices
}

fn platform_device_fallback_name() -> String {
    #[cfg(target_os = "macos")]
    {
        return "External Volume".to_string();
    }
    #[cfg(target_os = "windows")]
    {
        return "Removable Drive".to_string();
    }
    #[cfg(target_os = "linux")]
    {
        return "Removable Media".to_string();
    }
    #[allow(unreachable_code)]
    "Device".to_string()
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn marks_missing_device_as_unavailable() {
        let device = DeviceInfo {
            id: "a".to_string(),
            display_name: "EOS R6".to_string(),
            mount_path: PathBuf::from("/tmp/example"),
            availability: DeviceAvailability::Available,
        };

        let validated = validate_selected_device(&device, &[]).unwrap();
        assert!(!validated.is_available());
    }

    #[test]
    fn builds_device_info_from_mount_path() {
        let device = build_device_info("", PathBuf::from("/Volumes/NIKON Z 6_2"));
        assert_eq!(device.display_name, "NIKON Z 6_2");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn discovers_macos_volumes_from_directory_entries() {
        let root = tempdir().unwrap();
        fs::create_dir(root.path().join("NIKON Z 6_2")).unwrap();
        fs::create_dir(root.path().join("SD_CARD")).unwrap();

        let devices = discover_macos_volume_devices_from_root(root.path());

        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].display_name, "NIKON Z 6_2");
        assert_eq!(devices[1].display_name, "SD_CARD");
    }
}
