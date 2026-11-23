# midSunday — Hardware Intelligence & Adaptive Resource Manager

**Status:** Proposed architecture requirement; not implemented or benchmarked.  
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
