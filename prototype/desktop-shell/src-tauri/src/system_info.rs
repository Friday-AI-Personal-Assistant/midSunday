//! Read-only, local hardware inventory for the first HIARM baseline.
//! This intentionally does not enumerate processes, host names, network interfaces,
//! device serial numbers, or private user files.
use serde::Serialize;
use sysinfo::{get_current_pid, CpuRefreshKind, Disks, ProcessRefreshKind, ProcessesToUpdate, System};

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GpuMemorySegment {
    pub budget_bytes: u64,
    pub current_usage_bytes: u64,
    pub available_for_reservation_bytes: u64,
    pub current_reservation_bytes: u64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GpuMemoryBudget {
    pub status: String,
    /// QueryVideoMemoryInfo is sampled for node zero; this is not a count of adapter nodes.
    pub node_index: u32,
    pub local: Option<GpuMemorySegment>,
    pub local_error_code: Option<i32>,
    pub non_local: Option<GpuMemorySegment>,
    pub non_local_error_code: Option<i32>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GpuAdapter {
    pub name: String,
    pub vendor_id: u32,
    pub device_id: u32,
    pub dedicated_video_memory_bytes: u64,
    pub dedicated_system_memory_bytes: u64,
    pub shared_system_memory_bytes: u64,
    pub is_software: bool,
    pub memory_budget: GpuMemoryBudget,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GpuDetection {
    pub status: String,
    pub adapters: Vec<GpuAdapter>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HardwareDisk {
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub free_percent: u8,
    pub status: String,
}

/// Classifies a volume's general free-space headroom. This is not a model-fit guarantee.
pub fn assess_storage_status(total_bytes: u64, available_bytes: u64) -> (u8, String) {
    const GIB: u64 = 1024 * 1024 * 1024;
    if total_bytes == 0 {
        return (0, "unknown".to_owned());
    }
    let available = available_bytes.min(total_bytes);
    let free_percent = ((available as u128 * 100) / total_bytes as u128) as u8;
    let status = if available < 2 * GIB || free_percent < 5 {
        "critical"
    } else if available < 10 * GIB || free_percent < 10 {
        "low"
    } else {
        "healthy"
    };
    (free_percent, status.to_owned())
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
pub struct CpuInstructionSets {
    pub status: String,
    pub features: Vec<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProcessMemorySnapshot {
    pub status: String,
    /// Resident set size (physical memory currently resident).
    pub resident_bytes: Option<u64>,
    /// Virtual address-space size; this is not equivalent to physical RAM use.
    pub virtual_bytes: Option<u64>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModelStorageAssessment {
    pub status: String,
    pub mount_point: Option<String>,
    pub available_bytes: Option<u64>,
    pub free_percent: Option<u8>,
    pub volume_status: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HardwareSnapshot {
    /// Unix timestamp in seconds captured when this snapshot was collected.
    pub captured_at_unix_seconds: u64,
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub architecture: String,
    pub cpu_model: Option<String>,
    pub cpu_instruction_sets: CpuInstructionSets,
    pub logical_core_count: usize,
    pub physical_core_count: Option<usize>,
    pub total_memory_bytes: u64,
    pub available_memory_bytes: u64,
    pub process_memory: ProcessMemorySnapshot,
    pub disks: Vec<HardwareDisk>,
    pub model_storage: ModelStorageAssessment,
    pub gpu: GpuDetection,
    pub assessment: CapabilityAssessment,
}

/// Detects optional CPU features with Rust runtime probes. These are CPU/OS capabilities,
/// not proof that an installed inference runtime was compiled with matching optimized kernels.
pub fn detect_cpu_instruction_sets() -> CpuInstructionSets {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let mut features = Vec::new();
        if std::is_x86_feature_detected!("sse2") { features.push("SSE2".to_owned()); }
        if std::is_x86_feature_detected!("sse3") { features.push("SSE3".to_owned()); }
        if std::is_x86_feature_detected!("ssse3") { features.push("SSSE3".to_owned()); }
        if std::is_x86_feature_detected!("sse4.1") { features.push("SSE4.1".to_owned()); }
        if std::is_x86_feature_detected!("sse4.2") { features.push("SSE4.2".to_owned()); }
        if std::is_x86_feature_detected!("avx") { features.push("AVX".to_owned()); }
        if std::is_x86_feature_detected!("avx2") { features.push("AVX2".to_owned()); }
        if std::is_x86_feature_detected!("fma") { features.push("FMA".to_owned()); }
        features.sort();
        CpuInstructionSets { status: "detected".to_owned(), features }
    }
    #[cfg(target_arch = "aarch64")]
    {
        let mut features = Vec::new();
        if std::arch::is_aarch64_feature_detected!("neon") { features.push("NEON".to_owned()); }
        features.sort();
        CpuInstructionSets { status: "detected".to_owned(), features }
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
    {
        CpuInstructionSets { status: "unsupported_architecture".to_owned(), features: Vec::new() }
    }
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

/// Queries one process-specific DXGI memory segment. This is a point-in-time snapshot.
#[cfg(target_os = "windows")]
fn query_gpu_memory_segment(
    adapter: &windows::Win32::Graphics::Dxgi::IDXGIAdapter3,
    group: windows::Win32::Graphics::Dxgi::DXGI_MEMORY_SEGMENT_GROUP,
) -> Result<GpuMemorySegment, i32> {
    use windows::Win32::Graphics::Dxgi::DXGI_QUERY_VIDEO_MEMORY_INFO;
    let mut info = DXGI_QUERY_VIDEO_MEMORY_INFO::default();
    unsafe { adapter.QueryVideoMemoryInfo(0, group, &mut info) }
        .map_err(|error| error.code().0)?;
    Ok(GpuMemorySegment {
        budget_bytes: info.Budget,
        current_usage_bytes: info.CurrentUsage,
        available_for_reservation_bytes: info.AvailableForReservation,
        current_reservation_bytes: info.CurrentReservation,
    })
}

/// Enumerates Windows display adapters through DXGI and samples the current process's
/// local/non-local video-memory budgets where IDXGIAdapter3 is available.
#[cfg(target_os = "windows")]
pub fn detect_gpu_adapters() -> GpuDetection {
    use windows::core::Interface;
    use windows::Win32::Graphics::Dxgi::{
        CreateDXGIFactory1, IDXGIAdapter3, IDXGIFactory1,
        DXGI_ADAPTER_FLAG_SOFTWARE, DXGI_MEMORY_SEGMENT_GROUP_LOCAL,
        DXGI_MEMORY_SEGMENT_GROUP_NON_LOCAL,
    };

    let factory = match unsafe { CreateDXGIFactory1::<IDXGIFactory1>() } {
        Ok(factory) => factory,
        Err(_) => return GpuDetection { status: "unavailable".to_owned(), adapters: Vec::new() },
    };

    let mut adapters = Vec::new();
    let mut enumeration_ended = false;
    let mut descriptor_failures = 0_usize;
    for index in 0..32_u32 {
        let adapter = match unsafe { factory.EnumAdapters1(index) } {
            Ok(adapter) => adapter,
            Err(_) => { enumeration_ended = true; break; }
        };
        let desc = match unsafe { adapter.GetDesc1() } {
            Ok(desc) => desc,
            Err(_) => { descriptor_failures += 1; continue; }
        };
        let name = String::from_utf16_lossy(&desc.Description)
            .trim_end_matches('\0')
            .trim()
            .to_owned();
        let memory_budget = match adapter.cast::<IDXGIAdapter3>() {
            Ok(adapter3) => {
                let local_result = query_gpu_memory_segment(&adapter3, DXGI_MEMORY_SEGMENT_GROUP_LOCAL);
                let non_local_result = query_gpu_memory_segment(&adapter3, DXGI_MEMORY_SEGMENT_GROUP_NON_LOCAL);
                let local = local_result.as_ref().ok().cloned();
                let local_error_code = local_result.err();
                let non_local = non_local_result.as_ref().ok().cloned();
                let non_local_error_code = non_local_result.err();
                let status = match (local.is_some(), non_local.is_some()) {
                    (true, true) => "detected",
                    (true, false) | (false, true) => "partial",
                    (false, false) => "unavailable",
                };
                GpuMemoryBudget {
                    status: status.to_owned(),
                    node_index: 0,
                    local,
                    local_error_code,
                    non_local,
                    non_local_error_code,
                }
            }
            Err(_) => GpuMemoryBudget {
                status: "unsupported_interface".to_owned(),
                node_index: 0,
                local: None,
                local_error_code: None,
                non_local: None,
                non_local_error_code: None,
            },
        };
        adapters.push(GpuAdapter {
            name: if name.is_empty() { "Unnamed display adapter".to_owned() } else { name },
            vendor_id: desc.VendorId,
            device_id: desc.DeviceId,
            dedicated_video_memory_bytes: desc.DedicatedVideoMemory as u64,
            dedicated_system_memory_bytes: desc.DedicatedSystemMemory as u64,
            shared_system_memory_bytes: desc.SharedSystemMemory as u64,
            is_software: desc.Flags & (DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) != 0,
            memory_budget,
        });
    }

    let status = if adapters.is_empty() {
        if descriptor_failures > 0 { "unavailable" } else if enumeration_ended { "no_adapters" } else { "unavailable" }
    } else if descriptor_failures > 0 {
        "partial"
    } else {
        "detected"
    };
    GpuDetection { status: status.to_owned(), adapters }
}

#[cfg(not(target_os = "windows"))]
pub fn detect_gpu_adapters() -> GpuDetection {
    GpuDetection { status: "unsupported_platform".to_owned(), adapters: Vec::new() }
}

/// Queries only the current midSunday process; no other process details are collected.
pub fn collect_current_process_memory() -> ProcessMemorySnapshot {
    let Ok(pid) = get_current_pid() else {
        return ProcessMemorySnapshot { status: "unavailable".to_owned(), resident_bytes: None, virtual_bytes: None };
    };
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        ProcessRefreshKind::nothing().with_memory(),
    );
    match system.process(pid) {
        Some(process) => ProcessMemorySnapshot {
            status: "detected".to_owned(),
            resident_bytes: Some(process.memory()),
            virtual_bytes: Some(process.virtual_memory()),
        },
        None => ProcessMemorySnapshot { status: "unavailable".to_owned(), resident_bytes: None, virtual_bytes: None },
    }
}

/// Matches an existing configured model file to the most specific enumerated volume.
pub fn assess_model_storage(model_path: Option<&str>, disks: &[HardwareDisk]) -> ModelStorageAssessment {
    let Some(path_text) = model_path.map(str::trim).filter(|value| !value.is_empty()) else {
        return ModelStorageAssessment { status: "not_configured".to_owned(), mount_point: None, available_bytes: None, free_percent: None, volume_status: None };
    };
    let Ok(model_file) = std::fs::canonicalize(path_text) else {
        return ModelStorageAssessment { status: "model_path_unavailable".to_owned(), mount_point: None, available_bytes: None, free_percent: None, volume_status: None };
    };
    if !model_file.is_file() {
        return ModelStorageAssessment { status: "model_file_not_found".to_owned(), mount_point: None, available_bytes: None, free_percent: None, volume_status: None };
    }
    let matched = disks.iter().filter_map(|disk| {
        let mount = std::fs::canonicalize(&disk.mount_point).ok()?;
        if model_file.starts_with(&mount) { Some((mount.components().count(), disk)) } else { None }
    }).max_by_key(|(depth, _)| *depth).map(|(_, disk)| disk);
    match matched {
        Some(disk) => ModelStorageAssessment {
            status: "matched".to_owned(),
            mount_point: Some(disk.mount_point.clone()),
            available_bytes: Some(disk.available_bytes),
            free_percent: Some(disk.free_percent),
            volume_status: Some(disk.status.clone()),
        },
        None => ModelStorageAssessment { status: "volume_not_detected".to_owned(), mount_point: None, available_bytes: None, free_percent: None, volume_status: None },
    }
}

/// Collects a one-time snapshot and queries only midSunday's own process memory.
pub fn collect_hardware_snapshot(model_path: Option<String>) -> HardwareSnapshot {
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
        .map(|disk| {
            let total_bytes = disk.total_space();
            let available_bytes = disk.available_space().min(total_bytes);
            let (free_percent, status) = assess_storage_status(total_bytes, available_bytes);
            HardwareDisk {
                mount_point: disk.mount_point().display().to_string(),
                total_bytes,
                available_bytes,
                free_percent,
                status,
            }
        })
        .collect::<Vec<_>>();
    disks.sort_by(|left, right| left.mount_point.cmp(&right.mount_point));

    let model_storage = assess_model_storage(model_path.as_deref(), &disks);
    let total_memory_bytes = system.total_memory();
    let available_memory_bytes = system.available_memory();
    let logical_core_count = system.cpus().len();

    HardwareSnapshot {
        captured_at_unix_seconds: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        os_name: System::name(),
        os_version: System::long_os_version().or_else(System::os_version),
        architecture: std::env::consts::ARCH.to_owned(),
        cpu_model,
        cpu_instruction_sets: detect_cpu_instruction_sets(),
        logical_core_count,
        physical_core_count: System::physical_core_count(),
        total_memory_bytes,
        available_memory_bytes,
        process_memory: collect_current_process_memory(),
        disks,
        model_storage,
        gpu: detect_gpu_adapters(),
        assessment: assess_capability(total_memory_bytes, available_memory_bytes, logical_core_count),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hardware_snapshot_is_serializable_and_contains_only_inventory_fields() {
        let snapshot = HardwareSnapshot {
            captured_at_unix_seconds: 1_795_600_000,
            os_name: Some("Windows".to_owned()),
            os_version: Some("Windows 10".to_owned()),
            architecture: "x86_64".to_owned(),
            cpu_model: Some("Example CPU".to_owned()),
            cpu_instruction_sets: CpuInstructionSets { status: "detected".to_owned(), features: vec!["SSE2".to_owned(), "AVX2".to_owned()] },
            logical_core_count: 8,
            physical_core_count: Some(4),
            total_memory_bytes: 16 * 1024 * 1024 * 1024,
            available_memory_bytes: 8 * 1024 * 1024 * 1024,
            process_memory: ProcessMemorySnapshot { status: "detected".to_owned(), resident_bytes: Some(512 * 1024 * 1024), virtual_bytes: Some(2 * 1024 * 1024 * 1024) },
            disks: vec![HardwareDisk {
                mount_point: "C:\\".to_owned(),
                total_bytes: 512 * 1024 * 1024 * 1024,
                available_bytes: 256 * 1024 * 1024 * 1024,
                free_percent: 50,
                status: "healthy".to_owned(),
            }],
            model_storage: ModelStorageAssessment { status: "not_configured".to_owned(), mount_point: None, available_bytes: None, free_percent: None, volume_status: None },
            gpu: GpuDetection { status: "unsupported_platform".to_owned(), adapters: Vec::new() },
            assessment: assess_capability(16 * 1024 * 1024 * 1024, 8 * 1024 * 1024 * 1024, 8),
        };
        let value = serde_json::to_value(&snapshot).unwrap();
        assert_eq!(value["capturedAtUnixSeconds"], 1_795_600_000);
        assert_eq!(value["logicalCoreCount"], 8);
        assert_eq!(value["processMemory"]["residentBytes"], 512_u64 * 1024 * 1024);
        assert!(value.get("processes").is_none());
        assert_eq!(value["physicalCoreCount"], 4);
        assert_eq!(value["disks"][0]["availableBytes"], 256_u64 * 1024 * 1024 * 1024);
        assert_eq!(value["disks"][0]["freePercent"], 50);
        assert_eq!(value["disks"][0]["status"], "healthy");
        assert!(value.get("hostName").is_none());
        assert!(value.get("processes").is_none());
        assert!(value.get("networkInterfaces").is_none());
    }

    #[test]
    fn process_memory_probe_reports_only_the_current_process_or_unavailable() {
        let snapshot = collect_current_process_memory();
        assert!(matches!(snapshot.status.as_str(), "detected" | "unavailable"));
        if snapshot.status == "detected" {
            assert!(snapshot.resident_bytes.is_some());
            assert!(snapshot.virtual_bytes.is_some());
        } else {
            assert!(snapshot.resident_bytes.is_none());
            assert!(snapshot.virtual_bytes.is_none());
        }
    }

    #[test]
    fn hardware_snapshot_does_not_claim_gpu_detection() {
        let snapshot = HardwareSnapshot {
            captured_at_unix_seconds: 0,
            os_name: None,
            os_version: None,
            architecture: "unknown".to_owned(),
            cpu_model: None,
            cpu_instruction_sets: CpuInstructionSets { status: "unsupported_architecture".to_owned(), features: Vec::new() },
            logical_core_count: 0,
            physical_core_count: None,
            total_memory_bytes: 0,
            available_memory_bytes: 0,
            process_memory: ProcessMemorySnapshot { status: "unavailable".to_owned(), resident_bytes: None, virtual_bytes: None },
            model_storage: ModelStorageAssessment { status: "not_configured".to_owned(), mount_point: None, available_bytes: None, free_percent: None, volume_status: None },
            disks: Vec::new(),
            gpu: GpuDetection { status: "unsupported_platform".to_owned(), adapters: Vec::new() },
            assessment: assess_capability(0, 0, 0),
        };
        assert_eq!(snapshot.gpu.status, "unsupported_platform");
        assert_eq!(snapshot.assessment.suggested_mode, "Undetermined");
        assert_eq!(snapshot.cpu_instruction_sets.status, "unsupported_architecture");
    }

    #[test]
    fn cpu_instruction_set_detection_returns_only_known_feature_names() {
        let detected = detect_cpu_instruction_sets();
        assert!(matches!(detected.status.as_str(), "detected" | "unsupported_architecture"));
        let allowed = ["SSE2", "SSE3", "SSSE3", "SSE4.1", "SSE4.2", "AVX", "AVX2", "FMA", "NEON"];
        assert!(detected.features.iter().all(|feature| allowed.contains(&feature.as_str())));
        assert!(detected.features.windows(2).all(|pair| pair[0] < pair[1]));
        if detected.status == "unsupported_architecture" { assert!(detected.features.is_empty()); }
    }


    #[test]
    fn storage_assessment_handles_zero_capacity_and_clamps_free_space() {
        assert_eq!(assess_storage_status(0, 0), (0, "unknown".to_owned()));
        assert_eq!(assess_storage_status(10 * 1024 * 1024 * 1024, 20 * 1024 * 1024 * 1024), (100, "healthy".to_owned()));
    }

    #[test]
    fn storage_assessment_uses_conservative_free_space_thresholds() {
        let gib = 1024_u64 * 1024 * 1024;
        assert_eq!(assess_storage_status(100 * gib, 4 * gib).1, "critical");
        assert_eq!(assess_storage_status(100 * gib, 4 * gib).0, 4);
        assert_eq!(assess_storage_status(100 * gib, 8 * gib).1, "low");
        assert_eq!(assess_storage_status(100 * gib, 20 * gib).1, "healthy");
        assert_eq!(assess_storage_status(100 * gib, gib).1, "critical");
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

#[cfg(test)]
mod gpu_tests {
    use super::*;

    #[test]
    fn gpu_inventory_serializes_distinct_memory_categories() {
        let inventory = GpuDetection {
            status: "detected".to_owned(),
            adapters: vec![GpuAdapter {
                name: "Example integrated GPU".to_owned(), vendor_id: 0x8086, device_id: 1,
                dedicated_video_memory_bytes: 128 * 1024 * 1024,
                dedicated_system_memory_bytes: 0,
                shared_system_memory_bytes: 8 * 1024 * 1024 * 1024,
                is_software: false,
                memory_budget: GpuMemoryBudget {
                    status: "detected".to_owned(),
                    node_index: 0,
                    local_error_code: None,
                    non_local_error_code: None,
                    local: Some(GpuMemorySegment {
                        budget_bytes: 4 * 1024 * 1024 * 1024,
                        current_usage_bytes: 512 * 1024 * 1024,
                        available_for_reservation_bytes: 2 * 1024 * 1024 * 1024,
                        current_reservation_bytes: 0,
                    }),
                    non_local: Some(GpuMemorySegment {
                        budget_bytes: 8 * 1024 * 1024 * 1024,
                        current_usage_bytes: 256 * 1024 * 1024,
                        available_for_reservation_bytes: 4 * 1024 * 1024 * 1024,
                        current_reservation_bytes: 0,
                    }),
                },
            }],
        };
        let value = serde_json::to_value(inventory).unwrap();
        assert_eq!(value["adapters"][0]["dedicatedVideoMemoryBytes"], 128_u64 * 1024 * 1024);
        assert_eq!(value["adapters"][0]["sharedSystemMemoryBytes"], 8_u64 * 1024 * 1024 * 1024);
        assert_eq!(value["adapters"][0]["vendorId"], 0x8086);
        assert_eq!(value["adapters"][0]["memoryBudget"]["local"]["budgetBytes"], 4_u64 * 1024 * 1024 * 1024);
        assert_eq!(value["adapters"][0]["memoryBudget"]["local"]["currentUsageBytes"], 512_u64 * 1024 * 1024);
        assert_eq!(value["adapters"][0]["memoryBudget"]["status"], "detected");
    }

    #[test]
    fn unsupported_gpu_detection_is_explicit_and_empty() {
        let inventory = GpuDetection { status: "unsupported_platform".to_owned(), adapters: Vec::new() };
        assert!(inventory.adapters.is_empty());
        assert_eq!(inventory.status, "unsupported_platform");
    }
}
