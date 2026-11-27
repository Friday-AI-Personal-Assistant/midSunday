# midSunday — Complete Product & Development Roadmap

> **Product:** midSunday  
> **Descriptor:** AI Personal Companion  
> **Roadmap version:** 0.1  
> **Status:** Foundation / planning  
> **Repository:** `Friday-AI-Personal-Assistant/midSunday`  
> **Last updated:** 2026-11-27  
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
- Windows compatibility target: Windows 10 and Windows 11. The initial supported Windows 10 baseline will be Windows 10 version 22H2, with earlier Windows 10 builds investigated and supported only when the selected Tauri/WebView2/runtime stack passes compatibility testing. Windows 11 supported releases will be tested as first-class targets. Windows 7 and 8/8.1 are not official targets.
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
- [x] Implement the first cross-platform local inventory snapshot for OS/version/architecture, CPU brand and core counts, installed/available RAM, and storage-volume capacity; expose manual refresh in Settings.
- [ ] Add DXGI per-process GPU memory-budget/usage detection and validate GPU/CPU inventory against real Windows hardware; the initial adapter inventory and CPU instruction-set probe are now implemented.
- [ ] Research and design the adaptive resource manager's model recommendations and safe workload scheduling.
- [x] Compare OpenJarvis hardware auto-detection/recommendation with the proposed midSunday design; do not assume its setup-time heuristic is sufficient for runtime adaptation.
- [ ] Review all candidate licences, model cards, voice licences, and redistribution terms.
- [ ] Record benchmark machines, test scripts, model versions, and results.

**Exit gate:** The stack and default model/voice choices are selected based on repeatable tests, documented licences, and realistic hardware profiles. Unverified claims remain clearly labelled as targets.

**Benchmark protocol status (2026-11-24):** The repeatable protocol is documented in [BENCHMARK-PROTOCOL.md](docs/BENCHMARK-PROTOCOL.md), and a synthetic text seed set is drafted in [BENCHMARK-DATASET-V1.md](docs/BENCHMARK-DATASET-V1.md). Fluent Urdu review, speech audio sets, holdout cases, and dataset freeze remain pending; no benchmark has been executed.

### 2026-11-25 — First local hardware inventory prototype

- Added a read-only, on-demand hardware snapshot for OS/version/architecture, CPU model and core counts, installed/available RAM, and storage-volume total/free capacity.
- Added an on-demand, privacy-limited snapshot of midSunday's own resident and virtual memory; no other processes are enumerated or exposed.
- Added a Settings → System & hardware panel with manual refresh and clear disclosure that GPU detection and adaptive recommendations are not included yet.
- Kept hardware inspection local; the snapshot excludes process lists, host names, network interfaces, serial numbers, and user-file contents.
- Added Rust serialization/privacy regression tests and moved hardware enumeration to a blocking worker so it does not freeze the interface.

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

### Appearance & theme system — approved visual direction (planned)

The product owner approved five optional themes. **Sunday Green is the default signature identity**; the other four are equally available to every user. Theme choice is personal preference and must never be gender-locked or associated with gender stereotypes.

#### Approved theme identities and proposed palette starting points

These HEX values are initial design tokens, not final locked colours. Refine them during visual design and verify accessible contrast for every foreground/background pairing.

| Theme | Visual direction | Primary | Deep / strong | Accent | Light surface | Dark surface | Main text |
|---|---|---|---|---|---|---|---|
| **Sunday Green** (default) | Calm, modern, professional; the midSunday signature | `#176B45` | `#104B35` | `#36A878` | `#F4F8F5` | `#111B16` | `#17211B` |
| **Midnight Blue** | Deep navy, subtle blue highlights, premium and technology-inspired | `#315B9A` | `#14243D` | `#65A6F2` | `#F3F6FB` | `#101827` | `#172235` |
| **Lavender Dream** | Soft lavender, gentle purple and light backgrounds; elegant and relaxing | `#8064B5` | `#58427F` | `#B39AE8` | `#F8F5FC` | `#1C1726` | `#282133` |
| **Rose Blossom** | Blush pink, rose accents and warm neutral surfaces; soft and welcoming | `#C05A7A` | `#873B57` | `#E58AA5` | `#FFF6F8` | `#24171D` | `#302128` |
| **Warm Sand** | Cream, beige and subtle brown; peaceful, warm and minimal | `#94734E` | `#655039` | `#C4A77D` | `#FAF7F0` | `#201C16` | `#2C271F` |

#### Required behaviour and design rules

- [ ] Establish **Sunday Green** as the first complete design system: navigation/sidebar, chat workspace, typography, buttons, cards, dialogs, settings, notifications, empty/loading/error states, and focus states.
- [ ] Build the remaining four themes on the same shared component system and semantic design tokens; they must feel like five expressions of one product, not five separate applications.
- [ ] Add **Settings → Appearance & Themes** with all five named theme choices and clear visual previews/swatches.
- [ ] Support instant live preview before the user applies or keeps a theme; provide an obvious way to revert/cancel.
- [ ] Provide light and dark appearances for each theme, refining each palette for both modes rather than merely inverting colours.
- [ ] Include **Follow System / System Default** so the app follows the operating system's light/dark preference, plus a manual light/dark override.
- [ ] Persist the selected theme and appearance mode locally on the device and restore them after restart. No account, cloud service, or network connection is required.
- [ ] Apply theme tokens consistently throughout every screen and shared component; avoid scattered hard-coded colours that make themes incomplete.
- [ ] Maintain readable text, accessible contrast, visible keyboard focus, usable control states, and clear status/error meanings in every theme and mode. Colour must not be the only way to communicate status.
- [ ] Theme switching must not change application functionality, user data, model behaviour, privacy settings, or performance in a material way.
- [ ] Keep theme selection entirely local; do not transmit the user's appearance preference.
- [ ] Consider future custom accent colours and user-created themes after the five built-in themes are stable.

#### Implementation guidance

Use semantic tokens (for example, primary, primary-strong, accent, surface, elevated-surface, text, muted-text, border, focus, success, warning, and danger) rather than tying components directly to a theme's raw HEX values. A theme should supply token values for light and dark modes. The proposed HEX palette above is a starting reference for design work; contrast testing and refinement take precedence over preserving a particular value.

**Status:** Theme names, overall visual directions, default theme, and equal availability are approved product decisions. Palette HEX values are proposals. Layout, final colour contrast, previews, persistence, system-following behaviour, and implementation remain planned work; recording this specification does not mean the theme feature has been implemented.

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
- [x] Implement explicit user-approved memory creation and editing rules with 2,000-character / 4 KiB per-item limits. A temporary 100-item cap was later removed.
- [x] Let users inspect, correct, and delete individual memories, or clear all memories separately.
- [x] Let users disable memory; it is disabled by default and remains stored but is excluded from prompts while disabled.
- [x] Keep conversation-history deletion independent from saved memory deletion.
- [x] Export saved memories as a separate versioned JSON file.
- [x] Add conversation-history export (versioned JSON); full database backup and restore remain pending.
- [x] Add validated, version-1 saved-memory JSON import with transactional merge, exact-duplicate skipping, no overwrites, and regression tests. This is a convenience import, not encrypted Backup & Restore; full database restore tests remain pending.
- [ ] Define retention and cleanup behaviour.
- [ ] Keep memory local by default.
- [ ] Require explicit consent before any cloud provider receives memory or chat context.
- [ ] Test data integrity after crashes, upgrades, and interrupted writes.
- [x] Add the first Awareness foundation migration (SQLite schema v3), preserving conversations and explicit saved memories from schema v2.
- [x] Persist Awareness master, conversation-learning, and in-app-search-learning preferences independently; all default OFF and remain user-controlled.
- [x] Add an Awareness settings panel that clearly states automatic learning is not active yet, so saved preferences cannot be mistaken for active collection.
- [ ] Evolve explicit saved memory into a dedicated **Awareness** system; preserve existing user-created memories during migration.
- [x] Remove the temporary fixed 100-item cap from explicitly saved memories; retain per-entry validation and the separate bounded prompt-context budget. Saved entries remain until user deletion, with device storage as the practical limit; no automatic eviction is performed. Storage-usage indicators and low-space warnings remain future work.
- [x] Add a local Awareness suggestion review queue: screened candidates remain pending until the user approves them; rejection stores only a one-way fingerprint and metadata to prevent relearning.
- [x] Connect a user-triggered, local-model-only conversation analysis pipeline to the guarded suggestion queue. The user chooses the open conversation and explicitly starts analysis; candidates are source-linked, screened, deduplicated, and remain pending until approval. No background analysis, automatic collection, in-app-search extraction, or cloud transfer is active.
- [ ] Make every learning source, feature, category, and optional capability independently controllable with clear on/off settings; provide pause/resume controls without forcing deletion.
- [ ] Keep Awareness disabled until the user explicitly enables it; do not silently enable new learning sources after an update.
- [ ] Store learned knowledge and its provenance locally, including when it was learned, its source, whether it is observed or inferred, and an appropriate confidence indicator.
- [ ] Make every learned Awareness item visible and inspectable in Settings → Awareness, with edit/rewrite, individual delete, category controls, export, and clear-all actions.
- [ ] Add an inspectable Activity & Learning History so users can understand which permitted interactions contributed to an awareness item.
- [ ] Ensure deleting or rejecting an item also prevents the same evidence from immediately recreating it; support a user-controlled "do not learn this again" / exclusion mechanism.
- [x] Retrieve only relevant local Awareness context for a response; a separate OFF-by-default chat-use switch gates local keyword matching, and the prompt receives at most 12 relevant approved items / 2 KiB. Entries are explicitly framed as untrusted reference data, never instructions or permissions. This is an initial bounded retrieval implementation; indexed semantic retrieval remains future work.
- [ ] Treat all stored awareness content as untrusted user data, never as system instructions or permission grants.
- [ ] Initially limit automatic learning to interactions that happen inside midSunday. Any future browser, operating-system, or third-party-app source must be a separately named, explicitly permissioned, independently switchable integration.
- [ ] Never collect or store passwords, authentication codes, tokens, API keys, private keys, payment-card/banking credentials, or other secrets. Do not capture keystrokes, password fields, or clipboard contents as a background learning source.
- [ ] Do not monitor or ingest private messages, email contents, files, microphone audio, or activity in other applications unless a future narrowly scoped feature is explicitly requested, separately permissioned, and independently switchable. Raw voice recordings must not be retained by default.
- [x] Add independently switchable Awareness category exclusions, enforced in Rust at the guarded automatic-ingestion boundary; manual user-entered items remain an explicit user action.
- [ ] Avoid automatically inferring or retaining sensitive personal attributes beyond user-approved, non-excluded categories; when in doubt, do not learn it.
- [ ] Keep all Awareness data on-device by default. No background upload, telemetry of personal content, cloud memory, or external model transfer; any future cloud use requires separate informed consent and a clear data-transfer explanation.
- [ ] Provide user-controlled export, backup/restore, deletion, and retention settings. Saved-memory JSON export/import is implemented as a convenience merge; the unified encrypted Backup & Restore remains planned. Deletion must cover source evidence and derived awareness where applicable, including safeguards against accidental re-learning.
- [ ] Build a unified **Backup & Restore** feature for user-selected personal data, while keeping all data local by default and making cloud backup entirely optional.
- [ ] Support two clearly labelled backup formats: (a) a simple, portable local backup with a prominent warning that it is not protected by a separate backup password and should be stored securely; (b) a password-protected, authenticated-encryption backup that is encrypted on-device before it is written or uploaded.
- [ ] Make password-protected encrypted backup the recommended/default choice in the Backup & Restore experience, while keeping simple unencrypted backup available as an explicitly chosen convenience option with a clear security warning.
- [ ] Never silently downgrade an encrypted backup to an unencrypted backup. If encryption, password access, key derivation, or secure archive creation fails, stop the operation and explain the failure; require the user to make a separate, explicit choice before creating any simple unencrypted backup. Preserve the selected protection mode for scheduled backups and fail safely if it cannot be honoured.
- [ ] Require encryption for every backup sent to a cloud provider. Never upload the simple unencrypted backup format to Google Drive or another cloud destination.
- [ ] Let users choose backup contents independently: saved memories, Awareness items and provenance, pending Learning Review suggestions, user profile, companion/personality preferences, application settings, and optionally conversation history. Exclude passwords, API keys, tokens, credentials, model binaries, caches, and diagnostic logs by default; never include secrets in a backup.
- [ ] Provide manual backup plus optional automatic daily and weekly schedules. Scheduling is OFF by default, user-configurable, visible, pausable, and must not run before the user chooses a destination and backup protection mode.
- [ ] Add optional Google Drive as a storage destination, not as an application account or runtime dependency. Google must receive no access to midSunday's local application, conversations, model runtime, filesystem, or controls; the Google integration may only perform the narrowly authorised Drive file operations explicitly requested by the user.
- [ ] Support connecting and disconnecting optional cloud backup providers (starting with Google Drive) from Backup & Restore. A user must be able to upload a protected backup on one computer, connect the same provider on a newly installed midSunday desktop, download and restore that backup, and then disconnect the provider.
- [ ] On disconnect, revoke and remove midSunday's locally held provider authorisation tokens/credentials and stop all future provider access. Disconnecting must not delete remote backup files; make remote backup deletion a separate, clearly confirmed action, and allow users to manage/delete files directly with the provider.
- [ ] Provide a clear cloud-backup inventory so users can identify the backup files midSunday created, choose individual files for deletion, and understand that deleting a cloud copy does not delete local data or other copies.
- [ ] Provide encrypted-backup password management in Backup & Restore settings. Users may change the default password used for future encrypted backups after proving access with the current password or the independently held recovery key. If the password is forgotten, the recovery key may authorise setting a new default password; it must not silently recover or rewrite old archives.
- [ ] Changing the default encrypted-backup password applies to future backups only. Existing backup archives remain encrypted with their original password/recovery key and must continue to be restorable with those original credentials. Clearly warn users before any separately requested re-encryption/rotation of existing archives; never imply that changing the default password updates old files.
- [ ] If a user loses both the password and recovery key for an existing encrypted backup, neither midSunday nor the cloud provider can recover it. Do not offer a password reset that bypasses authenticated encryption or falsely suggests that changing the default password unlocks old archives.
- [ ] Use the narrowest feasible Google OAuth/Drive scope (prefer app-created or user-selected backup files only); do not request broad access to the user's entire Drive, Gmail, Contacts, Photos, or other Google services. Do not require Google sign-in to install, launch, or use midSunday.
- [ ] Keep Google integration isolated in an optional adapter. Google receives only the encrypted backup archive and unavoidable file metadata (such as filename, size, timestamps, and account/storage metadata); it never receives the backup password or encryption key. Explain that Google is a third-party provider and that cloud storage is not risk-free.
- [ ] Use a documented, versioned backup archive with a manifest, included-data inventory, schema/app compatibility information, integrity checks, and migration support so future midSunday versions can restore older backups safely.
- [ ] Implement restore preview, integrity/authentication verification, compatibility checks, and a safe merge/replace choice. Never silently overwrite newer local data; preserve rollback/recovery if restore fails.
- [ ] Provide backup history/status, last-success timestamp, failure notices, retention controls, delete-backup and disconnect-provider controls, and a way to verify a backup before relying on it.
- [ ] Explain password/recovery-key loss clearly: password-protected backups cannot be recovered by midSunday or Google if the user loses the only decryption secret. Never transmit or store the backup password/recovery key with Google.
- [ ] Test interrupted backups, corrupted/tampered archives, wrong passwords, disk-full conditions, offline operation, duplicate restores, schema upgrades, and restore across Windows, Linux, and macOS.
- [ ] Investigate encryption at rest using operating-system-protected keys; do not claim local data is encrypted until implemented and tested.
- [x] Add a versioned SQLite FTS5 index for saved memories, maintained by insert/update/delete triggers and rebuilt from existing records during migration.
- [x] Add local all-terms memory search with bounded query parsing, relevance ordering, 25-result pagination, and tests for index lifecycle and query-syntax safety.
- [ ] Extend indexed retrieval to Awareness and conversation history; embeddings remain optional and must be locally processed by default.

**Awareness privacy and control contract (binding product requirements):**
1. No passwords, credentials, authentication secrets, payment details, or private data are to be collected or stored as Awareness.
2. Every optional setting and learning source must have a clear, user-accessible on/off control. A master pause switch must also be available.
3. Learning requires explicit opt-in. Conversations and in-app searches have separate switches; other sources are off unless separately authorised.
4. Users can inspect, correct, rewrite, export, and delete what midSunday has learned. They can clear all Awareness data and independently manage source-history retention.
5. Awareness remains local by default, with no hidden cloud transfer or covert monitoring.
6. "No fixed item-count limit" does not mean infinite storage; actual capacity is bounded by the user's device. midSunday must explain storage use and must not silently delete records.
7. The system must distinguish observed facts from inferences, show provenance, and avoid treating learned text as trusted instructions.
8. Backups are opt-in; local storage remains the default. Password-protected encrypted backups are the recommended option; simple unencrypted backups remain available only as an explicit convenience choice, are local-only, and carry a clear warning. Cloud backups must always be encrypted on-device before upload.
9. Never silently downgrade an encrypted backup to an unencrypted backup. If the selected protection mode cannot be honoured, fail closed and ask the user for a separate explicit decision before creating a simple backup; scheduled backups must never change protection mode automatically.
10. Google Drive is only an optional backup-file destination. Google must not receive access to midSunday itself, its local runtime, conversations, files, or controls; use the narrowest feasible Drive-only authorisation and disclose unavoidable backup-file metadata.
11. Backup passwords and recovery keys never leave the device. Users control backup contents, schedules, destinations, retention, deletion, and restoration.
12. Users can disconnect a cloud provider after migrating/restoring on another desktop. Disconnect revokes local authorisation but never silently deletes cloud files; remote deletion is a separate explicit action.
13. The encrypted-backup default password is changeable from settings only after authentication with the current password or recovery key. A changed default protects future backups; old archives retain their original credentials unless the user separately requests a clearly explained re-encryption.

**Exit gate:** Users understand what is stored, can control every learning source and setting, and can inspect/export/delete learned knowledge and relevant source evidence without hidden copies or unexpected cloud transfer. The initial Awareness schema, opt-in preferences, category exclusions, Rust-enforced guarded ingestion boundary, pending-review queue, and explicitly user-triggered local conversation analysis are implemented. No background extraction, automatic activity collection, in-app-search extraction, or cloud transfer is active. Scalable retrieval, import/restore, retention controls, and real desktop crash/restart verification remain open.

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
- [ ] Test Windows 10 version 22H2 as the initial Windows 10 baseline and test current supported Windows 11 releases; investigate earlier Windows 10 builds before claiming support for them.
- [ ] Verify the required Microsoft Edge WebView2 Runtime is present, healthy, and compatible; provide a clear installation/repair path when it is missing or damaged.
- [ ] Verify Windows installer prerequisites and native dependencies before application launch where possible; show a useful installer or Windows-native error if the app cannot start because a prerequisite is missing.
- [ ] Test Windows scaling, multiple displays, and accessibility settings.
- [ ] Publish a compatibility matrix based on actual results.
 
### Diagnostics, health checks, and user-facing error reporting

- [ ] Run a startup health check for required application files, runtime components, database/schema state, model registration and file integrity, permissions, and available disk space.
- [ ] Monitor important runtime conditions and detect recoverable failures such as model-engine startup errors, missing/corrupt model files, database failures, microphone/speaker problems, and optional network-feature failures.
- [ ] Capture application crashes and Rust panics safely, and use frontend error boundaries so one broken screen does not silently break the whole interface.
- [ ] Present clear native-style dialogs that explain what happened, its severity, what the user can do next, and whether retry or a safe repair is available.
- [ ] Distinguish informational messages, warnings, and critical errors; never label a recoverable issue as a fatal app failure.
- [ ] Add Settings → Diagnostics with health status, recent error details, log location, and guided checks/repair actions.
- [ ] Keep diagnostic logs on the device by default; exclude conversation content, memories, passwords, API keys, and private files.
- [ ] Allow users to review and explicitly export diagnostic reports; never upload logs or enable telemetry automatically.
- [ ] Provide a limited pre-launch/installer prerequisite check for failures that prevent the main application from starting at all.
- [ ] Test missing DLL/runtime, damaged app files, database migration failures, low disk space/RAM, model corruption, permission denial, device loss, and unexpected shutdown scenarios.
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
- [ ] Create Windows installer and uninstall flow with clear prerequisite checks and recovery guidance for missing or damaged runtime components.
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
- Backup & Restore is optional; no Google account or cloud service is required for core midSunday use.
- Google Drive integration must be isolated from the midSunday application/runtime and limited to the narrowest feasible Drive backup-file permissions; no broad Google-account scopes or access to Gmail, Contacts, Photos, or other services.
- Cloud backups must be encrypted locally before upload; never send backup passwords/recovery keys to Google. Disclose that the provider can still observe necessary file metadata and that third-party storage carries residual risk.
- Simple unencrypted backup files are local-only, clearly labelled, and accompanied by a storage-security warning.
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
- [ ] Define user-selectable resource-aware Automatic / Lightweight / Balanced / High capability modes.
- [x] Implement a first non-binding capability assessment from installed RAM and CPU topology, including a separate current-memory-pressure warning; keep model compatibility and automatic changes out of scope until GPU/runtime data and benchmarks exist.
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


### 2026-11-25 — Awareness source lifecycle and deletion integrity

- [x] Upgrade the local SQLite schema from version 6 to version 7 and add indexes for source-linked Awareness items and pending suggestions.
- [x] Treat conversation-derived Awareness items and pending suggestions as dependent on their source messages.
- [x] When a user deletes one conversation, remove its derived Awareness items and pending suggestions in the same database transaction.
- [x] When a user clears all conversations, remove all conversation-derived Awareness items and pending suggestions in the same transaction.
- [x] Record metadata-only deletion events for Awareness items removed with their source conversation; never copy source text into the history.
- [x] Add regression tests for individual conversation deletion and clear-all cleanup.
- [x] Add a user-facing, read-only source-evidence viewer for pending conversation-derived suggestions; it retrieves only the original user-authored message and fails closed if it no longer exists.
- [x] Add source-evidence viewing to saved conversation-derived Awareness items as well as pending suggestions. The viewer returns only the original user-authored message and fails closed when unavailable.
- [x] Add an explicit source-linked deletion action for saved conversation-derived Awareness items. The Rust layer verifies the exact item/message relationship and source existence, then atomically deletes only the saved fact, records metadata history, and adds a one-way relearning suppression fingerprint; the original conversation remains unchanged.
- [x] Keep the saved item's evidence panel read-only for original message content while offering a separate, clearly confirmed action to remove the linked saved fact and block matching relearning.
- [ ] Add equivalent source lifecycle handling for in-app search evidence when a local search-history store is implemented.

Conversation-derived Awareness must not outlive the conversation messages that support it. Deleting a source conversation now removes its linked learned items and pending suggestions atomically, while preserving unrelated manually entered Awareness items, saved memories, and other conversations. This is a lifecycle safeguard, not an active learning feature; automatic extraction remains disabled and unimplemented.


## Awareness learning safety and review integrity
- Conversation-derived Awareness candidates and learned items must reference a real, existing local message ID; reject missing, stale, or fabricated source references.
- Suggestion approval must be one atomic SQLite transaction: recheck source existence, master/source/category settings, sensitive-data screening and suppression; insert the permanent item and metadata history; remove the pending suggestion only when every step succeeds.
- Suggestion rejection must atomically record a one-way suppression fingerprint and remove the pending suggestion.
- Failed review actions must leave the pending queue and permanent Awareness state consistent.
- Manual user-entered Awareness remains independent of conversation source references and automatic-learning switches.
- Automatic extraction remains inactive until separately implemented and explicitly opted in.

### 2026-11-25 — Windows 10/11 compatibility and diagnostics direction

- Confirmed Windows 10 and Windows 11 as the intended Windows support targets, while retaining one shared cross-platform application codebase.
- Set Windows 10 version 22H2 as the initial compatibility baseline; earlier Windows 10 builds must be evaluated against the actual Tauri, WebView2, and runtime dependencies before being advertised as supported.
- Added requirements for startup health checks, runtime/crash detection, clear user-facing error dialogs, a Diagnostics centre, local-by-default logs, and user-approved diagnostic export only.
- Added installer/pre-launch prerequisite checks for failures such as missing or damaged WebView2/runtime components that could prevent midSunday itself from opening.
- Windows 7 and Windows 8/8.1 are not official support targets.

---

# INTERNAL / CONFIDENTIAL — Commercial licensing and delivery plan

> This section is an internal planning record, not public product documentation. Revisit and remove or replace it before the public release. Commercial terms, prices, update periods, payment providers, and infrastructure remain subject to final legal, technical, and cost review.

## 1. Commercial licence model

- Initial target price: **US$20 one-time purchase** for a lifetime-use licence. The price is a planning target, not a final public offer.
- One licence may be active on **one computer at a time**. The owner may deactivate it and transfer it to another computer, subject to reasonable anti-abuse controls.
- Lifetime use does **not** mean lifetime feature updates or lifetime technical support.
- Include updates released during the first **six months from the purchase date**. This period covers feature updates, improvements, bug fixes, and security updates released within that window.
- After the included update period ends, the customer may continue using the licensed version they already have indefinitely. Updates released after that period require a separately purchased upgrade or update entitlement; no subscription is required to keep using the existing version.
- Consider a fair discounted upgrade price rather than automatically charging the full initial purchase price again. Final upgrade pricing and the exact meaning of a major-version upgrade must be defined before sales begin.
- Do not promise lifetime technical support. Define a realistic support policy separately.

## 2. Separate licensing infrastructure

- Use a small, independent licensing service, with Cloudflare Workers and D1 as an initial candidate, subject to current free-tier quotas, terms, availability, and a tested migration path.
- The licensing service manages purchase verification, licence issuance, activation/deactivation, device transfers, recovery, replacement, revocation, and update entitlements.
- Keep licensing infrastructure separate from the midSunday application and its local data. It must not become a chat, memory, model, file, microphone, or backup service.
- Keep requests lightweight and event-driven. Do not run a continuously active server or scheduled workload unless a demonstrated requirement justifies it.
- Free-tier quotas are limits, not a guarantee of unlimited free service. Disclose unavoidable payment-processing or email-delivery costs before choosing providers.

## 3. Licence administration and founder/test licences

- Provide a private administration process for creating, issuing, reviewing, suspending, revoking, and replacing licences.
- Support manually issued founder, development, demonstration, and QA licences without requiring a customer payment.
- Support batch issuance only for controlled testing or approved promotions.
- Record purchase references, licence status, entitlement dates, activation events, and administrative actions with minimal necessary metadata.
- Protect administration with strong authentication, least privilege, and audit records. Never embed an administrative master key in the desktop application.

## 4. Official website purchase and licence delivery

- The official website may display the licence key and recovery code on a secure checkout-success page after the backend independently verifies that payment succeeded.
- Also send the licence details to the buyer's email as a convenient backup, where a suitable email-delivery provider is available.
- Never issue a licence solely because a browser was redirected to a success URL. Verify payment through a trusted server-side provider response or signed webhook.
- Make webhook processing idempotent so retries cannot create duplicate licences or duplicate entitlements.
- Do not place licence keys, recovery codes, or other secrets in public URLs, analytics events, or browser-visible payment metadata.
- Provide a secure way to retrieve the purchase later if the buyer closes the success page or the email is delayed. Email delivery failure must not silently invalidate a verified purchase.
- Compare provider free tiers and limits before implementation. Do not promise that email sending or payment processing will be free.

## 5. Buyer email and ownership recovery

- Associate each purchased licence with a buyer-provided, verified email address for receipts, ownership records, and recovery. Support ordinary email providers and custom domains.
- Do not require Google Sign-In or a Google account. An email address is an ownership/recovery contact, not permission for Google to access midSunday.
- Do not make continued access to one email inbox the sole proof of ownership.
- Provide a separate recovery code at purchase/issuance. Store only a secure verifier/hash server-side, not the plaintext recovery code.
- If the buyer loses the licence key but still controls the verified email, allow a verified recovery flow. If the buyer loses email access but has the key, require additional purchase evidence. If both are lost, use a manual purchase-verification process.
- After ownership is verified, allow the buyer email to be updated and the key/recovery material to be reissued. Revoke the superseded key.
- Clearly explain that losing both the key/recovery material and the ability to prove purchase may make recovery impossible.

## 6. Customer activation

- Activation should be simple: the customer enters the licence key in midSunday. Do not ask them to copy or submit a Device ID manually.
- The application may generate a privacy-conscious device binding automatically and send only the minimum required activation data to the licensing service.
- The service returns a signed entitlement that the application can verify locally. Define the signing format, key rotation, entitlement fields, and replay protections during implementation.
- The entitlement should identify the licence, activation status, permitted device count, and included-update expiry without exposing unnecessary buyer data.
- Do not transmit conversations, memories, local model contents, files, microphone data, or backups during activation.

## 7. Deactivation and device transfer

- Add a clear **Deactivate this computer** action in Settings → Licence.
- After successful deactivation, the server releases that activation so the same key can be used on another computer.
- Explain that deactivation normally requires an internet connection. Provide a support-assisted recovery path for a lost, broken, or inaccessible computer.
- Apply reasonable transfer limits and human review for suspicious patterns, without trapping legitimate customers after hardware replacement or OS reinstall.

## 8. Lost, stolen, replaced, and revoked licences

- Provide a support process for reporting a stolen or compromised key.
- Verify ownership before cancelling or replacing a licence.
- Revoke the old key and issue a replacement linked to the verified owner and the remaining purchase/update entitlement.
- Keep a minimal audit trail of replacement and revocation events.
- Clarify how a revoked entitlement affects an offline computer: a device that never reconnects cannot learn about a server-side revocation until it checks again.

## 9. Offline use and verification policy — decision required

- Core app functionality and user data must remain local. Do not require continuous connectivity or a permanently running licensing server.
- Activation-only is the preferred first implementation for minimal cost and customer friction. Under this model, an already activated installation continues offline, but remote revocation or device-transfer changes cannot take effect on that device until it contacts the service again.
- A periodic verification policy with a defined offline grace period is an alternative, but must not be introduced without a clear customer-facing explanation and tested outage behaviour.
- Finalise whether activation-only or periodic verification is used before implementation and before publishing licence terms.
- Never make local conversations, memories, backups, or files inaccessible merely because the licensing service is temporarily unavailable.

## 10. Security architecture and realistic limits

- Keep private signing keys and payment-provider secrets on the server, outside the public repository and desktop package.
- Verify signed entitlements in the Rust application layer; do not rely only on frontend checks.
- Use rate limits, abuse detection, secure transport, strict input validation, and protected administrative access.
- Avoid collecting a hardware fingerprint more detailed than needed for one-active-device enforcement. Document what device information is collected and why.
- Treat client-side checks as a deterrent, not perfect DRM. A modified client can potentially bypass local enforcement; do not promise uncrackable protection.
- Do not silently lock users out of their local data during a server outage or routine service migration.

## 11. Developer, founder, and testing licences

- Provide manually created internal licences for the founder, developers, testers, and approved demonstrations.
- Keep test and production licensing environments separate, with visibly distinct test keys and records.
- Test licences must not grant hidden administrative powers or introduce a master bypass into the released application.
- Document how internal licences are created, rotated, and revoked.

## 12. Privacy and data separation

- The licensing backend may store only the minimum information needed for licence ownership and enforcement: licence identifiers or secure key verifiers, verified contact email, purchase reference/status, entitlement dates, activation binding, and necessary audit metadata.
- It must not have access to private chats, saved memories, Awareness data, prompts, local models, user files, microphone recordings, or backup contents.
- No Google Sign-In, Google Drive access, or Google account permission is required for licensing.
- Publish a plain-language privacy notice describing licensing data, device binding, retention, and support/recovery handling before sales begin.

## 13. Reliability, recovery, and provider migration

- Back up licensing records securely and test restoration before accepting real purchases.
- Keep a documented export/migration path so the service can move away from Cloudflare or another provider without invalidating legitimate customer licences.
- Design signed entitlements so already-issued licences can be validated locally during a temporary backend outage, within the final chosen verification policy.
- Monitor quota usage and service errors without collecting application content.
- Keep a manual support procedure for verified purchases if automated email or website delivery fails.

## 14. Implementation and release acceptance criteria

- Payment is verified server-side before a paid licence is issued.
- Checkout success page and email delivery do not expose secrets in URLs or analytics.
- Repeated payment webhooks cannot issue duplicate licences.
- Manual founder/test licence issuance works without payment.
- Key activation succeeds without asking the customer to provide a Device ID.
- One active computer is enforced; normal deactivation and transfer work.
- Lost-key, lost-email, recovery-code, replacement, and stolen-key support flows are tested.
- Signed entitlements validate locally and survive temporary licensing-service outages according to the published policy.
- The six-month included-update period is calculated from verified purchase time; the existing licensed version remains usable after expiry, while later updates require a valid upgrade entitlement.
- Licensing requests contain no private application content, and the application remains usable with local data intact if the licensing service is unavailable.
- Free-tier limits, payment fees, email costs, legal terms, refund rules, and support obligations are reviewed before launch.



### 2026-11-25 — First HIARM capability assessment

- Added a local, non-binding starting-mode assessment using installed RAM and logical CPU count, with conservative Lightweight, Balanced, and High capability suggestions.
- Added an independent warning for low currently available memory; missing hardware values produce an Undetermined result rather than a guessed tier.
- Displayed the rationale and limitations in Settings → System & hardware.
- Kept the assessment informational only: it does not choose or download models, alter settings, start/stop workloads, or claim benchmark-backed compatibility.
- Added Rust tests for memory/CPU thresholds, low-memory headroom, and incomplete hardware data. GPU/VRAM and real-device validation remain future work.


### 2026-11-25 — CPU instruction-set inventory

- [x] Add runtime detection for common x86/x86-64 SIMD features (SSE family, AVX, AVX2, FMA) using Rust's OS-aware feature probes.
- [x] Add AArch64 NEON detection and explicit unsupported-architecture status elsewhere.
- [x] Display detected CPU/OS capabilities in Settings → System & hardware.
- [x] Add regression coverage for serialisation and the detector's feature allow-list.
- [ ] Verify inference-backend build flags and dispatch before using these fields for runtime recommendations.

This is capability inventory only; it does not change CPU thread settings or claim that the installed AI runtime can use each detected instruction set.


### 2026-11-25 — Windows GPU adapter inventory

- [x] Enumerate Windows display adapters through DXGI 1.1 and report adapter name, vendor/device IDs, dedicated video memory, dedicated system memory, shared system memory, and software-adapter flags.
- [x] Preserve multiple adapters and label unsupported/unavailable detection explicitly; keep GPU inspection local and read-only.
- [x] Display adapter inventory in Settings → System & hardware with clear separation between dedicated VRAM capacity and shared system memory.
- [ ] Add process-specific DXGI video-memory budget/usage sampling and validate results against real Windows systems.

This is hardware inventory only. It does not prove inference-runtime acceleration, report currently free VRAM, select a model, or configure GPU offload.


### 2026-11-25 — Windows process-specific GPU memory budget snapshot

- [x] Query DXGI local and non-local memory budget/usage for the current midSunday process where `IDXGIAdapter3` is available.
- [x] Preserve independent status for each adapter and each memory segment; failed queries never hide adapter inventory.
- [x] Display process usage, OS budget, and reservation headroom as a refresh-time snapshot without reserving or changing memory.
- [ ] Compare reported values against Windows Task Manager and validate integrated/discrete multi-GPU systems on real devices.
- [ ] Add event-driven budget-change notifications only when a real runtime consumer needs them and lifecycle handling is designed.

This is not total free VRAM and does not prove that the AI inference runtime is using the adapter. No GPU memory is reserved or model offload enabled.


### 2026-11-25 — HIARM GPU query diagnostics

- [x] Label the DXGI memory-budget sample explicitly as node index 0; do not imply it covers every node on multi-node adapters.
- [x] Preserve the HRESULT independently for failed local and non-local memory-segment queries, while retaining successful results from the other segment.
- [x] Display the node index and hexadecimal query failure codes in Settings → System & hardware.
- [ ] Investigate reliable adapter-node enumeration and mapping to the selected inference runtime before using node-specific values for model placement.
- [ ] Compare live readings with Windows Task Manager on physical integrated/discrete and multi-GPU systems.

The node-zero query is diagnostic inventory only. It does not establish which adapter or node an AI runtime uses, and does not reserve GPU memory.


### 2026-11-25 — HIARM snapshot freshness

- [x] Include a collection timestamp in every hardware snapshot.
- [x] Display the actual localised last-checked time instead of a generic session label.
- [x] Keep the timestamp as in-memory snapshot metadata; do not create a hardware activity history or transmit it.
- [ ] Add explicit stale-data messaging only if future background refresh or long-lived snapshots are introduced.



### 2026-11-25 — HIARM storage headroom indicators

- [x] Calculate free-space percentage for each enumerated volume with safe handling for zero capacity and inconsistent values.
- [x] Classify volumes as Healthy, Low, Critical, or Unknown using conservative absolute and percentage thresholds.
- [x] Display used-space bars and plain-language free-space status in Settings → System & hardware.
- [x] Add Rust regression tests for threshold boundaries, zero capacity, and free-space clamping.
- [x] Identify the configured model storage volume and display its containing volume's available space and health status.
- [ ] Compare free space with a model-specific download/install/runtime estimate.
- [ ] Validate volume enumeration and threshold usefulness on real Windows, Linux, and macOS devices.

These indicators describe general volume headroom only. They do not guarantee that a model can be downloaded, installed, loaded, or run.

### 2026-11-25 — HIARM configured model storage-volume matching

- [x] Pass the configured GGUF model path into the local hardware refresh.
- [x] Resolve an existing model file and match it to the most specific enumerated volume using canonical filesystem paths.
- [x] Display the matched volume, available bytes, free percentage, and general volume health in Settings.
- [x] Report unconfigured, inaccessible, missing, and unmatched paths explicitly; do not guess a drive.
- [ ] Validate path and mount matching on physical Windows, Linux, and macOS devices and add model-specific storage-footprint estimates.


### 2026-11-25 — HIARM GGUF model metadata intelligence (Batch #113)

- [x] Add a bounded, read-only GGUF v2/v3 metadata inspector without a new third-party dependency.
- [x] Extract architecture, embedded model name, GGML file-type/known quantisation label, parameter count, context length, layer/block count, embedding length, attention-head count, tensor count, file size, and container metadata counts when present.
- [x] Expose inspection through an explicit Settings → Local model library action; run file parsing on a blocking worker.
- [x] Keep inspection local and read-only: never load tensor weights, copy or modify the model, start inference, download files, or transmit model paths/metadata.
- [x] Bound metadata entries, strings, and arrays; reject malformed, truncated, unsupported, or unreasonable structures safely; skip unknown supported metadata.
- [x] Add regression tests for representative metadata, quantisation labels, malformed headers, unsupported versions, truncation, count limits, unknown fields/arrays, and extension validation.
- [ ] Validate against a representative real-world GGUF corpus across model architectures, quantisations, and exporter versions.
- [ ] Expand architecture-specific metadata fields only from verified GGUF specifications and fixtures.
- [ ] Use measured model/runtime/context profiles to estimate RAM, VRAM, KV cache, and disk staging requirements; do not infer compatibility from metadata alone.
- [ ] Build runtime-aware compatibility checks and hardware recommendations only after measured benchmark and real-device validation.

The metadata is supplied by the GGUF file and is descriptive, not authoritative. Missing or inaccurate fields remain unavailable or are displayed as reported; this batch does not select models, alter context settings, guarantee compatibility, or perform resource management.

### 2026-11-25 — Extended GGUF architecture metadata (Batch #115)

- [x] Read selected standard GGUF architecture fields for grouped-query attention KV heads, key/value head lengths, RoPE dimensions, and expert counts.
- [x] Read standard general quantisation-version and alignment values, preserving the distinction between quantisation version and file-type quantisation label.
- [x] Display the added fields in the local model library, with absent values explicitly marked unavailable or not provided.
- [x] Extend synthetic GGUF regression fixtures and assertions for representative architecture metadata.
- [ ] Validate parser behaviour against a representative real-world GGUF corpus across multiple architectures, quantisations, and exporter versions.
- [ ] Use verified metadata plus runtime-specific measurements to build conservative context/KV-cache and model resource estimates; do not infer full model RAM/VRAM from parameter count or file size alone.

This batch extends descriptive metadata only. It does not claim runtime compatibility or predict memory use, select models, or change context/resource settings.


### 2026-11-25 — Conservative GGUF KV-cache estimate (Batch #116)

- [x] Add a user-triggered KV-cache estimate using GGUF layer count, KV-head count, key/value head lengths, and the currently configured context length.
- [x] Calculate key and value cache separately with an explicit FP16 (2-byte) assumption and checked integer arithmetic.
- [x] Report insufficient metadata rather than guessing missing dimensions; reject invalid context, zero dimensions, and arithmetic overflow.
- [x] Display the estimate and its assumptions beside the selected model's library metadata.
- [x] Add regression coverage for representative arithmetic, missing metadata, invalid context, zero dimensions, and overflow.
- [ ] Validate estimates against actual llama.cpp runtime measurements across multiple architectures, cache types, and sequence/batch settings.
- [ ] Extend the resource model to measured model-weight residency, runtime overhead, compute buffers, CPU/GPU offload, and real hardware headroom.

Batch #116 is an informational estimate only. It is not a total RAM/VRAM prediction or model-fit guarantee, does not select a model, and does not change context or runtime settings. See `docs/HARDWARE-INTELLIGENCE.md` section 24.


### 2026-11-26 — Managed llama-server process memory snapshot (Batch #117)

- [x] Expose the PID of the managed child process internally through the process supervisor without returning it to the UI.
- [x] Add a targeted sysinfo query for that one known child PID; do not enumerate or expose unrelated processes.
- [x] Add an explicit Settings action to refresh resident and virtual memory for the running inference process.
- [x] Report not-running and unavailable states without guessing; keep the measurement local, read-only, and on demand.
- [x] Document that process memory is not model-only memory, virtual memory is not physical RAM, and GPU allocations are outside this snapshot.
- [ ] Validate measurements against operating-system process monitors on real Windows, Linux, and macOS machines.
- [ ] Compare before-load, after-load, and sustained-inference samples across representative model/runtime configurations before using measurements for recommendations.

This is a process-wide diagnostic measurement only. It does not predict model fit, separate model weights from KV cache, report GPU allocations, alter runtime settings, or select/stop models.

### 2026-11-26 — Inference-generation memory boundary report (Batch #121)

- [x] Take a targeted memory sample for the managed llama-server child immediately before each local generation request.
- [x] Take a second targeted sample immediately after a successful generation completes, without enumerating unrelated processes.
- [x] Report resident and virtual memory at both boundaries and calculate signed end-minus-start deltas only when both values are available.
- [x] Label incomplete samples explicitly and keep timestamps local; do not create background polling or a persistent telemetry history.
- [x] Display the latest successful generation's boundary readings and deltas in Settings, with clear interpretation limits.
- [x] Add regression tests for positive, negative, zero, missing, and out-of-range memory deltas.
- [ ] Compare boundary readings with operating-system process monitors during real model generation on Windows, Linux, and macOS.
- [ ] Collect representative repeated measurements across models, context lengths, and workloads before using memory changes for hardware recommendations.

This is a pair of process-wide boundary readings, not peak memory or model-only allocation measurement. It does not identify weights/KV cache/compute buffers, measure GPU allocations, guarantee model fit, or change runtime settings. See docs/HARDWARE-INTELLIGENCE.md section 26.

### 2026-11-26 — Managed inference-process peak memory sampling (Batch #122)

- [x] Add a request-scoped sampler that observes only the already-managed llama-server child PID while generation is active.
- [x] Sample resident and virtual memory at a conservative 250 ms interval, stopping and joining the sampler when generation ends.
- [x] Include the before/after boundary readings when calculating maximum observed memory.
- [x] Report peak observed resident/virtual values, valid sample counts, interval sample counts, sampling interval, and explicit availability status.
- [x] Keep sampling local and temporary: no idle background monitor, process enumeration, PID exposure, persistent history, or network telemetry.
- [x] Add regression tests for peak tracking, interval counts, and missing readings.
- [ ] Compare peak readings with operating-system process monitors on real Windows, Linux, and macOS devices.
- [ ] Evaluate sampler overhead and missed short-lived spikes across short/long generations and multiple model sizes before using results for adaptive resource recommendations.

The reported peak is the maximum observed at the sample points, not a guaranteed true peak. A short spike between 250 ms samples may be missed. It remains process-wide and does not measure GPU allocations or guarantee model fit. See docs/HARDWARE-INTELLIGENCE.md section 27.


### 2026-11-26 — Model-startup peak memory sampling (Batch #124)

- [x] Sample only the known managed llama-server child while the local server is becoming ready and loading the selected model.
- [x] Capture initial, readiness-poll, and final samples; retain maximum observed resident and virtual memory with valid sample count and sampling interval.
- [x] Expose the latest successful startup report through runtime status and display it in Settings.
- [x] Keep the probe local and bounded by startup readiness: no process enumeration, PID exposure, idle monitoring, persistent telemetry history, or network transfer.
- [x] Report unavailable measurements explicitly and label the values as process-wide observed maxima rather than model-only or guaranteed peaks.
- [ ] Compare startup peaks against operating-system process monitors on real Windows, Linux, and macOS devices across representative model sizes and quantisations.
- [ ] Evaluate whether startup sampling cadence and overhead are suitable for future measured resource profiles; do not enable automatic model selection or resource changes from this report alone.

This report measures the whole managed inference process during startup. It may miss short spikes between readiness polls and excludes GPU memory. It does not guarantee model fit or runtime compatibility. See docs/HARDWARE-INTELLIGENCE.md section 28.


### 2026-11-26 — Startup duration and memory-growth context (Batch #126)

- [x] Measure elapsed time from immediately before managed process launch until the local server readiness check succeeds.
- [x] Preserve the first valid managed-process resident/virtual sample as a startup baseline.
- [x] Report signed observed peak-minus-initial resident and virtual memory deltas, with safe handling of missing values and integer-range overflow.
- [x] Display startup duration, initial footprint, observed peaks, and growth values together in Settings.
- [x] Keep all measurements local and in-memory; do not persist telemetry or use values to change runtime settings or select models.
- [x] Add regression tests for positive, negative, zero, missing, and out-of-range deltas.
- [ ] Compare timing and memory-growth values against operating-system monitors on real Windows, Linux, and macOS devices across representative model sizes and quantisations.
- [ ] Collect repeated controlled runs before treating startup duration or memory growth as a performance benchmark or using it for adaptive recommendations.

Startup duration includes process launch, server initialisation, model loading, and readiness response; it is not a pure model-load duration. Memory growth is based on the first valid sample and maximum observed sample, not a guaranteed peak or model-only allocation. See docs/HARDWARE-INTELLIGENCE.md section 29.


### 2026-11-26 — GGUF tensor storage analysis (Batch #128)

- [x] Parse GGUF tensor descriptors without reading tensor weight payloads.
- [x] Validate tensor names, dimensions, alignment, offsets, and known payload boundaries with bounded counts.
- [x] Calculate payload sizes only for explicitly supported, verified GGML block layouts.
- [x] Group tensor counts and known payload bytes by GGML type; expose partial/unknown status instead of guessing unsupported formats.
- [x] Add a user-triggered tensor storage analysis panel to the local model library.
- [x] Keep stored tensor payload size distinct from GGUF file size, runtime RAM/VRAM, KV cache, and model-fit recommendations.
- [ ] Validate the parser against a representative real-world GGUF corpus from multiple architectures and quantisations.
- [ ] Expand supported GGML types only after verifying upstream block layouts and adding fixtures for each.

The reported known payload total excludes inter-tensor alignment padding and leaves unsupported tensor types unknown. It is not a RAM/VRAM estimate or a model-fit guarantee. See docs/HARDWARE-INTELLIGENCE.md section 30.


### 2026-11-26 — GGUF tensor payload range integrity (Batch #131)

- [x] Sort tensor descriptors by their relative data offsets for payload-range validation without assuming descriptor listing order is storage order.
- [x] Reject overlapping payload ranges when both tensor layouts have known, verified sizes.
- [x] Preserve checked file-boundary arithmetic and use checked arithmetic for analysed/unknown aggregate counters.
- [x] Add regression fixtures for overlapping known ranges and valid non-overlapping ranges with descriptors listed out of storage order.
- [ ] Validate overlap and offset behaviour against a representative real-world GGUF corpus across architectures, quantisations, and exporter versions.

Unknown tensor types still have unknown payload lengths, so their ranges cannot be included in overlap checks. The validation is structural and does not read weights, calculate runtime memory, or guarantee model compatibility. See docs/HARDWARE-INTELLIGENCE.md section 31.


### 2026-11-26 — GGUF overlap checks include unknown tensor starts (Batch #132)

- [x] Compare each known tensor payload end with the next descriptor offset, regardless of whether the next tensor type has a known payload layout.
- [x] Reject a known payload that reaches into an unsupported tensor's declared start.
- [x] Keep unsupported payload sizes unknown; do not guess their lengths or claim complete overlap validation.
- [x] Add regression coverage for a known F32 payload overlapping an unsupported tensor start.
- [ ] Validate descriptor offsets and overlap behaviour against a representative real-world GGUF corpus across architectures, quantisations, and exporter versions.

See docs/HARDWARE-INTELLIGENCE.md section 32.


### 2026-11-26 — Host-memory evidence check for GGUF models (Batch #133)

- [x] Add a user-triggered, local-only report combining GGUF metadata, tensor descriptor analysis, the configured context length, and a fresh host-RAM snapshot.
- [x] Compare only the theoretical one-sequence FP16 KV-cache estimate with currently available system RAM; flag when the KV cache alone exceeds the available reading.
- [x] Keep model file size and known tensor payload bytes labelled as on-disk facts, never as required RAM.
- [x] Report missing GGUF dimensions and unavailable hardware memory as explicit unknown states.
- [x] Explain that a KV-cache estimate below available RAM is not a model-fit or runtime-compatibility guarantee.
- [x] Add regression tests for above/below available RAM, missing metadata, and unavailable hardware readings.
- [ ] Validate estimates against measured llama.cpp host/GPU allocations across supported models, cache types, and offload configurations.
- [ ] Build a full compatibility engine only after representative real-device measurements establish reliable model-weight, runtime, compute-buffer, and accelerator requirements.

The Batch #133 report is informational evidence only. It does not select models, alter context or resource settings, load weights, or claim CPU/GPU compatibility. See docs/HARDWARE-INTELLIGENCE.md section 33.


### 2026-11-26 — GGUF-reported context-length advisory (Batch #135)

- [x] Compare the configured context length with the GGUF-reported context length when available.
- [x] Distinguish within-reported, exceeds-reported, missing, and unusable context metadata states.
- [x] Preserve the theoretical KV-cache calculation when the configured context exceeds the reported value, while displaying a clear advisory.
- [x] Explain that the GGUF value is not a verified runtime maximum and that RoPE scaling/context extension may be runtime-dependent.
- [x] Include the advisory in both per-model KV-cache estimates and host-memory evidence.
- [x] Add regression tests for within, above, and missing reported context values.
- [ ] Validate reported-context interpretation against a representative GGUF corpus and supported llama.cpp runtime configurations.

This is an informational metadata comparison only. It does not reject a context, change settings, prove runtime support, or guarantee model fit. See docs/HARDWARE-INTELLIGENCE.md section 34.


### 2026-11-26 — GGUF tensor element-count cross-check (Batch #137)

- [x] Calculate each tensor's declared element count from its dimensions with checked 64-bit multiplication.
- [x] Aggregate declared tensor elements across all descriptors, including unsupported quantisation layouts whose dimensions remain readable.
- [x] Read optional `general.parameter_count` metadata during tensor analysis and compare it with the descriptor-derived element total.
- [x] Display both values and a neutral comparison status; do not conflate tensor elements with unique model parameters.
- [x] Reject zero-sized dimensions and arithmetic overflow rather than wrapping or inventing counts.
- [ ] Validate comparison behaviour against a representative real-world GGUF corpus, especially tied/shared tensors and exporter-specific parameter-count conventions.

The comparison is an advisory structural cross-check only. It does not establish model correctness, runtime compatibility, or memory requirements, and it does not read tensor weights or change model settings. See `docs/HARDWARE-INTELLIGENCE.md` section 35.


### 2026-11-26 — GGUF tensor descriptor name integrity (Batch #140)

- [x] Read each tensor descriptor name using a strict 64-byte bound rather than skipping arbitrary name bytes.
- [x] Reject empty, truncated, and invalid UTF-8 tensor names with clear parser errors.
- [x] Reject duplicate tensor names before calculating element counts or payload ranges.
- [x] Preserve exact, case-sensitive name comparison and avoid exposing tensor names in telemetry or network requests.
- [x] Add regression fixtures for empty, invalid UTF-8, duplicate, and distinct valid names.

This is a structural GGUF container check only; it does not validate architecture-specific tensor naming conventions, prove model completeness, or guarantee runtime compatibility. See docs/HARDWARE-INTELLIGENCE.md section 36.


### 2026-11-26 — GGUF metadata-key uniqueness (Batch #141)

- [x] Reject duplicate metadata keys in both the metadata inspector and tensor-storage analyser.
- [x] Apply the check to all keys, including unknown metadata fields, to avoid ambiguous parser behaviour.
- [x] Preserve support for distinct unknown keys and bounded type-aware skipping.
- [x] Add regression tests covering both inspector paths and valid distinct unknown keys.

This is a structural GGUF metadata check. It does not validate architecture-specific values or guarantee model compatibility. See docs/HARDWARE-INTELLIGENCE.md section 37.


### 2026-11-26 — GGUF metadata key and boolean encoding validation (Batch #142)

- [x] Reject empty metadata keys in both the metadata inspector and tensor-storage analyser.
- [x] Validate scalar GGUF boolean values as exactly 0 or 1.
- [x] Validate boolean-array elements using the same canonical 0/1 encoding rule.
- [x] Preserve valid scalar and array booleans and add malformed/valid regression fixtures for both inspector paths.

This is structural container validation only; it does not establish model compatibility or runtime support. See docs/HARDWARE-INTELLIGENCE.md section 38.


### 2026-11-26 — GGUF metadata array scanning efficiency and bounds (Batch #143)

- [x] Replace per-element seeks for fixed-width numeric arrays with checked aggregate byte-length calculation and a single bounded skip.
- [x] Keep the maximum array-item limit and reject multiplication overflow or truncated payloads.
- [x] Validate boolean arrays in fixed-size chunks, avoiding allocations proportional to untrusted array counts.
- [x] Preserve bounded per-string validation for string arrays and add regression coverage for valid and truncated numeric arrays in both inspection paths.

This is parser resilience and efficiency work only; it does not load model weights or establish runtime compatibility. See docs/HARDWARE-INTELLIGENCE.md section 39.


### 2026-11-26 — GGUF selected metadata value-type validation (Batch #145)

- [x] Require recognised architecture and model-name metadata to use string values.
- [x] Require recognised counts, dimensions, file-type codes, quantisation versions, and alignment fields to use non-negative integer values.
- [x] Reject 32-bit metadata values that exceed the supported range instead of silently presenting them as unavailable.
- [x] Apply strict type/range checks to alignment and parameter-count metadata in tensor-storage analysis as well as metadata inspection.
- [x] Preserve bounded skipping of unknown metadata keys and add regression coverage for wrong types and integer overflow.

This validates metadata representation only; it does not prove that metadata values are truthful or that a model is runtime-compatible. See docs/HARDWARE-INTELLIGENCE.md section 40.


### 2026-11-26 — GGUF recognised metadata ambiguity checks (Batch #146)

- [x] Map each recognised architecture-specific metadata suffix to one semantic field before parsing its value.
- [x] Reject multiple competing keys for the same recognised field instead of silently using the last value encountered.
- [x] Require recognised architecture-specific suffixes to have a non-empty namespace prefix.
- [x] Preserve unrelated vendor-specific metadata keys and bounded skipping behaviour.
- [x] Add regression coverage for competing architecture aliases and unrelated unknown suffixes.

This prevents ambiguous metadata from silently changing the displayed dimensions. It does not verify architecture truthfulness or runtime compatibility. See docs/HARDWARE-INTELLIGENCE.md section 41.


### 2026-11-26 — GGUF tensor data-offset uniqueness (Batch #147)

- [x] Reject duplicate tensor data offsets after sorting descriptors, before payload-size evaluation.
- [x] Apply the rule to known and unsupported tensor types, including mixed known/unknown pairs.
- [x] Add regression coverage proving an unsupported tensor cannot share its offset with a known tensor.
- [x] Preserve a separate regression test for genuinely overlapping, distinct known payload ranges.
- [x] Document that descriptor integrity is validated without reading model weights or guessing unsupported payload sizes.

See `docs/HARDWARE-INTELLIGENCE.md` section 42.


### 2026-11-26 — GGUF architecture metadata namespace consistency (Batch #148)

- [x] Require recognised architecture-specific metadata keys to use the namespace declared by `general.architecture`.
- [x] Make the check independent of metadata entry order, including architecture-specific fields that appear before the general architecture entry.
- [x] Reject architecture-specific recognised fields when `general.architecture` is missing.
- [x] Preserve unrelated vendor-specific metadata through the existing bounded unknown-field path.
- [x] Add regression tests for mismatched namespaces and missing architecture declarations.

This checks internal metadata consistency only; it does not prove the declared architecture is truthful or that the model is runtime-compatible. See `docs/HARDWARE-INTELLIGENCE.md` section 43.


### 2026-11-26 — Consistent GGUF tensor-count safety limit (Batch #149)

- [x] Centralise the maximum supported inspection count as `MAX_TENSORS = 1,000,000`.
- [x] Enforce the same tensor-count limit in metadata-only inspection and tensor-storage analysis.
- [x] Reject excessive header counts before metadata interpretation or descriptor allocation.
- [x] Add a regression test proving both inspection paths reject the same over-limit file.

This is a defensive parser bound, not a model-loadability guarantee. See `docs/HARDWARE-INTELLIGENCE.md` section 44.


### 2026-11-26 — GGUF tensor-descriptor allocation preflight (Batch #150)

- [x] Define the minimum byte length of a valid tensor descriptor.
- [x] Check the declared descriptor count against remaining file bytes using checked arithmetic before count-sized allocations.
- [x] Reject impossible/truncated descriptor declarations before allocating descriptor vectors or tensor-name sets.
- [x] Add a regression test using a tiny file with an impossible declared tensor count.

This is an allocation-safety preflight; detailed descriptor and payload validation remains in place. See `docs/HARDWARE-INTELLIGENCE.md` section 45.


### 2026-11-26 — Aggregate GGUF metadata-key memory budget (Batch #151)

- [x] Add a shared 8 MiB aggregate budget for metadata-key bytes retained during inspection.
- [x] Enforce the budget in both metadata-only inspection and tensor-storage analysis.
- [x] Replace entry-count-based HashSet preallocation with incremental growth under the byte budget.
- [x] Use checked accounting and reject over-budget metadata before retaining additional keys.
- [x] Add regression coverage proving both inspection paths reject an over-budget file.

This bounds retained key text; it is not a complete cap on all metadata bytes or the GGUF file size. See `docs/HARDWARE-INTELLIGENCE.md` section 46.


## 2026-11-26 — Batch #152

- Corrected the GGUF aggregate metadata-key memory regression fixture: each generated key now remains within the existing 1 MiB individual string limit, while nine distinct keys collectively exceed the 8 MiB aggregate budget. This makes the test exercise the intended aggregate guard in both inspection paths. Production safety limits and parser behaviour are unchanged.


### 2026-11-26 — GGUF tensor-descriptor collection memory bounds (Batch #153)

- [x] Replace per-tensor heap-allocated dimension vectors with a fixed-size four-dimension inline array.
- [x] Remove tensor-count-based preallocation for descriptor and tensor-name collections.
- [x] Add checked aggregate accounting with an 8 MiB retained tensor-name text budget.
- [x] Reject over-budget tensor-name sets before retaining the next name.
- [x] Add regression coverage for aggregate tensor-name storage exhaustion.

These are parser-owned collection bounds, not a total process-memory guarantee; the existing one-million descriptor cap and file-size preflight remain in force. See `docs/HARDWARE-INTELLIGENCE.md` section 48.


### 2026-11-26 — Aggregate GGUF metadata-section work budget (Batch #154)

- [x] Add a shared 64 MiB cap for the encoded metadata section in both GGUF inspection paths.
- [x] Measure consumed metadata bytes with checked file-position accounting after each entry, including skipped unknown values and arrays.
- [x] Preserve existing per-string, array-item, metadata-entry, and aggregate metadata-key limits.
- [x] Add a sparse-file regression test proving both inspectors reject an over-budget metadata section without allocating a matching in-memory fixture.

This limits parser work across the complete metadata section; it is not a maximum GGUF file-size limit. See docs/HARDWARE-INTELLIGENCE.md section 49.


### 2026-11-26 — GGUF tensor data-offset file-boundary validation (Batch #156)

- [x] Read tensor-analysis file length from the opened file handle used for parsing.
- [x] Require every tensor data offset to resolve strictly inside the file, rejecting starts at EOF as well as beyond EOF.
- [x] Apply the boundary check to unsupported tensor types whose payload sizes remain unknown.
- [x] Add regression coverage for at-EOF, beyond-EOF, and valid in-file unsupported tensor starts.

This validates data-start locations without guessing unsupported payload sizes or reading tensor weights. See docs/HARDWARE-INTELLIGENCE.md section 50.


### 2026-11-26 — GGUF tensor-descriptor collection memory budget (Batch #157)

- [x] Add a conservative per-descriptor bookkeeping estimate for retained descriptor records and duplicate-name indexing.
- [x] Enforce a checked 64 MiB aggregate descriptor-collection budget before allocating descriptor/name collections.
- [x] Keep the independent 8 MiB tensor-name text budget and one-million tensor-count ceiling.
- [x] Add regression coverage for over-budget rejection before allocation and the exact in-budget boundary.

This bounds parser-side retained collection memory; it is not a GGUF file-size or model-runtime-memory limit. See docs/HARDWARE-INTELLIGENCE.md section 51.


### 2026-11-26 — GGUF metadata string file-boundary preflight (Batch #158)

- [x] Check each declared GGUF string length against the remaining bytes in the opened file before allocating its buffer.
- [x] Apply the shared preflight to metadata keys, selected and unknown string values, string arrays, and tensor names.
- [x] Read metadata-only inspector file size from the same opened handle used for parsing.
- [x] Add regression coverage for a truncated string declaration that must fail before buffer allocation.

This reduces avoidable allocations for malformed/truncated input and ties reported file size to the parsed handle; concurrent external file mutation remains outside this guarantee. See `docs/HARDWARE-INTELLIGENCE.md` section 52.


### 2026-11-26 — GGUF string-array aggregate work preflight (Batch #159)

- [x] Pass the metadata-section start position into unknown-value array skipping.
- [x] Enforce the aggregate 64 MiB metadata-section budget after each string in a string array.
- [x] Preserve the per-string cap, array-item cap, boolean validation, and efficient fixed-width array skipping.
- [x] Add sparse-file regression coverage proving a single oversized string-array entry is stopped mid-entry.

This closes a per-entry accounting gap for string arrays; it does not change tensor parsing or load model weights. See docs/HARDWARE-INTELLIGENCE.md section 53.


### 2026-11-26 — GGUF string-array budget fixture correction (Batch #160)

- [x] Correct the sparse-file regression fixture to include the complete array header, string-length field, and declared string payload.
- [x] Keep the production 64 MiB mid-entry enforcement unchanged; this batch corrects test-fixture sizing only.
- [x] Re-run the shared Rust test suite on Windows, Linux, and macOS.

The previous fixture was eight bytes too short and therefore triggered the correct truncated-string guard before reaching the aggregate metadata-section guard. See docs/HARDWARE-INTELLIGENCE.md section 53.


### 2026-11-26 — GGUF string-array fixture read-handle correction (Batch #161)

- [x] Reopen the sparse regression fixture read-only after writing it, because `File::create` supplies a write-only handle.
- [x] Keep the corrected encoded array size and production metadata-section enforcement unchanged.
- [x] Re-run the Rust core tests on Windows, Linux, and macOS.

The previous fixture correction fixed the encoded byte count but left the test reading through a write-only file handle. See docs/HARDWARE-INTELLIGENCE.md section 53.


### 2026-11-26 — GGUF string-array section-budget allocation preflight (Batch #162)

- [x] Check each string-array payload length against the remaining 64 MiB metadata-section budget before allocating its buffer.
- [x] Preserve the existing per-string cap and same-handle file-boundary preflight.
- [x] Add regression coverage proving an over-budget payload is rejected before the parser reads it.

This prevents a single bounded string allocation from temporarily overshooting the aggregate metadata-section limit. See docs/HARDWARE-INTELLIGENCE.md section 54.


### 2026-11-26 — GGUF metadata-key aggregate allocation preflight (Batch #163)

- [x] Compare each declared metadata-key length with the remaining aggregate 8 MiB key budget before allocation or payload reads.
- [x] Apply the preflight consistently in both metadata inspection paths.
- [x] Add regression coverage proving an over-budget key payload is not consumed.

See docs/HARDWARE-INTELLIGENCE.md section 55.


### 2026-11-26 — GGUF metadata string section-budget preflight (Batch #165)

- [x] Apply the aggregate 64 MiB metadata-section boundary to metadata-key payloads before allocation or reading.
- [x] Apply the same preflight to scalar string values, including selected fields and unknown metadata strings.
- [x] Preserve the separate 8 MiB aggregate metadata-key budget, per-string cap, and opened-file truncation check.
- [x] Add regression tests proving over-budget metadata-key and scalar-string payloads are rejected while their payload bytes remain unread.

This closes the remaining single-string allocation gap in the metadata-section work budget. It does not change tensor parsing, load model weights, or transmit data. See docs/HARDWARE-INTELLIGENCE.md section 56.


### 2026-11-26 — GGUF array-payload section-budget preflight (Batch #166)

- [x] Check fixed-width numeric array payload lengths against the remaining metadata-section budget before seeking.
- [x] Check boolean-array payload lengths against the same budget before chunked reads.
- [x] Preserve checked array-size arithmetic, array-item caps, boolean validation, and file-boundary checks.
- [x] Add regression tests proving over-budget numeric and boolean array payloads remain untouched.

This prevents array parsing from crossing the 64 MiB metadata boundary before the normal end-of-entry check. See docs/HARDWARE-INTELLIGENCE.md section 57.


### 2026-11-26 — GGUF string-length-prefix section-boundary preflight (Batch #167)

- [x] Check that a complete 8-byte GGUF string-length prefix fits inside the remaining 64 MiB metadata-section budget before reading it.
- [x] Apply the guard through the shared reader used by metadata keys, scalar strings, and unknown string-array elements.
- [x] Add regression tests proving metadata-key and scalar-string length prefixes remain unread when they cross the section boundary.
- [x] Preserve existing string-payload, aggregate-key, per-string, and opened-file preflight checks.

This closes the final-seven-byte boundary gap for GGUF string-length prefixes without changing ordinary metadata parsing or tensor handling. See docs/HARDWARE-INTELLIGENCE.md section 58.


### 2026-11-26 — GGUF string-array boundary regression fixture correction (Batch #167 follow-up)

- [x] Update the existing mid-entry string-array test to assert that an out-of-section 8-byte length prefix remains unread.
- [x] Keep the Batch #167 shared-reader preflight unchanged; this follow-up corrects the older test's cursor expectation only.

See docs/HARDWARE-INTELLIGENCE.md section 59.


### 2026-11-26 — GGUF metadata field and scalar section-boundary preflight (Batch #169)

- [x] Preflight the four-byte metadata value-type tag before reading it in both metadata inspection paths.
- [x] Preflight fixed-width numeric and boolean scalar payloads against the remaining 64 MiB metadata-section budget.
- [x] Preflight the complete 12-byte array element-type/count header before consuming either field.
- [x] Add sparse-file regression coverage proving boundary-crossing type tags and scalar payloads remain unread.

This extends the aggregate metadata-section boundary to the remaining fixed-size metadata headers and scalar values. Existing string and array payload protections remain in place. See docs/HARDWARE-INTELLIGENCE.md section 60.


### 2026-11-26 — GGUF metadata boundary regression fixture cursor correction (Batch #169 follow-up)

- [x] Seek to the intended metadata boundary after reopening each sparse regression file.
- [x] Preserve the Batch #169 parser checks; correct fixture cursor setup only.
- [x] Re-run the complete cross-platform Rust test workflow.

See docs/HARDWARE-INTELLIGENCE.md section 61.


### 2026-11-26 — GGUF alignment validation parity (Batch #171)

- [x] Share one tensor-alignment validator between metadata inspection and tensor-storage analysis.
- [x] Reject declared alignment values that are zero, non-power-of-two, or greater than 4096 bytes in both paths.
- [x] Preserve unspecified metadata alignment and the tensor analyser's existing 32-byte default.
- [x] Add cross-inspector regression coverage for valid values 1, 32, and 4096 and invalid values 0, 3, and 8192.

This prevents the metadata-only inspector from presenting a malformed declared alignment as usable when tensor analysis would reject it. See docs/HARDWARE-INTELLIGENCE.md section 62.


### 2026-11-26 — GGUF architecture metadata semantic validation (Batch #172)

- [x] Reject blank `general.architecture` identifiers.
- [x] Require positive values for recognised architecture dimensions and counts when present.
- [x] Ensure attention key/value heads do not exceed total attention heads.
- [x] Ensure expert-used count does not exceed total expert count.
- [x] Add regression tests for zero-valued fields, valid relationships, invalid relationships, and blank architecture identifiers.

These checks reject internally contradictory model metadata before HIARM uses it for hardware and context recommendations. Optional fields remain optional. See docs/HARDWARE-INTELLIGENCE.md section 63.


### 2026-11-26 — GGUF architecture semantic regression fixture correction (Batch #173)

- [x] Include `general.architecture = "llama"` in the fixture that exercises `llama.*` metadata keys.
- [x] Preserve strict architecture namespace validation and all positive-dimension/count relationship assertions.
- [x] Keep production parser behaviour unchanged; correct the incomplete synthetic fixture only.

See docs/HARDWARE-INTELLIGENCE.md section 64.


### 2026-11-26 — HIARM pressure-aware starting profiles (Batch #174)

- [x] Keep the installed-RAM tier and CPU-topology assessment as the baseline.
- [x] Clamp available-memory readings to installed RAM before evaluating headroom.
- [x] Reduce the suggested profile by one level when current available RAM falls below the conservative headroom threshold.
- [x] Preserve Lightweight as the safest profile when the machine is already constrained.
- [x] Add regression tests for Balanced-to-Lightweight and High-capacity-to-Balanced downgrades, healthy headroom, and inconsistent memory readings.
- [x] Keep the assessment advisory only; it does not change runtime settings or claim model compatibility.

See docs/HARDWARE-INTELLIGENCE.md section 65.


### 2026-11-26 — GPU inventory assessment (Batch #175)

- [x] Add a derived GPU inventory assessment to the serializable hardware snapshot.
- [x] Distinguish unsupported, unavailable, no-adapter, software-only, and detected hardware states.
- [x] Exclude software renderers from physical-adapter memory summaries.
- [x] Report the largest dedicated-memory figure and largest OS local-segment budget without pooling memory across adapters.
- [x] Keep GPU inventory separate from claims about inference backend, compute capability, model fit, or performance.
- [x] Add cross-platform tests for mixed adapters, software-only inventory, missing memory budgets, and unavailable states.

See docs/HARDWARE-INTELLIGENCE.md section 66.


### 2026-11-26 — GPU memory reservation-headroom assessment (Batch #176)

- [x] Derive per-physical-adapter reservation headroom from the current process's DXGI local-segment budget.
- [x] Classify reported headroom as critical, constrained, or at least 1 GiB and 20%, without treating it as free VRAM.
- [x] Report the lowest per-adapter reservation headroom and its percentage without pooling values across GPUs.
- [x] Exclude software adapters and skip missing or zero-sized local budgets.
- [x] Preserve partial GPU adapter-enumeration status rather than reporting a complete inventory.
- [x] Display the summary and limitations in Settings → System & hardware.
- [x] Keep the assessment advisory; it does not affect model selection, runtime settings, or capability profiles.
- [x] Add regression coverage for thresholds, mixed adapters, missing budgets, and unsupported/unavailable states.

See docs/HARDWARE-INTELLIGENCE.md section 67.


### 2026-11-26 — GPU context in GGUF model memory evidence (Batch #177)

- [x] Attach a fresh GPU inventory summary to each model-memory evidence report.
- [x] Include physical adapter count, usable local-budget coverage, per-adapter headroom bands, and lowest reported headroom.
- [x] Keep GPU context separate from host-RAM and KV-cache calculations.
- [x] Preserve unsupported/unavailable GPU states without inventing values.
- [x] Show GPU context in the model library's memory-evidence panel.
- [x] Add regression coverage proving GPU headroom does not change the host-RAM verdict.
- [x] Explicitly avoid claims about free VRAM, model fit, or inference-backend offload support.

See docs/HARDWARE-INTELLIGENCE.md section 68.

## Planned feature — Live System Monitor (HIARM companion)

**Status:** Approved for implementation after Batch #177. This is a planned live-observation feature; it is not yet implemented. Keep it distinct from the existing on-demand hardware inventory and model-memory evidence panels.

**Goal:** Give users a clean, understandable, continuously refreshed view of how their computer is performing, similar to a lightweight system-monitor dashboard, while keeping measurements local and honest about platform/driver limitations.

### User experience

- [ ] Add a dedicated **System Monitor** view reachable from Settings → System & hardware (or its own clearly labelled navigation entry if the UX design supports it better).
- [ ] Present clear summary cards for CPU, RAM, GPU, storage, network, and battery (battery only where applicable), with current values and readable units.
- [ ] Show live history graphs/sparklines for supported metrics, with sensible time windows and no distracting visual noise.
- [ ] Show total system resource usage and midSunday's own process usage as separate measurements; never confuse the two.
- [ ] RAM: show total, used, available, and percentage, with definitions consistent across operating systems.
- [ ] CPU: show overall utilisation and, where reliable, per-core utilisation; explain that readings are sampled over an interval.
- [ ] GPU: show per-physical-adapter utilisation and memory usage/budget where reliable APIs expose them; show temperature, engine breakdown, and vendor details only when supported.
- [x] Storage: show per-volume capacity, used/free space, and system-wide read/write activity rates with generic volume labels; keep mount paths and device identifiers out of the live monitor.
- [x] Network: show aggregate system-wide upload/download throughput without collecting interface names; clearly label scope and unavailable/warming-up states.
- [x] Battery: show aggregate charge percentage and charging/discharging state where reported; do not invent remaining-time estimates.
- [ ] Provide clear unavailable/unsupported/permission-limited states instead of fabricated, stale, or misleading zero values.
- [ ] Keep the layout responsive, accessible, keyboard-friendly, and readable at different window sizes and display scales.

### Measurement and architecture

- [ ] Implement a Rust-side, cross-platform metrics provider with a stable serializable snapshot schema and explicit per-metric availability/source metadata.
- [ ] Use appropriate native operating-system interfaces: Windows system/performance and graphics APIs; Linux procfs, sysfs, and supported graphics interfaces; macOS system-statistics and supported graphics interfaces.
- [ ] Treat GPU monitoring as capability-dependent: adapters, vendors, drivers, and OS versions expose different metrics. Do not assume a single portable GPU utilisation/temperature API exists.
- [ ] Sample at a conservative interval (initial target around 1–2 seconds, subject to profiling); avoid expensive polling and suspend or reduce sampling when the monitor is not visible.
- [ ] Keep collection asynchronous and bounded so monitoring cannot freeze the UI or interfere with local model inference.
- [ ] Calculate rate-based metrics from timestamped counter deltas, handle counter resets/overflow, and identify first-sample or unavailable states correctly.
- [ ] Keep system-wide metrics, per-process metrics, and GPU adapter metrics clearly separated; do not infer GPU model-loading or inference support from utilisation readings.
- [ ] Make monitoring read-only. It must not kill processes, change power plans, alter system settings, close applications, or automatically change model/runtime configuration.

### Privacy, performance, and validation

- [ ] Keep monitoring local by default; do not transmit hardware metrics or usage history to a server.
- [ ] Do not enumerate or expose other applications' process names/details unless a separate, explicitly approved feature requires it.
- [ ] Bound in-memory graph history and avoid persistent collection by default; any future export must be user-initiated and clearly explained.
- [ ] Add Rust unit tests for metric calculations, counter deltas, missing values, invalid readings, and platform-independent snapshot handling.
- [ ] Add frontend tests for unavailable states, unit formatting, graph history bounds, and separation of system versus midSunday process readings.
- [ ] Build and test on Windows, Linux, and macOS; validate real readings on physical machines where possible because hosted CI cannot prove hardware sensor support or accuracy.
- [ ] Document per-platform limitations and distinguish measured values from estimates or OS-reported budgets.

**Relationship to HIARM:** The System Monitor observes current resource usage. HIARM may later consume selected, validated measurements to improve advisory workload/model recommendations, but monitoring and adaptation remain separate. No automatic system changes are permitted without a separately designed, user-approved control.


### 2026-11-26 — Live System Monitor foundation (Batch #178)

- [x] Add a local Rust sampler for CPU utilisation, per-logical-processor utilisation, system RAM, and midSunday's own resident/virtual memory.
- [x] Keep the sampler stateful so CPU percentages are calculated from successive OS samples; identify the initial baseline as warming up.
- [x] Restrict process refreshes to the current midSunday process; do not enumerate unrelated applications.
- [x] Add an opt-in Settings → System & hardware live panel with a 1.5-second polling target, bounded recent CPU/RAM history, and an explicit stop control.
- [x] Pause sampling when the user leaves Settings and clear the live session when monitoring is stopped.
- [x] Keep GPU live utilisation explicitly marked as not yet implemented; network throughput, battery telemetry, and live system-wide disk activity are now implemented in the opt-in monitor.
- [ ] Validate real metric accuracy on physical Windows, Linux, and macOS devices; hosted CI validates builds/tests, not sensor accuracy.

This is the first vertical slice of the roadmap feature, not the completed System Monitor. See the Live System Monitor section in ROADMAP.md.


### 2026-11-26 — Network and process I/O monitoring (Batch #180)

- [x] Add aggregate system-wide upload/download rates from network byte-counter deltas, without collecting or displaying interface names.
- [x] Add explicit network warm-up/unavailable states and guard against counter resets and zero-duration samples.
- [x] Add midSunday-only disk read/write rates using the current process's I/O counters; do not enumerate or expose other processes.
- [x] Extend the bounded live history with download/upload activity.
- [x] Add regression tests for rate calculations, counter resets, serialization, and absence of interface-name fields.
- [x] Add system-wide disk read/write activity from operating-system disk counters, separately from midSunday process I/O.
- [ ] Add live GPU utilisation/memory using platform-aware collectors and explicit unsupported states.
- [ ] Validate network and process I/O accuracy on physical Windows, Linux, and macOS devices.


### 2026-11-27 — Battery telemetry (Batch #181)

- [x] Add cross-platform battery detection through the maintained starship-battery library (Windows, Linux, and macOS support).
- [x] Report energy-weighted aggregate charge across detected batteries and a combined charging/discharging/full/empty/paused state.
- [x] Preserve no-battery, partial, unavailable, unknown, and mixed-state outcomes without fabricating values.
- [x] Keep battery vendor, model, serial number, and other device identifiers out of the snapshot.
- [x] Display battery charge and state in the opt-in live monitor; retain explicit local-only sampling and bounded history.
- [x] Add regression tests for mixed battery states, valid charge bounds, and snapshot serialization.
- [ ] Validate battery readings on physical laptops across Windows, Linux, and macOS; desktop CI cannot prove battery hardware availability or accuracy.


### 2026-11-27 — System-wide disk activity (Batch #182)

- [x] Add OS-reported disk read/write throughput across detected storage devices using sysinfo disk I/O counters.
- [x] Keep system-wide disk activity separate from midSunday process-only disk I/O.
- [x] Sum per-device deltas without exposing mount paths, volume labels, device names, or serial identifiers.
- [x] Add explicit first-sample warm-up and no-disk states; keep idle zero-throughput distinct from unavailable data.
- [x] Extend bounded recent-activity history with system-wide disk read/write samples.
- [x] Add regression tests for elapsed-time handling, idle I/O, rate conversion, and snapshot serialization.
- [ ] Validate counter accuracy on physical Windows, Linux, and macOS devices; OS caching, virtual volumes, RAID, and driver reporting can affect readings.


### 2026-11-27 — Live GPU telemetry foundation (Batch #183)

- [x] Add a stable live GPU snapshot with explicit overall and per-adapter status, utilisation, VRAM allocation, source, and limitations fields.
- [x] Add a bounded, read-only Linux AMDGPU DRM sysfs collector for driver-reported GPU busy percentage and VRAM used/total bytes.
- [x] Keep adapter labels generic; do not expose GPU names, PCI identifiers, or serial/device identifiers in live monitoring.
- [x] Reject malformed/out-of-range utilisation and zero-capacity memory readings; clamp used VRAM to the reported total.
- [x] Display live GPU utilisation and allocated VRAM as distinct values in the System Monitor.
- [x] Preserve explicit unsupported-platform, unsupported-driver, no-adapter, partial, and unavailable states rather than presenting zero as a fabricated reading.
- [x] Add a Windows PDH GPU Engine / dedicated Adapter Memory collector with generic per-adapter labels, explicit unsupported states, and DXGI capacity context where available.
- [ ] Add and validate macOS GPU telemetry using supported public or carefully reviewed platform interfaces.
- [ ] Add Linux Intel and NVIDIA collectors with driver-specific semantics and tests.
- [ ] Validate live readings on physical hardware; hosted CI cannot validate sensor accuracy.

This is the first GPU telemetry slice, not universal GPU support. The initial live collector reads documented AMDGPU driver counters on Linux. Windows and macOS remain explicitly unsupported in this batch; the existing Windows DXGI inventory and process-scoped memory-budget snapshot are separate from system-wide live utilisation. See docs/HARDWARE-INTELLIGENCE.md section 72.


### 2026-11-27 — Windows live GPU telemetry (Batch #185)

- [x] Add a local PDH collector for Windows GPU Engine utilisation and dedicated adapter-memory allocation counters.
- [x] Take two samples for rate-style GPU Engine counters and select the peak individual engine per adapter rather than summing concurrent engines.
- [x] Group PDH instances by adapter LUID internally and expose only generic GPU labels; do not return adapter names, LUIDs, PIDs, or engine instance names.
- [x] Pair dedicated-memory allocation with DXGI dedicated-memory capacity only where a matching capacity is available; never label shared memory as VRAM.
- [x] Preserve explicit unavailable, unsupported-driver, no-adapter, and partial states; show dedicated-memory bytes even when total capacity is unavailable.
- [x] Add regression tests for peak-engine selection and malformed/out-of-range utilisation values.
- [ ] Validate PDH counter presence, attribution, and accuracy on physical Windows 10 and Windows 11 systems with different GPU vendors and driver versions.
- [ ] Add and validate macOS GPU telemetry and Linux Intel/NVIDIA collectors.

Windows readings are driver/build-dependent. The displayed percentage is the busiest individual engine, not a universal whole-GPU utilisation value. See docs/HARDWARE-INTELLIGENCE.md section 73.


### 2026-11-27 — Live storage capacity monitoring (Batch #188)

- [x] Add per-volume total, used, and available capacity to the opt-in live System Monitor.
- [x] Use generic labels such as Volume 1; do not expose mount paths, volume labels, filesystem names, device names, or serial identifiers in live metrics.
- [x] Reuse the existing conservative free-space thresholds; represent zero-capacity readings as unknown rather than a fabricated percentage.
- [x] Keep per-volume capacity separate from aggregate system-wide disk read/write throughput and midSunday process-only disk I/O.
- [x] Add serialization and boundary regression coverage for the storage snapshot.
- [ ] Validate reported capacity and volume enumeration on physical Windows, Linux, and macOS devices; virtual volumes, network mounts, permissions, and filesystem semantics may differ.

See docs/HARDWARE-INTELLIGENCE.md section 74.


### 2026-11-27 — GPU monitor clarity and per-adapter history (Batch #189)

- [x] Show dedicated-memory allocation as a separate visual meter only when both used and total capacity are available.
- [x] Keep used-byte readings visible when total dedicated-memory capacity is unavailable; do not invent a percentage.
- [x] Add accessible, labelled meter semantics to supported GPU utilisation and dedicated-memory readings.
- [x] Record bounded recent utilisation history independently for each generic GPU label; preserve missing samples as gaps instead of treating them as zero.
- [x] Keep adapter labels generic and avoid exposing hardware identifiers.
- [ ] Validate live GPU utilisation and memory readings against physical devices and driver tools on supported platforms; history inherits each collector's platform-specific measurement semantics.

See docs/HARDWARE-INTELLIGENCE.md section 75.


### 2026-11-27 — Per-core CPU dashboard and truthful history gaps (Batch #190)

- [x] Display current utilisation for each logical processor in a responsive, individually labelled grid.
- [x] Expose overall CPU, per-core CPU, system RAM, and aggregate battery meters with accessible labels and bounded values.
- [x] Preserve missing CPU, RAM, network, and system-disk samples as visible history gaps rather than converting unavailable readings into zero activity.
- [x] Keep the 36-sample in-memory history bounded and local; retain the existing 1.5-second sampling cadence and pause behavior.
- [x] Keep per-core readings separate from overall CPU utilisation and avoid exposing process or hardware identifiers.
- [ ] Validate per-core reporting and UI behavior on physical Windows, Linux, and macOS systems with different CPU topologies and power states.

See docs/HARDWARE-INTELLIGENCE.md section 76.


### 2026-11-27 — Per-processor CPU activity history (Batch #191)

- [x] Extend the bounded, in-memory monitor history with per-logical-processor CPU samples.
- [x] Display separate compact history rows for each reported logical processor, aligned to the shared 36-sample timeline.
- [x] Preserve unavailable or missing per-processor readings as visible gaps; retain genuine zero utilisation as a valid idle sample.
- [x] Keep processor labels generic (C1, C2, etc.), avoid hardware identifiers, and retain opt-in/local-only sampling.
- [ ] Validate history alignment, processor hot-plug/topology changes, and display scaling on physical Windows, Linux, and macOS devices.

See docs/HARDWARE-INTELLIGENCE.md section 77.


### 2026-11-27 — CPU history topology-change handling (Batch #192)

- [x] Record the logical-processor count with each bounded CPU history sample.
- [x] Render a visible gap for historical per-processor samples captured under a different logical-processor count, rather than mapping old ordinals onto a changed topology.
- [x] Preserve real zero readings and missing-value semantics.
- [x] Keep labels ordinal and generic; do not imply stable physical-core identity.
- [ ] Validate CPU topology changes and hot-plug behavior on physical devices; the OS-provided logical-processor ordering is not a persistent hardware identity.

See docs/HARDWARE-INTELLIGENCE.md section 78.


### 2026-11-27 — System Monitor history controls and sampling integrity (Batch #193)

- [x] Timestamp each in-memory history point using the operating-system snapshot's capture time.
- [x] Show the actual elapsed span represented by the retained history instead of assuming every poll completed exactly on schedule.
- [x] Prevent overlapping live-metrics requests when a platform collector takes longer than the nominal 1.5-second polling interval.
- [x] Add a user-controlled Clear history action that clears only the temporary chart history; it does not stop monitoring or alter current metric cards.
- [x] Keep the history bounded to 36 points and local to the active app session; do not persist or transmit samples.
- [ ] Validate timing, slow collector behavior, and history controls on physical Windows, Linux, and macOS devices.

See docs/HARDWARE-INTELLIGENCE.md section 79.


### 2026-11-27 — System Monitor measurement availability panel (Batch #194)

- [x] Add a concise per-source availability overview for CPU, system RAM, network, midSunday process memory and disk I/O, system-wide disk I/O, storage capacity, GPU telemetry, and battery.
- [x] Translate collector status codes into clear user-facing states while preserving distinctions between available, partial, warming-up, unavailable, and unsupported measurements.
- [x] Show only aggregate counts and generic labels; do not expose network interface names, mount paths, GPU vendor/device identifiers, or battery identifiers.
- [x] Explain that warming-up and unavailable readings are not zero activity.
- [x] Keep the overview tied to the live snapshot and update it with the existing opt-in monitor polling.
- [ ] Validate status presentation for no-battery desktops, unsupported GPU drivers, permission-limited systems, and devices with multiple adapters/volumes.

See docs/HARDWARE-INTELLIGENCE.md section 80.


### 2026-11-27 — Linux Intel GPU telemetry (Batch #197)

- [x] Add Intel DRM GT engine-busy sampling when the kernel exposes cumulative per-engine busy nanosecond counters.
- [x] Calculate utilisation from two bounded local samples and report the busiest individual engine per adapter rather than summing concurrent engines.
- [x] Keep generic adapter labels and avoid returning PCI IDs, engine names, or other hardware identifiers.
- [x] Reuse optional driver-published dedicated-memory counters only when both used and nonzero total capacity are available.
- [x] Add regression tests for idle readings, valid rates, counter resets, out-of-range deltas, and zero elapsed time.
- [ ] Validate Intel integrated and discrete GPU counters across physical Linux devices, kernel versions, and driver generations.
- [ ] Add a separately validated NVIDIA collector; NVIDIA remains explicitly unsupported in this batch.

See docs/HARDWARE-INTELLIGENCE.md section 81.


### 2026-11-27 — System Monitor GPU memory history (Batch #200)

- [x] Add a separate bounded history row for per-adapter dedicated-GPU-memory allocation, using only driver readings with both used and non-zero total capacity.
- [x] Keep GPU memory allocation history separate from GPU compute/utilisation history; missing values remain visible gaps, never fabricated zeroes.
- [x] Reuse the existing 36-sample, timestamped, in-memory history and existing opt-in polling cycle; no extra sampling or persistence.
- [x] Label Linux Intel engine readings as the busiest sampled engine, Windows PDH readings as the busiest/peak engine, and AMDGPU's driver value as GPU busy.
- [ ] Validate adapter reordering/replacement and memory reporting on physical Windows, Linux, and macOS devices. macOS live GPU telemetry is still unsupported.


### 2026-11-27 — macOS Metal resource-allocation telemetry (Batch #203)

- [x] Add a macOS-only Metal collector using Apple's current allocated resource size and recommended working-set threshold.
- [x] Show Metal allocation separately from dedicated VRAM, with explicit labels and a distinct working-set history row.
- [x] Preserve actual allocation bytes if they exceed the recommended threshold; cap only the visual ratio.
- [x] Keep GPU utilisation unavailable on macOS; do not infer whole-system activity from resource allocation.
- [x] Avoid GPU workloads, extra polling, device identifiers, persistence, and network transfer.
- [ ] Validate readings on physical Apple silicon and Intel Macs, including multi-GPU configurations and OS versions.

See docs/HARDWARE-INTELLIGENCE.md section 83.


### 2026-11-27 — Linux NVIDIA NVML live GPU telemetry (Batch #204)

- [x] Add an optional Linux-only NVML wrapper that dynamically loads the NVIDIA driver-provided library at runtime; do not bundle NVML, install drivers, invoke nvidia-smi, or require CUDA Toolkit.
- [x] Cache NVML initialisation once per process and bound enumeration to 16 devices.
- [x] Read device-wide NVIDIA GPU utilisation and framebuffer used/total memory where supported.
- [x] Keep generic labels and avoid GPU names, UUIDs, PCI IDs, serials, and process lists.
- [x] Preserve unsupported, unavailable, partial, and out-of-range states without fabricating zero readings.
- [x] Reuse existing opt-in polling and bounded local history.
- [ ] Validate telemetry on physical NVIDIA GPUs across driver branches, GPU generations, hybrid systems, and permission configurations.

See docs/HARDWARE-INTELLIGENCE.md section 84.


### 2026-11-27 — Source-aware GPU System Monitor presentation (Batch #205)

- [x] Distinguish NVIDIA device-wide utilisation, AMDGPU driver GPU-busy percentage, Intel busiest-engine utilisation, and Windows peak-engine utilisation in card labels.
- [x] Label NVIDIA memory as framebuffer allocation and use a distinct FB MEM history row.
- [x] Update accessible history descriptions to name source-specific metrics.
- [x] Preserve opt-in polling, missing-sample gaps, generic labels, and bounded in-memory history without adding hardware reads or telemetry.

See docs/HARDWARE-INTELLIGENCE.md section 85.


### 2026-11-27 — Time-aware System Monitor history (Batch #206)

- [x] Add oldest, midpoint, and newest local-time anchors to the shared Recent activity timeline using existing sample timestamps.
- [x] Include the capture time and measured value/state in individual history-bar hover descriptions.
- [x] Add accessible descriptions to CPU, RAM, network, and system-wide disk history rows.
- [x] Explain that network and disk throughput bar heights are relative to each row's recent peak, not a shared absolute scale.
- [x] Preserve the existing 36-point bound, timestamped samples, polling cadence, missing-value gaps, opt-in behavior, and in-memory-only retention.
- [ ] Validate timestamp readability and history layout at narrow and wide window sizes on physical Windows, Linux, and macOS devices.

See docs/HARDWARE-INTELLIGENCE.md section 86.


### 2026-11-27 — Numeric System Monitor history summary (Batch #207)

- [x] Add a compact accessible table showing latest, average, peak, and valid-sample count for CPU and RAM.
- [x] Include aggregate network download/upload and system-wide disk read/write history.
- [x] Include per-adapter GPU utilisation and memory history with source-aware framebuffer/Metal labels.
- [x] Keep unavailable newest values explicit and exclude missing/non-finite readings from averages and peaks.
- [x] Derive all statistics from the existing bounded in-memory history; add no polling, hardware reads, persistence, identifiers, or network activity.
- [ ] Validate table readability, horizontal scrolling, and values on physical Windows, Linux, and macOS devices.

See docs/HARDWARE-INTELLIGENCE.md section 87.


### 2026-11-27 — System Monitor CSV export (Batch #208)

- Added an explicit local Export CSV action for the existing bounded Recent activity history.
- Export covers capture timestamps, total and per-logical-processor CPU, system RAM, aggregate network throughput, system-wide disk throughput, and per-adapter GPU utilisation/memory history.
- Preserved missing readings as empty cells and logical-processor topology changes as blank values; source-specific GPU labels distinguish Metal working-set and NVIDIA framebuffer allocation.
- CSV generation uses only the retained in-memory samples after an explicit user click; no new polling, persistence, identifiers, process enumeration, or network transfer.
- Added CSV quoting and spreadsheet-formula injection guards for text cells.


### 2026-11-27 — Timestamp-aware System Monitor sampling gaps (Batch #209)

- Added consistent horizontal spacing for delayed intervals across CPU, per-logical-processor, RAM, network, system-disk, GPU utilisation, and GPU memory history.
- Derive gap cues from actual adjacent capture timestamps; use the normal 1.5-second cadence as the baseline and show a cue when elapsed time exceeds 2.25 seconds.
- Add elapsed-time details to affected sample tooltips and explain the gap cue to users.
- Keep missing metric readings visible as gaps; do not interpolate, fabricate zeroes, or imply continuous measurements.
- Preserve the existing bounded 36-sample in-memory window, opt-in polling cadence, and local-only collection.
- Physical validation remains required for slow collectors, system sleep/resume, app load, and narrow-window readability across Windows, Linux, and macOS.

### 2026-11-27 — Per-logical-processor numeric history summary (Batch #210)

- [x] Extend the System Monitor Window summary with latest, average, peak, and valid-sample count for each logical processor in the newest retained sample.
- [x] Compare a processor index only against samples with the same logical-processor count as the newest sample; exclude mismatched topologies rather than implying processor identity across topology changes.
- [x] Preserve missing/non-finite readings as unavailable and exclude them from statistics.
- [x] Derive all rows from the existing 36-point in-memory history without adding polling, hardware reads, persistence, identifiers, or network activity.
- [ ] Validate topology changes and long-table usability on physical Windows, Linux, and macOS devices.

See docs/HARDWARE-INTELLIGENCE.md section 90.

### 2026-11-27 — Visible CPU-topology change disclosure (Batch #211)

- [x] Detect whether retained System Monitor samples contain different logical-processor counts.
- [x] Display a clear status note when CPU topology changed during the retained history.
- [x] Explain that per-processor statistics use only samples matching the newest processor count and exclude older-topology samples.
- [x] Keep the note hidden when processor count is consistent; derive it only from existing in-memory sample metadata.
- [ ] Validate hotplug/topology-change scenarios where supported and narrow-window readability on physical Windows, Linux, and macOS devices.

See docs/HARDWARE-INTELLIGENCE.md section 91.

