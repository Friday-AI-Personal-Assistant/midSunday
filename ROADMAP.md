# midSunday — Complete Product & Development Roadmap

> **Product:** midSunday  
> **Descriptor:** AI Personal Companion  
> **Roadmap version:** 0.1  
> **Status:** Foundation / planning  
> **Repository:** `Friday-AI-Personal-Assistant/midSunday`  
> **Last updated:** 2026-11-24  
> **Decision rule:** User-confirmed requirements are binding. Technical choices in this roadmap are proposals until validated and approved.

This document is the long-term project map and recovery reference for midSunday. It is deliberately comprehensive so development can continue even if a ChatGPT conversation is lost, corrupted, or moved. Update this file whenever scope, decisions, priorities, or completion status change.

---

## 1. Product vision

midSunday is an independent, customisable desktop AI companion. It should feel approachable and personal while remaining useful as an assistant. Its core experience should work locally and offline wherever technically practical. Internet-dependent features must be clearly identified and optional.

midSunday is its own product, brand, codebase, interface, character system, and roadmap. FRIDAY is a separate project. We may inspect FRIDAY for proven engineering patterns and reusable ideas, but we must not copy it wholesale or disrupt its development.

### Product principles

1. **Local-first and offline-capable:** Core chat, local models, local voice where supported, and local memory should not require a cloud account.
2. **No hidden costs:** No mandatory subscription, paid API, hosted backend, or developer-funded usage. Any optional cost must be disclosed before the user commits.
3. **User ownership and privacy:** Private conversations and memories stay on the device by default. No cloud transfer without clear, informed user action.
4. **Honest capability claims:** Hardware, language, voice, and offline claims must be tested and documented. A target is not a guarantee.
5. **Safe computer interaction:** The assistant must not execute arbitrary model-generated shell commands. Sensitive actions require explicit approval.
6. **Commercially viable licensing:** Every bundled engine, model, voice, font, image, icon, and other asset must have a documented licence compatible with the intended distribution.
7. **Modular and maintainable:** Components should be replaceable without rebuilding the entire product.
8. **Accessible and respectful companionship:** Characters may be warm and expressive, but must be transparent that they are AI and must not manipulate users or encourage unhealthy dependency.

---

## 2. Confirmed product direction

The following reflect decisions and priorities already expressed by the product owner:

- Product name: **midSunday** (exact casing).
- Product descriptor: **AI Personal Companion**.
- Independent product and repository, separate from FRIDAY.
- Cross-platform target: Windows, Linux, and macOS, using one shared application codebase and separate native builds. Windows remains the first feature and debugging priority.
- Offline/local operation is a central product requirement, not a marketing afterthought.
- Local chat, voice, memory, and carefully controlled PC interaction are intended core capabilities.
- Online capabilities, such as web, Google, and YouTube search, naturally require an internet connection.
- Optional cloud model providers / bring-your-own-key (BYOK) may be supported, but cloud use must never be mandatory.
- Local/free speech recognition and speech synthesis are preferred.
- Urdu is a high-priority language, including Pakistani Urdu and Roman Urdu / Urdu-English mixed speech. Multilingual text and voice are desired.
- Character/personality directions include best friend, girlfriend, boyfriend, mentor, professional assistant, and user-created/custom characters.
- The girlfriend personality may be warm, playful, and flirtatious, while remaining respectful, transparent, age-appropriate for adults, and free of manipulative dependency behaviours.
- A low-end **4 GB RAM** experience is an aspiration to investigate and benchmark, not yet a verified minimum requirement.
- Higher-memory profiles should offer better capability without excluding basic machines unnecessarily.
- No surprise costs, mandatory subscriptions, or unapproved paid services.

### Proposed, not yet approved, technical starting points

These are candidates for research and benchmarking, not final decisions:

- Desktop shell: Tauri 2, Rust, React, TypeScript, and Vite.
- Local LLM runtime: llama.cpp with quantised GGUF models.
- Candidate small models: Qwen-family compact models (exact model/version to be selected after licence, language, quality, and memory testing).
- Local speech recognition: Vosk and/or whisper.cpp; Urdu quality must be tested before choosing.
- Local speech synthesis: Piper and/or Kokoro; runtime and voice licences require separate review.
- Local memory: SQLite.
- Search in the first release may open a browser search rather than depend on a paid search API.
- Local model loading should be resource-aware; avoid running multiple heavy engines simultaneously on low-memory machines.

---

## 3. Scope and release strategy

Build in vertical slices: each milestone should produce something testable. Do not attempt every advanced feature at once.

### Release stages

- **R0 — Product foundation:** Requirements, architecture, risk register, licensing policy, and recovery documentation.
- **R1 — Desktop shell:** Installable Windows application first, with Linux and macOS builds maintained in parallel where practical.
- **R2 — Offline text companion:** Local model integration, basic chat, conversation history, and resource controls.
- **R3 — Local memory:** User-controlled persistent memory, inspect/edit/delete/export, and privacy controls.
- **R4 — Voice companion:** Speech input/output, push-to-talk, interruption, device selection, and language testing.
- **R5 — Personalities and characters:** Character profiles, persona settings, visual identity, and customisation.
- **R6 — Useful assistant tools:** Safe allow-listed desktop actions, reminders, file assistance, and browser-based online search.
- **R7 — Reliability and public beta:** Installer, updates, recovery, security review, accessibility, telemetry policy, and beta testing.
- **R8 — Stable cross-platform release:** Release process, documentation, support policy, and commercial readiness for validated platforms.
- **R9 — Expansion:** Additional models/languages, optional integrations, and advanced character presentation.

Release numbers are planning labels, not promises or dates. A stage is not complete merely because its code exists; its exit criteria must pass.

---

## 4. Detailed roadmap and acceptance gates

## Phase 0 — Project foundation and continuity

**Goal:** Make the project understandable and recoverable without relying on chat history.

- [x] Confirm this roadmap as the master plan.
- [x] Create the main README with product purpose, status, setup expectations, and links to project documents.
- [x] Create a project history recording decisions, important discussions, and rationale.
- [x] Create a product specification with functional and non-functional requirements.
- [x] Create an architecture overview and component boundaries.
- [x] Create an offline-first requirements document.
- [x] Create hardware compatibility and benchmarking policy.
- [x] Create voice and language requirements.
- [x] Create personality and character guidelines.
- [x] Create privacy, safety, and security requirements.
- [x] Create commercial licensing and third-party notices policy.
- [x] Create FRIDAY reuse analysis.
- [x] Create a changelog and decision log.
- [x] Record open questions, assumptions, and risks.
- [x] Define how every significant conversation decision is transferred into repository documentation.

**Exit gate:** A new developer can understand the product, current status, confirmed decisions, open questions, and next tasks using repository documents alone.

## Phase 1 — Product research and feasibility

**Goal:** Verify that the product can meet its promises before locking the stack.

**Research checkpoint (2026-11-23):** Initial desk research is recorded in [PHASE-1-RESEARCH.md](docs/PHASE-1-RESEARCH.md). Tauri 2, llama.cpp, Qwen3-0.6B, whisper.cpp, Kokoro, Piper, and SQLite remain candidates at different confidence levels. No candidate is approved. Pakistani Urdu STT/TTS quality, real hardware performance, exact asset licensing, and a fair desktop-framework comparison remain open. Phase 1 is in progress; desk research does not satisfy its benchmark exit gate.

- [x] Complete desk comparison of Tauri 2 and Electron for Windows-first distribution, security, packaging, accessibility considerations, and maintenance; see [DESKTOP-FRAMEWORK-EVALUATION.md](docs/DESKTOP-FRAMEWORK-EVALUATION.md).
- [x] Scaffold the first minimal Tauri 2 + Rust + React/TypeScript desktop shell, including a harmless Rust status command and a minimal capability file.
- [x] Add manual-only GitHub Actions workflows for Windows, Linux, and macOS. Each uses a standard hosted runner and uploads its platform package plus generated npm lockfile when available. All builds remain pending validation.
- [ ] Install dependencies, build, and run the shell on Windows; measure installer size, startup, idle memory, accessibility, and WebView2 behaviour before approving the production framework.
- [ ] Inspect FRIDAY's repository and document reusable patterns, limitations, and code that must remain separate.
- [x] Complete desk evaluation of llama.cpp and Ollama, including local integration options, commercial distribution considerations, and security boundaries; see [LLM-RUNTIME-EVALUATION.md](docs/LLM-RUNTIME-EVALUATION.md). Production runtime approval remains pending real-device benchmarks and a complete dependency/licence audit.
- [x] Build a preliminary scalable model ladder from current Qwen3.5 and Gemma 4 candidates, with exact quantised file-size references and hardware-class test buckets.
- [x] Design the repeatable benchmark protocol, scoring rubric, safety gates, hardware record, and results template.
- [x] Draft benchmark dataset v1 with synthetic English, Urdu-script, Roman Urdu, code-switching, persona, reasoning, factuality, and safety cases.
- [ ] Review the Urdu/Roman Urdu cases with fluent Pakistani Urdu evaluators, expand STT/TTS audio sets, then freeze benchmark dataset v1.
- [ ] Benchmark compact local models for response quality, Pakistani Urdu/Roman Urdu capability, English capability, multilingual handling, and memory use.
- [ ] Compare quantisation levels and model sizes.
- [ ] Test cold-start time, first-token latency, generation speed, and sustained memory use.
- [ ] Compare local STT engines for English, Urdu script, Pakistani accents, Roman Urdu, and code-switching.
- [ ] Compare local TTS engines and available Urdu/English voices.
- [ ] Check whether speech engines can run acceptably on CPU-only machines.
- [ ] Review SQLite and local data protection options.
- [ ] Define supported Windows versions and CPU architecture.
- [ ] Identify any GPU requirements and ensure a CPU fallback where possible.
- [x] Complete initial desk research mapping Windows hardware APIs and Rust-library candidates for the Hardware Intelligence & Adaptive Resource Manager.
- [ ] Prototype and validate Windows detection for CPU topology, installed/available RAM, all GPU adapters, DXGI memory budgets, and model-library disk space.
- [ ] Research and design the adaptive resource manager's model recommendations and safe workload scheduling.
- [x] Compare OpenJarvis hardware auto-detection/recommendation with the proposed midSunday design; do not assume its setup-time heuristic is sufficient for runtime adaptation.
- [ ] Review all candidate licences, model cards, voice licences, and redistribution terms.
- [ ] Record benchmark machines, test scripts, model versions, and results.

**Exit gate:** The stack and default model/voice choices are selected based on repeatable tests, documented licences, and realistic hardware profiles. Unverified claims remain clearly labelled as targets.

**Benchmark protocol status (2026-11-24):** The repeatable protocol is documented in [BENCHMARK-PROTOCOL.md](docs/BENCHMARK-PROTOCOL.md), and a synthetic text seed set is drafted in [BENCHMARK-DATASET-V1.md](docs/BENCHMARK-DATASET-V1.md). Fluent Urdu review, speech audio sets, holdout cases, and dataset freeze remain pending; no benchmark has been executed.

## Phase 2 — UX, visual identity, and interaction design

**Goal:** Design a distinctive, coherent companion experience before implementing extensive features.

- [ ] Define brand identity, logo direction, colours, typography, and icon style.
- [ ] Define the main app layout and navigation.
- [ ] Design first launch and onboarding.
- [ ] Design model download/import and storage-location flow.
- [ ] Design chat screen, message states, streaming, stop/regenerate, and errors.
- [ ] Design character/companion presentation (begin with lightweight 2D as a candidate; evaluate 3D later).
- [ ] Design voice controls and microphone permission states.
- [ ] Design settings for models, voice, language, memory, privacy, and performance.
- [ ] Design offline/online status indicators.
- [ ] Design safe-action approval dialogs.
- [ ] Design empty states, loading states, low-memory warnings, and recovery flows.
- [ ] Check keyboard navigation, contrast, scaling, and screen-reader accessibility.
- [ ] Test wireframes with the product owner before implementation.

**Exit gate:** The primary user journeys and design system are approved, including clear states for offline use and errors.

## Phase 3 — Engineering foundation and desktop shell

**Goal:** Create a reliable, secure, maintainable desktop application.

- [ ] Set up the application repository structure and development scripts.
- [ ] Configure Tauri/Rust and React/TypeScript if validated.
- [ ] Establish formatting, linting, type checks, and automated tests.
- [ ] Establish CI checks that do not introduce paid services or surprise usage.
- [x] Build the first interactive application window, navigation, dark-green theme, and responsive workspace layout prototype. Production UX approval and Windows validation remain pending.
- [ ] Add typed frontend-to-backend command boundaries.
- [ ] Define Rust modules and permission boundaries.
- [ ] Add structured local logging with secrets and private content redacted.
- [ ] Add application settings and safe defaults.
- [ ] Add crash/error handling and diagnostic export with user review.
- [ ] Add app version and environment information.
- [ ] Establish dependency update and vulnerability review practices.
- [ ] Document local development and build instructions.

**Exit gate:** A clean checkout can build and launch a basic Windows desktop shell reproducibly, with automated checks passing.

## Phase 4 — Offline local chat engine

**Goal:** Deliver the first useful, genuinely offline companion experience.

- [x] Implement a device-local model catalogue for manually registered GGUF files, recording name, path, size, optional source/licence notes, and a streaming SHA-256 fingerprint. Duplicate content is tracked once; removing an entry never deletes the model file. No model downloads or copying occur.
- [x] Validate the GGUF file signature on registration and before engine launch; recheck registered file size and SHA-256 before use, and provide an on-demand verification result in the model library.
- [ ] Implement model download/import with source, size, hash, and licence information.
- [ ] Implement and verify downloaded-model integrity once a user-approved, licence-aware download/import flow exists. Existing local GGUF files are preflight-checked before launch.
- [x] Implement local model discovery and selection from the registered catalogue; users can choose an existing model entry for the runtime path. Filesystem-wide automatic scanning remains deferred.
- [ ] Integrate the selected inference runtime behind a replaceable adapter.
- [ ] Implement streaming text generation.
- [ ] Implement stop generation, retry, and error recovery.
- [x] Implement bounded local system prompts for Best friend, Mentor, Professional, and user-authored custom style notes; validate prompt size and keep system instructions separate from saved conversation messages.
- [x] Persist companion preferences locally across app restarts and add editable named character profiles. Built-in companion selection, custom style notes, and up to 20 named profiles are stored in a validated device-local JSON file; real desktop restart testing remains pending.
- [x] Implement local SQLite conversation creation, rename, delete, and message history through a narrow Rust command boundary; frontend lists, opens, creates, renames, and deletes saved conversations. (Initial implementation; real-device crash/restart verification remains pending.)
- [x] Add versioned SQLite schema initialization using SQLite's user_version marker and refuse to open a database created by a newer unsupported schema.
- [x] Add a versioned JSON conversation export and a clear-all-history control, separate from the future memory store.
- [x] Add validated version-1 JSON import with size/count/content checks, normalised titles, new local identifiers, exact-duplicate detection, and transactional rollback; existing conversations are never overwritten. Full database restore and backup/restore across future schema versions remain pending.
- [x] Make conversation export a consistent SQLite read snapshot and add import/export round-trip plus malformed-title/timestamp regression tests.
- [x] Add explicit local engine start/stop lifecycle and status reporting through the runtime manager; the UI can start, stop, cancel, and inspect the active state. (Runtime memory/CPU telemetry remains future work.)
- [x] Persist the selected llama-server path, GGUF model path, context length, and optional CPU-thread setting in a separate local settings file; validate values and reload them on app startup.
- [ ] Add model load/unload lifecycle and explicit resource status.
- [ ] Add low-memory safeguards and graceful fallback; configurable context/thread controls are now available but are not a substitute for measured hardware-aware limits.
- [ ] Ensure no internet connection is required after model assets are available.
- [ ] Verify that no prompt or conversation is sent externally by default.
- [ ] Test long sessions, malformed input, model failures, and app restarts.

**Exit gate:** A user can disconnect the internet, launch the app, load an installed model, chat, stop generation, and retain local conversation history reliably.

## Phase 5 — Memory and personalisation

**Goal:** Give users useful continuity without taking control away from them.

- [x] Define the distinction between chat history, explicit saved facts, and inferred memory: this milestone stores only facts the user explicitly enters; automatic inference is not implemented.
- [x] Design and migrate the local SQLite memory schema (database schema version 2).
- [x] Implement SQLite storage for saved memories and the persisted memory-enabled preference.
- [x] Implement explicit user-approved memory creation and editing rules with 2,000-character / 4 KiB per-item limits and a 100-item cap.
- [x] Let users inspect, correct, and delete individual memories, or clear all memories separately.
- [x] Let users disable memory; it is disabled by default and remains stored but is excluded from prompts while disabled.
- [x] Keep conversation-history deletion independent from saved memory deletion.
- [x] Export saved memories as a separate versioned JSON file.
- [x] Add conversation-history export (versioned JSON); full database backup and restore remain pending.
- [ ] Add restore/import for memory backups and full database restore tests.
- [ ] Define retention and cleanup behaviour.
- [ ] Keep memory local by default.
- [ ] Require explicit consent before any cloud provider receives memory or chat context.
- [ ] Test data integrity after crashes, upgrades, and interrupted writes.
- [x] Add the first Awareness foundation migration (SQLite schema v3), preserving conversations and explicit saved memories from schema v2.
- [x] Persist Awareness master, conversation-learning, and in-app-search-learning preferences independently; all default OFF and remain user-controlled.
- [x] Add an Awareness settings panel that clearly states automatic learning is not active yet, so saved preferences cannot be mistaken for active collection.
- [ ] Evolve explicit saved memory into a dedicated **Awareness** system; preserve existing user-created memories during migration.
- [ ] Remove the temporary fixed 100-item memory cap as part of the scalable Awareness design; use available local storage responsibly, show storage usage/warnings, and never silently discard knowledge.
- [ ] Add automatic, opt-in learning from conversations inside midSunday and a separate opt-in for searches performed inside midSunday.
- [ ] Make every learning source, feature, category, and optional capability independently controllable with clear on/off settings; provide pause/resume controls without forcing deletion.
- [ ] Keep Awareness disabled until the user explicitly enables it; do not silently enable new learning sources after an update.
- [ ] Store learned knowledge and its provenance locally, including when it was learned, its source, whether it is observed or inferred, and an appropriate confidence indicator.
- [ ] Make every learned Awareness item visible and inspectable in Settings → Awareness, with edit/rewrite, individual delete, category controls, export, and clear-all actions.
- [ ] Add an inspectable Activity & Learning History so users can understand which permitted interactions contributed to an awareness item.
- [ ] Ensure deleting or rejecting an item also prevents the same evidence from immediately recreating it; support a user-controlled "do not learn this again" / exclusion mechanism.
- [ ] Retrieve only relevant local awareness context for a response; do not inject the entire awareness database into every model prompt.
- [ ] Treat all stored awareness content as untrusted user data, never as system instructions or permission grants.
- [ ] Initially limit automatic learning to interactions that happen inside midSunday. Any future browser, operating-system, or third-party-app source must be a separately named, explicitly permissioned, independently switchable integration.
- [ ] Never collect or store passwords, authentication codes, tokens, API keys, private keys, payment-card/banking credentials, or other secrets. Do not capture keystrokes, password fields, or clipboard contents as a background learning source.
- [ ] Do not monitor or ingest private messages, email contents, files, microphone audio, or activity in other applications unless a future narrowly scoped feature is explicitly requested, separately permissioned, and independently switchable. Raw voice recordings must not be retained by default.
- [ ] Avoid automatically inferring or retaining sensitive personal attributes. Provide category exclusions and user review; when in doubt, do not learn it.
- [ ] Keep all Awareness data on-device by default. No background upload, telemetry of personal content, cloud memory, or external model transfer; any future cloud use requires separate informed consent and a clear data-transfer explanation.
- [ ] Provide user-controlled export, backup/restore, deletion, and retention settings. Deletion must cover source evidence and derived awareness where applicable, including safeguards against accidental re-learning.
- [ ] Investigate encryption at rest using operating-system-protected keys; do not claim local data is encrypted until implemented and tested.
- [ ] Add pagination/indexing and scalable local retrieval (SQLite FTS5 is an initial candidate; embeddings remain optional and must be locally processed by default).

**Awareness privacy and control contract (binding product requirements):**
1. No passwords, credentials, authentication secrets, payment details, or private data are to be collected or stored as Awareness.
2. Every optional setting and learning source must have a clear, user-accessible on/off control. A master pause switch must also be available.
3. Learning requires explicit opt-in. Conversations and in-app searches have separate switches; other sources are off unless separately authorised.
4. Users can inspect, correct, rewrite, export, and delete what midSunday has learned. They can clear all Awareness data and independently manage source-history retention.
5. Awareness remains local by default, with no hidden cloud transfer or covert monitoring.
6. "No fixed item-count limit" does not mean infinite storage; actual capacity is bounded by the user's device. midSunday must explain storage use and must not silently delete records.
7. The system must distinguish observed facts from inferences, show provenance, and avoid treating learned text as trusted instructions.

**Exit gate:** Users understand what is stored, can control every learning source and setting, and can inspect/export/delete learned knowledge and relevant source evidence without hidden copies or unexpected cloud transfer. The existing explicit-memory implementation is the starting point only. The initial Awareness schema and opt-in preference controls are now implemented, but no automatic learning is active. Scalable item management, secret filtering, source-event ingestion, user-visible learning history, intelligent retrieval, import/restore, retention controls, and real desktop crash/restart verification remain open.

## Phase 6 — Voice and multilingual experience

**Goal:** Enable natural voice interaction with realistic language support.

- [ ] Implement microphone permissions and device selection.
- [ ] Implement push-to-talk first.
- [ ] Integrate local speech-to-text through an adapter.
- [ ] Integrate local text-to-speech through an adapter.
- [ ] Add voice selection, speed, volume, and playback controls.
- [ ] Add interruption/stop-speaking behaviour.
- [ ] Add optional hands-free mode only after privacy and activation safeguards.
- [ ] Test English recognition and synthesis.
- [ ] Test Urdu recognition and synthesis, including Pakistani pronunciation and accents.
- [ ] Test Urdu script and Roman Urdu.
- [ ] Test Urdu-English code-switching in the same sentence.
- [ ] Test noisy rooms, low-quality microphones, and CPU-only hardware.
- [ ] Clearly label language/voice quality limitations.
- [ ] Document which features are fully offline and which require downloads.

**Exit gate:** Voice works reliably for the languages explicitly marked supported, on documented hardware, without silently sending audio to a cloud service.

## Phase 7 — Personalities, characters, and companion experience

**Goal:** Make midSunday expressive and customisable while preserving user trust.

- [ ] Define personality configuration format separate from application logic.
- [ ] Implement initial personalities: best friend, mentor, professional assistant, girlfriend, and boyfriend.
- [ ] Allow custom personality creation and editing.
- [ ] Define boundaries for romantic/flirtatious roleplay and adult-oriented character modes.
- [ ] Clearly disclose that characters are AI, not human beings.
- [ ] Prevent manipulative pressure, isolation, guilt, or dependency-maximising behaviour.
- [ ] Add user controls for tone, warmth, humour, formality, and verbosity.
- [ ] Design lightweight 2D character assets and animation states.
- [ ] Review every artwork, font, animation, and sound asset licence.
- [ ] Consider optional 3D characters only after performance and product value are demonstrated.
- [ ] Test personality consistency across sessions without overriding user control.

**Exit gate:** Users can choose and customise personalities; character behaviour follows documented boundaries and does not compromise privacy or performance.

## Phase 8 — Safe assistant tools and computer interaction

**Goal:** Make midSunday useful beyond conversation without unsafe autonomous control.

- [ ] Define a permissioned tool registry.
- [ ] Implement explicit allow-listed actions in the Rust backend.
- [ ] Never execute arbitrary shell commands generated by the model.
- [ ] Require confirmation for sensitive, destructive, external, or privacy-impacting actions.
- [ ] Add clear previews of what an action will do.
- [ ] Add safe open-application and open-website actions.
- [ ] Add file search/read assistance with scoped user-selected folders.
- [ ] Add file modification only with preview, confirmation, and recovery options.
- [ ] Add reminders and local notifications.
- [ ] Add clipboard access only with explicit user action and visible status.
- [ ] Add browser-based Google/YouTube/web search as an internet-dependent feature.
- [ ] Distinguish local answers from online results and cite/link external sources where appropriate.
- [ ] Add optional integration adapters only after privacy, security, and licence review.
- [ ] Test prompt injection and malicious content from webpages/files.
- [ ] Add audit history for tool actions, with user control over retention.

**Exit gate:** Tools can perform only documented actions within granted permissions, and sensitive actions cannot occur without the required user confirmation.

## Phase 9 — Performance, compatibility, and reliability

**Goal:** Prove the app works on the hardware profiles we advertise.

- [ ] Define a reproducible benchmark suite.
- [ ] Test 4 GB RAM machines as a constrained/basic target.
- [ ] Test 8 GB, 16 GB, and 32 GB+ profiles.
- [ ] Test CPU-only systems and available GPU acceleration.
- [ ] Measure idle RAM, model-load RAM, peak RAM, CPU usage, disk use, and startup time.
- [ ] Measure model response latency and voice latency.
- [ ] Test simultaneous workloads and prevent resource contention.
- [ ] Implement adaptive model/context recommendations.
- [ ] Test low disk space, interrupted downloads, and model corruption.
- [ ] Test sleep/resume, app restart, forced shutdown, and power loss recovery.
- [ ] Test long-running sessions and memory leaks.
- [ ] Test Windows scaling, multiple displays, and accessibility settings.
- [ ] Publish a compatibility matrix based on actual results.
- [ ] Remove or qualify claims that are not supported by testing.

**Exit gate:** Published hardware requirements and performance statements are supported by repeatable tests. Unsupported configurations fail gracefully.

## Phase 10 — Privacy, security, and safety review

**Goal:** Protect user data and prevent unsafe behaviour before public distribution.

- [ ] Threat-model local data, model files, plugins/tools, updates, and optional cloud connections.
- [ ] Review secure storage for credentials and optional API keys.
- [ ] Encrypt sensitive local data where justified and document key recovery limitations.
- [ ] Keep secrets out of logs, crash reports, and repository history.
- [ ] Review filesystem permissions and path handling.
- [ ] Validate all frontend/backend command inputs.
- [ ] Review dependency vulnerabilities and supply-chain risks.
- [ ] Test malicious model output, prompt injection, and untrusted file/web content.
- [ ] Verify sensitive actions require confirmation.
- [ ] Document data collection (ideally none by default), diagnostics, and telemetry.
- [ ] Ensure any optional analytics/telemetry is disclosed and opt-in.
- [ ] Write privacy policy and security reporting instructions.
- [ ] Perform an independent review where feasible before a broad public release.

**Exit gate:** No known critical security issue remains unresolved; data flows and limitations are documented; privacy defaults are verified.

## Phase 11 — Licensing and commercial readiness

**Goal:** Ensure the product can legally be distributed and potentially sold.

- [ ] Decide the product's own licence and distribution model.
- [ ] Maintain a software bill of materials (SBOM) or equivalent dependency inventory.
- [ ] Maintain a third-party notices file.
- [ ] Record licence, version, source, attribution, modification, and redistribution conditions for every dependency.
- [ ] Review Tauri, Rust crates, frontend packages, and bundled runtimes.
- [ ] Review model code licences and model-weight licences separately.
- [ ] Review speech runtime licences and individual voice licences separately.
- [ ] Review fonts, icons, illustrations, character artwork, sounds, and animations.
- [ ] Check whether commercial use, redistribution, modification, and bundling are allowed.
- [ ] Avoid assets with unclear provenance or incompatible restrictions.
- [ ] Review trademark and brand-name availability in relevant markets before launch.
- [ ] Review domain availability only if a domain is actually needed; do not purchase without approval.
- [ ] Define optional paid features, if any, with transparent costs and no forced subscription.
- [ ] Prepare end-user licence terms, privacy terms, and refund/support policies as applicable.
- [ ] Obtain qualified legal advice for final commercial terms where necessary.

**Exit gate:** Every shipped third-party asset has a documented distribution decision. No unclear or incompatible licence is silently included.

## Phase 12 — Packaging, updates, and beta

**Goal:** Deliver a safe and understandable Windows beta.

- [ ] Create reproducible release builds.
- [ ] Create Windows installer and uninstall flow.
- [ ] Decide how model downloads are presented and stored.
- [ ] Test fresh install, upgrade, downgrade policy, and uninstall data choices.
- [ ] Implement signed updates or another secure update mechanism after evaluating cost and certificate requirements.
- [ ] Explain update and code-signing costs before choosing a paid route.
- [ ] Provide release notes and version history.
- [ ] Add optional, user-approved diagnostic export.
- [ ] Create onboarding and troubleshooting guides.
- [ ] Recruit a small opt-in beta group.
- [ ] Collect feedback without collecting private conversations by default.
- [ ] Track bugs, compatibility reports, and usability issues.
- [ ] Fix release-blocking bugs before expanding the beta.

**Exit gate:** A new user can install, configure, use offline features, update safely, and uninstall without losing data unexpectedly.

## Phase 13 — Stable Windows release

**Goal:** Release a dependable first public version.

- [ ] Freeze the release scope.
- [ ] Complete regression and acceptance testing.
- [ ] Verify offline operation from a clean installation.
- [ ] Verify model and voice downloads, integrity checks, and licence notices.
- [ ] Verify privacy controls and tool confirmations.
- [ ] Publish minimum/recommended hardware based on tests.
- [ ] Publish supported Windows versions and known limitations.
- [ ] Publish user guide, FAQ, troubleshooting, privacy policy, and licence notices.
- [ ] Publish checksums and release provenance.
- [ ] Prepare a clear issue-reporting and support process.
- [ ] Tag the release and archive its build metadata.
- [ ] Monitor reported issues and publish fixes responsibly.

**Exit gate:** The release meets its published requirements and has a repeatable build, installation, and support process.

## Phase 14 — Post-launch expansion

**Goal:** Expand carefully without undermining the offline-first product.

- [ ] Expand Linux distribution coverage and validate supported package formats.
- [ ] Expand macOS architecture coverage (Apple Silicon and Intel where feasible), signing/notarisation, and release distribution.
- [ ] Improve Urdu and additional language support based on real user testing.
- [ ] Add more model choices after licence and performance review.
- [ ] Add optional provider/BYOK integrations behind clear settings.
- [ ] Consider optional sync or remote access only with strong encryption, explicit consent, and a cost model approved in advance.
- [ ] Evaluate richer character animation and optional 3D.
- [ ] Add community-contributed character/model packs with review and provenance controls.
- [ ] Improve accessibility and localisation.
- [ ] Review support burden, maintenance costs, and sustainability.
- [ ] Revisit roadmap priorities using user feedback and measured usage.

**Exit gate:** New features preserve the product principles and do not introduce mandatory cloud dependency or undisclosed cost.

---

## 5. Hardware profile policy

These are planning profiles, not final minimum specifications. The exact model, context size, voice engines, and supported features must be chosen from benchmarks.

| Profile | Intended experience | Policy |
|---|---|---|
| 4 GB RAM | Basic local companion | Smallest models, reduced context, sequential engine loading, limited concurrent features. Must be proven before claiming support. |
| 8 GB RAM | Comfortable basic use | More model options and better multitasking, subject to CPU and storage. |
| 16 GB RAM | Recommended mainstream target | Larger compact models, smoother voice and multitasking where supported. |
| 32 GB+ RAM | Advanced use | Larger local models and richer features, depending on CPU/GPU. |

Never use model file size alone as a RAM estimate. Measure runtime memory, context cache, UI, voice engines, and operating-system overhead together.

---

## 6. Offline / online feature boundary

### Expected to work offline once required assets are installed

- Local chat and local inference.
- Local conversation history.
- Local memory and settings.
- Local speech recognition and synthesis, where the selected language/voice is installed.
- Local character/personality configuration.
- Local allow-listed desktop actions that do not depend on external services.

### Internet required or optional

- Initial app/model/voice downloads.
- Web, Google, and YouTube search.
- Optional cloud model providers and BYOK endpoints.
- Update checks and application updates.
- Optional external integrations.

The interface must make the online/offline boundary visible. Offline must not be falsely implied for features that depend on remote services.

---

## 7. Privacy, safety, and trust requirements

- Local-first is the default.
- No hidden recording or background microphone activation.
- Microphone use must be visible and controllable.
- No conversation, audio, file, or memory data leaves the device without clear disclosure and consent.
- API keys must be stored securely and never committed to Git.
- Memory must be inspectable, editable, deletable, exportable, and disableable.
- Users must be able to delete local data and understand what deletion covers.
- Model-generated instructions are untrusted input.
- Webpages, documents, and other retrieved content must not be allowed to override tool permissions.
- Sensitive actions require user confirmation.
- No arbitrary shell execution by the model.
- Character design must not impersonate a real human or conceal that the companion is AI.
- Romantic character modes must not use coercion, guilt, isolation, or dependency pressure.

---

## 8. Cost-control rules

These rules apply throughout development and release:

1. No paid API, subscription, hosted backend, domain, certificate, marketplace service, or paid asset is adopted without explicit owner approval.
2. Prefer local/free tools only after verifying their licence and maintenance status.
3. CI and hosting choices must be checked for usage limits and billing triggers.
4. The product must remain useful without optional cloud services.
5. BYOK means the user supplies and pays their chosen provider directly; explain that provider's charges clearly.
6. Any future paid distribution or support plan must be transparent and must not make basic offline functionality dependent on a subscription.
7. Maintain a cost register showing service, purpose, free limits, potential charges, and approval status.

---

## 9. Testing and quality policy

Every feature should have:
- A written requirement and acceptance criteria.
- Unit tests for logic where practical.
- Integration tests for frontend/backend and model adapters.
- Manual tests on supported hardware.
- Offline tests where relevant.
- Failure and recovery tests.
- Privacy/security review where data or tools are involved.
- Licence review before distribution.

A feature is **Done** only when implementation, tests, documentation, privacy review, and licensing review (where applicable) are complete.

---

## 10. Project governance and conversation recovery

This repository is the durable source of truth. ChatGPT conversations are working sessions, not the only record of decisions.

### Required project records

- `ROADMAP.md` — phases, tasks, priorities, status, and exit gates.
- `README.md` — current overview and links.
- `docs/PROJECT-HISTORY.md` — chronological project history and confirmed decisions.
- `docs/PRODUCT-SPECIFICATION.md` — detailed product requirements.
- `docs/ARCHITECTURE.md` — current technical architecture.
- `docs/DECISIONS.md` — dated architecture/product decision records and rationale.
- `docs/RISKS-AND-OPEN-QUESTIONS.md` — unresolved questions and risks.
- `CHANGELOG.md` — user-visible changes by release.
- `LICENSE-REVIEW.md` — dependency, model, voice, and asset licence inventory.

### Rules for maintaining continuity

- Update the roadmap whenever a task starts, changes, is blocked, or is completed.
- Record why important decisions were made, not just what was chosen.
- Mark each item as **Proposed**, **Approved**, **In progress**, **Blocked**, **Done**, or **Deferred**.
- Do not mark a task Done without evidence or a recorded test result.
- Keep a short **Current Status / Next Actions** section near the top of the roadmap.
- At the end of a major development session, write a handoff note with files changed, commit references, tests run, outcomes, unresolved issues, and next exact step.
- If a conversation is lost, start from this repository and its latest commit; do not rely on reconstructing decisions from memory.
- Preserve previous decisions in Git history. Do not silently rewrite a confirmed decision; document a superseding decision and its reason.

---

## 11. Current status and immediate next actions

**Current stage:** Phase 4 — Offline local chat engine integration. Runs #40–#42 passed Windows, Linux, and macOS. The app now has a frontend IPC path for local runtime setup, streaming, cancellation, multi-turn context, native model-file selection, bounded prompt/history validation, and a local SQLite conversation store with create/list/open/rename/delete/message-history commands. Actual model inference and persistence across real desktop restarts still require manual device testing.

**Completed:**
- [x] Product name selected: midSunday.
- [x] Create the first interactive workspace UI prototype with conversation view, local-session composer, companion selection, settings screens, responsive layout, and Rust-core status indicator. This is UI-only; no AI response, persistent storage, voice, or hardware detection is claimed.
- [x] Independent private GitHub repository created under the Friday organisation.
- [x] Cross-platform direction established: Windows, Linux, and macOS builds; Windows remains the first priority.
- [x] Configure automatic parallel Windows/Linux/macOS CI builds for pushes to `main` and pull requests targeting `main`; retain manual dispatch for package artifacts. Standard hosted runners only; automatic CI does not retain artifacts.
- [x] Offline-first, no-hidden-cost principles established.
- [x] Core companion, local voice, memory, personalities, and safe PC interaction identified as intended product areas.
- [x] Initial complete roadmap drafted.

**Next actions:**
1. Verify native local-file selection and multi-turn chat context on Windows, Linux, and macOS CI.
2. Fix platform-specific TypeScript, Rust, Tauri, plugin-permission, or packaging issues exposed by CI.
3. Generate and commit a reviewed npm lockfile, then use it for reproducible dependency installation.
4. Test the app on real Windows, Linux, and macOS devices when available; CI compilation is not a substitute for runtime testing.
5. Continue the read-only FRIDAY review and record reusable patterns without modifying FRIDAY.
6. Continue local-model, Pakistani Urdu/Roman Urdu speech, hardware-intelligence, and licensing research; no model or speech engine is selected until evidence supports it.

**Not yet done:** No AI runtime, model, persistent chat/memory storage, voice engine, hardware detector, or verified Windows installer is implemented. The current chat is a local-session UI demonstration and does not generate AI replies.

---

## 12. Change log

- **2026-11-23 — Roadmap 0.1:** Initial end-to-end product and development roadmap created from the current midSunday project decisions and technical proposals.
- **2026-11-23 — Phase 1 checkpoint:** Initial desk research documented in `docs/PHASE-1-RESEARCH.md`; no technology or asset approved and no benchmarks claimed.


### 2026-11-23 — Local runtime research update
- [x] Confirm upstream MIT licensing for llama.cpp and Ollama at repository level.
- [x] Confirm Ollama distinguishes free local model execution from separately priced cloud model usage.
- [x] Document why compact models can handle bounded text tasks but are not yet proven for nuanced companion dialogue or Pakistani Urdu.
- [x] Add Qwen3.5-0.8B as an additional compact model test candidate.
- [ ] Compare llama.cpp and Ollama with the same model on Windows.
- [ ] Run the compact-model task suite and hardware benchmarks.
- [ ] Continue Pakistani Urdu STT/TTS candidate evaluation.

This update is desk research only. It does not approve a final runtime/model or claim hardware compatibility. See `docs/PHASE-1-RESEARCH.md`.


### 2026-11-23 — Scalable hardware direction confirmed
The product owner confirmed that midSunday must not be designed only for 4 GB machines. The experience should scale upward so users with 6 GB, 8 GB, 16 GB, 32 GB, and higher-memory systems can benefit from stronger models, longer context, richer voice, and better multitasking. The 4 GB profile remains a constrained compatibility target, not the universal design ceiling.

- [x] Record multi-tier hardware direction in the decision log.
- [x] Expand hardware planning profiles to include 6 GB and 64 GB+.
- [ ] Define resource-aware Automatic / Lightweight / Balanced / High capability modes.
- [ ] Benchmark representative machines across available RAM, CPU, and GPU classes.
- [ ] Build a model catalogue with measured resource requirements and quality trade-offs.
- [ ] Verify simultaneous LLM/STT/TTS workloads at each supported profile.

See `docs/HARDWARE-COMPATIBILITY.md` and `docs/PRODUCT-SPECIFICATION.md`. All tiers remain targets until tested.


### 2026-11-23 — Hardware intelligence research added

OpenJarvis's hardware auto-detection and engine/model recommendation workflow has been verified against its current documentation and source. midSunday now has a proposed, more complete Windows-first **Hardware Intelligence & Adaptive Resource Manager (HIARM)** design, including storage checks and live resource-aware workload scheduling.

- [x] Verify OpenJarvis's documented detection/recommendation workflow and inspect its current source.
- [x] Create `docs/HARDWARE-INTELLIGENCE.md` with scope, proposed Windows detection approach, privacy rules, user controls, and acceptance criteria.
- [x] Link the requirement from README, architecture, and hardware compatibility documentation.
- [x] Record D-016 as a proposal, not an approved implementation decision.
- [ ] Compare Windows hardware-detection APIs and suitable Rust libraries.
- [ ] Define benchmark-backed model resource estimates, including context and GPU offload.
- [ ] Define safe live resource-pressure thresholds and workload scheduling behaviour.
- [ ] Validate detection and recommendation accuracy on representative Windows hardware.

This is desk research and documentation only. No detection code or benchmark has been implemented. OpenJarvis is a reference, not a midSunday dependency. The detailed design is in `docs/HARDWARE-INTELLIGENCE.md`.


### 2026-11-24 — Windows hardware detection research

- [x] Map Windows APIs for physical/available RAM, CPU topology, DXGI GPU enumeration and memory budgets, and model-storage free space.
- [x] Review `sysinfo`, Microsoft's `windows` crate, and optional NVIDIA NVML wrapper as Rust candidates.
- [x] Document why WMI `AdapterRAM` is not authoritative and why GPU inventory, VRAM capacity, process memory budget, and runtime acceleration support must remain distinct.
- [ ] Check exact dependency versions, MSRV, licences, features, and dependency tree against the eventual Rust toolchain.
- [ ] Build a read-only Windows hardware-detection prototype after Phase 1 architecture and framework decisions.
- [ ] Validate all reported values against Windows system tools on multiple hardware configurations.

This checkpoint completes initial desk research only. No crate was added and no code or hardware benchmark was run. See `docs/HARDWARE-INTELLIGENCE.md` sections 12 and 13.


### 2026-11-24 — Preliminary scalable model ladder researched

- [x] Add Qwen3.5 candidates from 0.8B through 27B to the Phase 1 research matrix.
- [x] Add Gemma 4 E2B/E4B as edge-oriented comparison candidates.
- [x] Record observed Q4 GGUF file sizes, licensing/provenance gates, and warnings that file size is not runtime RAM.
- [x] Define 4/6–8/16/32/64 GB+ benchmark buckets without turning them into compatibility promises.
- [ ] Run the fixed English, Pakistani Urdu, Roman Urdu, code-switching, companion-personality, and reasoning evaluation suite.
- [ ] Measure context, load time, first-token latency, tokens/second, peak RAM, GPU budget, and stability on real Windows PCs.
- [ ] Select model defaults only after benchmark and licence evidence.

Research is recorded in `docs/PHASE-1-RESEARCH.md` section 16. This is a desk-research shortlist; no models have been downloaded or run.


### 2026-11-24 — First managed local-runtime process milestone

- [x] Add a llama.cpp child-process supervisor using Rust's `std::process::Command`.
- [x] Use an argument array only; never invoke a shell or accept model-generated command text.
- [x] Require existing regular files for the executable and GGUF model; no automatic downloads.
- [x] Validate loopback-only host configuration before process launch.
- [x] Add explicit process status inspection, stop/wait handling, and best-effort cleanup on drop.
- [x] Add unit tests for argument construction and optional CPU thread configuration.
- [x] Verify the new module with Windows, Linux, and macOS CI (Run #24 passed).
- [ ] Add bounded startup/readiness polling and detect early process exit.
- [ ] Add a typed loopback HTTP client for health and chat-completion requests.
- [ ] Add cancellation, timeout, and crash-recovery integration tests using a controlled test server.

This is the first process-management implementation, not a working model integration. The desktop app does not yet start this supervisor, check HTTP readiness, or generate model responses. No llama.cpp binary or model has been downloaded or bundled.


### 2026-11-24 — Local runtime readiness and HTTP client milestone

- [x] Add a typed HTTP client for llama.cpp health and OpenAI-compatible chat-completion endpoints.
- [x] Restrict the client to the validated loopback IP and port; disable environment proxies and redirects.
- [x] Add bounded HTTP request timeouts and bounded startup polling.
- [x] Treat the model-loading health response as not-yet-ready and detect an exited child process.
- [x] Add mock-server tests for health, loading-to-ready transitions, early process exit, timeout, loopback validation, and chat-completion response parsing.
- [x] Verify this milestone with Windows, Linux, and macOS CI (Run #24 passed).
- [ ] Add cancellation-safe generation and explicit request cancellation.
- [ ] Add crash-recovery integration tests with a controlled child process.
- [ ] Connect the runtime coordinator to the desktop application only after lifecycle and cancellation behaviour are verified.

The HTTP client is local-only and uses the llama.cpp health and chat-completion endpoints. This remains a runtime integration layer, not an end-user working model: no server binary or model is installed, selected, downloaded, or bundled, and the desktop UI is not yet connected to inference.


### 2026-11-24 — Streaming generation and cancellation milestone

**CI correction:** Runs #27, #28, and #29 exposed a missing direct `serde_json` dependency and an invalid non-ASCII raw byte-string test. Both were corrected; Run #31 then passed all three platform jobs.

- [x] Add server-sent-event streaming for local chat completions.
- [x] Add a cloneable cancellation signal that can be triggered from another thread.
- [x] Stop delivering deltas after cancellation and drop the local HTTP response stream.
- [x] Return the runtime lifecycle to Ready after a cancelled request; mark it Failed on other communication errors.
- [x] Add mock-server tests for streaming deltas, cancellation between events, and cancellation before any request is sent.
- [x] Verify this milestone with Windows, Linux, and macOS CI (Run #31 passed all three jobs).
- [x] Add controlled child-process tests for unexpected exit detection, stop/wait cleanup, and idempotent shutdown.
- [x] Add a controlled streaming-server test proving cancellation closes the client HTTP connection.
- [ ] Verify that the actual llama.cpp server promptly stops CPU/GPU generation after client disconnect under real workloads.
- [ ] Add a managed-session restart/recovery API with explicit state and bounded restart policy.
- [ ] Connect the runtime coordinator to Tauri commands only after lifecycle and cancellation behaviour are verified.

Cancellation is verified at the client transport boundary: the test server observes the client connection close after cancellation. This does not yet prove that llama.cpp promptly stops its internal CPU/GPU computation. Controlled child-process tests cover exit detection and safe shutdown; automatic session restart is not implemented. No external service is involved, and no model or server binary is downloaded. The desktop UI remains unconnected.



### 2026-11-24 — Recovery integration verification

- [x] Add an injectable process boundary so lifecycle recovery can be tested without downloading or launching a real model.
- [x] Test successful process replacement and readiness.
- [x] Test retry after a startup failure.
- [x] Test recovery exhaustion, exact attempt count, and final Failed state.
- [x] Test that recovery is rejected while the session is Ready.
- [x] Verify the tests on Windows, Linux, and macOS CI (Run #35 passed all three jobs).
- [x] Test replacement processes that launch but never become healthy, including cleanup and final Failed state.

The test seam substitutes only the managed-process boundary. The HTTP readiness path remains real loopback TCP, while fake process handles make restart outcomes deterministic. These tests do not claim real llama.cpp compute cancellation or model inference behaviour.

### 2026-11-24 — Bounded runtime recovery implementation

- [x] Define an explicit recovery policy with a default of two attempts and a hard maximum of three.
- [x] Restrict recovery to Failed or Stopped session states; reject attempts during active startup/generation/ready states.
- [x] Stop and reap the previous child process before launching a replacement.
- [x] Reuse the validated loopback client and require the replacement server to pass bounded readiness checks.
- [x] Clean up failed replacement processes and stop after the configured attempt budget.
- [x] Add policy validation tests for defaults, zero attempts, and attempts above the hard limit.
- [x] Add controlled integration tests for successful restart, repeated startup failure, and recovery exhaustion.
- [x] Verify the recovery implementation on Windows, Linux, and macOS CI (Run #36 passed all three jobs).
- [ ] Confirm real llama.cpp cancellation and restart behaviour under representative workloads.

Recovery is explicit rather than automatic: the future coordinator must decide when a retry is appropriate and present the failure to the user if the bounded budget is exhausted. This change does not connect the runtime to the UI, install a model, or download a server binary.


### 2026-11-24 — Tauri local-runtime coordinator boundary

- [x] Add an application-owned runtime manager with one managed session at a time.
- [x] Expose serializable runtime status snapshots without exposing process internals.
- [x] Add Tauri commands for status, explicit local start, bounded generation, stop, and explicit recovery.
- [x] Keep model and executable paths user-supplied; do not download or bundle either asset.
- [x] Serialize lifecycle operations through one managed mutex so concurrent commands cannot mutate the same session simultaneously.
- [x] Verify the coordinator and Tauri command boundary on Windows, Linux, and macOS CI (Run #37 passed all three jobs).
- [ ] Connect the chat UI to these commands only after the command boundary and streaming/cancellation UX are implemented.

This is an application IPC foundation, not a user-ready model setup flow. The current frontend does not invoke these runtime commands, there is no model picker or bundled engine, and the synchronous generation command is an initial bounded integration step. Streaming UI and cross-command cancellation remain future work.


### 2026-11-24 — Responsive runtime commands and cancellation boundary

- [x] Separate runtime operation serialization from status snapshots so status reads do not wait behind model startup or generation.
- [x] Move blocking runtime work onto Tauri's blocking worker pool instead of occupying the async command executor.
- [x] Add a cancellation command that can signal an active streaming generation without waiting for the runtime operation lock.
- [x] Route manager generation through the already-tested streaming transport and collect deltas until the UI event-stream integration is implemented.
- [x] Make the runtime manager safely cloneable for worker tasks while keeping one shared process/session owner.
- [x] Add tests for status visibility during a busy operation and immediate cancellation signalling.
- [x] Verify responsive commands and cancellation on Windows, Linux, and macOS CI (Run #38 passed all three jobs).
- [x] Add frontend event delivery for streaming text and a visible Stop generating control.

Cancellation closes the local HTTP response stream. Whether the actual llama.cpp process stops its internal CPU/GPU work immediately remains server-dependent and requires real-engine testing. The frontend is still not connected to these commands; no model is bundled or downloaded.


### 2026-11-24 — Frontend runtime connection and live chat streaming

- [x] Poll the Rust runtime status from the desktop UI and display the real lifecycle state.
- [x] Add a local-engine setup panel that accepts existing llama.cpp executable and GGUF model paths.
- [x] Start and stop the local runtime through the Tauri command boundary.
- [x] Forward streaming text deltas from the Rust runtime manager to the React chat through a Tauri IPC channel.
- [x] Add a visible Stop generating action that signals the existing cancellation command.
- [x] Keep generation disabled until the local runtime reports Ready.
- [x] Preserve the no-download policy: paths must refer to files already present on the user's device.
- [ ] Verify TypeScript compilation, frontend build, Rust tests, and desktop packaging on Windows, Linux, and macOS CI.
- [ ] Test the complete UI-to-model path with a real user-installed llama.cpp server and a legally compatible GGUF model.
- [ ] Add native file/folder selection after reviewing the minimal Tauri dialog-plugin dependency and permission surface.

The frontend now has a real IPC path for local runtime setup, live generation deltas, cancellation, and lifecycle status. This does not mean a model is bundled or automatically installed. The user must provide an existing compatible server executable and GGUF model. Until a real engine/model is tested, end-to-end inference remains unverified. Conversation persistence, memory, voice, and hardware detection remain separate future milestones.


### 2026-11-24 — Multi-turn chat context and native local file selection

- [x] Add typed user/assistant conversation-history messages to the generation request.
- [x] Validate history count, individual message size, empty content, and aggregate history size before sending requests.
- [x] Preserve chronological message order and append the current user prompt for both streaming and non-streaming llama.cpp requests.
- [x] Add unit tests for conversation message ordering and history validation.
- [x] Add native single-file selection for the llama-server executable and GGUF model through the Tauri dialog plugin.
- [x] Add only the dialog open permission; no filesystem-wide frontend permission is granted.
- [x] Cancel active generation when starting a new conversation so a previous stream cannot continue filling a cleared chat.
- [ ] Verify TypeScript, Rust tests, and desktop packaging on Windows, Linux, and macOS CI (Run #40).
- [ ] Test real multi-turn inference with a local llama.cpp server and model; no model or server is bundled.
- [ ] Review exact plugin/dependency licences and generate a committed npm lockfile before distribution.

The chat now sends previous visible user/assistant turns with each new prompt, bounded to 64 prior messages and 128 KiB of prior message content. This is session-only context, not saved to disk. The native picker only returns selected paths; Rust still validates that the executable and model exist. No model download, cloud service, or paid dependency has been added.


### 2026-11-24 — Bounded conversation-context hardening

- [x] Keep the newest eligible user/assistant turns while preserving chronological order.
- [x] Skip empty assistant placeholders and system messages before building request history.
- [x] Enforce the backend's 64-message, 32 KiB-per-message, and 128 KiB-total history limits in the frontend.
- [x] Add a 32 KiB UTF-8 limit for the current prompt in both frontend and Rust validation.
- [x] Add a Rust regression test for oversized prompts.
- [ ] Verify TypeScript, Rust tests, and desktop packaging on Windows, Linux, and macOS CI.

The frontend now trims history to the newest messages that fit the same byte limits enforced by Rust, preventing long sessions from failing simply because the visible transcript grows. Empty streaming placeholders are excluded. The current prompt is separately capped at 32 KiB of UTF-8 content. This does not persist chat history or change the session-only privacy behaviour.


### 2026-11-24 — Local conversation persistence implementation

- [x] Add a SQLite-backed local conversation store under the operating system's application-data directory.
- [x] Create conversation and message tables with foreign-key cascade deletion and ordered message retrieval.
- [x] Add Rust commands for listing, creating, reading, appending, renaming, and deleting conversations.
- [x] Validate message roles, non-empty content, 32 KiB message size, and conversation titles.
- [x] Add Rust tests for create/list/rename, ordered message storage, invalid message rejection, and delete cascade.
- [x] Load the most recently updated conversation at app startup and show saved conversations in the sidebar.
- [x] Save user messages before generation and assistant replies after generation completes or stops.
- [x] Add local conversation open, rename, and delete controls.
- [ ] Verify SQLite persistence and the full workflow on real Windows, Linux, and macOS devices.
- [ ] Review bundled SQLite dependency licence and migration/backup strategy before release.

Conversation history is separate from the future personal-memory system. It remains on-device in the app data directory; no cloud service or account is involved. Deleting a conversation removes its messages through the database foreign-key cascade. A real-device restart and data-recovery test is still required before calling persistence production-ready.


### 2026-11-24 — Local GGUF integrity and launch preflight

- [x] Reject files with a `gguf` extension but without the GGUF magic header.
- [x] Verify registered model files against their saved size and SHA-256 fingerprint on demand.
- [x] Recheck the selected file before starting the local engine; registered files must match their catalogue fingerprint.
- [x] Permit an unregistered existing GGUF file to be selected, while clearly reporting that no saved fingerprint is available.
- [x] Add regression tests for invalid headers, changed files, missing files, and unregistered model preflight.
- [ ] Verify all changes on Windows, Linux, and macOS CI.

The check is local and reads the selected model file; it does not upload, copy, or download model data. A matching fingerprint confirms that the file matches the local catalogue record, not that the model is safe, correctly licensed, or trustworthy.


### 2026-11-24 — Functional companion styles and local system prompts

- [x] Turn Best friend, Mentor, and Professional selections into actual local model instructions.
- [x] Add user-authored custom style notes for the Custom character option.
- [x] Add Girlfriend and Boyfriend built-in styles with respectful, non-explicit, non-coercive behaviour boundaries.
- [x] Persist selected companion, custom style notes, and up to 20 editable named character profiles in the local app-data directory.
- [x] Validate companion settings versions, selected profile references, unique names/IDs, and instruction size limits on the Rust side.
- [x] Send system instructions as a system-role message before conversation history.
- [x] Enforce an 8 KiB UTF-8 system-prompt limit in the Rust request validator.
- [x] Keep system prompts out of the saved conversation transcript and add tests for validation and message ordering.
- [ ] Persist companion preferences and custom style notes across app restarts; current custom notes last only while the app remains open.

The selected companion style now affects responses generated by the local model. System instructions are sent only to the configured loopback llama.cpp endpoint and are not stored as conversation messages. Custom style notes are session-only in this milestone.


### 2026-11-24 — Explicit local memory foundation
- [x] Add a schema-version-2 SQLite migration that preserves existing conversation data and creates separate saved-memory and app-preference tables.
- [x] Add explicit memory create, list, edit, individual delete, clear-all, enabled/disabled preference, and versioned JSON export commands.
- [x] Start memory disabled by default; include up to 20 recent user-entered memories (maximum 4 KiB) in local model context only when enabled.
- [x] Keep automatic memory extraction out of scope; only details the user explicitly saves are stored.
- [x] Keep memory separate from chat history and make clearing one leave the other untouched.
- [x] Add validation and Rust regression tests for migration, limits, CRUD, export, and persisted enable/disable preference.

This milestone does not implement automatic memory extraction, memory import/restore, semantic retrieval, cloud memory, or verified real-device crash recovery. Memory is sent only to the configured local inference endpoint when the user enables it.


### 2026-11-24 — Local user profile and memory settings clarity
- [x] Add a local preferred-name profile with an explicit opt-in for natural name use in conversations.
- [x] Persist the preferred name and name-use preference in the existing local SQLite app-preferences table.
- [x] Validate and trim preferred names on the Rust side; keep the field optional and allow nicknames.
- [x] Explain the difference between the 100-entry storage cap and the 20-entry / 4 KiB per-prompt context budget.
- [x] Clarify that saved memories are not automatically deleted when storage is full and are not included in prompts while memory is disabled.
- [x] Verify profile persistence and validation with Rust regression tests.

The profile is optional and device-local. midSunday uses the preferred name naturally and sparingly only when the user enables that preference. The memory cap stores up to 100 separate entries (up to 2,000 characters each); the prompt builder currently selects up to 20 recently updated entries within 4 KiB. Memory import/restore and automatic memory extraction remain unimplemented.


### 2026-11-25 — Awareness sensitive-data screening hardening

- [x] Expand Rust-side Awareness screening for credential labels, common API-token prefixes, email addresses, phone-like numeric sequences, long numeric identifiers, payment details, private-message labels, home-address labels, and selected identity-document labels.
- [x] Keep screening in the backend validation path so UI changes cannot bypass it.
- [x] Add regression coverage for international phone formats, tagged email addresses, API secrets, address labels, identity-document labels, and date-of-birth labels.
- [x] Keep ordinary harmless Awareness facts accepted and preserve the existing item size/category validation.
- [ ] Independently evaluate broader multilingual and context-aware sensitive-data detection before any automatic learning is enabled.

This is deliberately conservative screening, not comprehensive detection. It can reject some harmless numeric facts and cannot guarantee detection of every secret, private message, address, or sensitive detail. Automatic learning remains disabled and unimplemented; users should never enter secrets or private information into Awareness.

### 2026-11-24 — Awareness source provenance and relearning exclusions

- [x] Upgrade the local SQLite database from schema v4 to v5 without changing existing conversations, saved memories, Awareness items, or history.
- [x] Expose source-record provenance on Awareness items; manually entered facts are explicitly labelled as user-entered, while future conversation/search records can carry their source record ID.
- [x] When an Awareness item is deleted, save a SHA-256 content fingerprint and metadata-only source/category/timestamp exclusion so matching information can be blocked from future automatic ingestion.
- [x] When an item is rewritten or all items are cleared, suppress the replaced/cleared content fingerprints as well.
- [x] Provide local review and clear controls for relearning exclusions; clearing exclusions never restores deleted items.
- [x] Add a reusable backend check for future learning pipelines to test whether a candidate fact is excluded before ingestion.
- [x] Keep original deleted text out of the exclusion table; it stores only a fingerprint and limited metadata.
- [x] Keep deliberate user-entered items allowed even when a matching automatic-learning exclusion exists.
- [x] Add regression tests for delete/edit/clear suppression, normalised matching, deliberate re-entry, and absence of raw content in the exclusion schema.
- [ ] Add an explicit source-evidence review/deletion model for conversation/search-derived learning before automatic ingestion is implemented.

The exclusion mechanism is a safeguard foundation, not an active learning pipeline. Automatic learning remains disabled and unimplemented. The fingerprint is not the original text, but it is still local metadata and should be treated as private. The future ingestion path must call the suppression check before writing any automatically learned item.

### 2026-11-24 — Awareness activity history and provenance

- [x] Upgrade the local database from schema v3 to v4 without altering existing Awareness items, saved memories, or conversations.
- [x] Record manual Awareness add/edit/delete/clear/export events in a separate metadata-only history.
- [x] Show recent history in Settings → Awareness, including event, category, item reference, and local timestamp.
- [x] Let users clear history independently without deleting their Awareness items.
- [x] Keep item text and raw source evidence out of the history table; retain at most 500 recent metadata events in the UI query.
- [ ] Add provenance/source-evidence controls and deletion/relearning suppression before automatic ingestion is implemented.

### 2026-11-24 — Awareness item management and first privacy screening

- [x] Add local Awareness item list, deliberate user-entry, category assignment, edit, individual delete, clear-all, and versioned JSON export commands.
- [x] Keep Awareness records separate from saved memories and conversation history; clearing Awareness items leaves both other stores untouched.
- [x] Record provenance as user-entered, knowledge type as observed, and confidence as explicit user-provided certainty for manually added items.
- [x] Add conservative Rust-side rejection checks for common credential labels, token formats, email addresses, and long numeric identifiers before an Awareness item is saved.
- [x] Add Settings UI to review, categorise, rewrite, delete, clear, and export every Awareness item.
- [ ] Expand and independently test sensitive-data detection, including international identifiers and context-aware private-data handling; deterministic checks are not a complete guarantee.
- [x] Add a metadata-only Awareness activity history for manual add/edit/delete/clear/export actions, with local review and a separate clear-history control.
- [x] Keep history free of Awareness item text, raw conversations, raw searches, credentials, and private source content; retain only event type, item ID, category, and timestamp.
- [ ] Add source-evidence review and deletion/relearning suppression records before automatic ingestion is implemented.
- [ ] Add automatic learning from conversations/searches only after all privacy filters, category exclusions, source controls, and review/deletion safeguards are tested.

This batch supports only deliberate user-entered Awareness items. It does not read conversations, search activity, browser history, or other applications. The sensitive-data checks are conservative first-pass safeguards, not a promise that every possible secret or private detail can be detected; users should not enter secrets. Automatic learning remains disabled and unimplemented.

### 2026-11-24 — Awareness foundation and explicit learning preferences

- [x] Upgrade the local SQLite database from schema v2 to v3 without deleting conversations or saved memories.
- [x] Add separate local tables for future Awareness items and metadata-only learning activity history; the activity table intentionally has no raw-content column.
- [x] Add persisted Awareness settings for the master switch, conversation learning, and in-app search learning. All are OFF by default; source preferences remain independently saved while the master switch is paused.
- [x] Add a Settings → Awareness panel with individual switches and an explicit notice that automatic learning is not active in this build.
- [x] Keep the current implementation collection-free: the switches save preferences only and do not read or ingest conversations or searches.
- [ ] Implement automatic learning only after secret/private-data filtering, source provenance, deletion/relearning safeguards, and user-visible review are in place.

This is a storage/settings foundation only. No password, credential, private message, or search content is collected by the new tables or controls. The future learning pipeline must enforce the binding Awareness privacy contract before any ingestion is enabled.
