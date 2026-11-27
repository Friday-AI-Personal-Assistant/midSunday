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


## 31. GGUF tensor payload range integrity (2026-11-26)

Before reporting known tensor payload totals, the analyser orders descriptors by their relative data offsets and verifies that every calculable, supported payload range ends before the next calculable supported range begins. Overlapping known ranges are rejected as malformed rather than double-counted. Descriptor order in the file is not assumed to be storage order. Checked arithmetic remains in place for range ends and aggregate counts.

This is a structural integrity check, not a complete proof of the file's contents. Unsupported tensor types have unknown payload lengths, so their ranges cannot be checked for overlap with known or other unknown tensors. Inter-tensor padding is still excluded from the known payload total. The analyser remains local, user-triggered, read-only, and does not load tensor weights.

Regression fixtures cover overlapping known ranges and valid non-overlapping ranges whose descriptor order differs from their storage order.


## 32. GGUF known-payload overlap against unknown tensor starts (2026-11-26)

The tensor analyser now compares each known-size payload end with the next descriptor's starting offset even when that next tensor type is unsupported or has an invalid/unknown block layout. A known tensor payload that extends into the next tensor's declared start is rejected. This closes a gap where the next descriptor's start was known but its payload length was not.

Unknown tensor payload lengths remain unmeasurable, so the analyser cannot establish their ending boundaries or prove non-overlap with later tensors. The check is deliberately one-sided: it protects known payload ranges against every following descriptor start without inventing sizes for unsupported formats. It remains local, read-only, and does not load tensor weights.

Regression coverage includes a known F32 payload overlapping the start of an unsupported tensor.


## 33. Host-memory evidence check for a registered GGUF model (2026-11-26)

The local model library adds an explicit, user-triggered evidence check. It re-inspects the GGUF metadata and tensor descriptors, calculates the same conservative one-sequence FP16 KV-cache estimate for the currently configured context, and takes a fresh local snapshot of total and currently available system RAM. The report presents the model file size and known tensor-payload total as file facts, separately from the KV-cache estimate, and shows how large the estimated KV cache is relative to currently available host RAM when both values are known.

The result is deliberately not a compatibility verdict. If the estimated KV cache alone exceeds available RAM, the report raises a host-memory warning. If it is below available RAM, the report says only that this one estimated component is below the current reading; it does not say the model fits. Model-weight residency, runtime and allocator overhead, compute buffers, batching, parallel sequences, cache quantisation, GPU memory, CPU/GPU offload, other applications, and runtime-specific behaviour remain unmeasured. Known tensor payload bytes are stored-file measurements and must not be treated as required RAM.

Missing GGUF dimensions and unavailable hardware readings produce explicit unknown states rather than guessed values. The check runs locally on a blocking worker, does not load model weights, does not enumerate unrelated processes, does not transmit paths or hardware information, and does not select a model or change settings. Its snapshot can become stale immediately as system memory changes.

Regression tests cover KV-cache-above-available-RAM, KV-cache-below-available-RAM without a fit guarantee, missing metadata, and unavailable hardware memory.


## 34. GGUF-reported context-length advisory (2026-11-26)

KV-cache estimates and host-memory evidence now compare the configured context length with the GGUF-reported `.context_length` value when it is present. The report distinguishes a configured context within the reported value, one that exceeds it, a missing value, and an unusable zero value. The KV-cache arithmetic is still shown when its required dimensions are available; exceeding the reported context does not silently block the estimate.

This comparison is deliberately advisory. GGUF metadata is file-supplied and may be inaccurate or incomplete; the reported context value is not treated as a verified runtime maximum. Some inference runtimes support RoPE scaling or context extension, while actual support depends on the model, runtime, and configuration. Being within the reported value does not guarantee runtime support or model fit. No context setting is changed, and no compatibility verdict is produced.

Regression tests cover configured context within, above, and without a reported context value, including preservation of the arithmetic estimate when the configured context exceeds the reported value.


## 35. GGUF tensor element-count cross-check (2026-11-26)

Tensor storage analysis now multiplies the declared dimensions of every tensor using checked 64-bit arithmetic and sums those element counts across the descriptor table. This count is independent of the tensor's quantisation layout, so descriptors with unsupported payload-size layouts can still contribute their declared element count when their dimensions are valid. The analyser also reads the optional `general.parameter_count` metadata value and reports whether the aggregate descriptor element count matches, exceeds, or falls below that file-reported value.

This is a structural cross-check, not a definitive parameter-count or model-integrity verdict. A tensor-element total is not necessarily identical to a model's unique parameter count: tied/shared weights, exporter conventions, incomplete or malformed descriptors, and file-supplied metadata can affect the comparison. The UI labels both values separately and presents the comparison as an advisory. Dimension or aggregate overflow and zero-sized dimensions are rejected rather than wrapped or guessed. No tensor weights are read, no model is loaded, and no hardware or runtime settings are changed.

Regression coverage includes dimension-product aggregation, optional/matching/mismatching reported counts, and overflow/zero-dimension handling.


## 36. GGUF tensor descriptor name integrity (2026-11-26)

Tensor storage analysis now reads each bounded tensor name instead of blindly skipping its bytes. It rejects empty names, names longer than the supported 64-byte inspection limit, truncated names, invalid UTF-8, and duplicate names before analysing tensor dimensions or payload ranges. Names are compared exactly and case-sensitively; no names or tensor contents are uploaded, and tensor weights remain unread.

This is a structural container check only. It does not establish that tensor names match a particular model architecture, that all expected tensors are present, or that a model can run. Regression coverage includes empty, invalid-UTF-8, duplicate, and distinct valid names.


## 37. GGUF metadata-key uniqueness (2026-11-26)

Both GGUF metadata inspection paths now track metadata keys and reject duplicate keys, including duplicates for unknown fields. This prevents ambiguous files from silently overwriting earlier values or producing different results depending on which inspector is used. Distinct unknown keys remain supported and their values continue to be skipped using the bounded type-aware parser. Duplicate detection is exact and case-sensitive. No metadata is transmitted, no tensor weights are read, and no runtime settings are changed.

Regression coverage verifies duplicate-key rejection in both inspectors and confirms that distinct unknown keys remain accepted.


## 38. GGUF metadata key and boolean encoding validation (2026-11-26)

Both GGUF inspection paths now reject empty metadata keys. Scalar boolean values and boolean-array elements must use the GGUF-defined canonical byte encodings 0 or 1; other byte values are rejected instead of being silently accepted as truthy or false-like data. Valid scalar and array booleans remain supported. These checks validate container encoding only and do not establish model compatibility. Regression coverage exercises valid and invalid scalar/array booleans and empty keys in both inspectors. Tensor weights remain unread and no metadata is transmitted.


## 39. GGUF metadata array scanning efficiency and bounds (2026-11-26)

Unknown fixed-width metadata arrays are now skipped in one checked byte-range operation rather than seeking once per element. The element-count limit remains in force, multiplication is checked for overflow, and the file boundary is validated before accepting the array. Boolean arrays are scanned in fixed-size chunks so every value is validated as canonical 0 or 1 without allocating memory proportional to the declared count. String arrays retain per-string length and UTF-8 validation. Truncated arrays fail cleanly in both inspection paths. This improves malformed-file resilience without reading tensor weights or changing model/runtime settings.


## 40. GGUF selected metadata value-type validation (2026-11-26)

Recognised metadata fields are now required to contain the expected broad value category: architecture/name fields must be strings, while counts, dimensions, file-type codes, quantisation versions, and alignment must be non-negative integers. Values that previously appeared as silently missing when encoded with an incompatible GGUF scalar type now produce an explicit inspection error. Values narrowed to 32-bit fields are checked for range overflow. Tensor-storage analysis applies the same strict checks to general.alignment and general.parameter_count. Unknown metadata keys remain supported through the existing bounded type-aware skip path.

Regression tests cover wrong string/integer encodings in both inspection paths, string-versus-number mismatches, and 32-bit range overflow. These checks validate container metadata representation only; they do not establish architecture correctness, model compatibility, or runtime support.


## 41. GGUF recognised-field ambiguity checks (2026-11-26)

The metadata inspector now maps recognised architecture-specific suffixes to one semantic field and rejects files that provide multiple competing keys for that field (for example, both llama.block_count and mistral.block_count). This prevents the parser from silently allowing whichever matching key appears last to overwrite the earlier value. Recognised suffixes must also have a non-empty namespace prefix. Unrelated vendor-specific keys remain supported and are skipped through the existing bounded parser.

Regression coverage verifies that competing architecture aliases are rejected while unrelated namespaced metadata with the same unrecognised suffix remains accepted. This is an ambiguity check, not proof that the selected architecture metadata is truthful or that the model is runtime-compatible. Tensor weights remain unread and no data is transmitted.


## 42. GGUF tensor data-offset uniqueness (2026-11-26)

Tensor descriptors are sorted by their relative data offsets before payload-range analysis. The inspector now rejects duplicate offsets before evaluating tensor sizes, including when one or both tensor types have unknown payload layouts. Two descriptors cannot legitimately identify the same start of tensor data; accepting that structure could otherwise double-count or ambiguously interpret the same bytes. This check remains metadata/descriptor-only: tensor weights are not read, and unknown payload sizes remain unknown.


## 43. GGUF architecture metadata namespace consistency (2026-11-26)

Recognised architecture-specific metadata fields must now use the namespace declared by `general.architecture`. For example, a file declaring `mistral` cannot supply `llama.block_count` and have that value presented as Mistral metadata. The check is order-independent: architecture-specific keys may appear before or after the general architecture entry. Files that provide architecture-specific recognised fields without declaring `general.architecture` are rejected as ambiguous. Unrelated vendor-specific keys remain on the bounded unknown-metadata path. Regression coverage exercises mismatched namespaces with reversed metadata order and missing architecture declarations. This validates internal metadata consistency only; it does not prove the architecture claim is truthful or guarantee runtime compatibility. Tensor weights remain unread and no data is transmitted.


## 44. Consistent GGUF tensor-count safety limit (2026-11-26)

Both GGUF entry points now enforce the same explicit maximum of 1,000,000 tensor descriptors before proceeding. The metadata-only inspector previously reported the header's tensor count without applying the structural inspector's cap. Centralising the limit prevents the two inspection paths from disagreeing about whether an excessive declared count is acceptable, and keeps the bound visible as a named parser constant. A regression test feeds the same over-limit header to both paths and confirms that both reject it. This is a defensive inspection limit, not a claim that every model below the cap is practical to load.


## 45. GGUF tensor-descriptor allocation preflight (2026-11-26)

Before allocating descriptor storage or the tensor-name uniqueness set, tensor-storage analysis now checks whether the declared descriptor count could physically fit in the remaining file. The lower-bound calculation accounts for the smallest legal descriptor shape (one-byte name, one dimension, type code, and data offset) and uses checked multiplication and subtraction. Impossible counts in tiny or truncated files are rejected before count-sized allocations. The existing one-million-descriptor safety cap remains in force. This is an allocation-safety preflight, not a complete file-validity check; actual descriptor fields and payload ranges are still validated while parsing. No tensor weights are read and no file data is transmitted.

Regression coverage confirms that a tiny file declaring a tensor descriptor is rejected before descriptor allocation.


## 46. Aggregate GGUF metadata-key memory budget (2026-11-26)

Both GGUF inspection paths now enforce an 8 MiB aggregate budget for metadata-key bytes retained by duplicate-key tracking. The existing per-string limit remains in place, but a per-string limit alone could still permit many individually valid large keys to accumulate substantial memory. The key sets now grow incrementally rather than reserving capacity directly from the untrusted metadata-entry count. Checked accounting rejects a file as soon as the aggregate key budget is exceeded. This is a bound on retained key text, not a total metadata-file-size limit; scalar values and arrays continue to use their existing bounded readers/skippers. Regression coverage verifies that both inspection paths reject the same over-budget fixture.


## 47. Aggregate GGUF metadata-key memory regression fixture (Batch #152)

- Corrected the aggregate metadata-key budget regression fixture so each generated key is exactly within the existing 1 MiB per-string limit while the nine distinct keys collectively exceed the 8 MiB aggregate limit.
- This ensures the test reaches and verifies the aggregate-key guard in both metadata inspection paths, rather than stopping earlier at the individual-string guard.
- No production limit or parser behaviour was relaxed; the change corrects test data only.


## 48. GGUF tensor-descriptor collection memory bounds (2026-11-26)

Tensor-storage analysis now avoids allocating a separate dimensions vector for every tensor: each descriptor stores up to four dimensions in a fixed-size inline array. Descriptor and tensor-name collections grow incrementally rather than reserving capacity directly from the untrusted header count. Retained tensor-name text is additionally limited to 8 MiB with checked accounting, complementing the existing one-million tensor-count cap and minimum-descriptor file-size preflight. A regression fixture verifies that analysis rejects an aggregate tensor-name set that exceeds the budget. These limits bound parser-owned descriptor/name data; they are not a total process-memory guarantee and do not load tensor weights.


## 49. Aggregate GGUF metadata-section work budget (2026-11-26)

Both GGUF inspection paths now cap the total encoded metadata section at 64 MiB, measured with checked file-position accounting after each metadata entry. Existing per-string, array-item, entry-count, and aggregate key limits remain in force. This additional bound prevents a file from multiplying individually bounded arrays and values across many entries into an arbitrarily large metadata scan. The cap is intentionally generous for ordinary model metadata; files exceeding it are rejected before tensor descriptor processing. A sparse-file regression test exercises the limit in both inspectors without allocating a matching in-memory fixture. This is a parser-work bound, not a maximum GGUF file-size limit.


## 50. GGUF tensor data-offset file-boundary validation (2026-11-26)

Tensor-storage analysis now obtains the file length from the same opened file handle used to parse the GGUF container, avoiding a separate path-metadata/open race for boundary checks. Every tensor data offset must resolve to a byte strictly inside the file. An offset exactly at EOF is rejected even when the tensor type is unsupported and its payload length cannot be calculated; offsets beyond EOF remain rejected as well. Regression coverage checks both invalid boundaries and confirms that an unsupported tensor with an in-file start remains explicitly unknown rather than being assigned a guessed payload size. This validates tensor start locations only; unsupported payload lengths remain unknown, and tensor weights are never read.


## 51. GGUF tensor-descriptor collection memory budget (2026-11-26)

Tensor-storage analysis now estimates the memory required for its retained tensor-descriptor records and duplicate-name index before allocating either collection. A conservative 192-byte bookkeeping estimate per descriptor is checked with overflow-safe arithmetic against a 64 MiB aggregate budget. Tensor-name text remains governed by its separate 8 MiB aggregate limit, and the existing one-million tensor-count ceiling remains an independent upper bound. Regression tests prove an over-budget declared count is rejected before descriptor allocation and a count at the budget boundary proceeds to the existing file-layout preflight. This is an inspection-memory bound, not a limit on GGUF file size or a claim about runtime model memory.


## 52. GGUF metadata string file-boundary preflight (2026-11-26)

Before allocating a metadata string buffer, the parser now checks the declared byte length against the remaining bytes in the same opened file handle. A truncated string declaration is rejected before allocating its (otherwise individually bounded) buffer. This applies consistently to metadata keys, selected values, unknown string values, string arrays, and tensor names because they share the same string reader. The metadata-only inspector also obtains its reported file size from the opened parsing handle rather than a separate path metadata lookup. These checks reduce avoidable allocation and keep file-size reporting tied to the parsed handle; they do not eliminate changes made to a file concurrently by another process. Regression coverage verifies early rejection of a declared string payload that is not present. Tensor weights remain unread and no data is transmitted.


## 53. GGUF string-array aggregate work preflight (2026-11-26)

Unknown metadata string arrays now enforce the aggregate 64 MiB metadata-section budget after each string, rather than waiting until the entire array entry has been consumed. This closes a gap where one individually bounded but very large string array could exceed the section budget substantially before the normal per-entry check ran. Existing per-string and array-item caps remain in force, and known numeric arrays retain their checked bulk-skip path. Regression coverage uses a sparse file to prove an oversized single string-array entry is rejected during the entry, without constructing a correspondingly large in-memory fixture. Tensor weights remain unread and no data is transmitted.


## 54. GGUF string-array section-budget preflight (2026-11-26)

Unknown metadata string arrays now check the declared string payload against the remaining aggregate metadata-section budget immediately after reading its length prefix. If the payload would cross the 64 MiB section boundary, inspection stops before allocating or reading that string. The existing per-string limit and opened-file truncation preflight remain in force, and ordinary string reads retain their existing behaviour. Regression coverage verifies that an over-budget string is rejected with the file cursor still positioned before its payload, proving the payload was not consumed. Tensor weights remain unread and no data is transmitted.


## 55. GGUF metadata-key aggregate allocation preflight (2026-11-26)

Metadata keys now check their declared byte length against the remaining aggregate 8 MiB key-storage budget immediately after the length prefix is read. A key that would exceed the remaining budget is rejected before its payload buffer is allocated or its bytes are read. This applies to both metadata inspection paths and complements the existing per-string cap, duplicate-key checks, and metadata-section budget. A focused regression test verifies rejection with the file cursor still directly after the length prefix.

## 56. GGUF metadata string section-budget preflight (2026-11-26)

Metadata keys and scalar string values now check the remaining 64 MiB metadata-section budget immediately after reading their length prefixes. A declared key or string payload that would cross the section boundary is rejected before allocating or reading the payload. This complements the aggregate 8 MiB metadata-key budget, per-string limit, opened-file boundary check, and the existing string-array preflight. Regression tests verify both metadata-key and scalar-string payloads remain unread when the section budget would be exceeded. Tensor weights remain unread and no data is transmitted.

## 57. GGUF array-payload section-budget preflight (2026-11-26)

Before skipping fixed-width numeric metadata arrays or scanning boolean arrays, the parser now checks the entire declared payload against the remaining 64 MiB metadata-section budget. An array that would cross the boundary is rejected while the file cursor is still at the start of its payload. This prevents numeric-array seeks from crossing into later GGUF structures and prevents boolean-array reads from consuming bytes beyond the metadata budget. Existing checked multiplication, array-item limits, boolean-value validation, and file truncation checks remain in force. Regression tests cover both numeric and boolean arrays. Tensor weights remain unread and no data is transmitted.


## 58. GGUF string-length-prefix section-boundary preflight (2026-11-26)

The shared GGUF string reader now verifies that the complete 8-byte string-length prefix fits inside the remaining 64 MiB metadata-section budget before reading that prefix. Previously, the declared string payload was correctly preflighted before allocation or reading, but a length prefix positioned within the final seven bytes of the section could itself be consumed across the boundary. Metadata keys, scalar string values, and unknown string-array elements all use the shared reader, so the guard applies consistently. Regression tests verify that both key and scalar-string length prefixes remain unread when they would cross the section boundary. Existing payload, per-string, aggregate-key, and opened-file bounds remain unchanged; tensor weights remain unread.


## 59. GGUF string-array boundary regression fixture alignment (2026-11-26)

Updated the existing mid-entry string-array regression test to reflect the stricter shared string-reader contract: when fewer than eight metadata-section bytes remain, the length prefix is rejected before it is consumed. The fixture now asserts the cursor remains at the start of that prefix, not after it. This is a test expectation correction; the Batch #167 parser guard remains unchanged.


## 60. GGUF metadata field and scalar section-boundary preflight (2026-11-26)

Both metadata inspection paths now preflight the four-byte metadata value-type tag before reading it. Scalar numeric and boolean payloads are checked against the remaining 64 MiB metadata-section budget before consumption, and array element-type/count headers are checked as a complete 12-byte unit before either field is read. String values retain their dedicated prefix and payload checks; array payload checks remain unchanged. Sparse-file regression coverage proves a type tag and an eight-byte scalar payload remain unread when they would cross the section boundary. Existing file truncation, value validation, array-count, and memory limits remain in force; tensor weights remain unread.


## 61. GGUF metadata boundary regression fixture cursor correction (2026-11-26)

Corrected the Batch #169 sparse-file regression fixture to seek to the tested boundary after reopening each file handle. Reopening a file resets its cursor to byte zero; the prior fixture therefore exercised an ordinary in-range read instead of the intended boundary. The production parser checks are unchanged. The regression now positions the reopened handle explicitly before asserting that the metadata type tag and scalar payload remain unread.


## 62. GGUF alignment validation parity (2026-11-26)

The metadata-only inspector now validates a declared `general.alignment` value using the same rule as tensor-storage analysis: it must be a power of two from 1 through 4096 bytes. Previously, tensor analysis rejected malformed alignment while the metadata inspector could report it as if it were usable. Both paths now share one validator, and regression coverage checks valid boundary values (1 and 4096), the default/common value (32), and invalid values (0, 3, and 8192). Files without an explicit alignment continue to use the GGUF default in tensor analysis and remain reported as unspecified by the metadata inspector. Tensor weights remain unread.


## 63. GGUF architecture metadata semantic validation (2026-11-26)

Recognised architecture-specific dimensions and counts are now required to be positive when present: context length, block count, embedding length, attention head counts and key/value lengths, RoPE dimensions, and expert counts. The inspector also rejects a blank `general.architecture` identifier, a key/value head count greater than the total attention head count, and an expert-used count greater than the total expert count. These checks are order-independent and do not require optional metadata fields to be present. Regression coverage exercises zero values for every recognised positive field, valid paired counts, both inconsistent pairings, and blank architecture identifiers. Tensor weights remain unread.


### Section 64 — GGUF architecture semantic test-fixture correction

- Corrected the positive-dimension and count-relationship regression fixture to include the required `general.architecture = "llama"` metadata entry.
- Preserved the parser rule that architecture-specific metadata must match a declared architecture, independent of metadata ordering.
- Kept the production validation rules unchanged; this is a test-fixture correction, not a relaxation of GGUF validation.


## 65. HIARM pressure-aware starting profiles (Batch #174)

The first HIARM capability assessment now considers both installed RAM and the currently available RAM snapshot. It continues to classify the machine by installed-memory tier and CPU logical-processor count, then conservatively reduces the suggested starting profile by one level when available RAM falls below the existing headroom threshold. A high-capacity suggestion becomes Balanced; a Balanced suggestion becomes Lightweight; a Lightweight suggestion remains Lightweight. Available memory is clamped to installed memory so inconsistent operating-system readings cannot create artificial headroom.

This is a recommendation only: it does not change runtime settings, terminate applications, select or download a model, or guarantee that a model will fit. The assessment still explicitly excludes GPU/VRAM, runtime acceleration, model-specific benchmarks, context size, and speech workloads. Regression tests cover both downgrade levels, comfortable headroom, and impossible available-memory readings.


## 66. GPU inventory assessment (Batch #175)

The hardware snapshot now includes a derived GPU inventory assessment alongside the raw adapter inventory. It distinguishes unsupported detection, unavailable detection, an empty adapter list, software-only adapters, and detected hardware adapters. Software renderers are excluded from hardware memory summaries. For physical adapters, the summary reports the largest dedicated-memory figure and largest available OS-reported local-segment budget, plus how many adapters exposed a local budget.

These figures are deliberately not added together across multiple GPUs: model workloads cannot be assumed to pool their memory. A local-segment budget is not free VRAM, and integrated GPUs may use shared system RAM. The assessment is inventory context only; it does not establish compute capability, driver or inference-backend support, model compatibility, or performance. Tests cover unsupported/unavailable/empty states, software-only inventory, mixed software/integrated/discrete adapters, and missing local-budget data. No new platform-specific dependency or network access is introduced.


## 67. GPU memory reservation-headroom assessment (Batch #176)

HIARM now derives a conservative GPU memory-headroom summary from each physical adapter's DXGI local-segment budget for the current midSunday process. It counts adapters with critical reported headroom (below 512 MiB or 10%), constrained reported headroom (below 1 GiB or 20%), and at least 1 GiB plus 20% reported reservation headroom. The summary also reports the lowest per-adapter available-for-reservation value and its percentage of that adapter's OS budget. Software adapters, zero-sized budgets, and adapters without a valid local budget are not scored. Inventory status now preserves a partial adapter-enumeration result instead of labelling it fully detected.

These are point-in-time OS reservation-headroom indicators, not free VRAM, model-fit predictions, or proof that an inference backend can use a GPU. Values are clamped to their reported budget and are never summed across adapters. The UI labels the current-process scope and explicitly separates this evidence from the RAM/CPU starting profile. The assessment is advisory only and does not reserve memory, change settings, select a model, or reduce a profile automatically. Cross-platform tests cover each headroom band, software exclusion, missing/zero budgets, partial inventory, and unavailable/unsupported states.


## 68. GPU context alongside GGUF model memory evidence (Batch #177)

The model-memory evidence report now includes a fresh GPU inventory and per-adapter reservation-headroom snapshot alongside the existing host-RAM and FP16 KV-cache evidence. It reports the number of physical adapters, how many expose a usable local budget, critical/constrained/healthy headroom counts, and the lowest per-adapter reservation-headroom reading when available. The UI keeps this GPU information in a separate section of the report.

GPU context does not alter the host-RAM verdict or KV-cache calculation. It is not free VRAM, a model-fit prediction, or proof that the installed llama.cpp build or another inference backend supports GPU offload. Memory is never pooled across adapters. This addition is evidence-only and does not select a model, change offload layers, reserve memory, or modify runtime settings. Regression tests verify that constrained GPU headroom is carried into the report without changing an otherwise unchanged host-RAM result.


## 69. Live network throughput and midSunday process I/O (Batch #180)

The opt-in Live System Monitor now reports aggregate system-wide download/upload throughput using the operating system's per-interface byte counters. It sums counters without exposing interface names, IP addresses, or per-interface identity. Rates are derived from successive totals and elapsed sample time; the first sample warms up, and counter decreases (for example, interface changes or resets) do not produce misleading negative rates.

The monitor also reports read/write throughput for the midSunday process only, using that process's I/O counters. This is not system-wide disk activity. Operating-system caching and platform-specific accounting may affect the values, so the UI labels the scope explicitly. The recent activity view now includes bounded download/upload history.

The implementation reuses the existing sysinfo dependency and refreshes only the current midSunday process. It does not enumerate other processes, expose network interface names, store history persistently, or send telemetry anywhere. GPU utilisation, battery telemetry, and system-wide disk activity remain future work. Tests cover rate calculations, counter resets, unavailable/warm-up states, and serialized snapshot shape.


## 70. Live battery telemetry (Batch #181)

The Live System Monitor now uses the cross-platform `starship-battery` library to query battery state on supported Windows, Linux, and macOS systems. Charge is aggregated across detected batteries using reported current energy divided by summed full energy, rather than taking an arbitrary first battery or averaging percentages equally. State is reported as charging, discharging, empty, full, paused, unknown, or mixed when batteries disagree.

No battery produces an explicit `no_battery` / `not_present` result; query failures and partial readings remain distinct. The monitor intentionally omits remaining-time estimates because they can be unstable and are not consistently trustworthy across devices. Battery vendor, model, and serial identifiers are not collected. Sampling remains opt-in, local, read-only, and active only while the Settings monitor is visible. CI checks compilation and pure aggregation behavior; physical-device testing is still required.


## 71. Live system-wide disk activity (Batch #182)

The Live System Monitor now refreshes OS-reported disk I/O counters through sysinfo's `Disks` API and aggregates read/write throughput across the detected storage devices. The aggregate deliberately does not expose mount paths, volume labels, device names, or serial identifiers. It is a separate metric from the midSunday process's own I/O counters, which remain visible independently.

The first sample is a warm-up; an idle system can correctly report zero bytes per second, while an empty device list is reported separately. Values reflect the operating system's disk/volume accounting and may include activity from all applications. Virtual volumes, RAID, filesystem caches, drivers, and platform-specific accounting can change what is counted; this is not guaranteed to represent raw physical-device traffic. Sampling remains opt-in, local, read-only, bounded, and paused when the Settings monitor is not visible. Physical-device validation remains necessary.


## 72. Live GPU telemetry foundation (Batch #183)

The opt-in System Monitor now has a dedicated per-adapter live GPU schema. The first native collector targets Linux systems whose AMDGPU DRM driver exposes `gpu_busy_percent`, `mem_info_vram_used`, and `mem_info_vram_total` under `/sys/class/drm/card*/device`. Utilisation is validated to the 0–100 range. VRAM allocation is clamped to the reported capacity, and a zero or missing capacity is not converted into a percentage.

The collector uses generic labels such as GPU 1 and does not expose adapter names, PCI identifiers, serial numbers, or other device identifiers in live metrics. It reads only a small fixed set of driver-published files during the existing opt-in sample cycle. No process enumeration, privileged helper, shell command, network access, or persistent history is introduced.

GPU utilisation and VRAM allocation are separate readings. Allocated VRAM is not free memory, an inference-fit guarantee, or proof that the local model runtime can offload to that adapter. Unsupported drivers, missing sysfs data, no adapters, and unsupported operating systems have explicit states. This first collector does not yet support Linux Intel/NVIDIA, Windows, or macOS live GPU telemetry; Windows DXGI inventory and process-specific reservation-budget snapshots remain distinct evidence. Further native collectors and physical-device validation are required.


## 73. Windows live GPU telemetry (Batch #185)

Windows live GPU sampling now uses language-neutral Performance Data Helper (PDH) counters for `\\GPU Engine(*)\\Utilization Percentage` and `\\GPU Adapter Memory(*)\\Dedicated Usage`. The collector takes two closely spaced PDH samples because GPU Engine utilisation is a rate-style counter. It groups engine and memory instances by the adapter LUID internally, then emits only generic labels such as GPU 1; LUIDs, adapter names, PIDs, engine instance names, and device identifiers are never returned to the UI.

For each adapter, the displayed utilisation is the maximum valid individual engine percentage in the sample. It is deliberately labelled **peak engine**, not total GPU utilisation: engines can operate concurrently, and summing their percentages would be misleading. Dedicated-memory allocation is reported separately. Where available, its total is paired with DXGI dedicated-memory inventory; if no reliable total is available, the used-byte reading remains visible without a fabricated percentage. Shared GPU memory is not labelled VRAM.

PDH query/counter failures, missing counters, missing instances, and unsupported drivers retain explicit unavailable/unsupported/partial states. The collector is local, read-only, bounded to two counter families, does not enumerate processes, and requires no elevated helper, shell command, network request, or persistent history. PDH readings and DXGI inventory may differ by driver and Windows build. This batch does not establish live sensor accuracy on physical hardware; Windows 10/11 device testing remains required. See also section 72 for the Linux AMDGPU collector and the separate macOS/Linux Intel/NVIDIA work still outstanding.


## 74. Live per-volume storage capacity (Batch #188)

The opt-in Live System Monitor now reports each enumerated storage volume's total capacity, used capacity, available capacity, and used percentage. Each entry is assigned a generic session label such as "Volume 1"; mount paths, volume labels, filesystem names, device names, and serial identifiers are not included in the live snapshot. The order and count may vary when volumes are added, removed, or re-enumerated.

Capacity is calculated from the operating system's reported total and available space. Zero-capacity or otherwise unknown readings do not produce a fabricated percentage. Existing conservative free-space classifications are reused for low/critical headroom hints. This is a per-volume capacity view, distinct from aggregate system-wide disk throughput and midSunday's own process-only disk I/O. It does not identify which volume contains a model or guarantee that a model download, conversion, or runtime workload will fit.

Collection remains local, read-only, opt-in, and active only while the monitor is running in Settings. The live payload contains generic labels and byte counts only. Hosted CI validates compilation, serialization, and calculation behavior; physical-device validation is still required across Windows, Linux, and macOS, especially for network mounts, virtual volumes, permissions, and filesystem-specific reporting.


## 75. GPU monitor clarity and per-adapter history (Batch #189)

The Live System Monitor presents GPU utilisation and dedicated-memory allocation as separate measurements. A dedicated-memory percentage meter appears only when the collector provides both used bytes and a reliable total capacity. If used bytes exist but total capacity does not, the UI shows the byte count and explicitly says the total is unavailable; it never fabricates a percentage. GPU utilisation and memory-allocation meters expose accessible labels and bounded 0–100 values.

Recent GPU activity is retained separately for each generic adapter label in the same bounded 36-sample, approximately 54-second local history used by the monitor. A missing reading is represented as a visual gap, not a zero-utilisation sample. Adapter labels remain generic and no vendor, model, PCI, LUID, or serial identifiers are added to the UI or history.

The history is observational only. It inherits each platform collector's meaning: for example, Windows reports the busiest individual GPU engine rather than universal whole-GPU utilisation. Cross-platform values should not be compared as if their counter semantics were identical. Sampling remains opt-in, local, read-only, and paused when the user leaves the Settings view. Hosted CI checks compilation and frontend integration, not physical sensor accuracy.


## 76. Per-core CPU dashboard and truthful history gaps (Batch #190)

The Live System Monitor now displays each operating-system-reported logical processor as a separate, labelled utilisation meter alongside total CPU usage. The grid adapts to the available window width and uses accessible meter labels and bounded percentage values. These are logical processors as reported by the operating system; they are not necessarily distinct physical cores or independent CPU packages.

The monitor's bounded 36-sample history now retains unavailable CPU, RAM, network, and system-wide disk readings as missing samples. The chart draws a visible gap for those samples rather than silently replacing missing data with zero, which could incorrectly imply an idle processor or no traffic. Valid idle readings remain zero when the collector actually reports zero. Sampling remains opt-in, local, read-only, and paused when the user leaves Settings.

The overall CPU, per-logical-processor, system RAM, and aggregate battery indicators expose accessible meter semantics when a valid percentage exists. A meter is omitted when its percentage is unavailable rather than rendering a misleading empty bar. Hosted CI can validate compilation and frontend integration, but physical-device checks are still needed to confirm OS reporting and layout across different CPU topologies, power states, and display scales.


## 77. Per-logical-processor CPU activity history (Batch #191)

The Live System Monitor now retains a per-logical-processor CPU utilisation sample alongside each overall CPU, RAM, network, disk, and GPU history point. The existing bounded 36-sample timeline is reused, so per-processor history remains in memory only and is discarded when the monitor history is cleared or the app session ends. Each processor is identified only by a generic ordinal label such as C1; the display does not expose CPU topology identifiers or hardware IDs.

The history rows use the current operating-system-reported logical-processor count and align each sample to the shared timeline. If a sample does not contain a reading for a processor, that point is shown as a gap rather than zero. A genuine reported 0% remains a valid idle reading. The rows are observational and do not imply that logical processors are separate physical cores or CPU packages. Sampling remains opt-in, local, read-only, and paused when the user leaves Settings. Hosted CI validates compilation and integration; physical-device testing remains necessary for topology changes, hot-plug behaviour, and display scaling.


## 78. CPU history across logical-processor topology changes (Batch #192)

Each CPU history point now records the logical-processor count reported with that sample. When the current reported count differs from a historical point, per-processor history for that point is rendered as a gap rather than mapping an old ordinal (for example C4) onto a potentially different processor in the changed topology. Samples with matching counts retain their readings, including genuine 0% idle values; missing readings remain gaps.

The monitor deliberately uses generic ordinal labels, not hardware IDs. An unchanged count does not prove that the operating system's processor ordering is stable after every hot-plug or topology event, because the cross-platform metrics interface does not provide a portable persistent identity for each logical processor. The display therefore remains an ordinal view, not a physical-core identity map. Sampling remains opt-in, local, bounded, and read-only. Physical-device validation is still required for hot-plug and topology-change behavior.


## 79. System Monitor history controls and sampling integrity (Batch #193)

Each retained System Monitor history point now carries the capture timestamp supplied by the live operating-system snapshot. The Recent activity heading calculates and displays the elapsed time between the oldest and newest retained samples, rather than describing 36 samples as a fixed 54-second window even when the collector or operating system responds more slowly.

The polling loop now allows only one live-metrics request at a time. If a sample takes longer than the nominal 1.5-second interval, subsequent timer ticks are skipped until that request settles; this avoids piling up concurrent collector requests and prevents out-of-order samples from distorting the bounded timeline. A completed request still releases the guard after success or failure.

A Clear history control removes only the temporary 36-point chart history. It does not stop sampling, erase the current live metric cards, or affect hardware inventory. The history remains in memory only and is discarded when monitoring stops or the app session ends. No samples are persisted or transmitted. Physical-device testing is still required to validate actual collector latency and timing behavior across supported operating systems.


## 80. System Monitor measurement availability panel (Batch #194)

The live dashboard now includes a Measurement availability overview for CPU, system memory, aggregate network counters, midSunday process memory, midSunday process disk I/O, system-wide disk I/O, storage capacity, GPU telemetry, and battery reporting.

Each source displays its existing collector status in plain language: available, partially available, warming up, unavailable, or a specific no-device/unsupported state. The panel also shows limited aggregate context such as logical processor count, interface count, disk count, generic volume count, GPU adapter count, and battery count. It does not expose interface names, mount paths, GPU vendor/device identifiers, battery identifiers, or unrelated process details.

The panel explicitly explains that warming-up requires another sample and that unavailable/unsupported readings are not zero activity. It is driven by the same live snapshot as the metric cards, so no extra polling, background service, persistence, or network transfer is introduced. This is a reporting/clarity improvement, not a claim that additional hardware collectors have been implemented. Physical-device validation remains necessary for driver and permission-specific states.


## 81. Linux Intel DRM GT live GPU telemetry (Batch #197)

On Linux systems whose Intel DRM kernel driver exposes the GT engine `busy` counters, the live System Monitor now samples those cumulative nanosecond counters twice and calculates a utilisation rate over the measured interval. It reports the busiest individual engine per generic GPU adapter label; it does not sum concurrent engines, expose engine names, or claim a universal whole-GPU utilisation percentage. The collector is bounded to a limited number of GT/engine entries and uses local read-only sysfs access.

Where the same adapter also exposes driver-published dedicated-memory used/total counters, those values may be shown separately. Missing or zero total capacity never produces a percentage. Unsupported kernels/drivers, inaccessible counters, counter resets, and invalid rates remain unavailable/partial rather than being converted to zero. AMDGPU's existing `gpu_busy_percent` path remains unchanged. NVIDIA adapters now use the optional `nvml-wrapper` integration. It dynamically loads the driver-provided NVML shared library at runtime and initialises it once; midSunday does not bundle NVML, install a driver, invoke `nvidia-smi`, or require the CUDA toolkit. For each accessible NVIDIA device, it reads NVML's device-wide GPU utilisation percentage and framebuffer memory used/total bytes. These are whole-device NVML readings, not per-engine maxima. Device enumeration is capped at 16 entries, labels are generic and per-session, and the collector does not request GPU names, UUIDs, PCI identifiers, serials, or process lists. If NVML is absent, cannot initialise, or a query is unsupported/denied, NVIDIA readings remain unavailable; an NVIDIA-only system reports unsupported driver telemetry rather than fabricated zeros. Mixed-vendor systems retain available AMD/Intel readings and report partial coverage when NVIDIA cannot be queried. The NVML handle is initialised once per process rather than reloaded every 1.5-second sample. Other Linux drivers remain unsupported until separately implemented and validated.

The sampling interval introduces a small delay in the live snapshot when an Intel GT counter is available. This collector has regression coverage for rate bounds and counter resets, but hosted CI cannot validate physical sensor accuracy, driver counter availability, hybrid-GPU attribution, or kernel-version differences. Physical-device testing remains required before making broad hardware-support claims.


## 82. GPU dedicated-memory allocation history and source-aware labels (Batch #200)

The bounded Recent activity timeline now records a separate GPU dedicated-memory allocation percentage for each generic adapter label when the live collector supplies both a valid used-memory value and a non-zero total capacity. These samples are displayed in their own `GPU 1 MEM` history rows, separate from GPU utilisation. The rows share the existing timestamped 36-point timeline and remain in memory only; clearing history or stopping the monitor removes them. A missing or incomplete memory reading remains a gap and is never converted to zero. If no valid memory percentages have been observed for an adapter, no memory-history row is shown.

The live GPU card now labels Linux Intel DRM GT utilisation as the busiest sampled engine, Windows PDH utilisation as the peak/busiest individual engine, and the AMDGPU driver value as GPU busy. This avoids presenting engine-level readings as universal whole-GPU utilisation. Dedicated-memory allocation remains distinct from GPU compute utilisation and does not represent free memory, system-wide GPU memory pressure, or memory available to an AI model.

The change reuses the existing opt-in sample, does not add polling, filesystem reads, hardware identifiers, persistence, or network activity, and keeps the 36-sample bound. The history is keyed only by the generic per-session adapter label; adapter reordering or replacement may therefore change which physical adapter a label represents after re-enumeration. Hosted CI can validate compilation and integration, but real-device validation is still required for driver reporting, adapter changes, and display behaviour across Windows, Linux, and macOS. macOS live GPU telemetry remains unsupported in this batch.


## 83. macOS Metal resource-allocation telemetry (Batch #203)

The macOS System Monitor now queries Metal devices through the Rust `metal` crate (macOS-only dependency). For each device exposed by `Device::all()`, it reads `current_allocated_size()` and `recommended_max_working_set_size()`. The UI reports the allocation as Metal resources visible to midSunday and compares it with Apple's recommended working-set threshold. It deliberately does not call that threshold physical VRAM or total system GPU memory. The reported allocation is preserved even when it exceeds the recommendation; the visual ratio is capped at 100% and the actual byte values remain visible.

macOS adapters are assigned generic per-session labels; device names, registry IDs, and other hardware identifiers are not collected. The collector reports utilisation as unavailable and marks the overall capability partial because these Metal properties do not provide whole-system GPU activity. On Apple silicon, unified memory is shared between CPU and GPU, so the reading must not be interpreted as dedicated VRAM. The measurement is process-visible Metal resource allocation, not all applications' GPU memory use, system-wide memory pressure, free memory, or guaranteed AI-model headroom.

The System Monitor's existing opt-in sample and 36-point in-memory history are reused. Metal allocation history uses a separate `WS` row (recommended working-set ratio) rather than the dedicated-memory `MEM` label. No GPU workload is launched, no memory is allocated by the collector, and no device identifiers or telemetry are transmitted. If Metal returns no devices or a zero recommended working-set value, the UI reports unavailable values instead of inventing percentages.

Hosted macOS CI validates compilation and tests only; real-device validation remains required on Apple silicon and supported Intel Macs, including multi-GPU systems and OS/driver versions. No macOS whole-system GPU utilisation percentage is claimed.


## 84. Linux NVIDIA NVML live GPU telemetry (Batch #204)

Linux systems with a working NVIDIA driver and accessible NVML library can now report NVML device-wide GPU utilisation and framebuffer memory allocation. The NVML integration is an optional Linux-only dependency that dynamically loads the shared library supplied by the installed NVIDIA driver. midSunday does not bundle NVML, install proprietary drivers, invoke a command-line utility, or require the CUDA toolkit. NVML initialisation is cached once per process; device enumeration is bounded to 16 entries.

Only generic per-session GPU labels and numeric utilisation/memory readings enter the live snapshot. The collector does not query GPU names, UUIDs, PCI identifiers, serial numbers, or process lists. GPU utilisation is the device-wide percentage reported by NVML; it is not an engine maximum or a measurement of midSunday's own GPU workload. Framebuffer memory used/total is a driver-reported allocation/capacity pair, not guaranteed free memory or AI-model headroom. Unsupported, inaccessible, lost, or missing readings remain unavailable/partial rather than being fabricated as zero. If NVML cannot initialise, an NVIDIA-only DRM system reports unsupported driver telemetry; mixed-vendor systems keep available AMD/Intel values and indicate partial coverage.

The implementation includes bounds checking for NVML utilisation values and reuses the existing opt-in sampling and 36-point in-memory history. Hosted CI validates compilation and regression tests only; physical NVIDIA systems with different driver branches, GPU generations, hybrid graphics, and permission configurations still require validation.


## 85. Source-aware GPU monitor presentation (Batch #205)

The live System Monitor now distinguishes the meaning of each GPU source in its card heading and accessible meter label: NVIDIA NVML is device-wide GPU utilisation; AMDGPU is the driver's GPU-busy percentage; Intel DRM GT is the busiest sampled engine; Windows PDH is the peak/busiest individual engine; and macOS Metal allocation is not presented as GPU utilisation. NVIDIA framebuffer memory is labelled as framebuffer allocation, not generic free memory. Its history uses a distinct `FB MEM` label, while other dedicated-memory history remains `MEM` and Metal recommended-working-set history remains `WS`. The history graph's accessible description now names the source-specific metric. Existing missing-sample gaps, opt-in sampling, generic adapter labels, and bounded in-memory history are unchanged. No new hardware queries, polling, persistence, identifiers, or network activity are introduced. This is a presentation improvement only and does not replace physical hardware validation.


## 86. Time-aware System Monitor history (Batch #206)

The Recent activity timeline now shows oldest, midpoint, and newest capture times using the timestamps already attached to each sample. Hovering an individual history bar includes that sample's local capture time alongside its value or unavailable state. CPU, RAM, network, and system-disk history rows expose concise accessible descriptions; throughput rows explain that bar height is scaled relative to the recent peak in that row. The shared 36-point history, timestamp source, 1.5-second target cadence, opt-in behavior, and in-memory-only retention are unchanged. Missing values remain visible gaps, and this presentation change adds no sampling, persistence, hardware queries, or network activity.


## 87. Numeric System Monitor history summary (Batch #207)

The Recent activity panel now includes a compact, accessible numeric summary table for total CPU, system RAM, aggregate network download/upload, system-wide disk read/write, and each currently reported GPU utilisation/memory series. For the retained in-memory sample window it shows the newest sample, arithmetic average, peak, and valid-sample count. If the newest value is missing, Latest is explicitly unavailable; averages and peaks include only finite readings and never turn missing samples into zero. GPU memory labels distinguish NVIDIA framebuffer allocation and macOS Metal working-set allocation from other dedicated-memory readings. This is derived entirely from the existing bounded history and current adapter metadata: no extra polling, hardware queries, persistent storage, identifiers, or network activity are added. Physical device and narrow-window layout validation remains necessary.


## 88. Explicit local CSV export for System Monitor history (Batch #208)

The Recent activity panel now offers an explicit Export CSV action for the bounded history currently visible in the session. The file includes capture timestamps in local and UTC forms, total CPU, per-logical-processor CPU, system RAM, aggregate network throughput, system-wide disk throughput, and the generic per-adapter GPU utilisation and memory-allocation series. GPU columns retain source context where available; macOS Metal working-set allocation is distinguished from NVIDIA framebuffer and other GPU memory allocation. Missing or invalid readings are exported as empty cells, never as zero. Per-logical-processor readings are blank for samples captured under a different logical-processor count, preserving the topology-change safeguards.

Export is generated locally from the existing in-memory history only after the user clicks the button. It does not add polling, persistence, identifiers, process lists, or network activity. The export is limited to the retained sample window (at most 36 points); clearing history and stopping monitoring continue to discard that temporary data. CSV cells are quoted and text values beginning with spreadsheet formula characters are guarded. Hosted CI validates the frontend build; native save/download behavior and spreadsheet compatibility still require physical-device testing on Windows, Linux, and macOS.


## 89. Time-proportional sampling-gap cues in System Monitor history (Batch #209)

The Recent activity bars now add a small, bounded horizontal separation when consecutive OS capture timestamps are more than 2.25 seconds apart (the normal target cadence is 1.5 seconds). The same timestamp-derived spacing is applied consistently to total CPU, per-logical-processor CPU, RAM, network, system disk, GPU utilisation, and GPU memory rows. Tooltips identify the elapsed time since the previous sample when a delayed interval occurs, and a short note explains that wider spaces represent longer sampling gaps. Missing metric readings remain explicit gaps and are not converted to zero or interpolated.

This is a presentation-only correction derived from timestamps already attached to each in-memory sample. It adds no polling, hardware reads, persistence, identifiers, or network activity; the 36-point bound and target sampling cadence remain unchanged. Physical testing is still required to confirm the spacing and tooltip readability under real slow collectors, app load, sleep/resume, and narrow window sizes on Windows, Linux, and macOS.

## 90. Per-logical-processor numeric history summary (Batch #210)

The Recent activity Window summary now includes a separate latest, average, peak, and valid-sample count for each logical processor in the newest retained sample. A processor index is compared only across samples with the same logical-processor count as the newest sample. Earlier samples captured under a different CPU topology are excluded from that processor's statistics rather than being incorrectly matched by index. Missing and non-finite values remain unavailable and are not treated as zero.

The rows are derived from the existing bounded, timestamped in-memory history. This adds no hardware queries, polling, persistence, identifiers, or network activity. The table remains horizontally scrollable; systems with many logical processors may produce a longer summary. Hosted CI validates compilation and tests, while physical validation is still needed for topology changes and table usability on Windows, Linux, and macOS.

## 91. Visible CPU-topology change disclosure (Batch #211)

When retained System Monitor samples contain more than one logical-processor count, the Window summary now displays a status note explaining that CPU topology changed during the history. It clarifies that per-logical-processor statistics are calculated only from samples matching the newest sample's processor count, and that older-topology samples are excluded rather than interpreted as zero. The note is hidden when all retained samples share the same logical-processor count.

This disclosure is derived from the existing in-memory sample metadata. It adds no hardware queries, polling, persistence, identifiers, or network activity, and does not change the statistics or CSV export. Physical validation should cover processor hotplug/topology changes where supported, plus narrow-window readability across Windows, Linux, and macOS.



## 92. System Monitor capture-timing quality summary (Batch #212)

The Recent activity Window summary now reports the number of valid adjacent capture intervals, their average duration, the longest observed interval, and the number of delayed intervals. An interval is counted only when the difference between adjacent retained sample timestamps is finite and positive; intervals longer than 2.25 seconds are classified as delayed, matching the existing visible sampling-gap cues. With fewer than two usable timestamps, timing values remain unavailable rather than being shown as zero.

This is a presentation-only summary derived from timestamps already held in the bounded in-memory history. It does not infer missing metric readings, alter sample values, interpolate gaps, add polling, query hardware, persist data, collect identifiers, or send network activity. The summary reflows to two columns on narrow windows; physical readability and unusual clock/timestamp behavior still require validation on Windows, Linux, and macOS.


## 93. Robust capture-interval distribution summary (Batch #213)

The Recent activity capture-timing summary now includes the fastest and median valid interval in addition to the existing valid-interval count, arithmetic average, longest gap, and delayed-interval count. The median is calculated from a sorted copy of the positive, finite adjacent timestamp differences; for an even number of intervals it is the arithmetic midpoint of the two central values. The fastest and median figures help distinguish typical sampling cadence from occasional long pauses that can pull the average upward.

All values are derived from the existing bounded in-memory timestamps. No new sampling, hardware queries, persistence, identifiers, or network activity are introduced. When no valid intervals exist, duration values remain unavailable. The six summary cells use a three-column layout that collapses to two columns on narrow windows. Hosted CI checks cross-platform compilation; physical readability and timing behavior still require validation on Windows, Linux, and macOS.


## 94. Robust capture-interval variability indicator (Batch #214)

The Recent activity capture-timing summary now includes the median absolute deviation of valid adjacent capture intervals. This robust variability measure is the median of the absolute distances between each valid interval and the median interval. It describes typical timing spread while being less sensitive than standard deviation to an occasional unusually long pause. Even-sized deviation sets use the midpoint of their two central values; with no valid intervals the value remains unavailable.

The calculation uses only the existing bounded in-memory timestamps and sorted temporary arrays. It adds no polling, hardware queries, persistence, identifiers, or network activity. The summary now contains seven timing indicators and remains responsive, using four columns on wide layouts and two columns on narrow windows. Hosted CI checks cross-platform compilation; physical layout and timing validation remain necessary on Windows, Linux, and macOS.


## 95. Capture-interval context in System Monitor CSV exports (Batch #215)

The local System Monitor CSV export now includes two timing columns for every retained sample: elapsed milliseconds since the preceding sample, and whether that valid interval exceeded the existing 2.25-second delayed-interval threshold. The first row has no preceding sample, and non-finite, zero, or negative timestamp differences remain blank rather than being treated as valid intervals. The delay flag is also blank whenever the interval is unavailable; otherwise it is Yes or No. These fields make the existing timeline gaps and capture-timing summary easier to analyse in spreadsheet or data tools.

The values are calculated from timestamps already held in the bounded in-memory history at the moment the user explicitly exports. No additional polling, hardware reads, persistence, identifiers, process enumeration, or network activity is introduced. Existing CSV quoting, formula-injection protection, missing metric handling, and 36-sample limit remain unchanged. Hosted CI checks compilation; native file download and spreadsheet compatibility still require physical testing on Windows, Linux, and macOS.


## 96. midSunday process-resource history (Batch #216)

The opt-in System Monitor's Recent activity timeline and numeric Window summary now include midSunday's own resident-memory usage, process disk-read rate, and process disk-write rate. These are kept separate from system RAM and system-wide disk throughput so users can distinguish the app's resource footprint from activity across the whole device. The history and explicit CSV export include the same three per-sample values; CSV headers identify them as midSunday process metrics and retain their native byte or bytes-per-second units.

Only finite, non-negative process readings are retained; unavailable or invalid values remain gaps and blank CSV cells rather than being shown as zero. The values are captured from the same OS snapshot and existing 1.5-second opt-in polling cycle, stored only in the existing bounded 36-sample in-memory history, and discarded when history is cleared or the app session ends. No extra polling, process enumeration, executable paths, identifiers, persistent storage, or network transfer is added. Hosted CI checks cross-platform compilation; resource-counter semantics and layout still require physical validation on Windows, Linux, and macOS.

## 97. midSunday virtual address-space history (Batch #217)

The opt-in System Monitor's Recent activity timeline, numeric Window summary, and explicit local CSV export now include midSunday's virtual address-space size alongside its resident-memory usage. The new metric is labelled separately (APP VIRT / virtual address space) so it is not confused with physical RAM, resident memory, system RAM availability, or GPU memory. Values use the OS snapshot's existing process virtual-memory reading and are retained only when finite and non-negative; unavailable or invalid values remain visible gaps and blank CSV cells.

This addition reuses the same OS snapshot and existing 1.5-second opt-in polling cycle, bounded 36-sample in-memory history, and user-triggered local CSV export. It adds no process enumeration, extra polling, hardware queries, executable paths, identifiers, persistent storage, or network activity. Virtual address-space size is an address-space measurement and must not be interpreted as committed physical memory or actual RAM consumption. Hosted CI validates cross-platform compilation; physical counter semantics and narrow-window layout still require validation on Windows, Linux, and macOS.

## 98. Battery charge history (Batch #218)

The opt-in System Monitor now retains the operating system's aggregate battery charge percentage in the same bounded Recent activity timeline as CPU, RAM, process, network, disk, and GPU readings. A dedicated BAT row shows charge history oldest-to-newest, and its tooltips include sample time and charge percentage. The Window summary reports latest, average, peak, and available sample count for battery charge. Explicit local CSV exports include a clearly labelled Battery charge (%) column.

Battery percentage is clamped to the valid 0–100 range; null, non-finite, or unavailable readings remain gaps in the timeline and blank CSV cells. This is charge percentage only: it does not infer remaining runtime, battery health, capacity degradation, or charging state. The value is sourced from the existing OS live snapshot and existing opt-in 1.5-second polling cycle; no extra battery queries, device identifiers, serial numbers, persistence, or network activity are added. Systems without a battery correctly show unavailable readings. Hosted CI checks cross-platform compilation; battery-equipped laptop and desktop-without-battery behavior still require physical validation on Windows, Linux, and macOS.


## 99. Battery operating-state history (Batch #219)

The opt-in System Monitor now records the combined operating state reported by the existing battery snapshot alongside charge percentage. A discrete BAT ST timeline row shows each retained sample as a categorical segment, with tooltips identifying its capture time and state. The Window summary reports how many retained samples were observed in each state, and explicit local CSV exports include a separate Battery state column. Supported states are charging, discharging, empty, full, paused, mixed, unknown, not present, and unavailable; unexpected values are safely normalised to unavailable.

Battery state remains distinct from charge percentage: it does not estimate remaining runtime, battery health, degradation, or power draw. The feature reuses the existing OS snapshot, opt-in 1.5-second polling, 36-sample in-memory history, and user-triggered local CSV export. It adds no battery queries, identifiers, serial numbers, persistence, or network activity. Systems without batteries remain explicitly represented as not present rather than being mistaken for a charging state. Hosted CI checks cross-platform compilation; physical battery-state transitions and layout still require validation on Windows, Linux, and macOS.
