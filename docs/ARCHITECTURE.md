# midSunday — Architecture (Proposed)

**Status:** Preliminary architecture proposal; subject to feasibility research and owner approval.

## 1. Architecture goals

- Keep core functionality local and offline-capable.
- Separate UI, application logic, model inference, speech, storage, and tool permissions.
- Make model and voice engines replaceable.
- Minimise resource use on modest hardware.
- Keep network access explicit and optional.
- Avoid unnecessary services and paid infrastructure.

## 2. Proposed high-level stack

| Layer | Candidate | Status |
|---|---|---|
| Desktop shell | Tauri 2 | Proposed |
| Native/backend logic | Rust | Proposed |
| UI | React + TypeScript + Vite | Proposed |
| Local LLM runtime | llama.cpp | Candidate |
| Model format | Quantised GGUF | Candidate |
| Speech recognition | Vosk / whisper.cpp | Candidates |
| Speech synthesis | Piper / Kokoro | Candidates |
| Local database | SQLite | Candidate |

No dependency is final until current licensing, maintenance, security, and performance are reviewed.

## 3. Logical components

### Desktop shell
Owns the native window, application lifecycle, OS integration, permissions, and packaging.

### User interface
Owns chat presentation, onboarding, settings, character presentation, and user confirmation screens. It must not directly execute privileged OS operations.

### Application coordinator
Coordinates conversations, model selection, context preparation, cancellation, and tool requests.

### Model adapter
Defines a stable interface for loading/unloading a model and generating responses. The first implementation may use one local runtime, but the app should not bind product logic to a single model vendor.

### Speech adapters
Separate interfaces for speech-to-text and text-to-speech. Audio capture must be permissioned and visibly indicated.

### Local data layer
Stores settings, conversations, memory, and metadata. SQLite is the leading candidate. Data schema migrations and export/restore must be planned from the beginning.

### Tool/permission broker
The only route from model intent to desktop actions. It validates action names and arguments, checks permissions, requests confirmation when required, and records an appropriate local audit entry.

### Online service adapters
Optional adapters for web search and cloud inference. They must be disabled or unavailable when offline and must disclose what data is sent externally.

## 4. Trust boundaries

- Treat model output as untrusted.
- Treat webpage, document, and imported content as untrusted.
- The UI cannot bypass backend permissions.
- Model adapters cannot directly invoke arbitrary OS commands.
- The tool broker exposes only documented actions.
- External providers receive no conversation or memory unless the user has clearly enabled and approved that flow.
- Secrets and API keys must never be stored in source code or logs.

## 5. Resource management

- Load only the selected model.
- Unload unused engines when appropriate.
- Avoid running multiple heavy model/STT/TTS engines simultaneously on constrained devices.
- Bound context size and provide clear controls.
- Make model load/unload state visible.
- Provide graceful low-memory failure and recovery.
- Measure actual process and system memory rather than estimating from model file size.

## 6. Data organisation (conceptual)

Potential local data groups:
- Application settings.
- Conversation metadata and messages.
- User-approved memory records.
- Model and voice asset metadata.
- Tool permission settings and action history.
- Schema migration and backup metadata.

Exact schema, encryption requirements, storage paths, and backup behaviour remain open until the threat model and platform research are complete.

## 7. Offline and network policy

The app should support a strict offline mode. Local chat, local history, local memory, and installed local speech assets should not require network access. Network-dependent features must use explicit adapters and visible UI status.

Do not add background network calls, analytics, telemetry, or update checks without documenting their purpose and user controls.

## 8. Build and test strategy

- Keep frontend and Rust backend contracts typed and validated.
- Unit-test domain logic and permission decisions.
- Integration-test model and speech adapters.
- Test network-disabled operation.
- Test failure recovery and interrupted operations.
- Run performance tests on documented hardware profiles.
- Maintain dependency and licence inventories.

## 9. Open architecture decisions

- Final Tauri configuration and Rust module boundaries.
- Model process strategy: embedded library vs managed subprocess.
- Model download and update mechanism.
- SQLite schema and encryption/key management.
- Audio capture pipeline and device abstraction.
- Windows notification and startup integration.
- Secure update distribution and code signing.
- Linux/macOS abstraction boundaries.

These decisions must be resolved with prototypes and evidence, not assumed.

## 10. Hardware Intelligence & Adaptive Resource Manager (proposed)

A dedicated **Hardware Intelligence & Adaptive Resource Manager (HIARM)** is proposed to detect the machine's capabilities, estimate current resource headroom, recommend suitable model/runtime configurations, and manage workloads safely.

Responsibilities:
- Separate hardware inventory (CPU, core topology, installed RAM, GPU adapters/VRAM, OS, storage capacity) from live resource state (available RAM, process memory, GPU memory where reliable, and free disk space).
- Validate actual runtime/driver acceleration support; do not infer compatibility from a GPU name alone.
- Estimate model/runtime/context/speech resource needs using benchmark-backed catalogue entries.
- Recommend configurations according to user preference (quality, speed, lightweight, battery life) and explain uncertainty.
- Monitor resource pressure and schedule optional STT/TTS/background workloads without terminating unrelated applications.
- Require user consent before large downloads or material model changes; never silently enable cloud inference.
- Offer Automatic, Lightweight, Balanced, and High capability presets as candidates, with manual override.

For Windows-first detection, evaluate documented Windows system, memory, process, graphics/DXGI, storage, and power APIs. Vendor-specific utilities may be optional capability providers, never mandatory for basic operation. Hardware inventory stays local; no telemetry or upload by default.

OpenJarvis is a reference for separating detection from engine/model recommendation. It is not a midSunday dependency, and its setup-time recommendation flow is not sufficient evidence for our desired live adaptive behaviour. Full requirements and acceptance criteria are in [HARDWARE-INTELLIGENCE.md](HARDWARE-INTELLIGENCE.md).

No implementation or API dependency is approved by this proposal.

## 11. Windows hardware-detection implementation direction (research recommendation)

The initial Windows API review points toward a layered design:
- `sysinfo` as a possible cross-platform baseline for CPU, memory, process, and disk information.
- Microsoft's `windows` Rust crate for precise Windows-native calls: `GlobalMemoryStatusEx`, `GetLogicalProcessorInformationEx`, DXGI adapter enumeration and process GPU-memory budgets, and `GetDiskFreeSpaceExW`.
- Optional vendor diagnostics (such as NVML) only when installed and useful; never a mandatory requirement for basic detection.
- The selected inference runtime must independently validate its own acceleration backend.

Important data semantics: installed RAM differs from currently available RAM; dedicated VRAM differs from shared system memory; DXGI's GPU memory budget is process-specific and dynamic, not a global free-VRAM reading. WMI `Win32_VideoController.AdapterRAM` should not be the authoritative VRAM source because of its 32-bit field and documented accuracy caveats.

Detailed API mapping, crate candidates, fallback/confidence statuses, and source links: [HARDWARE-INTELLIGENCE.md](HARDWARE-INTELLIGENCE.md#12-windows-detection-research--api-and-rust-library-shortlist-2026-11-24).

This is a research recommendation only. No dependency or implementation has been approved; check exact crate versions, MSRV, features, transitive dependencies, and licences against the eventual project toolchain before adding anything.


## Framework selection note (2026-11-24)

Tauri 2 + Rust + React + TypeScript + Vite is the proposed direction for the first Windows shell prototype, not an approved production lock. See [DESKTOP-FRAMEWORK-EVALUATION.md](DESKTOP-FRAMEWORK-EVALUATION.md) for the Tauri/Electron comparison, WebView2 offline-install trade-offs, security boundaries, and prototype acceptance criteria. Revisit this proposal if measured prototype results show a concrete blocker.
