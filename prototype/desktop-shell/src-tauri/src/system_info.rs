//! Read-only, local hardware inventory for the first HIARM baseline.
//! This intentionally does not enumerate processes, host names, network interfaces,
//! device serial numbers, or private user files.
use serde::Serialize;
use sysinfo::{CpuRefreshKind, Disks, System};

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HardwareDisk {
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HardwareSnapshot {
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub architecture: String,
    pub cpu_model: Option<String>,
    pub logical_core_count: usize,
    pub physical_core_count: Option<usize>,
    pub total_memory_bytes: u64,
    pub available_memory_bytes: u64,
    pub disks: Vec<HardwareDisk>,
    pub gpu_status: String,
}

/// Collects a one-time snapshot without inspecting running processes or user content.
pub fn collect_hardware_snapshot() -> HardwareSnapshot {
    let mut system = System::new();
    system.refresh_memory();
    system.refresh_cpu_list(CpuRefreshKind::everything());

    let cpu_model = system
        .cpus()
        .iter()
        .map(|cpu| cpu.brand().trim())
        .find(|brand| !brand.is_empty())
        .map(str::to_owned);

    let mut disks = Disks::new_with_refreshed_list()
        .iter()
        .map(|disk| HardwareDisk {
            mount_point: disk.mount_point().display().to_string(),
            total_bytes: disk.total_space(),
            available_bytes: disk.available_space(),
        })
        .collect::<Vec<_>>();
    disks.sort_by(|left, right| left.mount_point.cmp(&right.mount_point));

    HardwareSnapshot {
        os_name: System::name(),
        os_version: System::long_os_version().or_else(System::os_version),
        architecture: std::env::consts::ARCH.to_owned(),
        cpu_model,
        logical_core_count: system.cpus().len(),
        physical_core_count: System::physical_core_count(),
        total_memory_bytes: system.total_memory(),
        available_memory_bytes: system.available_memory(),
        disks,
        gpu_status: "not_detected_yet".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hardware_snapshot_is_serializable_and_contains_only_inventory_fields() {
        let snapshot = HardwareSnapshot {
            os_name: Some("Windows".to_owned()),
            os_version: Some("Windows 10".to_owned()),
            architecture: "x86_64".to_owned(),
            cpu_model: Some("Example CPU".to_owned()),
            logical_core_count: 8,
            physical_core_count: Some(4),
            total_memory_bytes: 16 * 1024 * 1024 * 1024,
            available_memory_bytes: 8 * 1024 * 1024 * 1024,
            disks: vec![HardwareDisk {
                mount_point: "C:\\".to_owned(),
                total_bytes: 512 * 1024 * 1024 * 1024,
                available_bytes: 256 * 1024 * 1024 * 1024,
            }],
            gpu_status: "not_detected_yet".to_owned(),
        };
        let value = serde_json::to_value(&snapshot).unwrap();
        assert_eq!(value["logicalCoreCount"], 8);
        assert_eq!(value["physicalCoreCount"], 4);
        assert_eq!(value["disks"][0]["availableBytes"], 256_u64 * 1024 * 1024 * 1024);
        assert!(value.get("hostName").is_none());
        assert!(value.get("processes").is_none());
        assert!(value.get("networkInterfaces").is_none());
    }

    #[test]
    fn hardware_snapshot_does_not_claim_gpu_detection() {
        let snapshot = HardwareSnapshot {
            os_name: None,
            os_version: None,
            architecture: "unknown".to_owned(),
            cpu_model: None,
            logical_core_count: 0,
            physical_core_count: None,
            total_memory_bytes: 0,
            available_memory_bytes: 0,
            disks: Vec::new(),
            gpu_status: "not_detected_yet".to_owned(),
        };
        assert_eq!(snapshot.gpu_status, "not_detected_yet");
    }
}
