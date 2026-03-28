use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
#[cfg(any(target_os = "windows", target_os = "linux"))]
use sysinfo::DiskKind;
use sysinfo::Disks;

use crate::core::types::{DeviceAvailability, DeviceInfo};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceEjectOutcome {
    Ejected,
    Unsupported(String),
    Failed(String),
}

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

pub fn eject_device(device: &DeviceInfo) -> DeviceEjectOutcome {
    #[cfg(target_os = "macos")]
    {
        return run_command(
            "diskutil",
            &[String::from("eject"), device.mount_path.display().to_string()],
        );
    }

    #[cfg(target_os = "linux")]
    {
        return run_command("umount", &[device.mount_path.display().to_string()]);
    }

    #[cfg(target_os = "windows")]
    {
        let mount = windows_mount_identifier(&device.mount_path);
        return run_command(
            "powershell",
            &[
                String::from("-NoProfile"),
                String::from("-Command"),
                format!(
                    "(New-Object -comObject Shell.Application).Namespace(17).ParseName('{}').InvokeVerb('Eject')",
                    mount
                ),
            ],
        );
    }

    #[allow(unreachable_code)]
    DeviceEjectOutcome::Unsupported("safe eject is not supported on this platform".to_string())
}

pub fn discover_devices() -> Vec<DeviceInfo> {
    let mut devices = BTreeMap::new();
    let disks = Disks::new_with_refreshed_list();
    debug_discovery_log(format!("sysinfo disks: {}", disks.len()));
    for disk in disks.iter().filter(|disk| should_include_disk(disk)) {
        debug_discovery_log(format!(
            "sysinfo accepted: name={} mount={} removable={}",
            disk.name().to_string_lossy(),
            disk.mount_point().display(),
            disk.is_removable()
        ));
        let device = build_device_info(
            disk.name().to_string_lossy().as_ref(),
            disk.mount_point().to_path_buf(),
        );
        devices.insert(device.id.clone(), device);
    }

    #[cfg(target_os = "linux")]
    {
        for device in discover_linux_mount_devices() {
            debug_discovery_log(format!(
                "linux fallback accepted: name={} mount={}",
                device.display_name,
                device.mount_path.display()
            ));
            devices.insert(device.id.clone(), device);
        }
    }

    #[cfg(target_os = "macos")]
    {
        for device in discover_macos_volume_devices() {
            devices.insert(device.id.clone(), device);
        }
    }

    devices.into_values().collect()
}

fn debug_discovery_log(message: impl AsRef<str>) {
    let Ok(path) = std::env::var("SORTED_DISCOVERY_LOG") else {
        return;
    };

    let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };

    let _ = writeln!(file, "{}", message.as_ref());
}

fn run_command(program: &str, args: &[String]) -> DeviceEjectOutcome {
    match Command::new(program).args(args).output() {
        Ok(output) if output.status.success() => DeviceEjectOutcome::Ejected,
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let detail = if !stderr.is_empty() {
                stderr
            } else if !stdout.is_empty() {
                stdout
            } else {
                format!("{program} exited with status {}", output.status)
            };
            DeviceEjectOutcome::Failed(detail)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            DeviceEjectOutcome::Unsupported(format!("{program} is not available on this system"))
        }
        Err(error) => DeviceEjectOutcome::Failed(error.to_string()),
    }
}

#[cfg(target_os = "windows")]
fn windows_mount_identifier(path: &Path) -> String {
    path.display()
        .to_string()
        .trim_end_matches(['\\', '/'])
        .to_string()
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
        return should_include_macos_disk(disk.mount_point());
    }

    #[cfg(target_os = "linux")]
    {
        return should_include_linux_disk(disk.is_removable(), disk.mount_point());
    }

    #[cfg(target_os = "windows")]
    {
        return should_include_windows_disk(disk.kind(), disk.is_removable(), disk.mount_point());
    }

    #[allow(unreachable_code)]
    {
        false
    }
}

fn has_browsable_mount_point(mount_point: &Path) -> bool {
    mount_point.is_dir()
}

#[cfg(target_os = "macos")]
fn should_include_macos_disk(mount_point: &Path) -> bool {
    mount_point.starts_with("/Volumes")
        && mount_point != Path::new("/Volumes")
        && has_browsable_mount_point(mount_point)
}

#[cfg(target_os = "linux")]
fn should_include_linux_disk(is_removable: bool, mount_point: &Path) -> bool {
    is_removable && has_browsable_mount_point(mount_point)
}

#[cfg(target_os = "linux")]
fn discover_linux_mount_devices() -> Vec<DeviceInfo> {
    discover_linux_mount_devices_with(Path::new("/proc/mounts"), linux_mount_device_is_removable)
}

#[cfg(target_os = "linux")]
fn discover_linux_mount_devices_with<F>(mounts_path: &Path, is_removable: F) -> Vec<DeviceInfo>
where
    F: Fn(&str) -> bool,
{
    let Ok(contents) = fs::read_to_string(mounts_path) else {
        debug_discovery_log(format!(
            "linux fallback: could not read mounts from {}",
            mounts_path.display()
        ));
        return Vec::new();
    };

    contents
        .lines()
        .filter_map(parse_linux_mount_entry)
        .filter(|(source, mount_path)| {
            let device_name = linux_mount_device_name(source);
            let removable = device_name.as_deref().is_some_and(&is_removable);
            debug_discovery_log(format!(
                "linux fallback candidate: source={} mount={} device={} browsable={} removable={}",
                source,
                mount_path.display(),
                device_name.as_deref().unwrap_or("<unknown>"),
                has_browsable_mount_point(mount_path),
                removable
            ));
            source.starts_with("/dev/") && has_browsable_mount_point(mount_path) && removable
        })
        .map(|(_, mount_path)| {
            let raw_name = mount_path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_string();
            build_device_info(&raw_name, mount_path)
        })
        .collect()
}

#[cfg(target_os = "linux")]
fn parse_linux_mount_entry(line: &str) -> Option<(String, PathBuf)> {
    let mut fields = line.split_whitespace();
    let source = decode_mount_field(fields.next()?);
    let mount_path = PathBuf::from(decode_mount_field(fields.next()?));
    Some((source, mount_path))
}

#[cfg(target_os = "linux")]
fn decode_mount_field(value: &str) -> String {
    value
        .replace("\\040", " ")
        .replace("\\011", "\t")
        .replace("\\012", "\n")
        .replace("\\134", "\\")
}

#[cfg(target_os = "linux")]
fn linux_mount_device_is_removable(device_name: &str) -> bool {
    fs::read_to_string(Path::new("/sys/block").join(device_name).join("removable"))
        .ok()
        .map(|value| value.trim() == "1")
        .unwrap_or(false)
}

#[cfg(target_os = "linux")]
fn linux_mount_device_name(source: &str) -> Option<String> {
    let device = Path::new(source).file_name()?.to_str()?;
    Some(linux_parent_block_device_name(device))
}

#[cfg(target_os = "linux")]
fn linux_parent_block_device_name(device: &str) -> String {
    if let Some(prefix) = device.strip_suffix(|ch: char| ch.is_ascii_digit()) {
        if !prefix.is_empty() && !prefix.ends_with('p') {
            return prefix.to_string();
        }
    }

    if let Some((prefix, _)) = device.rsplit_once('p') {
        if prefix.chars().last().is_some_and(|ch| ch.is_ascii_digit()) {
            return prefix.to_string();
        }
    }

    device.to_string()
}

#[cfg(target_os = "windows")]
fn should_include_windows_disk(kind: DiskKind, is_removable: bool, mount_point: &Path) -> bool {
    kind != DiskKind::Unknown(-1) && is_removable && has_browsable_mount_point(mount_point)
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

    #[cfg(target_os = "linux")]
    #[test]
    fn includes_linux_removable_mount_even_with_unknown_disk_kind() {
        let mount = tempdir().unwrap();

        assert!(should_include_linux_disk(true, mount.path()));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn excludes_linux_mount_when_not_removable() {
        let mount = tempdir().unwrap();

        assert!(!should_include_linux_disk(false, mount.path()));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn discovers_linux_removable_mounts_from_proc_mounts_fallback() {
        let root = tempdir().unwrap();
        let mount_path = root.path().join("share/external/DEV3301_1");
        fs::create_dir_all(&mount_path).unwrap();
        let mounts = root.path().join("mounts");
        fs::write(
            &mounts,
            format!(
                "/dev/sdd1 {} exfat rw,relatime 0 0\n/dev/md9 /mnt/HDA_ROOT ext4 rw 0 0\n",
                mount_path.display()
            ),
        )
        .unwrap();

        let devices =
            discover_linux_mount_devices_with(&mounts, |device_name| device_name == "sdd");

        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].mount_path, mount_path);
        assert_eq!(devices[0].display_name, "DEV3301_1");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn parses_linux_parent_block_device_names() {
        assert_eq!(linux_parent_block_device_name("sdd1"), "sdd");
        assert_eq!(linux_parent_block_device_name("mmcblk0p1"), "mmcblk0");
        assert_eq!(linux_parent_block_device_name("nvme0n1p1"), "nvme0n1");
        assert_eq!(linux_parent_block_device_name("sdd"), "sdd");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn excludes_windows_unknown_disk_kind() {
        let mount = tempdir().unwrap();

        assert!(!should_include_windows_disk(
            DiskKind::Unknown(-1),
            true,
            mount.path()
        ));
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

    #[test]
    fn reports_missing_eject_command_as_unsupported() {
        let outcome = run_command("__sorted_missing_command__", &[]);

        assert!(matches!(outcome, DeviceEjectOutcome::Unsupported(_)));
    }
}
