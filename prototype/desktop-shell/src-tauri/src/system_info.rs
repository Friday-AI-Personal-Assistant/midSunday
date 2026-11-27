//! Read-only, local hardware inventory for the first HIARM baseline.
//! This intentionally does not enumerate processes, host names, network interfaces,
//! device serial numbers, or private user files.
use serde::Serialize;
use starship_battery::{Manager as BatteryManager, State as BatteryState};
use sysinfo::{get_current_pid, CpuRefreshKind, DiskRefreshKind, Disks, Networks, Pid, ProcessRefreshKind, ProcessesToUpdate, System};
use std::sync::Mutex;
use std::time::{Duration, Instant};

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
pub struct GpuInventoryAssessment {
    /// Describes adapter inventory only; it does not establish inference-backend support.
    pub status: String,
    pub adapter_count: usize,
    pub hardware_adapter_count: usize,
    pub software_adapter_count: usize,
    /// Largest dedicated-memory figure reported by a non-software adapter.
    pub largest_dedicated_video_memory_bytes: Option<u64>,
    /// Largest OS-reported local segment budget; not equivalent to free VRAM.
    pub largest_local_memory_budget_bytes: Option<u64>,
    pub adapters_with_local_memory_budget: usize,
    pub limitations: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GpuMemoryHeadroomAssessment {
    pub status: String, pub assessed_adapter_count: usize, pub adapters_with_local_budget: usize,
    pub adapters_with_critical_headroom: usize, pub adapters_with_constrained_headroom: usize,
    pub adapters_with_reported_headroom: usize, pub lowest_available_for_reservation_bytes: Option<u64>,
    pub lowest_headroom_percent: Option<u8>, pub limitations: String,
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
    pub gpu_assessment: GpuInventoryAssessment,
    pub gpu_memory_headroom: GpuMemoryHeadroomAssessment,
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

    let available = available_memory_bytes.min(total_memory_bytes);
    let pressure_threshold = (total_memory_bytes / 6).max(2 * GIB);
    let current_memory_pressure = available < pressure_threshold;

    let cpu_reason = if logical_core_count < 4 {
        suggested_mode = "Lightweight";
        "The CPU reports fewer than 4 logical processors, so Lightweight is the safer starting point.".to_owned()
    } else if suggested_mode == "High capability" && logical_core_count < 8 {
        suggested_mode = "Balanced";
        "The CPU reports fewer than 8 logical processors, so Balanced is the more cautious starting point.".to_owned()
    } else {
        format!("The detected CPU reports {logical_core_count} logical processors.")
    };

    let rationale = if current_memory_pressure {
        let previous_mode = suggested_mode;
        suggested_mode = match suggested_mode {
            "High capability" => "Balanced",
            "Balanced" => "Lightweight",
            other => other,
        };
        format!(
            "{memory_reason} {cpu_reason} Available RAM is below the conservative headroom threshold of {} GiB, so the suggested profile is reduced from {previous_mode} to {suggested_mode} for this snapshot.",
            pressure_threshold / GIB
        )
    } else {
        format!("{memory_reason} {cpu_reason}")
    };

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

/// Summarises detected display adapters without claiming that an inference backend can use them.
pub fn assess_gpu_inventory(gpu: &GpuDetection) -> GpuInventoryAssessment {
    let hardware_adapters = gpu.adapters.iter().filter(|adapter| !adapter.is_software).collect::<Vec<_>>();
    let software_adapter_count = gpu.adapters.len().saturating_sub(hardware_adapters.len());
    let largest_dedicated_video_memory_bytes = hardware_adapters
        .iter()
        .map(|adapter| adapter.dedicated_video_memory_bytes)
        .max();
    let local_budgets = hardware_adapters.iter().filter_map(|adapter| {
        adapter.memory_budget.local.as_ref().map(|segment| segment.budget_bytes)
    }).collect::<Vec<_>>();

    let status = if gpu.status == "unsupported_platform" {
        "unsupported_platform"
    } else if gpu.status == "unavailable" {
        "unavailable"
    } else if gpu.adapters.is_empty() {
        "no_adapters"
    } else if hardware_adapters.is_empty() {
        "software_only"
    } else if gpu.status == "partial" {
        "partial"
    } else {
        "detected"
    };

    GpuInventoryAssessment {
        status: status.to_owned(),
        adapter_count: gpu.adapters.len(),
        hardware_adapter_count: hardware_adapters.len(),
        software_adapter_count,
        largest_dedicated_video_memory_bytes,
        largest_local_memory_budget_bytes: local_budgets.iter().copied().max(),
        adapters_with_local_memory_budget: local_budgets.len(),
        limitations: "Display-adapter inventory only. Dedicated memory and OS local-memory budgets are not guaranteed free memory; integrated GPUs may use shared system RAM. This does not detect inference-backend support, compute capability, driver/runtime compatibility, model fit, or performance.".to_owned(),
    }
}

/// Scores per-adapter OS reservation headroom; this is not free VRAM or model fit.
pub fn assess_gpu_memory_headroom(gpu: &GpuDetection) -> GpuMemoryHeadroomAssessment {
    const MIB: u64 = 1024 * 1024; const GIB: u64 = 1024 * MIB;
    let physical = gpu.adapters.iter().filter(|a| !a.is_software).collect::<Vec<_>>();
    let (mut count, mut critical, mut constrained, mut reported) = (0_usize, 0_usize, 0_usize, 0_usize);
    let (mut lowest_bytes, mut lowest_percent): (Option<u64>, Option<u8>) = (None, None);
    for adapter in &physical {
        let Some(local) = adapter.memory_budget.local.as_ref() else { continue };
        if local.budget_bytes == 0 { continue; }
        count += 1;
        let available = local.available_for_reservation_bytes.min(local.budget_bytes);
        let percent = ((available as u128 * 100) / local.budget_bytes as u128) as u8;
        if lowest_bytes.map_or(true, |v| available < v) { lowest_bytes = Some(available); lowest_percent = Some(percent); }
        if available < 512 * MIB || percent < 10 { critical += 1; }
        else if available < GIB || percent < 20 { constrained += 1; }
        else { reported += 1; }
    }
    let status = if gpu.status == "unsupported_platform" { "unsupported_platform" }
        else if gpu.adapters.is_empty() { if gpu.status == "unavailable" { "unavailable" } else { "no_adapters" } }
        else if count == 0 { "unavailable" }
        else if count < physical.len() || gpu.status == "partial" { "partial" }
        else { "detected" };
    GpuMemoryHeadroomAssessment {
        status: status.to_owned(), assessed_adapter_count: physical.len(), adapters_with_local_budget: count,
        adapters_with_critical_headroom: critical, adapters_with_constrained_headroom: constrained,
        adapters_with_reported_headroom: reported, lowest_available_for_reservation_bytes: lowest_bytes,
        lowest_headroom_percent: lowest_percent,
        limitations: "Point-in-time OS reservation headroom for the current process on DXGI node zero; not free VRAM, model fit, or proof of inference-backend support. Values are not pooled; software adapters and missing/zero local budgets are not scored.".to_owned(),
    }
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

/// Samples one known child PID supplied by midSunday's process supervisor.
/// This function never enumerates unrelated processes.
pub fn collect_known_process_memory(pid_value: u32) -> ProcessMemorySnapshot {
    let pid = Pid::from_u32(pid_value);
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

/// Reads only total and currently available system RAM for a one-time local estimate.
pub fn collect_system_memory_bytes() -> (u64, u64) {
    let mut system = System::new();
    system.refresh_memory();
    let total = system.total_memory();
    (total, system.available_memory().min(total))
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
    let gpu = detect_gpu_adapters();
    let gpu_assessment = assess_gpu_inventory(&gpu);
    let gpu_memory_headroom = assess_gpu_memory_headroom(&gpu);
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
        gpu,
        gpu_assessment,
        gpu_memory_headroom,
        assessment: assess_capability(total_memory_bytes, available_memory_bytes, logical_core_count),
    }
}

/// Bounded, read-only sampler for the live System Monitor. It refreshes CPU, memory,
/// aggregate network counters, and the current midSunday process only; it never exposes
/// interface names or enumerates unrelated processes.
pub struct SystemMonitor {
    state: Mutex<SystemMonitorState>,
}

struct SystemMonitorState {
    system: System,
    networks: Networks,
    disks: Disks,
    samples: u64,
    previous_network_totals: Option<(u64, u64)>,
    previous_sample_at: Option<Instant>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LiveSystemMetrics {
    pub captured_at_unix_millis: u64,
    pub cpu_status: String,
    pub cpu_usage_percent: Option<f32>,
    pub cpu_core_usage_percent: Vec<f32>,
    pub total_memory_bytes: u64,
    pub used_memory_bytes: u64,
    pub available_memory_bytes: u64,
    pub process_memory: ProcessMemorySnapshot,
    pub network_status: String,
    pub network_interface_count: usize,
    pub download_bytes_per_second: Option<u64>,
    pub upload_bytes_per_second: Option<u64>,
    pub process_disk_io_status: String,
    pub process_read_bytes_per_second: Option<u64>,
    pub process_write_bytes_per_second: Option<u64>,
    pub system_disk_status: String,
    pub system_disk_count: usize,
    pub system_disk_read_bytes_per_second: Option<u64>,
    pub system_disk_write_bytes_per_second: Option<u64>,
    pub gpu: GpuLiveMetrics,
    pub battery: BatteryMetrics,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GpuLiveAdapterMetrics {
    pub label: String,
    pub status: String,
    pub utilization_percent: Option<f32>,
    pub memory_used_bytes: Option<u64>,
    pub memory_total_bytes: Option<u64>,
    pub memory_utilization_percent: Option<f32>,
    pub source: String,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GpuLiveMetrics {
    pub status: String,
    pub adapter_count: usize,
    pub adapters: Vec<GpuLiveAdapterMetrics>,
    pub limitations: String,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BatteryMetrics {
    pub status: String,
    pub battery_count: usize,
    pub charge_percent: Option<f32>,
    pub state: String,
}

impl Default for SystemMonitor {
    fn default() -> Self {
        let mut system = System::new();
        system.refresh_cpu_list(CpuRefreshKind::everything());
        system.refresh_memory();
        let networks = Networks::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list_specifics(DiskRefreshKind::nothing().with_io_usage());
        Self { state: Mutex::new(SystemMonitorState { system, networks, disks, samples: 0, previous_network_totals: None, previous_sample_at: None }) }
    }
}

impl SystemMonitor {
    pub fn sample(&self) -> LiveSystemMetrics {
        let mut state = self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        state.system.refresh_cpu_usage();
        state.system.refresh_memory();
        state.networks.refresh(true);
        let now = Instant::now();
        let elapsed = state.previous_sample_at.map(|previous| now.saturating_duration_since(previous));
        let cpu_core_usage_percent = state.system.cpus().iter()
            .map(|cpu| normalise_cpu_usage_percent(cpu.cpu_usage()))
            .collect::<Vec<_>>();
        let cpu_usage_percent = if state.samples == 0 || cpu_core_usage_percent.is_empty() {
            None
        } else {
            Some(normalise_cpu_usage_percent(cpu_core_usage_percent.iter().sum::<f32>() / cpu_core_usage_percent.len() as f32))
        };
        state.samples = state.samples.saturating_add(1);
        let total_memory_bytes = state.system.total_memory();
        let available_memory_bytes = state.system.available_memory().min(total_memory_bytes);
        let network_interface_count = state.networks.iter().count();
        let network_totals = state.networks.iter().fold((0_u64, 0_u64), |(received, transmitted), (_, data)| {
            (received.saturating_add(data.total_received()), transmitted.saturating_add(data.total_transmitted()))
        });
        let download_bytes_per_second = state.previous_network_totals.zip(elapsed)
            .and_then(|(previous, duration)| counter_rate_bytes_per_second(network_totals.0, previous.0, duration));
        let upload_bytes_per_second = state.previous_network_totals.zip(elapsed)
            .and_then(|(previous, duration)| counter_rate_bytes_per_second(network_totals.1, previous.1, duration));
        let network_status = if network_interface_count == 0 {
            "unavailable"
        } else if download_bytes_per_second.is_some() && upload_bytes_per_second.is_some() {
            "detected"
        } else {
            "warming_up"
        }.to_owned();
        state.previous_network_totals = Some(network_totals);
        state.disks.refresh_specifics(true, DiskRefreshKind::nothing().with_io_usage());
        let system_disk_count = state.disks.len();
        let disk_io_totals = state.disks.iter().fold((0_u64, 0_u64), |(read, written), disk| {
            let usage = disk.usage();
            (read.saturating_add(usage.read_bytes), written.saturating_add(usage.written_bytes))
        });
        let (system_disk_read_bytes_per_second, system_disk_write_bytes_per_second) = disk_io_rates(disk_io_totals.0, disk_io_totals.1, elapsed);
        let system_disk_status = if system_disk_count == 0 { "no_disks" } else if system_disk_read_bytes_per_second.is_some() && system_disk_write_bytes_per_second.is_some() { "detected" } else { "warming_up" }.to_owned();
        let process_result = match get_current_pid() {
            Ok(pid) => {
                state.system.refresh_processes_specifics(
                    ProcessesToUpdate::Some(&[pid]), true,
                    ProcessRefreshKind::nothing().with_memory().with_disk_usage(),
                );
                state.system.process(pid).map(|process| {
                    let disk_usage = process.disk_usage();
                    (
                        ProcessMemorySnapshot { status: "detected".to_owned(), resident_bytes: Some(process.memory()), virtual_bytes: Some(process.virtual_memory()) },
                        elapsed.and_then(|duration| rate_bytes_per_second(disk_usage.read_bytes, duration)),
                        elapsed.and_then(|duration| rate_bytes_per_second(disk_usage.written_bytes, duration)),
                    )
                })
            }
            Err(_) => None,
        };
        let (process_memory, process_read_bytes_per_second, process_write_bytes_per_second, process_disk_io_status) =
            match process_result {
                Some((memory, read_rate, write_rate)) => (
                    memory,
                    read_rate,
                    write_rate,
                    if read_rate.is_some() && write_rate.is_some() { "detected".to_owned() } else { "warming_up".to_owned() },
                ),
                None => (
                    ProcessMemorySnapshot { status: "unavailable".to_owned(), resident_bytes: None, virtual_bytes: None },
                    None, None, "unavailable".to_owned(),
                ),
            };
        state.previous_sample_at = Some(now);
        LiveSystemMetrics {
            captured_at_unix_millis: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis().min(u64::MAX as u128) as u64,
            cpu_status: if cpu_usage_percent.is_some() { "detected".to_owned() } else if cpu_core_usage_percent.is_empty() { "unavailable".to_owned() } else { "warming_up".to_owned() },
            cpu_usage_percent,
            cpu_core_usage_percent,
            total_memory_bytes,
            used_memory_bytes: total_memory_bytes.saturating_sub(available_memory_bytes),
            available_memory_bytes,
            process_memory,
            network_status,
            network_interface_count,
            download_bytes_per_second,
            upload_bytes_per_second,
            process_disk_io_status,
            process_read_bytes_per_second,
            process_write_bytes_per_second,
            system_disk_status,
            system_disk_count,
            system_disk_read_bytes_per_second,
            system_disk_write_bytes_per_second,
            gpu: collect_live_gpu_metrics(),
            battery: collect_battery_metrics(),
        }
    }
}


/// Collects only driver-published GPU telemetry. Adapter names, PCI identifiers, and
/// serial/device identifiers are deliberately not exposed by the live monitor.
#[cfg(target_os = "linux")]
fn collect_live_gpu_metrics() -> GpuLiveMetrics {
    use std::fs;
    use std::path::Path;

    let root = Path::new("/sys/class/drm");
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return GpuLiveMetrics {
            status: "unavailable".to_owned(), adapter_count: 0, adapters: Vec::new(),
            limitations: "Linux DRM telemetry could not be read. GPU driver permissions or kernel support may limit measurements.".to_owned(),
        },
    };

    let mut adapters = Vec::new();
    let mut drm_device_count = 0_usize;
    for entry in entries.flatten() {
        let card_name = entry.file_name();
        let card_name = card_name.to_string_lossy();
        let Some(number) = card_name.strip_prefix("card") else { continue };
        if number.is_empty() || !number.chars().all(|character| character.is_ascii_digit()) { continue; }
        let device = entry.path().join("device");
        if !device.exists() { continue; }
        drm_device_count = drm_device_count.saturating_add(1);

        // gpu_busy_percent is a documented AMDGPU sysfs reading. Other drivers are
        // intentionally left unsupported until their own counter semantics are validated.
        let utilization_percent = read_gpu_percent(&device.join("gpu_busy_percent"));
        let memory_used = read_gpu_integer(&device.join("mem_info_vram_used"));
        let memory_total = read_gpu_integer(&device.join("mem_info_vram_total"));
        let (memory_used_bytes, memory_utilization_percent) = match (memory_used, memory_total) {
            (Some(used), Some(total)) => gpu_memory_usage(used, total),
            _ => (None, None),
        };
        let memory_total_bytes = memory_total.filter(|total| *total > 0);
        if utilization_percent.is_some() || memory_used_bytes.is_some() || memory_total_bytes.is_some() {
            let has_utilization = utilization_percent.is_some();
            adapters.push(GpuLiveAdapterMetrics {
                label: format!("GPU {}", adapters.len() + 1),
                status: if has_utilization { "detected" } else { "partial" }.to_owned(),
                utilization_percent,
                memory_used_bytes,
                memory_total_bytes,
                memory_utilization_percent,
                source: "Linux AMDGPU DRM sysfs".to_owned(),
            });
        }
    }

    let adapter_count = adapters.len();
    let status = if adapter_count == 0 {
        if drm_device_count == 0 { "no_adapters" } else { "unsupported_driver" }
    } else if adapters.iter().all(|adapter| adapter.utilization_percent.is_some()) {
        "detected"
    } else {
        "partial"
    };
    GpuLiveMetrics {
        status: status.to_owned(),
        adapter_count,
        adapters,
        limitations: "This first live collector supports AMDGPU driver-published GPU busy and VRAM counters on Linux. Other Linux GPU drivers, Windows, and macOS remain explicitly unsupported until dedicated collectors are validated. Memory values are driver-reported VRAM allocation, not proof that memory is free for AI inference.".to_owned(),
    }
}

#[cfg(target_os = "linux")]
fn read_gpu_integer(path: &std::path::Path) -> Option<u64> {
    std::fs::read_to_string(path).ok()?.trim().parse::<u64>().ok()
}

#[cfg(target_os = "linux")]
fn read_gpu_percent(path: &std::path::Path) -> Option<f32> {
    parse_gpu_percent_text(&std::fs::read_to_string(path).ok()?)
}

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy)]
union PdhFormattedValueUnion {
    long_value: i32,
    double_value: f64,
    large_value: i64,
    ansi_string: *mut i8,
    wide_string: *mut u16,
}

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy)]
struct PdhFormattedValue {
    c_status: u32,
    value: PdhFormattedValueUnion,
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct PdhFormattedCounterValueItem {
    name: *mut u16,
    value: PdhFormattedValue,
}

#[cfg(target_os = "windows")]
#[link(name = "pdh")]
extern "system" {
    fn PdhOpenQueryW(data_source: *const u16, user_data: usize, query: *mut *mut std::ffi::c_void) -> u32;
    fn PdhAddEnglishCounterW(query: *mut std::ffi::c_void, path: *const u16, user_data: usize, counter: *mut *mut std::ffi::c_void) -> u32;
    fn PdhCollectQueryData(query: *mut std::ffi::c_void) -> u32;
    fn PdhGetFormattedCounterArrayW(counter: *mut std::ffi::c_void, format: u32, buffer_size: *mut u32, item_count: *mut u32, items: *mut PdhFormattedCounterValueItem) -> u32;
    fn PdhCloseQuery(query: *mut std::ffi::c_void) -> u32;
}

#[cfg(target_os = "windows")]
const PDH_MORE_DATA_CODE: u32 = 0x8000_07D2;
#[cfg(target_os = "windows")]
const PDH_FMT_DOUBLE_CODE: u32 = 0x0000_0200;
#[cfg(target_os = "windows")]
const PDH_FMT_LARGE_CODE: u32 = 0x0000_0400;
#[cfg(target_os = "windows")]
const PDH_FMT_NOCAP100_CODE: u32 = 0x0000_8000;

/// Collects Windows GPU Engine activity and dedicated adapter-memory allocations
/// through language-neutral PDH counters. Only generic adapter labels are returned.
#[cfg(target_os = "windows")]
fn collect_live_gpu_metrics() -> GpuLiveMetrics {
    use std::ffi::c_void;

    let mut query: *mut c_void = std::ptr::null_mut();
    let opened = unsafe { PdhOpenQueryW(std::ptr::null(), 0, &mut query) };
    if opened != 0 || query.is_null() {
        return windows_gpu_unavailable("Windows PDH could not open a local performance-counter query.");
    }

    let engine_path = "\\GPU Engine(*)\\Utilization Percentage";
    let memory_path = "\\GPU Adapter Memory(*)\\Dedicated Usage";
    let engine_utf16 = engine_path.encode_utf16().chain(std::iter::once(0)).collect::<Vec<_>>();
    let memory_utf16 = memory_path.encode_utf16().chain(std::iter::once(0)).collect::<Vec<_>>();
    let mut engine_counter: *mut c_void = std::ptr::null_mut();
    let mut memory_counter: *mut c_void = std::ptr::null_mut();
    let engine_added = unsafe { PdhAddEnglishCounterW(query, engine_utf16.as_ptr(), 0, &mut engine_counter) } == 0;
    let memory_added = unsafe { PdhAddEnglishCounterW(query, memory_utf16.as_ptr(), 0, &mut memory_counter) } == 0;

    if !engine_added && !memory_added {
        unsafe { PdhCloseQuery(query); }
        return GpuLiveMetrics {
            status: "unsupported_driver".to_owned(), adapter_count: 0, adapters: Vec::new(),
            limitations: "Windows does not expose the expected GPU Engine or GPU Adapter Memory performance counters on this device. Driver and Windows-version support varies.".to_owned(),
        };
    }

    let first_sample = unsafe { PdhCollectQueryData(query) };
    if first_sample != 0 {
        unsafe { PdhCloseQuery(query); }
        return windows_gpu_unavailable("Windows PDH could not collect the initial GPU counter sample.");
    }
    // GPU Engine utilisation is a rate-style counter and needs two samples.
    std::thread::sleep(std::time::Duration::from_millis(80));
    let second_sample = unsafe { PdhCollectQueryData(query) };
    if second_sample != 0 {
        unsafe { PdhCloseQuery(query); }
        return windows_gpu_unavailable("Windows PDH could not collect the follow-up GPU counter sample.");
    }

    let engine_values = if engine_added { unsafe { pdh_read_counter_array(engine_counter, PDH_FMT_DOUBLE_CODE | PDH_FMT_NOCAP100_CODE) }.unwrap_or_default() } else { Vec::new() };
    let memory_values = if memory_added { unsafe { pdh_read_counter_array(memory_counter, PDH_FMT_LARGE_CODE) }.unwrap_or_default() } else { Vec::new() };
    unsafe { PdhCloseQuery(query); }
    assemble_windows_gpu_metrics(engine_values, memory_values)
}

#[cfg(target_os = "windows")]
fn windows_gpu_unavailable(reason: &str) -> GpuLiveMetrics {
    GpuLiveMetrics {
        status: "unavailable".to_owned(), adapter_count: 0, adapters: Vec::new(),
        limitations: format!("{reason} No fabricated zero readings are reported."),
    }
}

#[cfg(target_os = "windows")]
unsafe fn pdh_read_counter_array(counter: *mut std::ffi::c_void, format: u32) -> Option<Vec<(String, f64)>> {
    let mut buffer_size = 0_u32;
    let mut item_count = 0_u32;
    let status = PdhGetFormattedCounterArrayW(counter, format, &mut buffer_size, &mut item_count, std::ptr::null_mut());
    if status == 0 && item_count == 0 {
        return Some(Vec::new());
    }
    if status != PDH_MORE_DATA_CODE || buffer_size == 0 {
        return None;
    }

    // u64 storage guarantees sufficient alignment for PDH's pointer-containing records.
    let words = (buffer_size as usize).saturating_add(7) / 8;
    let mut buffer = vec![0_u64; words];
    let items = buffer.as_mut_ptr().cast::<PdhFormattedCounterValueItem>();
    let result = PdhGetFormattedCounterArrayW(counter, format, &mut buffer_size, &mut item_count, items);
    if result != 0 {
        return None;
    }

    let mut values = Vec::with_capacity(item_count as usize);
    for index in 0..item_count as usize {
        let item = &*items.add(index);
        if item.name.is_null() || item.value.c_status > 1 {
            continue;
        }
        let mut length = 0_usize;
        while length < 1024 && *item.name.add(length) != 0 {
            length += 1;
        }
        if length == 0 || length == 1024 {
            continue;
        }
        let name = String::from_utf16_lossy(std::slice::from_raw_parts(item.name, length));
        let numeric = if format & PDH_FMT_LARGE_CODE == PDH_FMT_LARGE_CODE {
            item.value.value.large_value as f64
        } else {
            item.value.value.double_value
        };
        if numeric.is_finite() && numeric >= 0.0 {
            values.push((name, numeric));
        }
    }
    Some(values)
}

#[cfg(target_os = "windows")]
fn assemble_windows_gpu_metrics(engine_values: Vec<(String, f64)>, memory_values: Vec<(String, f64)>) -> GpuLiveMetrics {
    use std::collections::{BTreeMap, BTreeSet};

    let mut engine_by_adapter: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for (instance, value) in engine_values {
        if let Some(key) = gpu_adapter_key_from_instance(&instance) {
            engine_by_adapter.entry(key).or_default().push(value);
        }
    }
    let mut memory_by_adapter: BTreeMap<String, u64> = BTreeMap::new();
    for (instance, value) in memory_values {
        if let Some(key) = gpu_adapter_key_from_instance(&instance) {
            let bytes = value.min(u64::MAX as f64) as u64;
            let total = memory_by_adapter.entry(key).or_default();
            *total = total.saturating_add(bytes);
        }
    }

    let totals = windows_gpu_dedicated_memory_totals();
    let mut keys = BTreeSet::new();
    keys.extend(engine_by_adapter.keys().cloned());
    keys.extend(memory_by_adapter.keys().cloned());

    let mut adapters = Vec::new();
    for key in keys {
        let utilization_percent = engine_by_adapter.get(&key).and_then(|values| max_gpu_engine_utilization(values));
        let memory_used_bytes = memory_by_adapter.get(&key).copied();
        let memory_total_bytes = totals.get(&key).copied().filter(|total| *total > 0);
        let (bounded_used, memory_utilization_percent) = match (memory_used_bytes, memory_total_bytes) {
            (Some(used), Some(total)) => gpu_memory_usage(used, total),
            (Some(used), None) => (Some(used), None),
            _ => (None, None),
        };
        if utilization_percent.is_some() || bounded_used.is_some() {
            adapters.push(GpuLiveAdapterMetrics {
                label: format!("GPU {}", adapters.len() + 1),
                status: if utilization_percent.is_some() && bounded_used.is_some() { "detected" } else { "partial" }.to_owned(),
                utilization_percent,
                memory_used_bytes: bounded_used,
                memory_total_bytes,
                memory_utilization_percent,
                source: "Windows PDH GPU Engine / Adapter Memory".to_owned(),
            });
        }
    }

    let adapter_count = adapters.len();
    let status = if adapter_count == 0 {
        if totals.is_empty() { "no_adapters" } else { "unsupported_driver" }
    } else if adapters.iter().all(|adapter| adapter.utilization_percent.is_some()) {
        "detected"
    } else {
        "partial"
    };
    GpuLiveMetrics {
        status: status.to_owned(),
        adapter_count,
        adapters,
        limitations: "Windows PDH GPU Engine counters report the busiest individual engine per adapter, not a single universal whole-GPU utilisation percentage. Dedicated Usage is an OS-reported allocation counter; its total is paired with DXGI dedicated-memory inventory where available. Shared GPU memory is not labelled VRAM. Counter availability and attribution vary by Windows build and graphics driver; physical-device validation is required.".to_owned(),
    }
}

#[cfg(target_os = "windows")]
fn windows_gpu_dedicated_memory_totals() -> std::collections::BTreeMap<String, u64> {
    use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE};
    let mut totals = std::collections::BTreeMap::new();
    let Ok(factory) = (unsafe { CreateDXGIFactory1::<IDXGIFactory1>() }) else { return totals; };
    for index in 0..32_u32 {
        let Ok(adapter) = (unsafe { factory.EnumAdapters1(index) }) else { break; };
        let Ok(desc) = (unsafe { adapter.GetDesc1() }) else { continue; };
        if desc.Flags & (DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) != 0 || desc.DedicatedVideoMemory == 0 {
            continue;
        }
        let key = format!("luid_0x{:08x}_0x{:08x}", desc.AdapterLuid.HighPart as u32, desc.AdapterLuid.LowPart);
        totals.insert(key, desc.DedicatedVideoMemory as u64);
    }
    totals
}

#[cfg(target_os = "windows")]
fn gpu_adapter_key_from_instance(instance: &str) -> Option<String> {
    let start = instance.to_ascii_lowercase().find("luid_")?;
    let tail = &instance[start..];
    let end = tail.find("_phys_").unwrap_or(tail.len());
    if end <= "luid_".len() { return None; }
    Some(tail[..end].to_owned())
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn collect_live_gpu_metrics() -> GpuLiveMetrics {
    GpuLiveMetrics {
        status: "unsupported_platform".to_owned(),
        adapter_count: 0,
        adapters: Vec::new(),
        limitations: "A live GPU telemetry collector is not yet available for this operating system. Existing GPU inventory remains separate point-in-time hardware information.".to_owned(),
    }
}

/// Reads aggregate battery telemetry without exposing vendor, model, or serial identifiers.
/// Charge is energy-weighted across installed batteries; remaining time is intentionally omitted.
fn collect_battery_metrics() -> BatteryMetrics {
    let manager = match BatteryManager::new() {
        Ok(manager) => manager,
        Err(_) => return BatteryMetrics { status: "unavailable".to_owned(), battery_count: 0, charge_percent: None, state: "unavailable".to_owned() },
    };
    let batteries = match manager.batteries() {
        Ok(batteries) => batteries,
        Err(_) => return BatteryMetrics { status: "unavailable".to_owned(), battery_count: 0, charge_percent: None, state: "unavailable".to_owned() },
    };
    let mut count = 0_usize;
    let mut current_energy = 0.0_f64;
    let mut full_energy = 0.0_f64;
    let mut states = Vec::new();
    let mut had_read_error = false;
    for result in batteries {
        let battery = match result {
            Ok(battery) => battery,
            Err(_) => { had_read_error = true; continue; }
        };
        count += 1;
        let charge = battery.state_of_charge().value as f64;
        let energy = battery.energy().value as f64;
        let full = battery.energy_full().value as f64;
        if charge.is_finite() && energy.is_finite() && full.is_finite() && full > 0.0 {
            current_energy += energy.max(0.0);
            full_energy += full;
        } else {
            had_read_error = true;
        }
        states.push(battery_state_label(battery.state()));
    }
    if count == 0 {
        return BatteryMetrics {
            status: if had_read_error { "unavailable" } else { "no_battery" }.to_owned(),
            battery_count: 0, charge_percent: None,
            state: if had_read_error { "unavailable" } else { "not_present" }.to_owned(),
        };
    }
    let charge_percent = if full_energy.is_finite() && full_energy > 0.0 && current_energy.is_finite() {
        Some(((current_energy / full_energy) * 100.0).clamp(0.0, 100.0) as f32)
    } else {
        None
    };
    BatteryMetrics {
        status: if had_read_error { "partial" } else if charge_percent.is_some() { "detected" } else { "unavailable" }.to_owned(),
        battery_count: count,
        charge_percent,
        state: combined_battery_state(&states),
    }
}

fn battery_state_label(state: BatteryState) -> &'static str {
    match state {
        BatteryState::Charging => "charging",
        BatteryState::Discharging => "discharging",
        BatteryState::Empty => "empty",
        BatteryState::Full => "full",
        BatteryState::Paused => "paused",
        BatteryState::Unknown => "unknown",
    }
}

fn combined_battery_state(states: &[&str]) -> String {
    let Some(first) = states.first() else { return "unknown".to_owned(); };
    if states.iter().all(|state| state == first) { (*first).to_owned() } else { "mixed".to_owned() }
}

fn disk_io_rates(read_bytes: u64, written_bytes: u64, elapsed: Option<Duration>) -> (Option<u64>, Option<u64>) {
    let Some(duration) = elapsed else { return (None, None); };
    (rate_bytes_per_second(read_bytes, duration), rate_bytes_per_second(written_bytes, duration))
}

fn max_gpu_engine_utilization(values: &[f64]) -> Option<f32> {
    values.iter().filter_map(|value| {
        if !value.is_finite() || !(0.0..=100.0).contains(value) { return None; }
        Some(*value as f32)
    }).reduce(f32::max)
}

#[cfg(target_os = "windows")]
fn gpu_adapter_key_from_instance(instance: &str) -> Option<String> {
    let start = instance.to_ascii_lowercase().find("luid_")?;
    let tail = &instance[start..];
    let end = tail.find("_phys_").unwrap_or(tail.len());
    if end <= "luid_".len() { return None; }
    Some(tail[..end].to_owned())
}

fn parse_gpu_percent_text(value: &str) -> Option<f32> {
    let parsed = value.trim().parse::<u64>().ok()?;
    if parsed > 100 { return None; }
    Some(parsed as f32)
}

fn gpu_memory_usage(used_bytes: u64, total_bytes: u64) -> (Option<u64>, Option<f32>) {
    if total_bytes == 0 { return (None, None); }
    let used = used_bytes.min(total_bytes);
    (Some(used), Some(((used as u128 * 100) / total_bytes as u128) as f32))
}

fn normalise_cpu_usage_percent(value: f32) -> f32 {
    if value.is_finite() { value.clamp(0.0, 100.0) } else { 0.0 }
}

fn rate_bytes_per_second(bytes: u64, elapsed: Duration) -> Option<u64> {
    let seconds = elapsed.as_secs_f64();
    if !seconds.is_finite() || seconds <= 0.0 { return None; }
    let rate = bytes as f64 / seconds;
    if !rate.is_finite() { return None; }
    Some(rate.round().clamp(0.0, u64::MAX as f64) as u64)
}

fn counter_rate_bytes_per_second(current: u64, previous: u64, elapsed: Duration) -> Option<u64> {
    current.checked_sub(previous).and_then(|delta| rate_bytes_per_second(delta, elapsed))
}

#[cfg(test)]
mod tests {
    #[test]
    fn gpu_engine_utilisation_uses_peak_engine_and_rejects_invalid_values() {
        assert_eq!(super::max_gpu_engine_utilization(&[12.0, 68.5, 34.0]), Some(68.5));
        assert_eq!(super::max_gpu_engine_utilization(&[f64::NAN, 101.0, -1.0]), None);
        assert_eq!(super::max_gpu_engine_utilization(&[0.0, 100.0]), Some(100.0));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_gpu_engine_instances_are_grouped_by_adapter_luid_without_exposing_it() {
        assert_eq!(
            super::gpu_adapter_key_from_instance("pid_123_luid_0x00000000_0x0000ABCD_phys_0_eng_3D"),
            Some("luid_0x00000000_0x0000abcd".to_owned())
        );
        assert_eq!(super::gpu_adapter_key_from_instance("pid_1_eng_3D"), None);
    }

    #[test]
    fn gpu_live_percent_parser_rejects_out_of_range_and_malformed_values() {
        assert_eq!(super::parse_gpu_percent_text("0"), Some(0.0));
        assert_eq!(super::parse_gpu_percent_text("100\n"), Some(100.0));
        assert_eq!(super::parse_gpu_percent_text("101"), None);
        assert_eq!(super::parse_gpu_percent_text("not-a-number"), None);
    }

    #[test]
    fn gpu_memory_usage_is_clamped_and_requires_a_nonzero_total() {
        assert_eq!(super::gpu_memory_usage(120, 100), (Some(100), Some(100.0)));
        assert_eq!(super::gpu_memory_usage(25, 100), (Some(25), Some(25.0)));
        assert_eq!(super::gpu_memory_usage(25, 0), (None, None));
    }

    #[test]
    fn system_disk_rates_require_an_elapsed_sample_and_handle_idle_io() {
        assert_eq!(super::disk_io_rates(1500, 3000, None), (None, None));
        assert_eq!(super::disk_io_rates(0, 0, Some(std::time::Duration::from_secs(2))), (Some(0), Some(0)));
        assert_eq!(super::disk_io_rates(4096, 2048, Some(std::time::Duration::from_secs(2))), (Some(2048), Some(1024)));
    }

    #[test]
    fn battery_states_are_combined_without_hiding_mixed_devices() {
        assert_eq!(super::combined_battery_state(&[]), "unknown");
        assert_eq!(super::combined_battery_state(&["charging", "charging"]), "charging");
        assert_eq!(super::combined_battery_state(&["charging", "discharging"]), "mixed");
    }

    #[test]
    fn battery_charge_is_bounded_in_live_snapshots() {
        let snapshot = super::SystemMonitor::default().sample();
        if let Some(percent) = snapshot.battery.charge_percent {
            assert!(percent.is_finite() && (0.0..=100.0).contains(&percent));
        }
        assert!(matches!(snapshot.battery.status.as_str(), "detected" | "partial" | "unavailable" | "no_battery"));
        assert!(matches!(snapshot.battery.state.as_str(), "charging" | "discharging" | "empty" | "full" | "paused" | "unknown" | "mixed" | "not_present" | "unavailable"));
        let json = serde_json::to_value(&snapshot).expect("battery metrics should serialize");
        assert!(json.get("battery").and_then(|value| value.get("chargePercent")).is_some());
        assert!(json.get("systemDiskReadBytesPerSecond").is_some());
        assert!(json.get("systemDiskWriteBytesPerSecond").is_some());
        assert!(json.get("systemDiskCount").is_some());
    }

    #[test]
    fn cpu_usage_is_clamped_and_non_finite_values_are_safe() {
        assert_eq!(super::normalise_cpu_usage_percent(-4.0), 0.0);
        assert_eq!(super::normalise_cpu_usage_percent(42.5), 42.5);
        assert_eq!(super::normalise_cpu_usage_percent(140.0), 100.0);
        assert_eq!(super::normalise_cpu_usage_percent(f32::NAN), 0.0);
        assert_eq!(super::normalise_cpu_usage_percent(f32::INFINITY), 0.0);
    }

    #[test]
    fn live_monitor_samples_only_current_process_and_returns_bounded_memory() {
        let monitor = super::SystemMonitor::default();
        let snapshot = monitor.sample();
        assert!(snapshot.available_memory_bytes <= snapshot.total_memory_bytes);
        assert!(snapshot.used_memory_bytes <= snapshot.total_memory_bytes);
        assert_eq!(snapshot.used_memory_bytes + snapshot.available_memory_bytes, snapshot.total_memory_bytes);
        assert!(snapshot.cpu_core_usage_percent.iter().all(|value| value.is_finite() && (0.0..=100.0).contains(value)));
        assert!(matches!(snapshot.process_memory.status.as_str(), "detected" | "unavailable"));
        assert!(matches!(snapshot.cpu_status.as_str(), "warming_up" | "detected" | "unavailable"));
    }
    use super::*;


    #[test]
    fn network_counter_rates_handle_deltas_resets_and_zero_elapsed() {
        use std::time::Duration;
        assert_eq!(super::counter_rate_bytes_per_second(3_072, 1_024, Duration::from_secs(2)), Some(1_024));
        assert_eq!(super::counter_rate_bytes_per_second(100, 200, Duration::from_secs(1)), None);
        assert_eq!(super::counter_rate_bytes_per_second(100, 100, Duration::ZERO), None);
        assert_eq!(super::rate_bytes_per_second(512, Duration::from_secs(2)), Some(256));
    }

    #[test]
    fn live_monitor_reports_network_and_mid_sunday_io_without_interface_names() {
        let monitor = super::SystemMonitor::default();
        let snapshot = monitor.sample();
        assert!(matches!(snapshot.network_status.as_str(), "warming_up" | "detected" | "unavailable"));
        assert!(matches!(snapshot.process_disk_io_status.as_str(), "warming_up" | "detected" | "unavailable"));
        assert!(snapshot.download_bytes_per_second.is_none() || snapshot.network_status == "detected");
        assert!(snapshot.upload_bytes_per_second.is_none() || snapshot.network_status == "detected");
        let json = serde_json::to_value(&snapshot).expect("live metrics should serialize");
        assert!(json.get("networkInterfaceCount").is_some());
        assert!(json.get("downloadBytesPerSecond").is_some());
        assert!(json.get("processReadBytesPerSecond").is_some());
        assert!(json.get("networkInterfaceNames").is_none());
    }

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
            gpu_assessment: assess_gpu_inventory(&GpuDetection { status: "unsupported_platform".to_owned(), adapters: Vec::new() }),
            gpu_memory_headroom: assess_gpu_memory_headroom(&GpuDetection { status: "unsupported_platform".to_owned(), adapters: Vec::new() }),
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
            gpu_assessment: assess_gpu_inventory(&GpuDetection { status: "unsupported_platform".to_owned(), adapters: Vec::new() }),
            gpu_memory_headroom: assess_gpu_memory_headroom(&GpuDetection { status: "unsupported_platform".to_owned(), adapters: Vec::new() }),
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
    fn capability_assessment_downgrades_profiles_under_current_memory_pressure() {
        let gib = 1024_u64 * 1024 * 1024;

        let balanced_hardware = assess_capability(16 * gib, gib, 8);
        assert_eq!(balanced_hardware.memory_tier, "Balanced");
        assert_eq!(balanced_hardware.suggested_mode, "Lightweight");
        assert!(balanced_hardware.current_memory_pressure);
        assert!(balanced_hardware.rationale.contains("reduced from Balanced to Lightweight"));

        let high_capacity_hardware = assess_capability(32 * gib, gib, 12);
        assert_eq!(high_capacity_hardware.memory_tier, "High-capacity");
        assert_eq!(high_capacity_hardware.suggested_mode, "Balanced");
        assert!(high_capacity_hardware.current_memory_pressure);
        assert!(high_capacity_hardware.rationale.contains("reduced from High capability to Balanced"));

        let comfortable = assess_capability(32 * gib, 8 * gib, 12);
        assert_eq!(comfortable.suggested_mode, "High capability");
        assert!(!comfortable.current_memory_pressure);
    }

    #[test]
    fn capability_assessment_clamps_available_memory_to_installed_ram() {
        let gib = 1024_u64 * 1024 * 1024;
        let assessment = assess_capability(8 * gib, 20 * gib, 8);
        assert!(!assessment.current_memory_pressure);
        assert_eq!(assessment.suggested_mode, "Lightweight");
        assert!(!assessment.rationale.contains("reduced from"));
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
    fn gpu_assessment_distinguishes_unsupported_unavailable_and_empty_inventory() {
        let unsupported = assess_gpu_inventory(&GpuDetection { status: "unsupported_platform".to_owned(), adapters: Vec::new() });
        assert_eq!(unsupported.status, "unsupported_platform");
        assert_eq!(unsupported.hardware_adapter_count, 0);

        let unavailable = assess_gpu_inventory(&GpuDetection { status: "unavailable".to_owned(), adapters: Vec::new() });
        assert_eq!(unavailable.status, "unavailable");

        let empty = assess_gpu_inventory(&GpuDetection { status: "no_adapters".to_owned(), adapters: Vec::new() });
        assert_eq!(empty.status, "no_adapters");
    }

    #[test]
    fn gpu_assessment_excludes_software_adapters_from_hardware_memory_summary() {
        let gpu = GpuDetection {
            status: "detected".to_owned(),
            adapters: vec![
                GpuAdapter {
                    name: "Software renderer".to_owned(), vendor_id: 0, device_id: 0,
                    dedicated_video_memory_bytes: u64::MAX,
                    dedicated_system_memory_bytes: 0, shared_system_memory_bytes: 0, is_software: true,
                    memory_budget: GpuMemoryBudget {
                        status: "unavailable".to_owned(), node_index: 0, local: None,
                        local_error_code: None, non_local: None, non_local_error_code: None,
                    },
                },
                GpuAdapter {
                    name: "Integrated GPU".to_owned(), vendor_id: 0x8086, device_id: 1,
                    dedicated_video_memory_bytes: 128 * 1024 * 1024,
                    dedicated_system_memory_bytes: 0, shared_system_memory_bytes: 8 * 1024 * 1024 * 1024, is_software: false,
                    memory_budget: GpuMemoryBudget {
                        status: "detected".to_owned(), node_index: 0,
                        local: Some(GpuMemorySegment {
                            budget_bytes: 3 * 1024 * 1024 * 1024,
                            current_usage_bytes: 512 * 1024 * 1024,
                            available_for_reservation_bytes: 2 * 1024 * 1024 * 1024,
                            current_reservation_bytes: 0,
                        }),
                        local_error_code: None, non_local: None, non_local_error_code: None,
                    },
                },
                GpuAdapter {
                    name: "Discrete GPU".to_owned(), vendor_id: 0x10de, device_id: 2,
                    dedicated_video_memory_bytes: 8 * 1024 * 1024 * 1024,
                    dedicated_system_memory_bytes: 0, shared_system_memory_bytes: 8 * 1024 * 1024 * 1024, is_software: false,
                    memory_budget: GpuMemoryBudget {
                        status: "partial".to_owned(), node_index: 0, local: None,
                        local_error_code: Some(-1), non_local: None, non_local_error_code: Some(-1),
                    },
                },
            ],
        };

        let assessment = assess_gpu_inventory(&gpu);
        assert_eq!(assessment.status, "detected");
        assert_eq!(assessment.adapter_count, 3);
        assert_eq!(assessment.hardware_adapter_count, 2);
        assert_eq!(assessment.software_adapter_count, 1);
        assert_eq!(assessment.largest_dedicated_video_memory_bytes, Some(8 * 1024 * 1024 * 1024));
        assert_eq!(assessment.largest_local_memory_budget_bytes, Some(3 * 1024 * 1024 * 1024));
        assert_eq!(assessment.adapters_with_local_memory_budget, 1);
        assert!(assessment.limitations.contains("does not detect inference-backend support"));
    }

    #[test]
    fn gpu_assessment_reports_software_only_inventory_without_calling_it_hardware() {
        let gpu = GpuDetection {
            status: "detected".to_owned(),
            adapters: vec![GpuAdapter {
                name: "Software renderer".to_owned(), vendor_id: 0, device_id: 0,
                dedicated_video_memory_bytes: 0, dedicated_system_memory_bytes: 0,
                shared_system_memory_bytes: 0, is_software: true,
                memory_budget: GpuMemoryBudget {
                    status: "unsupported_interface".to_owned(), node_index: 0, local: None,
                    local_error_code: None, non_local: None, non_local_error_code: None,
                },
            }],
        };
        let assessment = assess_gpu_inventory(&gpu);
        assert_eq!(assessment.status, "software_only");
        assert_eq!(assessment.adapter_count, 1);
        assert_eq!(assessment.hardware_adapter_count, 0);
        assert_eq!(assessment.software_adapter_count, 1);
        assert_eq!(assessment.largest_dedicated_video_memory_bytes, None);
        assert_eq!(assessment.largest_local_memory_budget_bytes, None);
    }

    #[test]
    fn gpu_memory_headroom_classifies_bands_and_excludes_software() {
        let make = |available: u64, software: bool| GpuAdapter {
            name: "test".to_owned(), vendor_id: 1, device_id: 2, dedicated_video_memory_bytes: 8 * 1024 * 1024 * 1024,
            dedicated_system_memory_bytes: 0, shared_system_memory_bytes: 0, is_software: software,
            memory_budget: GpuMemoryBudget { status: "detected".to_owned(), node_index: 0,
                local: Some(GpuMemorySegment { budget_bytes: 8 * 1024 * 1024 * 1024,
                    current_usage_bytes: 0, available_for_reservation_bytes: available, current_reservation_bytes: 0 }),
                local_error_code: None, non_local: None, non_local_error_code: None },
        };
        let gpu = GpuDetection { status: "detected".to_owned(), adapters: vec![
            make(400 * 1024 * 1024, false), make(900 * 1024 * 1024, false),
            make(4 * 1024 * 1024 * 1024, false), make(0, true),
        ]};
        let a = assess_gpu_memory_headroom(&gpu);
        assert_eq!(a.status, "detected"); assert_eq!(a.assessed_adapter_count, 3);
        assert_eq!(a.adapters_with_local_budget, 3); assert_eq!(a.adapters_with_critical_headroom, 1);
        assert_eq!(a.adapters_with_constrained_headroom, 1); assert_eq!(a.adapters_with_reported_headroom, 1);
        assert_eq!(a.lowest_available_for_reservation_bytes, Some(400 * 1024 * 1024));
        assert_eq!(a.lowest_headroom_percent, Some(4));
    }

    #[test]
    fn gpu_memory_headroom_handles_zero_missing_and_partial_inventory() {
        let make = |budget: u64, missing: bool| GpuAdapter {
            name: "test".to_owned(), vendor_id: 1, device_id: 2, dedicated_video_memory_bytes: budget,
            dedicated_system_memory_bytes: 0, shared_system_memory_bytes: 0, is_software: false,
            memory_budget: GpuMemoryBudget { status: "unavailable".to_owned(), node_index: 0,
                local: if missing { None } else { Some(GpuMemorySegment { budget_bytes: budget, current_usage_bytes: 0,
                    available_for_reservation_bytes: budget, current_reservation_bytes: 0 }) },
                local_error_code: None, non_local: None, non_local_error_code: None },
        };
        let no_budget = GpuDetection { status: "detected".to_owned(), adapters: vec![make(0, false), make(8, true)] };
        assert_eq!(assess_gpu_memory_headroom(&no_budget).status, "unavailable");
        let partial = GpuDetection { status: "detected".to_owned(), adapters: vec![make(1024, false), make(1024, true)] };
        assert_eq!(assess_gpu_memory_headroom(&partial).status, "partial");
        let source_partial = GpuDetection { status: "partial".to_owned(), adapters: vec![make(1024, false)] };
        assert_eq!(assess_gpu_inventory(&source_partial).status, "partial");
        assert_eq!(assess_gpu_memory_headroom(&source_partial).status, "partial");
        assert_eq!(assess_gpu_memory_headroom(&GpuDetection { status: "unsupported_platform".to_owned(), adapters: vec![] }).status, "unsupported_platform");
    }

    #[test]
    fn unsupported_gpu_detection_is_explicit_and_empty() {
        let inventory = GpuDetection { status: "unsupported_platform".to_owned(), adapters: Vec::new() };
        assert!(inventory.adapters.is_empty());
        assert_eq!(inventory.status, "unsupported_platform");
    }
}
