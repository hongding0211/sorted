use std::fs;

use chrono::{Local, TimeZone};
use tempfile::tempdir;

use sorted::core::{
    copy::{execute_copy, plan_copy},
    types::{ArchiveSettings, DeviceAvailability, DeviceInfo},
};

#[test]
fn copy_plan_and_execution_report_failures_and_successes() {
    let device_root = tempdir().unwrap();
    let destination_root = tempdir().unwrap();
    fs::create_dir_all(device_root.path().join("DCIM")).unwrap();
    fs::write(device_root.path().join("DCIM").join("frame.jpg"), "image").unwrap();
    fs::write(device_root.path().join("DCIM").join("clip.mov"), "video").unwrap();

    let settings = ArchiveSettings {
        destination_root: destination_root.path().to_path_buf(),
        date_format: "%Y-%m-%d".to_string(),
    };
    let device = DeviceInfo {
        id: "device-1".to_string(),
        display_name: "Field Card".to_string(),
        mount_path: device_root.path().to_path_buf(),
        availability: DeviceAvailability::Available,
    };

    let plan = plan_copy(
        &settings,
        "winter shoot",
        None,
        &device,
        device_root.path(),
        Local.with_ymd_and_hms(2026, 3, 27, 12, 0, 0).unwrap(),
    )
    .unwrap();

    assert_eq!(plan.files.len(), 2);

    let summary = execute_copy(&plan, |_| {}, || false).unwrap();
    assert_eq!(summary.copied_files, 2);
    assert!(summary.failures.is_empty());
    assert!(!summary.was_cancelled);
}
