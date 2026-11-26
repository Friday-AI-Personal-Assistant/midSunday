# midSunday — Hardware Intelligence & Adaptive Resource Manager

**Status:** First cross-platform CPU/RAM/storage inventory prototype implemented; GPU/DXGI detection and adaptive recommendations remain unimplemented and unbenchmarked.  
**Priority:** Phase 1 research and architecture design.  
**Reference project:** OpenJarvis (studied 2026-11-23).

## 1. Purpose

midSunday should understand the computer it is running on and recommend a practical configuration for that specific machine. It must not assume that installed RAM alone determines what the computer can handle.

The proposed **Hardware Intelligence & Adaptive Resource Manager (HIARM)** will detect hardware, estimate current resource headroom, recommend compatible engines/models/settings, and respond safely when resources become constrained.

This is a product/architecture name for our own proposed subsystem, not a claim that OpenJarvis uses the same name.

## 2. OpenJarvis reference: what it actually does

OpenJarvis documents hardware auto-detection during `jarvis init`. Its current implementation exposes a `HardwareInfo` structure containing platform, CPU brand, CPU count, total RAM, and optional GPU metadata (vendor, name, VRAM, compute capability, and GPU count). It then uses `recommend_engine()` and `recommend_model()` to suggest an inference engine and model, and writes the defaults into `config.toml`. Users can override the choices.

Its documented detection methods include NVIDIA `nvidia-smi`, AMD `rocm-smi`, Apple `system_profiler`, and platform-specific CPU/RAM queries. The current model recommendation logic uses reported GPU VRAM when available, otherwise total system RAM with a fixed reserve/usable-memory calculation.

**Important limitations for our design:**
- The documented hardware structure does not include storage capacity, free disk space, storage type/speed, or disk health.
- The main documented workflow detects hardware and chooses setup defaults; it is not evidence of continuous, workload-aware adaptation throughout a desktop session.
- Its memory heuristic is a starting rule, not a universal guarantee that a model will fit or perform well.
- GPU reporting and acceleration support vary by vendor, driver, OS, and hardware generation.
- A reported GPU name or VRAM amount does not prove that a particular inference backend can use that GPU.

We should learn from its clean separation of detection, engine recommendation, model recommendation, and user overrides—but design a more complete Windows-first system.

### OpenJarvis references
- Hardware detection and engine architecture: https://github.com/open-jarvis/OpenJarvis/blob/main/docs/architecture/engine.md
- Current detection/recommendation implementation: https://github.com/open-jarvis/OpenJarvis/blob/main/src/openjarvis/core/config.py
- Setup and model recommendation guide: https://open-jarvis.github.io/OpenJarvis/getting-started/installation/
- Configuration and user overrides: https://open-jarvis.github.io/OpenJarvis/getting-started/configuration/

## 3. What midSunday should detect

### Hardware inventory (static or rarely changing)
- Operating system edition/build and architecture.
- CPU model, vendor, physical/logical core counts, supported instruction sets, and relevant performance capabilities where available.
- Installed physical RAM and currently available/committed memory.
- GPU adapter(s), vendor/model, dedicated VRAM, shared GPU memory, and active driver version.
- Supported acceleration backends/features, verified by capability checks rather than inferred from a product name alone.
- Storage volumes, total/free capacity, model-library destination, and storage type/performance where reliable and permissioned.
- Battery/power state and power mode on portable Windows devices, where available.

### Live resource state (dynamic)
- Available system memory and memory pressure.
- Current midSunday process memory and model/runtime memory.
- GPU memory use and availability where the driver exposes reliable data.
- CPU load and thermal/throttling signals where available without intrusive or vendor-specific dependencies.
- Free space on the destination volume before downloads, extraction, conversion, or updates.
- Whether another midSunday model or speech workload is already active.

Detection must degrade gracefully. A missing sensor or unsupported GPU must not prevent the application from starting.

## 4. Windows-first detection approach

The first implementation target is Windows. Candidate mechanisms to evaluate include:
- Windows system APIs for OS, CPU topology/capabilities, physical/available memory, and process memory.
- DXGI and supported Windows graphics interfaces for adapter inventory and memory budgets.
- Vendor-supported tools/APIs (for example NVIDIA tooling) only as optional capability providers, not mandatory dependencies.
- Windows storage APIs for volume capacity and available space; storage type/speed should be reported only when detection is reliable.
- Power and battery APIs for laptop-aware scheduling.

These are research candidates, not approved dependencies. Prefer documented Windows APIs and Rust-compatible libraries with suitable licences. Do not require administrator privileges for ordinary hardware detection. Avoid running arbitrary shell commands or scraping sensitive system information.

## 5. Recommendation pipeline

Keep these stages separate and testable:

1. **Detect:** Collect hardware inventory and current resource state.
2. **Validate:** Determine which reported capabilities are actually usable by installed drivers and candidate runtimes.
3. **Estimate:** Estimate resource needs for a specific model, quantisation, context length, GPU offload, and selected speech pipeline.
4. **Recommend:** Rank compatible configurations by the user's preference (quality, speed, low resource use, or battery life).
5. **Explain:** Show why a choice is recommended, expected download/storage size, estimated memory needs, limitations, and uncertainty.
6. **Confirm:** Ask before downloading large assets, changing the selected model, or making a material resource trade-off.
7. **Monitor:** Observe resource pressure while midSunday is working.
8. **Adapt safely:** Reduce optional concurrency, pause/unload eligible background workloads, or offer a smaller configuration when necessary. Never silently replace the user's selected model or discard work.

A recommendation is not a guarantee. The catalogue should use measured benchmarks for exact model/runtime/quantisation combinations whenever available.

## 6. User control

Candidate presets:
- **Automatic:** Recommend and adjust safe scheduling based on measured resources.
- **Lightweight:** Prioritise responsiveness, lower memory use, and battery life.
- **Balanced:** Balance answer quality and responsiveness.
- **High capability:** Prefer stronger models and richer workloads when the machine can support them.

Users must also be able to:
- Inspect detected hardware and the reason behind a recommendation.
- Choose a different model/engine where compatible.
- Choose a smaller model even on a powerful computer.
- Set a preferred model-library location.
- Set resource limits or disable automatic workload changes.
- Refresh hardware detection.
- See when detection is incomplete or stale.

Automatic mode must not silently download a model, incur a charge, enable cloud inference, or send system details externally.

## 7. Runtime resource management

- Installed RAM is not the same as available RAM; use live headroom and a safety margin.
- Do not treat system RAM, dedicated VRAM, and shared GPU memory as interchangeable.
- Account for model weights, context/KV cache, runtime overhead, desktop UI, audio buffers, STT/TTS models, and Windows/other-app overhead.
- Prefer one active LLM by default until benchmarks justify more concurrency.
- Schedule STT/TTS around chat generation on constrained systems; allow more concurrency on capable systems when measured results support it.
- Check disk space before downloads and temporary extraction; preserve enough space for rollback/recovery.
- React to memory pressure gracefully: pause optional work, offer to unload an idle speech engine, or ask the user before switching configuration.
- Do not terminate unrelated user applications or processes.
- Do not over-promise performance based on a RAM tier or GPU marketing name.

## 8. Privacy and safety

- Hardware detection runs locally and does not require an account or network connection.
- Do not upload hardware inventory, process lists, device identifiers, or performance telemetry by default.
- Do not collect serial numbers or unique device identifiers unless a separately justified feature requires them and the user explicitly consents.
- Keep detection read-only; no registry modification, driver installation, overclocking, or system setting changes.
- Never use model output to make security or permission decisions.
- Hardware information shown in diagnostic exports must be user-reviewed and opt-in.

## 9. Acceptance criteria

The subsystem is not complete until tests demonstrate that it:
- Detects CPU, core counts, installed/available RAM, GPU/VRAM, OS, and storage free capacity correctly on supported Windows configurations.
- Clearly labels unsupported or unavailable measurements rather than inventing values.
- Distinguishes dedicated VRAM from shared memory.
- Checks available disk space before large model operations.
- Recommends only models whose runtime, quantisation, context, and acceleration path have been evaluated.
- Explains recommendations and allows manual override.
- Does not silently download, switch models, use cloud inference, or change system settings.
- Responds safely to memory/disk pressure without losing user data or killing unrelated processes.
- Passes repeatable tests on representative 4 GB, 6 GB, 8 GB, 16 GB, 32 GB, and 64 GB+ systems as hardware becomes available.
- Operates offline and produces no outbound hardware telemetry by default.

## 10. Open research questions

- Which Windows APIs and Rust crates provide the most reliable cross-vendor hardware inventory?
- How should we query NVIDIA, AMD, Intel, and integrated GPU capabilities without requiring multiple mandatory vendor utilities?
- Which GPU memory figures are reliable enough for model placement and runtime safety?
- How should CPU instruction sets and NPU availability affect model/runtime recommendations?
- How can we estimate memory for different model architectures, quantisations, context sizes, and GPU offload configurations?
- Which storage characteristics can be measured reliably without administrator access?
- What resource-pressure thresholds should trigger warnings, workload scheduling, or user prompts?
- Which measurements can be collected with low overhead and without invasive monitoring?

## 11. Current status

No detection code, Windows API dependency, runtime recommendation policy, or adaptive controller has been implemented or approved. OpenJarvis is a reference, not a dependency. This subsystem remains a Phase 1 research/design task until the Windows detection approach and acceptance tests are validated.

## 12. Windows detection research — API and Rust-library shortlist (2026-11-24)

**Research status:** Desk research complete for the initial Windows API map. No API has been integrated, no dependency has been added, and no Windows machine has been benchmarked.

### 12.1 Recommended layered approach

Use a layered detector rather than a single all-purpose library:

1. **Cross-platform baseline candidate:** `sysinfo` for OS information, CPU entries, system/process memory, process information, and disk/volume enumeration. It supports Windows and documents that its `System` instance should be retained and refreshed rather than recreated for every read. On Windows, its `free_memory` result is the same as `available_memory`, so the app must label this as available memory, not literal unused/free memory.
2. **Windows-native detail candidate:** Microsoft's `windows` Rust crate (or `windows-sys` where raw bindings are more appropriate) for exact Windows APIs and COM/DXGI. Microsoft documents this as the official Rust projection of Win32, COM, and WinRT APIs.
3. **Optional vendor diagnostics:** NVIDIA NVML through an optional adapter only if a supported NVIDIA driver exposes the library. Never make it a prerequisite for the app or for basic GPU identification.
4. **Runtime capability probe:** Ask the actual selected inference runtime to report/validate the acceleration backends it was built with, then verify by a controlled model-load/inference test. Hardware inventory alone cannot prove a runtime can use a GPU.

This separation limits mandatory dependencies and allows the app to degrade gracefully when a sensor, API, driver, or optional library is unavailable.

### 12.2 API-to-information map

| Information | Primary Windows mechanism to evaluate | Important caveat |
|---|---|---|
| Installed and available physical RAM | `GlobalMemoryStatusEx` / `MEMORYSTATUSEX` | Values are live and volatile; re-query before major model operations. |
| Logical/physical CPU topology | `GetLogicalProcessorInformationEx` | Parse core, package, cache and NUMA relationships; do not assume logical processor count equals physical cores. |
| CPU architecture / basic system information | `GetNativeSystemInfo` / `GetSystemInfo` | Architecture and topology do not by themselves identify CPU model or performance. |
| CPU brand/model and per-core data | Evaluate `sysinfo` first; consider Windows registry/WMI only if a missing field is essential | Keep vendor-specific performance-core details optional; avoid fragile parsing of marketing strings. |
| GPU adapter inventory | DXGI `IDXGIFactory1::EnumAdapters1` + `IDXGIAdapter1::GetDesc1` | Enumerate every adapter, including integrated and discrete GPUs; identify software/remote adapters and do not treat them as local accelerators. |
| Dedicated/shared GPU memory | `DXGI_ADAPTER_DESC1` fields | Describes adapter memory capacity, not currently available memory for midSunday. Shared memory is system memory, not extra dedicated VRAM. |
| Per-process GPU memory budget/usage | `IDXGIAdapter3::QueryVideoMemoryInfo` | Reports the OS budget and current usage for this process and memory segment; budget changes dynamically. It is not a global free-VRAM counter. |
| Volume total/free/available space | `GetDiskFreeSpaceExW` for the actual model-library path | Distinguish free space available to the user from total free space; check the destination path immediately before downloads/extraction. |
| Drive class | `GetDriveTypeW` | Distinguishes fixed/removable/network/RAM/CD-ROM; it does not tell SSD vs HDD. |
| Storage seek-penalty capability | `IOCTL_STORAGE_QUERY_PROPERTY` with `StorageDeviceSeekPenaltyProperty` | Optional, more complex device query; report unknown if unsupported. Do not infer exact read/write throughput from this flag. |
| Power/battery state | Windows power-status APIs | Useful for laptop scheduling, but should be a later refinement, not a blocker for hardware detection. |

### 12.3 GPU detection: avoid a misleading shortcut

DXGI is the best first generic Windows path found in this review:
- `EnumAdapters1` enumerates the system's display adapters.
- `GetDesc1` returns adapter name, vendor/device IDs, dedicated video memory, dedicated system memory, shared system memory, and adapter identity.
- `IDXGIAdapter3::QueryVideoMemoryInfo` reports the current process's OS-assigned memory budget and usage for local/non-local segments.

The model selector must keep these concepts separate:
- **Dedicated VRAM capacity:** hardware inventory.
- **Current process GPU budget/usage:** dynamic process-specific resource headroom.
- **Shared GPU memory:** system RAM that the GPU may use; never count it as dedicated VRAM.
- **Runtime acceleration support:** confirmed by the chosen backend, driver, and actual runtime probe.

Do not use WMI `Win32_VideoController.AdapterRAM` as the authoritative VRAM source. Its field is a 32-bit integer, and Microsoft's documentation warns that WMI values can be inaccurate on hardware not compatible with WDDM. WMI may still be a fallback for display metadata if needed, but it must be marked lower-confidence.

### 12.4 Rust library candidates

| Candidate | Potential role | Preliminary view |
|---|---|---|
| `sysinfo` | Cross-platform CPU, memory, process and disk baseline | Strong candidate to evaluate; MIT-licensed upstream repository; current docs list Windows support. Latest documented version 0.40.4 has Rust MSRV 1.95, so check the project's pinned toolchain and choose a compatible version before adoption. |
| `windows` | Direct, typed access to Win32 and DXGI/COM APIs | Leading Windows-specific binding candidate; Microsoft-maintained and dual-licensed MIT/Apache-2.0. Use only required API feature flags and pin a tested version. |
| `windows-sys` | Lower-level Windows FFI | Alternative where the typed `windows` projection is unnecessary; assess API safety and dependency impact. |
| `nvml-wrapper` | Optional NVIDIA telemetry (memory, utilization, power, etc.) | Optional only; runtime-loads NVML and can fail gracefully when it is absent. It does not solve AMD/Intel coverage. Upstream crate is MIT/Apache-2.0. |
| `gpu-info` / `gpu-info-rs` | Potential shortcut for GPU memory information | Research lead only. Review source, maintenance, API semantics, license, and multi-adapter behaviour before considering it; do not add it merely to avoid a small DXGI integration. |

**Dependency policy:** no crate is approved for production yet. Check the exact version, transitive dependencies, MSRV, licence, maintenance, Windows target coverage, and feature flags against the actual midSunday Rust toolchain before adding it.

### 12.5 Detection confidence and fallback policy

Every field should carry a value plus a confidence/status such as:
- `verified`: obtained from a reliable OS/runtime API.
- `reported`: driver or system metadata, not independently validated.
- `estimated`: derived from a documented heuristic.
- `unavailable`: API/driver did not provide it.
- `unsupported`: the platform or device does not expose it.

Never turn a failed query into zero and then interpret zero as no hardware. Never use an unknown value as proof that an adapter is unsupported. The UI should explain incomplete detection without blocking local CPU inference.

### 12.6 Initial implementation recommendation — not yet approved

When Phase 1 reaches implementation readiness, prototype this minimal stack first:
- `sysinfo` for general OS/CPU/memory/process/disk inventory where its output meets our needs.
- `windows` crate for `GlobalMemoryStatusEx`, CPU topology, DXGI adapter enumeration and video-memory budgeting, and exact free-space checks.
- No mandatory NVML, WMI, PowerShell, `nvidia-smi`, or `rocm-smi` dependency.
- Optional vendor telemetry only after generic Windows coverage is tested and a concrete gap is demonstrated.

The prototype should return a typed, serialisable hardware snapshot and a separate live-resource snapshot. It must not select or download models yet.

### 12.7 Sources

Microsoft Windows APIs:
- GlobalMemoryStatusEx: https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-globalmemorystatusex
- GetLogicalProcessorInformationEx: https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getlogicalprocessorinformationex
- Rust for Windows / windows crate: https://learn.microsoft.com/en-us/windows/dev-environment/rust/rust-for-windows
- DXGI adapter enumeration: https://learn.microsoft.com/en-us/windows/win32/api/dxgi/nn-dxgi-idxgiadapter1
- DXGI_ADAPTER_DESC1: https://learn.microsoft.com/en-us/windows/win32/api/dxgi/ns-dxgi-dxgi_adapter_desc1
- IDXGIAdapter1::GetDesc1: https://learn.microsoft.com/en-us/windows/win32/api/dxgi/nf-dxgi-idxgiadapter1-getdesc1
- DXGI video memory budget: https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_4/ns-dxgi1_4-dxgi_query_video_memory_info
- GetDiskFreeSpaceExW: https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getdiskfreespaceexw
- GetDriveTypeW: https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getdrivetypew
- Storage seek penalty: https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntddstor/ns-ntddstor-_device_seek_penalty_descriptor
- WMI video controller caveat: https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/win32-videocontroller

Rust libraries:
- sysinfo docs: https://docs.rs/sysinfo/latest/sysinfo/
- sysinfo repository/licence: https://github.com/GuillaumeGomez/sysinfo
- windows-rs: https://github.com/microsoft/windows-rs
- windows crate API docs: https://microsoft.github.io/windows-docs-rs/
- nvml-wrapper: https://docs.rs/nvml-wrapper/latest/nvml_wrapper/

**Conclusion:** A credible Windows-first detector appears feasible with standard OS APIs and a small number of Rust bindings. The strongest initial direction is `sysinfo` for the cross-platform baseline plus Microsoft's `windows` crate for precise Windows-specific functions. This remains a research recommendation, not a final dependency approval.

## 13. First inventory prototype (2026-11-25)

The first HIARM code slice uses the cross-platform `sysinfo` 0.37 crate to collect a one-time local snapshot of OS name/version, process architecture, CPU brand, logical and physical core counts where available, total/available RAM, and storage-volume total/free capacity.

The detector deliberately avoids process enumeration, host names, network interfaces, device serial numbers, and user-file inspection. It runs on demand and in a blocking worker so hardware enumeration does not block the UI. The Settings → System & hardware panel shows the result and allows manual refresh.

GPU adapter/VRAM enumeration, DXGI memory budgets, instruction-set feature validation, live resource monitoring, model recommendations, and workload scheduling are not implemented in this slice. An unavailable field must remain explicitly unavailable; it must never be guessed from RAM or CPU marketing names.

## 14. Model catalogue connection

Hardware detection must not hard-code one model per RAM tier. It should query a versioned model catalogue containing exact model revision, architecture, quantisation, file size, runtime compatibility, context profile, measured peak system RAM, GPU memory behaviour, quality scores, and licence/provenance records. Recommendations should use measured resource estimates for the selected runtime and context, then compare those estimates with live available RAM and DXGI process budgets.

The preliminary Qwen3.5/Gemma 4 candidate ladder and test buckets are recorded in [PHASE-1-RESEARCH.md](PHASE-1-RESEARCH.md#16-preliminary-local-model-ladder--qwen35-and-gemma-4-candidates-2026-11-24). These are research candidates, not installed models or approved defaults.


## 15. First non-binding capability assessment (2026-11-25)

The first assessment layer converts the existing local RAM and CPU inventory into a cautious starting-mode suggestion for Settings. It is deliberately separate from model selection and runtime validation.

### Provisional rules

- Below 6 GiB installed RAM: Constrained / Lightweight.
- 6 GiB to below 12 GiB: Entry / Lightweight.
- 12 GiB to below 24 GiB: Balanced / Balanced.
- 24 GiB to below 48 GiB: High-capacity / High capability, provided at least 8 logical processors are detected; otherwise Balanced.
- 48 GiB or more: Very high-capacity / High capability, provided at least 8 logical processors are detected; otherwise Balanced.
- Fewer than 4 logical processors caps the suggestion at Lightweight.
- Missing RAM or CPU topology yields Undetermined rather than a fabricated recommendation.
- Current available memory is separately flagged when it falls below the greater of 2 GiB or one-sixth of installed RAM. This is a transient warning, not a permanent hardware tier.

These thresholds are intentionally conservative and provisional. They do not certify that a model, context length, GPU offload, or simultaneous speech workload will run. GPU/VRAM, runtime acceleration, model-specific measured memory, and benchmark results are not part of this first assessment.

### Behaviour and privacy

- The assessment is computed locally from the existing snapshot.
- It is informational only: no model is selected, no setting is changed, no download occurs, and no process is terminated.
- The user may continue choosing settings manually.
- The assessment must be recalculated when the hardware snapshot is refreshed.
- Do not describe the suggested mode as a guaranteed compatibility result or benchmark-backed recommendation.

This is the first HIARM assessment slice, not the adaptive resource manager. Live monitoring, model catalogue estimates, GPU detection, and workload scheduling remain separate future work.


## 16. Runtime-detected CPU instruction sets (2026-11-25)

The hardware snapshot reports optional CPU instruction-set features using Rust standard-library runtime probes:
- x86/x86-64: SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, AVX, AVX2, and FMA where detected.
- AArch64: NEON where detected.
- Other architectures: explicitly report detection as unsupported rather than implying the processor has no capabilities.

On x86, Rust's AVX/AVX2 probes account for operating-system support for extended register state; a raw CPUID flag alone would be insufficient. Detection is read-only, local, and uses no external executable or vendor utility.

The UI displays the detected features as informational labels. These are CPU/OS capabilities only: they do not prove that llama.cpp or another inference runtime includes matching optimized kernels, and they do not establish GPU acceleration support. Runtime-specific compatibility checks remain separate.


## 17. Windows DXGI display-adapter inventory prototype (2026-11-25)

The Windows hardware snapshot now uses Microsoft's Rust `windows` bindings and DXGI 1.1 adapter enumeration to report display-adapter name, PCI/ACPI vendor and device IDs, dedicated video memory capacity, dedicated system memory, shared system memory, and software-adapter flags. Multiple adapters are retained rather than selecting one based on display output order.

The implementation is Windows-only and degrades to an explicit unsupported/unavailable status on other platforms or when DXGI cannot be queried. It performs read-only local enumeration; no vendor utility, administrator permission, network request, driver installation, or GPU benchmark is required. The enumeration has a defensive 32-adapter bound.

**Important limits:** These are adapter capacity figures, not current free VRAM, not the process-specific DXGI memory budget, and not proof that a selected inference runtime can accelerate on that adapter. Shared system memory remains separate from dedicated video memory. Software adapters are identified so they are not mistaken for physical hardware accelerators. DXGI remote-adapter classification is not treated as reliable because its remote flag is reserved.

The next GPU work remains process-specific `IDXGIAdapter3::QueryVideoMemoryInfo` budget/usage sampling, followed by validation against Windows tools on real multi-GPU and integrated/discrete systems. No model selection or automatic offload is enabled by this inventory slice.

References: Microsoft [EnumAdapters1](https://learn.microsoft.com/en-us/windows/win32/api/dxgi/nf-dxgi-idxgifactory1-enumadapters1), [DXGI_ADAPTER_DESC1](https://learn.microsoft.com/en-us/windows/win32/api/dxgi/ns-dxgi-dxgi_adapter_desc1), and the [windows 0.62.2 Rust bindings](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Graphics/Dxgi/).


## 18. Windows process-specific DXGI memory budget snapshot (2026-11-25)

The Windows inventory now attempts to query `IDXGIAdapter3::QueryVideoMemoryInfo` for node zero on each enumerated adapter. It records separate local and non-local segment values: the operating-system budget, current usage attributed to the midSunday process, available-for-reservation amount, and current reservation. Each segment is optional and reports an explicit query status; a missing interface or failed query does not erase the adapter inventory.

This is a point-in-time, read-only snapshot refreshed only when the user requests a hardware refresh. It does not register budget-change events, reserve memory, change GPU settings, create a graphics device, select an adapter, or prove that the local AI runtime uses that adapter. The OS can revise budgets after the snapshot. Non-local memory is system-backed and must not be presented as dedicated VRAM. The current process usage may be low or zero before a GPU workload has allocated resources.

Microsoft documents that the budget is the process's OS-provided target and that usage beyond it can cause performance penalties. See [IDXGIAdapter3::QueryVideoMemoryInfo](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_4/nf-dxgi1_4-idxgiadapter3-queryvideomemoryinfo), [DXGI_QUERY_VIDEO_MEMORY_INFO](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_4/ns-dxgi1_4-dxgi_query_video_memory_info), and the [windows 0.62.2 Rust bindings](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Graphics/Dxgi/struct.IDXGIAdapter3.html).

Real-device comparison against Windows Task Manager / GPU engine telemetry and multi-GPU hardware remains required before using these fields for recommendations or workload scheduling.


### Process-specific DXGI query diagnostics (2026-11-25)

The current Windows snapshot calls `IDXGIAdapter3::QueryVideoMemoryInfo` for **node index 0** on each enumerated adapter, separately for local and non-local segments. This is deliberately labelled as node zero; it does not enumerate or claim to cover every node in a multi-node adapter. Each segment query preserves its own result and, when it fails, the signed HRESULT is retained for diagnostics and rendered as an eight-digit hexadecimal code. An unsupported adapter interface is distinct from an attempted query that failed. The snapshot remains on-demand and read-only. Node enumeration, runtime-to-adapter mapping, and real-device comparison with Windows Task Manager remain open validation work.


### Snapshot freshness and timestamps

Every hardware snapshot includes `capturedAtUnixSeconds`, a Unix timestamp in seconds recorded when collection completes. The Settings panel renders this localised timestamp so users can distinguish a fresh manual refresh from an older snapshot. It is diagnostic metadata only; it is not persisted as a device history, uploaded, or used to infer hardware changes. A failed collection leaves the previous successful snapshot visible with its original timestamp and shows the separate error message.


## 19. Storage free-space health indicators (2026-11-25)

Each locally enumerated storage volume now includes a conservative free-space percentage and a simple status:
- **Critical:** less than 2 GiB free or less than 5% free.
- **Low:** less than 10 GiB free or less than 10% free, unless already critical.
- **Healthy:** at least 10 GiB and at least 10% free.
- **Unknown:** capacity is zero or unavailable.

Available bytes are clamped to the reported total before percentages are calculated, and zero-capacity volumes never cause division by zero. Settings shows used-space bars and the corresponding status. This is a general volume warning only: it does not identify the volume containing a configured model, estimate a model's full runtime footprint, predict download/install success, or guarantee adequate space for future models. Values are local, on-demand snapshot data and are not uploaded or stored as a history.


## 20. Current midSunday process memory snapshot (2026-11-25)

The on-demand hardware refresh now queries memory for **only the current midSunday process**. It reports resident memory (the process's current physical-memory footprint) and virtual address-space size as separate values. The UI may show resident memory as a percentage of installed physical RAM when that denominator is available.

The probe requests only the current process ID internally and does not return or persist the PID. It does not enumerate other processes or collect their names, command lines, memory use, or application identities. If the operating system does not provide the current process record, the fields remain unavailable rather than being guessed. Values are point-in-time diagnostics and are not a history or a measure of model-only memory.

Virtual address space must not be confused with physical RAM consumption. Resident memory includes all memory resident for the midSunday process, not just an AI model. This metric does not identify individual allocations, prove GPU use, or trigger resource-management actions. It remains local and on-demand.

## 21. Configured model storage-volume matching (2026-11-25)

The hardware refresh accepts the configured GGUF model path and resolves the existing file locally. It matches the canonical file path to the most specific enumerated storage-volume mount point. Settings displays the matched volume, available bytes, free percentage, and general volume health.

No model file contents are read. The full configured path is not copied into the hardware snapshot; only the matched mount point and capacity indicators are returned. Empty, missing, inaccessible, or unmatched paths produce explicit statuses. The check runs only when the user refreshes hardware information.

This is volume identification, not a model-fit calculation. It does not estimate download staging space, model conversion overhead, context/KV cache, runtime allocations, or GPU memory. Real-device validation remains required on Windows, Linux, and macOS.


## 22. Bounded GGUF metadata inspection (2026-11-25)

The local model library can explicitly inspect the metadata section of an existing .gguf file. The Rust inspector reads the GGUF v2/v3 header and bounded metadata records, then stops before tensor descriptors and tensor weight payloads. It reports container version, file size, tensor count, metadata-entry count, architecture, embedded model name, GGML file-type code and known description, parameter count, context length, block/layer count, embedding length, and attention-head count when those fields are present.

The implementation uses only Rust standard-library file I/O plus the existing Serde dependency; no third-party parser or runtime dependency is added. It enforces limits on metadata entry counts, string lengths, and array lengths, rejects malformed/truncated/unsupported structures with clear errors, and skips unknown supported scalar/array metadata without loading it into memory. Inspection is performed on a blocking worker and is initiated by the user for an individual catalogue entry.

**Privacy and safety:** inspection is local and read-only. It does not load model weights, copy or modify the GGUF file, start inference, download anything, or transmit the file path or metadata. GGUF metadata is untrusted input and may be missing, incomplete, or inaccurate. The displayed values are descriptive only—not proof of model compatibility, runtime kernel support, safe memory requirements, licence rights, or expected performance. No automatic model selection, context change, or resource allocation is performed.

Regression tests cover representative llama metadata, quantisation labels, unsupported versions, invalid magic, truncated data, excessive metadata counts, unknown string/array fields, and incorrect extensions. Architecture-specific metadata coverage and real-world model corpus validation remain ongoing work.

## 23. Extended GGUF architecture metadata (2026-11-25)

The metadata inspector additionally reads selected standard GGUF integer fields when present: grouped-query attention KV-head count, key/value head lengths, RoPE dimension count, expert count and experts used per token, quantisation-format version, and tensor alignment. Architecture-specific values are read from namespaced metadata keys, alongside the standard `general.quantization_version` and `general.alignment` keys. Both 32-bit and 64-bit unsigned integer encodings are accepted where supported by the GGUF format.

The UI reports absent values as unavailable. It preserves the distinction between attention heads and KV heads. Expert counts are descriptive metadata and do not imply runtime support for a particular MoE implementation. Quantisation version is distinct from the GGML file-type/quantisation label. If alignment is absent, the UI does not claim a file-specific value.

These additions are groundwork for later context/KV-cache estimates and architecture-aware model checks. They do not calculate memory use, choose a model, infer runtime support, or change launch settings. Metadata remains untrusted file-supplied information. Tests use a synthetic GGUF fixture with representative LLaMA-style architecture keys; real-world multi-architecture corpus validation remains open.


## 24. Conservative GGUF-derived KV-cache estimate (2026-11-25)

The local model library now offers an explicit per-model KV-cache estimate for the context length currently entered in Local AI engine settings. The calculation uses GGUF-reported layer count, KV-head count, and key/value head lengths. It calculates key and value cache separately using: layers × KV heads × head dimension × configured context tokens × 2 bytes per element, then sums both sides.

The estimate assumes one sequence and FP16 K/V values (2 bytes per element). It is calculated on a blocking Rust worker with checked integer arithmetic. Invalid context ranges, zero-valued dimensions, and overflow are rejected. If any required metadata is absent, the result is explicitly unavailable; missing dimensions are never guessed. The user triggers the estimate for each model, and changing the context setting requires a fresh estimate.

**Important limitations:** this is only a theoretical KV-cache estimate, not total model RAM or VRAM. It excludes model weights, runtime/allocator overhead, compute buffers, graph/workspace memory, batching or parallel sequences, padding, and backend-specific behaviour. Actual cache precision may differ from FP16. It does not determine CPU/GPU placement, prove backend support, guarantee that a model fits, select a model, or change settings. GGUF metadata remains untrusted and real-world multi-architecture/runtime validation is still required.

Regression tests cover the arithmetic with a synthetic fixture, missing metadata, invalid context, zero dimensions, and overflow.


## 25. Managed inference-process memory snapshot (2026-11-26)

Settings can explicitly sample resident and virtual memory for the single `llama-server` child process launched and owned by the current midSunday runtime session. The process supervisor supplies its child PID internally; the resource monitor refreshes only that exact PID and does not enumerate unrelated processes. The PID is not returned to the frontend or persisted.

The snapshot is user-triggered and point-in-time. If no managed child is active, or the process has exited or cannot be inspected, the result is reported as not running or unavailable rather than guessed. Resident memory is the process's current physical-memory footprint; virtual memory is address-space size and must not be interpreted as physical RAM consumption.

This measurement covers the whole llama-server process, not model weights alone. It may include runtime allocations, mapped files, buffers, and other process memory. It does not report GPU allocations, separate model weights from KV cache, measure peak memory, or prove the selected backend is using a particular accelerator. It does not change runtime settings, terminate processes, or trigger model selection. The query remains local and read-only.

## 26. Inference-generation memory boundary report (2026-11-26)

After each successful local text generation, the runtime manager takes two additional targeted memory samples for the same managed llama-server child: one immediately before the request and one immediately after it completes. The report includes both resident/virtual readings, signed end-minus-start deltas where both readings are available, a complete/partial status, and a local capture timestamp.

Only the already-known managed child PID is queried. No process enumeration, PID exposure, background polling, persistent telemetry history, or network transfer is introduced. If either boundary sample is unavailable, the corresponding delta remains unavailable rather than being inferred. The UI displays the most recent successful generation report and clears it when the runtime is restarted or stopped.

**Interpretation limits:** these are two point-in-time boundary samples, not continuous monitoring or peak-memory measurements. A positive resident delta means the process ended the request with more resident memory than it had immediately before; it does not prove that the request caused every change or identify model weights, KV cache, compute buffers, or individual allocations. Memory may rise and fall between samples. Virtual-memory delta is address-space change, not physical RAM use. GPU allocations remain outside this report. The report is diagnostic only and does not alter runtime settings, choose a model, or claim model-fit compatibility.

Regression tests cover positive, negative, zero, missing, and out-of-range signed deltas.

## 27. Managed inference-process peak memory sampling (2026-11-26)

During each local generation request, the runtime manager starts a short-lived sampler for the exact managed llama-server child PID. It takes targeted samples at a 250 ms interval while generation is active, then stops and joins the sampler before returning the response report. The before- and after-generation boundary readings are also included when calculating the observed maximum. The report includes peak observed resident/virtual memory, number of valid readings, number of interval readings, sampling interval, and explicit detected/unavailable/not-running status.

The sampler is request-scoped: it does not run in the background while idle, does not persist a history, and does not upload telemetry. The PID stays inside the Rust process boundary. Each interval read refreshes only the known PID through the existing targeted process query; unrelated processes are not enumerated.

**Interpretation limits:** this is the maximum value observed at the sampling points, not a guaranteed true peak. A short-lived spike between 250 ms samples may be missed. The values cover the entire llama-server process and do not isolate model weights, KV cache, compute buffers, or GPU memory. Sampling overhead is expected to be small but must be checked on real devices. This diagnostic does not alter inference settings, reserve memory, select a model, or guarantee model fit.

Regression tests verify maximum tracking, interval sample counting, and unavailable field handling.


## 28. Model-startup peak memory sampling (2026-11-26)

When the user starts the local inference engine, the runtime records targeted memory samples for only the exact managed `llama-server` child while the server is becoming ready and loading the selected GGUF model. It captures an initial sample, samples during readiness polling at no more than the configured poll frequency, and takes a final sample once readiness succeeds. The report retains the maximum observed resident and virtual memory, the number of valid samples, the sampling interval, an explicit detected/unavailable status, and a local capture timestamp.

The sample query uses the already-known child PID and does not enumerate unrelated processes. No PID is returned to the frontend. Sampling is bounded by the existing startup timeout and readiness polling; there is no idle/background monitor, persistent history, or network telemetry. The latest successful startup report is retained in the in-memory runtime status for display in Settings and is replaced on the next successful start.

**Interpretation limits:** the report describes the whole llama-server process, not model weights alone. It is the maximum observed at sample points, not a guaranteed true peak; spikes between polls may be missed. It excludes GPU allocations and does not prove the model is fully resident in physical RAM, guarantee model fit, or establish runtime compatibility. A failed startup does not produce a completed startup report. Real-device comparison against operating-system process monitors remains required before using these values for recommendations or adaptive resource management.

Regression coverage for sample aggregation and real-device validation remain follow-up work.


## 29. Startup duration and memory-growth context (2026-11-26)

The startup report now includes elapsed time from immediately before launching the managed server process until the local readiness check succeeds. It also records the first valid process-memory sample as the startup baseline and reports the observed peak-minus-initial resident and virtual memory deltas. These values help distinguish the process's early footprint from the highest value observed during model startup.

The deltas are signed and remain unavailable when either baseline or peak data is missing or the magnitude cannot be represented safely. Startup duration includes process launch, server initialization, model loading, and readiness response; it is not a pure model-load benchmark. Measurements are point-in-time observations for the managed process only, remain local/in-memory, and are not persisted as telemetry.

**Interpretation limits:** observed growth is not a model-weight size or a total system-memory impact measurement. It excludes GPU allocations, other processes, and transient spikes between samples. Different machines, models, quantisations, runtimes, context settings, and cache states affect results. No automatic hardware profile, model selection, or runtime tuning is enabled by these diagnostics.

Regression tests cover positive, negative, zero, missing, and out-of-range signed memory deltas.


## 30. GGUF tensor storage analysis (2026-11-26)

The model library can explicitly inspect GGUF tensor descriptors without loading tensor weights. The bounded read-only pass validates the container header and metadata, descriptor dimensions, data offsets, alignment, and file boundaries. It calculates payload bytes only for a deliberately limited set of GGML types whose block layouts are known and tested (F32, F16, Q4_0, Q4_1, Q5_0, Q5_1, Q8_0, Q8_1, Q2_K through Q8_K, I8/I16/I32/I64, F64, and BF16). Results are grouped by type and include analysed/unknown counts. Unsupported types and dimensions that do not satisfy the known block layout remain unknown; no guessed byte sizes are reported.

Known payload totals exclude inter-tensor alignment padding and do not claim to represent every byte of the GGUF file. This is stored tensor payload size only—not total file size, runtime RAM, VRAM, KV cache, allocator overhead, compute buffers, or a model-fit guarantee. The action is user-triggered, local, and read-only; it does not load weights, change settings, select a model, or contact a network service. The supported type list is intentionally conservative and must be expanded only with verified upstream layouts and regression fixtures.

Regression coverage includes supported block-size calculations, unknown types, invalid alignment, malformed descriptors, offsets, truncated files, and overflow boundaries.
