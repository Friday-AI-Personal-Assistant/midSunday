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
pub struct CapabilityAssessment {
    pub suggested_mode: String,
    pub memory_tier: String,
    pub rationale: String,
    pub current_memory_pressure: bool,
    pub limitations: String,
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
    pub assessment: CapabilityAssessment,
}

/// Produces a conservative, non-binding starting profile from installed RAM and CPU topology.
/// It never selects a model, changes settings, or treats the result as a compatibility guarantee.
pub fn assess_capability(
    total_memory_bytes: u64,
    available_memory_bytes: u64,
    logical_core_count: usize,
) -> CapabilityAssessment {
    const GIB: u64 = 1024 * 1024 * 1024;
    let limitations = "GPU/VRAM, runtime acceleration, model-specific benchmarks, context size, and speech workloads are not assessed. This is a starting point, not a model compatibility guarantee.";

    if total_memory_bytes == 0 || logical_core_count == 0 {
        return CapabilityAssessment {
            suggested_mode: "Undetermined".to_owned(),
            memory_tier: "Hardware data incomplete".to_owned(),
            rationale: "midSunday could not read enough RAM or CPU information to suggest a starting profile. Refresh the hardware information and choose settings manually.".to_owned(),
            current_memory_pressure: false,
            limitations: limitations.to_owned(),
        };
    }

    let (memory_tier, mut suggested_mode, memory_reason) = if total_memory_bytes < 6 * GIB {
        ("Constrained", "Lightweight", "Installed RAM is below 6 GiB.")
    } else if total_memory_bytes < 12 * GIB {
        ("Entry", "Lightweight", "Installed RAM is between 6 and 12 GiB.")
    } else if total_memory_bytes < 24 * GIB {
        ("Balanced", "Balanced", "Installed RAM is between 12 and 24 GiB.")
    } else if total_memory_bytes < 48 * GIB {
        ("High-capacity", "High capability", "Installed RAM is between 24 and 48 GiB.")
    } else {
        ("Very high-capacity", "High capability", "Installed RAM is at least 48 GiB.")
    };

    let rationale = if logical_core_count < 4 {
        suggested_mode = "Lightweight";
        format!("{memory_reason} The CPU reports fewer than 4 logical processors, so Lightweight is the safer starting point.")
    } else if suggested_mode == "High capability" && logical_core_count < 8 {
        suggested_mode = "Balanced";
        format!("{memory_reason} The CPU reports fewer than 8 logical processors, so Balanced is the more cautious starting point.")
    } else {
        format!("{memory_reason} The detected CPU reports {logical_core_count} logical processors.")
    };

    let pressure_threshold = (total_memory_bytes / 6).max(2 * GIB);
    let current_memory_pressure = available_memory_bytes < pressure_threshold;

    CapabilityAssessment {
        suggested_mode: suggested_mode.to_owned(),
        memory_tier: memory_tier.to_owned(),
        rationale,
        current_memory_pressure,
        limitations: limitations.to_owned(),
    }
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

    let total_memory_bytes = system.total_memory();
    let available_memory_bytes = system.available_memory();
    let logical_core_count = system.cpus().len();

    HardwareSnapshot {
        os_name: System::name(),
        os_version: System::long_os_version().or_else(System::os_version),
        architecture: std::env::consts::ARCH.to_owned(),
        cpu_model,
        logical_core_count,
        physical_core_count: System::physical_core_count(),
        total_memory_bytes,
        available_memory_bytes,
        disks,
        gpu_status: "not_detected_yet".to_owned(),
        assessment: assess_capability(total_memory_bytes, available_memory_bytes, logical_core_count),
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
            assessment: assess_capability(16 * 1024 * 1024 * 1024, 8 * 1024 * 1024 * 1024, 8),
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
            assessment: assess_capability(0, 0, 0),
        };
        assert_eq!(snapshot.gpu_status, "not_detected_yet");
        assert_eq!(snapshot.assessment.suggested_mode, "Undetermined");
    }

    #[test]
    fn capability_assessment_uses_conservative_ram_and_cpu_buckets() {
        let gib = 1024_u64 * 1024 * 1024;
        assert_eq!(assess_capability(4 * gib, 3 * gib, 8).suggested_mode, "Lightweight");
        assert_eq!(assess_capability(8 * gib, 6 * gib, 8).suggested_mode, "Lightweight");
        assert_eq!(assess_capability(16 * gib, 10 * gib, 8).suggested_mode, "Balanced");
        assert_eq!(assess_capability(32 * gib, 20 * gib, 8).suggested_mode, "High capability");
        assert_eq!(assess_capability(32 * gib, 20 * gib, 4).suggested_mode, "Balanced");
        assert_eq!(assess_capability(64 * gib, 50 * gib, 2).suggested_mode, "Lightweight");
    }

    #[test]
    fn capability_assessment_flags_low_current_memory_headroom() {
        let gib = 1024_u64 * 1024 * 1024;
        let assessment = assess_capability(16 * gib, gib, 8);
        assert!(assessment.current_memory_pressure);
        assert!(!assess_capability(16 * gib, 8 * gib, 8).current_memory_pressure);
    }

    #[test]
    fn capability_assessment_does_not_invent_missing_hardware() {
        let assessment = assess_capability(0, 0, 8);
        assert_eq!(assessment.suggested_mode, "Undetermined");
        assert!(assessment.rationale.contains("could not read enough"));
    }
}
