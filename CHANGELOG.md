# Changelog

All notable project changes are recorded here. The project is in foundation/prototype development; no application release has been made.

## 2026-11-25
- Added the local Awareness Learning Review queue with a versioned SQLite migration, Rust-enforced source/master/category/suppression checks, explicit approve/reject actions, and rejection fingerprints. Suggestions remain pending until user approval; the automatic local extractor is not connected yet.
- Fixed the legacy Awareness-settings regression test to update the migration-created default preference row instead of inserting a duplicate primary key.
- Added independently switchable local Awareness category exclusions in Settings; exclusions are validated, normalized, persisted backward-compatibly, and enforced in Rust by a guarded automatic-ingestion command. Added regression tests for master/source opt-in, category exclusions, deletion fingerprints, legacy settings, and invalid categories. No automatic extraction or activity collection is active yet.
- Hardened local Awareness screening for credential labels and token prefixes, email addresses, phone-like numeric sequences, long identifiers, payment details, private-message labels, address labels, and selected identity-document labels. Added Rust regression coverage; automatic learning remains disabled and screening is not comprehensive.

## 2026-11-24
- Added local Awareness relearning exclusions: deleted, replaced, and cleared item text is represented only by a SHA-256 fingerprint plus source/category/timestamp metadata; future ingestion can check exclusions before learning.
- Added source-record provenance to Awareness items, an exclusions review/clear control, and regression tests for normalized matching, deliberate re-entry, and no raw deleted text in the exclusion table.
- Added metadata-only Awareness activity history for manual add/edit/delete/clear/export events, with review and independent clear-history control; no Awareness item text or raw activity content is stored in the history.
- Added local Awareness item management: deliberate user-entry, categories, provenance metadata, edit/delete/clear controls, JSON export, and a separate UI list.
- Added conservative backend screening that rejects common password/credential labels, known token prefixes, email-like identifiers, and long numeric strings before saving Awareness items. This is a first-pass filter, not comprehensive sensitive-data detection; automatic learning remains disabled.
- Added the first local Awareness foundation: SQLite schema v3 preserves existing conversations and saved memories while creating separate future Awareness-item and metadata-only activity-history tables.
- Added persisted, independently controlled Awareness master, conversation-learning, and in-app-search-learning preferences; every switch defaults OFF, and source choices remain saved when the master switch is paused.
- Added a Settings → Awareness panel that clearly explains that automatic learning is not active yet; these controls save preferences only and do not collect conversation or search content.
- Added a local GGUF model library: manually register existing files, record source/licence metadata, compute streaming SHA-256 fingerprints, detect duplicate content, select registered models, and remove catalogue entries without deleting model files. No download or model copy is performed.
- Added device-local persistence for llama-server/model paths and inference tuning settings, restored them on startup, exposed context-length and CPU-thread controls, and validated their ranges. Settings contain paths and tuning values only, not chat content or credentials.
- Made the local HTTP cancellation test portable across operating systems: it accepts clean EOF and the standard TCP disconnect errors produced when a client cancels a response, while still failing on unexpected reads or I/O errors.
- Hardened conversation backup handling: imported titles are normalised using the same validation as newly created conversations, added round-trip and malformed-metadata tests, and exports now read from one consistent SQLite transaction so concurrent changes cannot create mixed snapshots.
- Added safe conversation backup import: validates version, size, counts, identifiers, timestamps, roles, and message limits; merges without overwriting existing chats, skips exact duplicates, remaps imported IDs, and commits atomically. Full database restore and cross-version restore remain future work.
- Added a versioned SQLite schema marker and a safe startup guard against opening databases from a newer unsupported schema.
- Added portable, versioned JSON export for all saved conversations and messages, plus a separate clear-all-history action in Settings. Clearing conversation history does not touch the future memory store; import/restore is intentionally not included yet.
- Added local SQLite conversation persistence: create/list/open/rename/delete conversations, save user and assistant messages, and reload the latest conversation at startup. The database lives in the OS application-data directory; real-device restart testing remains pending.
- Hardened multi-turn context handling: the frontend now excludes empty/system messages and sends the newest eligible history within the backend byte/count limits; current prompts are capped at 32 KiB in both frontend and Rust validation.
- Added multi-turn user/assistant conversation context to local llama.cpp requests, with strict history size/content validation and ordering tests.
- Added native file pickers for an existing llama-server executable and GGUF model using the Tauri dialog plugin with a narrowly scoped open permission. No model download or cloud service was introduced.
- Enabled automatic Windows, Linux, and macOS build checks on pushes to `main` and pull requests targeting `main`; all three independent workflows can run in parallel. Kept manual dispatch for on-demand package artifacts. Automatic CI runs do not upload build artifacts, limiting storage use; standard hosted runners are used, with no larger paid runners configured. The newly enabled automatic builds have not yet been validated.
- Added separate manual-only Linux and macOS prototype build workflows alongside the existing Windows workflow. Linux initially packages AppImage and Debian formats; macOS initially packages a DMG. Added cross-platform build documentation and updated the roadmap to make all three desktop platforms explicit targets, with Windows remaining the development priority. These workflows have not yet been run; successful CI builds will not replace later real-device testing.
- Updated the manual Windows workflow to retain the generated `package-lock.json` alongside the installer (when produced), with 7-day artifact retention, and clarified that the standard public-repository Windows runner is free while larger runners and artifact storage remain separate considerations.
- Replaced the static welcome screen with the first interactive companion workspace: conversation navigation, local-session message composer, suggested prompts, companion selection, privacy/system settings prototypes, responsive layouts, and a Rust-core status indicator. The UI clearly labels the chat as a prototype; it does not generate AI replies or persist messages. No model, voice, memory store, or hardware detector is connected.
- Added a manual-only GitHub Actions workflow to compile the prototype on a hosted Windows runner and retain the NSIS installer artifact for 7 days. Runs #1 and #2 stopped during setup because the pinned artifact-upload action v3 was deprecated; the workflow now pins the official v4 action. Run #3 passed dependency installation and the TypeScript/Vite frontend build, then stopped during Tauri packaging because the required Windows icon file was missing. The artifact upload succeeded and retained the generated npm lockfile. Added an initial Windows ICO asset in a follow-up commit; Windows installer validation is still pending. Further runs remain manual and subject to the zero-cost rule.
- Added instructions for manually starting the Windows build and retrieving its artifact.
- Completed the Tauri 2 versus Electron desk evaluation and recorded Tauri 2 as the provisional direction for the first Windows shell prototype; production approval remains pending measurements.
- Scaffolded the first Tauri 2 + Rust + React + TypeScript + Vite desktop shell under prototype/desktop-shell, including a harmless Rust status command and a minimal capability allow-list.
- The desktop application has not yet completed a Windows installer build or been launched on a Windows machine. No AI model, speech engine, cloud service, telemetry, or paid dependency has been added. The generated npm lockfile is available in the Run #3 artifact but has not yet been committed.

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


## 2026-11-24
- Added GGUF magic-header validation when registering models and before launching the local engine.
- Added on-demand SHA-256 and file-size verification for registered models, with clear missing/modified-file status.
- Added launch preflight so a registered model cannot silently change after it was fingerprinted; unregistered GGUF files remain usable with a clear no-saved-fingerprint status.
- Added Rust regression tests for invalid GGUF headers, modified/missing files, and unregistered model preflight.


## 2026-11-24
- Added explicit, user-controlled local memory with separate SQLite storage, edit/delete controls, persistent enable/disable preference, bounded prompt context, and versioned JSON export.
- Strengthened Girlfriend style to be loving, sweet, playful, romantic, and confidently flirty when welcomed; strengthened Boyfriend style to be loving, attentive, caring, and reassuring.
- Added database schema version 2 migration and regression tests for memory CRUD, limits, export, disabled-by-default behaviour, and preserving existing conversations.
- Added Girlfriend and Boyfriend companion styles with warm, affectionate, playful guidance and explicit boundaries against exclusivity pressure, isolation, guilt, and dependency-building.
- Added device-local persistence for selected companion preferences and custom style notes, plus create/edit/delete support for up to 20 named custom character profiles.
- Added a versioned, validated companion-settings store with Rust tests for persistence, malformed files, duplicate names, invalid selections, and size limits. Real desktop restart testing remains pending.
- Connected Best friend, Mentor, Professional, and Custom character choices to actual local system prompts.
- Added editable custom style notes, bounded to 4,000 characters in the UI.
- Added Rust-side 8 KiB system-prompt validation and ensured system instructions precede saved conversation context without entering the transcript.
- Added regression tests for system-prompt size limits and system-message ordering.


## 2026-11-24
- Added optional, device-local preferred-name settings with an opt-in for natural use in conversations.
- Added Rust validation and persistence tests for the user profile.
- Clarified in Settings that 100 saved memories is the storage-entry cap, while only up to 20 recent entries / 4 KiB are included in a local model prompt when memory is enabled.
- Clarified that full memory storage never triggers automatic deletion and that disabling memory keeps saved entries stored but out of model context.
