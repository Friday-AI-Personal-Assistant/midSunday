# midSunday

**AI Personal Companion**

midSunday is an independent, local-first desktop AI companion designed to combine useful assistance with a personal, customisable companion experience.

> Project status: **Planning / foundation**  
> Initial target: **Windows**  
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
The repository currently contains planning documentation. **No application code, AI runtime, model, voice engine, installer, or production feature is implemented yet.**

Phase 1 desk research is underway. Initial findings and unresolved questions are recorded in [Phase 1 feasibility research](docs/PHASE-1-RESEARCH.md). Practical benchmarks and licence clearance are still outstanding; no application implementation has begun.

## Development approach
Development proceeds in small, testable milestones. A task is complete only when its acceptance criteria, tests, documentation, privacy review, and applicable licence review are satisfied.

No paid usage or external service is to be enabled without explicit approval.

## Repository
Private repository: [Friday-AI-Personal-Assistant/midSunday](https://github.com/Friday-AI-Personal-Assistant/midSunday)
