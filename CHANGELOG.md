# Changelog

All notable project changes are recorded here. The project is in foundation/prototype development; no application release has been made.

## 2026-11-24
- Added a manual-only GitHub Actions workflow to compile the prototype on a hosted Windows runner and retain the NSIS installer artifact for 7 days. It is not triggered automatically; no run has been started.
- Added instructions for manually starting the Windows build and retrieving its artifact.
- Completed the Tauri 2 versus Electron desk evaluation and recorded Tauri 2 as the provisional direction for the first Windows shell prototype; production approval remains pending measurements.
- Scaffolded the first Tauri 2 + Rust + React + TypeScript + Vite desktop shell under prototype/desktop-shell, including a harmless Rust status command and a minimal capability allow-list.
- The shell has not yet been built or run on Windows; no AI model, speech engine, cloud service, telemetry, or paid dependency has been added.

## 2026-11-23
- Added Urdu-specific STT/TTS research, including commercial-licence blockers and lightweight experimental candidates.
- Expanded the read-only FRIDAY reuse review to cover frontend, native workspace code, capabilities, security/product documents, CI workflows, and repository hygiene.
- Added preliminary upstream licence observations to the licence register; no candidate is approved for bundling.
- Added initial Phase 1 feasibility research covering desktop frameworks, local LLM runtime/model candidates, STT/TTS, SQLite, hardware questions, and licensing risks.
- Updated roadmap and README with Phase 1 research status and next actions.
- Established the complete product and development roadmap.
- Added the initial project overview, product specification, architecture proposal, decision log, and risk register.
- Added offline-first, hardware compatibility, voice/language, personality/character, privacy/safety, and commercial licensing requirements.
- Added a read-only FRIDAY reuse analysis based on the files listed in that document.
- No application code or runtime features have been implemented.
