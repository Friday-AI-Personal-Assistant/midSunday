# midSunday

**AI Personal Companion**

midSunday is an independent, local-first desktop AI companion designed to combine useful assistance with a personal, customisable companion experience.

> Project status: **Interactive desktop workspace prototype implemented; cross-platform CI passing; local runtime integration in progress**  
> Build targets: **Windows, Linux and macOS** (Windows first priority)  
> Core direction: **Offline-capable, privacy-first, no mandatory subscription**

## What we are building
The long-term goal is a desktop companion that can provide:
- Local AI chat using models installed on the user's computer.
- User-controlled local memory and conversation history.
- Local voice input and speech output where supported.
- Customisable personalities and companion characters.
- Carefully permissioned desktop assistance.
- Optional internet-based search and optional cloud model providers.

The product must remain useful without a cloud account. Online features must be clearly identified. No paid service, subscription, hosted backend, domain, certificate, or paid asset may be adopted without explicit owner approval.

## Product principles
1. Local-first by default; no silent transfer of conversations, audio, files, or memory.
2. No hidden costs or mandatory subscriptions.
3. Honest, benchmark-backed hardware and language claims.
4. User control over memory, privacy, permissions, and companion behaviour.
5. Safe, allow-listed computer actions; never arbitrary model-generated shell execution.
6. Commercial licence review for every bundled dependency, model, voice, and creative asset.
7. Independent identity and codebase from FRIDAY.

## Project documentation

### Master planning and continuity
- [Complete product roadmap](ROADMAP.md)
- [Project history](docs/PROJECT-HISTORY.md)
- [Decision log](docs/DECISIONS.md)
- [Risks and open questions](docs/RISKS-AND-OPEN-QUESTIONS.md)
- [Changelog](CHANGELOG.md)

### Early prototype
- [First Tauri desktop-shell prototype](prototype/desktop-shell/README.md)
- [Windows build workflow guide](prototype/desktop-shell/WINDOWS-BUILD.md)
- [Cross-platform build workflows](docs/CROSS-PLATFORM-BUILDS.md)

### Product and engineering
- [Product specification](docs/PRODUCT-SPECIFICATION.md)
- [Proposed architecture](docs/ARCHITECTURE.md)
- [Offline-first requirements](docs/OFFLINE-FIRST-REQUIREMENTS.md)
- [Hardware compatibility and benchmark policy](docs/HARDWARE-COMPATIBILITY.md)
- [Hardware Intelligence & Adaptive Resource Manager](docs/HARDWARE-INTELLIGENCE.md)
- [Voice and language requirements](docs/VOICE-AND-LANGUAGES.md)
- [Personalities and characters](docs/PERSONALITIES-AND-CHARACTERS.md)
- [Privacy, security, and safety](docs/PRIVACY-AND-SAFETY.md)

### Reuse and commercial readiness
- [FRIDAY reuse analysis](docs/FRIDAY-REUSE-ANALYSIS.md)
- [Commercial licensing policy](docs/COMMERCIAL-LICENSING.md)
- [Licence review register](LICENSE-REVIEW.md)
- [Phase 1 feasibility research](docs/PHASE-1-RESEARCH.md)
- [Windows desktop framework evaluation](docs/DESKTOP-FRAMEWORK-EVALUATION.md)
- [Repeatable AI benchmark protocol](docs/BENCHMARK-PROTOCOL.md)
- [Benchmark dataset v1 (draft)](docs/BENCHMARK-DATASET-V1.md)

The roadmap is the master sequence of work. These documents provide detailed requirements, decisions, risks, and technical context. Update them as the project changes so development can recover without relying on chat history.

## Current status
The repository contains an interactive Tauri desktop workspace prototype alongside the project documentation. The workspace includes local llama.cpp runtime connection, multi-turn chat, and SQLite-backed conversation history with rename/delete, versioned JSON export, and clear-history controls. The model runtime is not bundled: users must provide a compatible executable and model file. Voice, long-term personal memory, hardware detection, import/restore, and real-device validation remain in progress.

Phase 1 feasibility research continues in parallel. Initial findings and unresolved questions are recorded in [Phase 1 feasibility research](docs/PHASE-1-RESEARCH.md). Automatic Windows/Linux/macOS CI builds now run on pushes to `main` and pull requests targeting `main`; Runs #13 and #14 passed on all three platforms. The Rust backend now defines a tested local-runtime adapter contract and lifecycle state machine, but no actual model runtime is connected. Practical benchmarks, real-device runtime testing, and licence clearance remain outstanding.

## Development approach
Development proceeds in small, testable milestones. A task is complete only when its acceptance criteria, tests, documentation, privacy review, and applicable licence review are satisfied.

No paid usage or external service is to be enabled without explicit approval.

## Repository
Public development repository: [Friday-AI-Personal-Assistant/midSunday](https://github.com/Friday-AI-Personal-Assistant/midSunday)
