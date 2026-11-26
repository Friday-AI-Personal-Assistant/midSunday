- Added a fresh GPU inventory and per-adapter reservation-headroom context to GGUF model-memory evidence, shown separately from host-RAM/KV-cache results and without claiming GPU model fit or offload support.\n- Added HIARM GPU memory reservation-headroom assessment with per-adapter critical/constrained bands, partial-inventory preservation, and a Settings summary; values remain advisory and are never pooled across GPUs.\n- Added mid-entry aggregate metadata-section enforcement for GGUF string arrays, with sparse-file regression coverage.
- Added GGUF string-payload boundary preflight before allocating string buffers, and tied metadata-only file-size reporting to the opened parser handle.
- Added a 64 MiB aggregate GGUF metadata-section work budget in both inspection paths, with checked per-entry accounting and sparse-file regression coverage.
- Reduced GGUF tensor-descriptor allocation overhead with inline fixed-size dimensions, incremental collection growth, and an 8 MiB aggregate tensor-name storage budget; added regression coverage.
- Corrected the Batch #151 aggregate GGUF metadata-key regression fixture so every key stays within the per-string limit while their combined size exceeds the aggregate limit; production safety limits are unchanged.
- Hardened GGUF tensor descriptor integrity by rejecting duplicate tensor data offsets, including mixed known and unsupported tensor types, before payload-size analysis; retained separate coverage for overlapping ranges at distinct offsets.
- Hardened GGUF metadata inspection against competing architecture-specific aliases: multiple keys that map to the same recognised field now fail explicitly instead of silently allowing the last value to win; unrelated vendor metadata remains supported.
# Changelog
- Fixed GGUF tensor element-count comparison compilation by storing an owned status string and updating all existing tensor-analysis evidence fixtures with the new fields.
- Added a GGUF tensor element-count cross-check using checked dimension arithmetic, with a separate comparison against optional file-reported parameter-count metadata; values are explicitly advisory and do not imply unique model parameters.
- Fixed Batch #133 TypeScript compatibility and null-safe formatting for optional KV-cache evidence values.
- Added a local, user-triggered GGUF host-memory evidence check comparing the theoretical one-sequence FP16 KV-cache estimate with fresh available system RAM; it explicitly excludes model-weight residency and does not claim model fit.
- Extended GGUF tensor overlap validation to check known payload ends against every next descriptor start, including unsupported tensor types whose payload sizes remain unknown.
- Reject overlapping known GGUF tensor payload ranges and check aggregate counters with checked arithmetic; descriptor storage order is validated independently of descriptor listing order.
- Fixed unsupported GGML tensor types to return an unknown payload size safely instead of entering block arithmetic with a zero block size.

## 2026-11-26
- Added user-triggered GGUF tensor descriptor analysis that groups tensor counts and known payload sizes by verified GGML type without loading model weights. Unsupported types remain explicitly unknown; stored payload size is not runtime RAM/VRAM.
- Added startup timing and baseline-relative memory-growth diagnostics for the managed llama-server process. Settings now shows elapsed time to readiness, the first valid resident/virtual sample, observed peaks, and signed peak-minus-initial growth; values remain local and informational, not pure model-load benchmarks or model-only allocations.
- Added bounded startup-phase peak memory sampling for the managed llama-server child while the local engine becomes ready, retaining observed resident/virtual maxima and displaying the latest successful startup report in Settings. Values are process-wide observed maxima, not guaranteed peaks or model-only memory.
- Fixed the generation command's Rust return type to forward the peak/boundary memory report expected by the frontend.
- Added request-scoped peak resident/virtual memory sampling for the managed llama-server process during generation, using targeted 250 ms samples plus before/after readings. Reported peaks are observed maxima, not guaranteed true peaks.
- Added a before/after memory boundary report around each successful local generation, including signed resident/virtual deltas for the managed llama-server child. These point-in-time samples are not peak or model-only memory measurements.
- Added an explicit, local-only memory snapshot for the single managed llama-server child process, reporting resident and virtual memory without enumerating unrelated processes. The snapshot is informational and does not represent model-only or GPU memory.

All notable project changes are recorded here. The project is in foundation/prototype development; no application release has been made.

## 2026-11-25
- Added an explicit GGUF-derived KV-cache estimate for the configured context length, with separate key/value figures, FP16 assumptions, checked arithmetic, missing-metadata handling, and regression tests. The estimate is not total model RAM/VRAM or a model-fit guarantee.
- Added explicit, bounded GGUF v2/v3 metadata inspection to the local model library, reporting architecture, model name, file type/known quantisation, parameter count, context and architecture dimensions where present. Inspection is local, read-only, runs off the UI thread, and never loads tensor weights or claims compatibility.
- Added HIARM matching of the configured GGUF model file to its containing storage volume, showing free-space headroom and explicit unresolved-path states without reading model contents or transmitting the path.
- Added an on-demand midSunday process-memory snapshot showing resident physical memory and virtual address space, with explicit unavailable states and a privacy boundary that queries no other processes.
- Added runtime-detected CPU instruction-set inventory for x86/x86-64 (SSE, AVX, AVX2, FMA) and AArch64 NEON, displayed in System & hardware with explicit runtime-compatibility caveats.
- Added a preliminary local HIARM capability assessment based on installed RAM and logical CPU count, including an independent low-current-memory warning and explicit limitations. It is informational only and does not select models or change settings.
- Added the first local HIARM hardware inventory baseline for OS, CPU, RAM, and storage, with on-demand refresh in Settings. GPU detection and adaptive recommendations remain future work.
- Recorded the internal commercial licensing plan: one-time lifetime-use target, six months of included updates, paid upgrades thereafter, verified website checkout, email/recovery delivery, one active device, transfer/revocation, and strict separation from private app data. Pricing, offline verification policy, payment/email providers, and final legal terms remain open.

## 2026-11-25
- Confirmed Windows 10 and Windows 11 as the intended Windows support targets; Windows 10 version 22H2 is the initial compatibility baseline, with earlier builds subject to dependency and real-device validation. Added planned diagnostics covering startup/runtime health, missing prerequisites, crash/error dialogs, a Diagnostics centre, local-only-by-default logs, and user-approved diagnostic export.
- Recorded cloud-provider migration and disconnect behaviour, separate remote-backup deletion controls, and encrypted-backup default-password rotation. Password changes affect future backups only; existing archives retain their original password/recovery-key protection, and a recovery key can authorise changing the default after a forgotten password.
- Corrected the legacy unversioned-database migration test to expect schema version 8 after the saved-memory FTS5 migration; the migration and data-preservation behavior were already passing.
- Added SQLite FTS5 indexing and local search for saved memories. A schema v8 migration builds the index for existing records and installs insert/update/delete triggers to keep it synchronized. Search treats input as bounded literal terms rather than raw FTS syntax, requires all terms to match, orders results by relevance and recency, and returns 25 results per page. Added regression coverage for index lifecycle, updates, deletion, clearing, pagination, and query safety. Search runs entirely in the local database.
- Added validated saved-memory JSON import as a non-destructive merge. Imports use a versioned format, enforce file/item limits and content/timestamp validation, run transactionally, remap database IDs, preserve source timestamps, skip exact duplicates, and never overwrite existing memories. Added regression tests for successful/repeated imports, malformed files, unsupported versions, duplicate IDs, and all-or-nothing validation. The convenience JSON format is not encrypted and remains separate from the planned encrypted Backup & Restore system.
- Updated the existing memory-size/export regression test to reflect removal of the old 100-entry cap; it now verifies 125 entries can be saved, exported, and cleared while empty and oversized entries remain rejected.
- Removed the arbitrary 100-entry cap from explicitly saved memories. Each entry retains its existing content validation, while the separate 20-entry / 4 KiB prompt-context budget remains unchanged. Added a Rust regression test proving more than 100 memories persist without automatic eviction.
- Added source-linked controls for saved conversation-derived Awareness facts. Users can review the original message and explicitly delete only the linked saved fact while preserving the conversation. The Rust transaction verifies the item/message relationship and source existence, records metadata-only history, and blocks matching relearning with a one-way fingerprint. Added regression coverage for mismatched-source rejection and successful deletion.
- Added an OFF-by-default control for using approved Awareness in local chat. When explicitly enabled alongside the Awareness master switch, the app retrieves only keyword-relevant approved items (maximum 12 entries / 2 KiB), frames them as untrusted reference data, and leaves new-learning permissions independent. Added Rust tests for opt-in gating and relevance filtering.
- Recorded the product decision for optional Backup & Restore: manual, daily, and weekly schedules; user-selectable backup contents; simple local-only and password-protected encrypted backup modes; versioned archives and safe restore; optional Google Drive storage with narrow Drive-only permissions and no access to midSunday itself. Cloud uploads must always be encrypted locally, and backup secrets must never be sent to Google.
- Extended the read-only source-evidence viewer to approved/saved conversation-derived Awareness items. Added Rust regression coverage confirming evidence lookup returns only existing user messages and rejects assistant messages, missing IDs, and invalid IDs.\n- Added a read-only source-evidence viewer to Learning Review. Conversation-derived suggestions can open the exact original user-authored message through a Rust-validated local lookup; assistant messages and missing/deleted sources are not exposed. The source is displayed locally and is never copied into activity history.
- Added an explicitly user-triggered local conversation analysis action: only the currently open conversation is sent to the already-running loopback model, source-linked screened candidates enter Learning Review, and nothing becomes permanent without approval. Added strict JSON parsing, source-message validation, duplicate suppression for pending/saved facts, UI consent/status messaging, and regression tests. No background analysis, in-app-search extraction, or cloud transfer was added.
- Corrected the remaining Awareness ingestion fixture to assert the actual database-generated source-message ID instead of a hard-coded ID; source provenance validation remains unchanged.
- Fixed four legacy Awareness tests to create real conversation messages and use their persisted IDs, preserving strict source-provenance validation.
- Hardened Awareness provenance and review actions: conversation-derived entries require an existing source message; approval and rejection use atomic transactions; settings and suppression are rechecked at approval. Added regression coverage for missing/deleted sources and failed approvals.
- Corrected the legacy unversioned-database migration test to expect schema version 7 after the source-lifecycle migration; the migration itself and data-preservation assertions were already passing.
- Added Awareness source-lifecycle integrity: deleting one conversation or clearing all conversations now atomically removes its linked learned Awareness items and pending suggestions, records metadata-only deletion history, and preserves unrelated manual Awareness, saved memories, and other chats. Added schema-v7 provenance indexes and regression tests.
- Corrected the legacy unversioned-database migration test to expect schema version 6 after the new Awareness review-queue migration; all original conversation data assertions remain unchanged.
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


## 2026-11-25
- Added Windows DXGI process-specific local and non-local GPU memory budget snapshots, including current usage, OS budget, reservation headroom, and explicit partial/unavailable states.
- Displayed GPU memory data in Settings → System & hardware as a user-triggered point-in-time snapshot; the check does not reserve memory or alter GPU settings.
- Added serialization regression coverage for the GPU memory-budget payload. Real-device validation against Windows telemetry remains pending.

- Improved Windows GPU memory-budget diagnostics: label the sampled DXGI node (currently node 0), preserve independent HRESULT failure codes for local/non-local segments, and display these details in System & hardware. Multi-node enumeration and real-device validation remain pending.

- Added a collection timestamp to hardware snapshots and show the actual localised last-checked time in Settings → System & hardware; the timestamp is diagnostic metadata only and is not uploaded or persisted as device history.


- Added local HIARM storage headroom indicators with safe free-space percentages, Healthy/Low/Critical/Unknown classifications, accessible used-space meters, and regression tests. Model-specific storage fit remains future work.
- Fixed nullable GGUF parameter-count and context-length rendering so the shared frontend TypeScript build succeeds on Windows, Linux, and macOS.
- Extended the GGUF inspector with selected GQA, key/value head, RoPE, MoE expert, quantisation-version, and alignment metadata; added UI fields and synthetic regression coverage.

- Fixed the startup-memory diagnostics test fixture to initialize the new report field and removed the resulting Rust warnings; the runtime behaviour is unchanged.

- Added a GGUF-reported context-length advisory to KV-cache and host-memory evidence reports, distinguishing within/above/missing metadata without treating file metadata as a hard runtime limit; added regression coverage.

- Fixed TypeScript null-safety in the GGUF-reported context display by using optional chaining and a nullish fallback for unavailable metadata.

- Added bounded GGUF tensor-name parsing and structural validation for empty, truncated, invalid UTF-8, and duplicate descriptor names, with regression coverage. Tensor weights remain unread.

- Added GGUF metadata-key uniqueness checks to both inspection paths, rejecting duplicate known or unknown keys while preserving distinct unknown metadata and adding regression coverage.

- Hardened GGUF metadata parsing in both inspection paths: reject empty metadata keys and non-canonical boolean encodings (only 0/1 accepted) for scalar values and array elements; added valid/invalid regression coverage. Tensor weights remain unread.

- Improved GGUF metadata array handling: fixed-width arrays now use checked bulk skipping, boolean arrays are validated in bounded chunks, and truncated numeric arrays are rejected with regression coverage in both inspector paths.

- Fixed the Batch #143 GGUF array regression fixture to include required 32-byte tensor-data alignment padding, so the valid zero-tensor file is accepted by both inspection paths while truncated numeric arrays remain rejected.


- Hardened recognised GGUF metadata fields: string fields reject non-string values, numeric fields reject incompatible/negative encodings, 32-bit fields reject overflow, and tensor analysis validates alignment and parameter-count types consistently. Unknown metadata remains supported; regression tests cover malformed encodings.

- Hardened GGUF architecture metadata consistency: recognised architecture-specific fields must match the declared `general.architecture` namespace regardless of metadata order, and are rejected when the declaration is missing. Added regression coverage; tensor weights remain unread.

- Unified the GGUF tensor-count safety limit across metadata inspection and tensor-storage analysis using a shared one-million-descriptor cap, with regression coverage for both entry points.


- Hardened GGUF tensor-storage analysis with a checked minimum-descriptor-size preflight, rejecting impossible header counts before allocating descriptor and tensor-name collections; added regression coverage.


- Added an 8 MiB aggregate GGUF metadata-key memory budget to both inspection paths and removed metadata-entry-count-based preallocation; oversized key sets now fail safely with regression coverage.


- Hardened GGUF tensor data-offset boundaries: tensor analysis now uses the opened file handle's length and rejects offsets at or beyond EOF, including unsupported tensor types; added regression coverage while preserving unknown payload-size reporting.


- Bounded GGUF tensor-storage inspection memory with a checked 64 MiB aggregate budget for descriptor records and duplicate-name indexing, applied before collection allocation; retained separate tensor-name and tensor-count limits and added boundary regression tests.

- Corrected the GGUF string-array metadata-budget sparse-file regression fixture to include the full array header and string-length field; production parser limits and behaviour are unchanged.

- Fixed the GGUF string-array metadata-budget regression fixture to reopen its sparse test file read-only before parser inspection; production parser behaviour is unchanged.

- Hardened GGUF string-array scanning so a declared string payload that would exceed the aggregate 64 MiB metadata-section budget is rejected before allocating or reading that payload; added regression coverage for the preflight.

- Hardened both GGUF metadata inspectors to reject a metadata key whose declared length exceeds the remaining aggregate 8 MiB key budget before allocating or reading its payload; added a cursor-position regression test.

- Hardened GGUF metadata-section accounting so metadata keys and scalar string values are rejected before allocation/read when their declared payload would cross the aggregate 64 MiB section budget; added cursor-boundary regression tests.

- Added preflight checks for fixed-width numeric and boolean GGUF metadata arrays so declared payloads that cross the 64 MiB section budget are rejected before seeking or reading; added cursor-boundary regression tests.

- Hardened GGUF string parsing so the complete 8-byte string-length prefix must fit inside the remaining aggregate metadata-section budget before it is read; added cursor-boundary regression tests for metadata keys and scalar strings.
- Corrected the existing GGUF string-array section-budget regression test to expect the length prefix to remain unread when it crosses the metadata boundary; parser behavior unchanged.

- Hardened GGUF metadata-section accounting further: both inspectors now preflight metadata value-type tags, fixed-width scalar payloads, and complete array headers before reading bytes that would cross the 64 MiB boundary; added sparse-file cursor-boundary regression coverage.

- Corrected the Batch #169 sparse-file regression fixture to restore its intended cursor position after reopening; parser changes remain unchanged.

- Unified GGUF declared tensor-alignment validation across metadata inspection and tensor-storage analysis; malformed values (zero, non-power-of-two, or above 4096 bytes) are now rejected consistently, with valid/invalid cross-inspector regression coverage.

- Strengthened GGUF architecture metadata validation: reject blank architecture identifiers, zero-valued dimensions/counts, KV-head counts above total heads, and expert-used counts above total experts; added focused regression coverage.

- Corrected the GGUF architecture-semantic regression fixture to declare `general.architecture = "llama"` before testing `llama.*` fields; production validation remains unchanged.


- HIARM now reduces its suggested starting profile when current available RAM is under the conservative headroom threshold, while preserving the installed-memory tier and making no automatic settings changes. Added cross-platform regression coverage.


- Added a derived GPU inventory assessment to HIARM snapshots, distinguishing hardware adapters from software renderers and reporting conservative per-adapter memory summaries without claiming inference compatibility or pooling GPU memory.


- Added the approved Live System Monitor feature to the roadmap: a local-first, cross-platform dashboard for CPU, RAM, GPU, storage, network, battery, and midSunday process usage, with bounded sampling, honest unsupported states, privacy safeguards, and real-hardware validation requirements. Implementation is scheduled after Batch #177.


- Began the Live System Monitor implementation with an opt-in, local CPU/RAM/process-memory dashboard, bounded recent-activity bars, and a stateful Rust sampler. GPU live utilisation, network, battery, and live disk activity remain planned follow-up work.


- Extended the Live System Monitor with aggregate system-wide network download/upload rates and midSunday-process-only disk read/write rates. Added bounded network history and counter-reset/warm-up handling; interface names and other process details remain uncollected.
