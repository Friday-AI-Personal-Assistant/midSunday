## 2026-11-28 — Batch #328
- Enforced the existing application resource ceilings at the `RuntimeConfig` boundary as well, so direct backend callers cannot request more than 32,768 context tokens or 256 CPU threads.
- Added exact-boundary and over-limit tests for both settings; loopback validation and existing defaults are unchanged.

## 2026-11-28 — Batch #327
- Bounded local model catalogue loading to 64 MiB, checking that the catalogue is a regular file and enforcing the limit again while reading to protect against file growth after the metadata check.
- Prevented writing a catalogue larger than the same limit and added regression coverage for oversized catalogue files and directory paths.

## 2026-11-28 — Three-run batch cycle: Runs #312–#314

- **Run #312:** Changed local inference settings persistence to write and sync a temporary file in the destination directory, then atomically replace the saved settings. Added a regression test for replacing and reloading existing settings.
- **Run #313:** Applied the same atomic-replacement guarantee to the local GGUF model catalogue, preventing a failed save from first deleting the previous catalogue. Added a regression test for multiple updates and reload.
- **Run #314:** Expanded first-launch setup readiness coverage for directory paths and invalid or oversized paths. These checks remain read-only and never launch the selected executable.

## 2026-11-28 — Restore candidate regression fix
- Fixed guarded candidate copying so the backup-only v2 manifest is removed from the live application database after SQLite online-backup completes. This prevents subsequent verified snapshots from failing while creating a fresh backup manifest.
- Corrected the mismatch regression test to use different aggregate inventories rather than different record text with identical counts.
- Kept all restore functionality backend-only; no command or UI is enabled.

## 2026-11-28 — Batch #293
- Added guarded live-connection copy and verification primitives for a previously staged, fully verified restore candidate.
- The copy uses SQLite's online-backup API against the existing live connection while the exclusive database guard is held; a separate verification snapshot runs the complete SQLite/manifest verifier and compares schema plus all aggregate content counts.
- Added tests for replacing a test database under the runtime/database gates, matching the candidate inventory, and rejecting a mismatched live inventory without changing it.
- These are backend primitives only. No restore command or UI is enabled, and the primitives are not yet wrapped in the encrypted safety-archive/rollback/crash-recovery transaction.

## 2026-11-28 — Batch #292
- Added an encrypted-backup candidate staging pipeline that decrypts a user-selected `.msbackup` only inside a private temporary workspace, then uses the isolated SQLite online-backup candidate verifier from Batch #291.
- The decrypted source workspace is explicitly closed before the candidate is returned; cleanup failures are surfaced and the candidate is discarded rather than silently retained.
- Added tests for encrypted candidate fidelity, temporary workspace cleanup, live database non-mutation, and wrong-password rejection without changing the encrypted source.
- This remains a backend-only preparation primitive. It does not replace the live database, initiate rollback, or enable restore in the UI; crash recovery and physical-device validation remain incomplete.

## 2026-11-28 — Batch #291
- Added SQLite's online-backup API as an explicitly enabled rusqlite capability and a private restore-candidate preparation primitive.
- The primitive first fully verifies a v2 source backup, copies it into an isolated temporary SQLite database, runs the complete integrity/foreign-key/schema/manifest verifier again, and compares every aggregate inventory and manifest field with the source.
- The candidate owns a private temporary workspace and is removed when dropped. Candidate preparation never opens or mutates the live ConversationStore connection and is not wired to a restore command or UI.
- Added regression tests for successful online-copy fidelity, candidate cleanup, live-database non-mutation, and invalid-source rejection. Actual live restore, rollback, crash recovery, and physical-device validation remain incomplete.

## 2026-11-28 — Batch #290
- Added a separately callable encrypted pre-restore safety-archive operation. It acquires the runtime operation gate first and then the exact live SQLite connection lock used by normal database operations.
- While both gates are held, it snapshots the current database through the restore guard, encrypts the snapshot with the existing Argon2id/XChaCha20-Poly1305 format, decrypts it into a private verification workspace, and checks SQLite integrity, foreign keys, schema, and the v2 manifest before publishing.
- Publishes with an atomic no-overwrite hard link and reports temporary plaintext cleanup failures. Added tests for successful verification, destination preservation, and fail-fast runtime/database contention.
- The operation is registered as a backend command but is not exposed in the UI or invoked by the restore preview. Actual restore, rollback, and crash recovery remain unavailable.

## 2026-11-28 — Batch #289
- Fixed the cross-platform desktop packaging failure caused by npm resolving @tauri-apps/api 2.13.0 alongside the Rust Tauri crate 2.12.x.
- Pinned the JavaScript Tauri API to 2.12.0 so its minor release line matches the Rust-side Tauri dependency. This prevents npm's broad ^2.0.0 range from silently advancing the API to an incompatible minor release during CI installs.
- No application behaviour or backup/restore capability changed.

## 2026-11-28 — Batch #286
- Added a typed exclusive database restore guard that can create a consistent, manifest-bearing, fully verified SQLite safety snapshot while retaining the already-held live connection lock. The guarded path avoids recursively acquiring the same mutex and therefore avoids self-deadlock during the pre-restore safety-copy stage.
- Refactored ordinary local backup creation to share the same snapshot implementation. Added a regression test that creates and verifies a safety snapshot under the restore guard and confirms live data remains unchanged. The snapshot is still plaintext SQLite at this stage; encrypted safety-archive orchestration and actual restore remain unavailable.

## 2026-11-28 — Batch #285
- Added a fail-fast exclusive database restore lock using the exact mutex shared by every ConversationStore read and write. A restore can now reserve the live SQLite connection without replacing the file underneath it; normal database operations remain blocked until the guard is released.
- Added regression tests proving restore refuses a busy database and that holding the restore guard excludes concurrent normal database work. This is a coordination primitive only; no restore operation or UI is enabled.

## 2026-11-28 — Batch #284
- Added a non-blocking exclusive runtime-operation gate for future database restores. It rejects starting, ready, and generating states and shares the same mutex as runtime start, generation, stop, and recovery so a new runtime operation cannot race a restore holding the guard.
- Added regression tests for allowed and rejected runtime states, fail-fast busy behaviour, and exclusive guard lifetime. Restore remains unavailable; no database data is changed.

## 2026-11-28 — Batch #283
- Added a dedicated encrypted Backup & Restore safety design covering live SQLite connection constraints, explicit confirmation, a user-selected encrypted pre-restore safety archive, runtime coordination, database locking, destination verification, rollback, crash recovery, failure-injection tests, and physical-device validation.
- Updated the roadmap's current status and immediate next actions to reflect the implemented local runtime and backup work. Actual restore remains unavailable; this batch changes documentation only.

## 2026-11-28 — Batch #281
- Corrected the foreign-key corruption regression fixture to disable SQLite write-time foreign-key enforcement only on its isolated test connection, allowing the read-only verifier to be tested against a deliberately damaged backup. Normal application connections remain unchanged.

## 2026-11-28 — Batch #279
- Strengthened read-only database backup verification with SQLite's full integrity check and an explicit foreign-key relationship check, in addition to schema, required-table, and manifest/inventory validation.
- Added regression tests for valid full-integrity backups and backups containing broken conversation-message relationships. Verification remains read-only; actual restore is still disabled.

## 2026-11-28 — Batch #278
- Fixed a command-registration regression from Batch #277: restored the existing encrypted-backup verification Tauri command while retaining the new read-only restore preview command.
- No backup format, restore behaviour, or personal data handling was changed by this repair.

## 2026-11-28 — Batch #277
- Added a read-only encrypted-backup restore impact preview. It authenticates and verifies the selected archive, then compares aggregate record counts with the current local database.
- The preview exposes no personal record contents or identifiers and does not modify the backup or live database. It explicitly explains that a future restore would replace current database content; actual restore remains disabled pending a proven live-connection, safety-copy, rollback, and crash-recovery design.

## 2026-11-28 — Batch #276
- Expanded encrypted-backup regression coverage for empty files, exact chunk boundaries, modified headers, oversized chunk declarations, trailing bytes, and destination no-overwrite guarantees.
- Confirmed malformed encrypted containers fail without leaving partial plaintext output. Restore and cloud backup remain unavailable.

## 2026-11-28 — Batch #270
- Added a separate password-protected .msbackup format using Argon2id and XChaCha20-Poly1305 authenticated encryption, processed in bounded 1 MiB chunks.
- Encrypted backups are decrypted to a temporary file and checked with the existing SQLite integrity and manifest verifier before publication. Existing .msbackup destinations are never overwritten.
- Added password-based encrypted-backup verification; the selected encrypted file remains unchanged and temporary plaintext is removed after checking.
- Kept the original unencrypted .sqlite3 backup option separate. Passwords are not saved; losing one means the encrypted backup cannot be recovered. Restore and cloud backup remain unavailable.

## 2026-11-28 — Batch #247
- Hardened diagnostic storage classification: zero-capacity volumes now report unavailable instead of a false critical warning, and inconsistent available-space values are safely clamped. Added threshold and Windows drive-path regression coverage.

## 2026-11-28
- Fixed Windows diagnostic volume matching so drive roots and nested paths use normalized separators and case-insensitive component boundaries. Also resolve the nearest existing ancestor for nested app-data/model locations that have not been created yet. Added regression tests; no paths or drive identifiers are exposed.

## 2026-11-28
- Added runtime health detection for unexpected termination of the single managed llama-server child. Existing status refresh now marks a ready/generating session as failed if its child has exited, clears stale active-session details, and presents a plain-language recovery hint. No unrelated processes are enumerated, no process ID is exposed, and no additional polling timer is introduced.

## 2026-11-28
- Added an active local runtime session summary with model file size, configured context length, CPU-thread setting, and monotonic uptime. Session details are cleared after a successful stop or failed startup, and the status response deliberately omits local file paths and process IDs. Model file size is explicitly described as storage size rather than a memory-fit estimate.

## 2026-11-28
- Fixed a JSX closing-brace error in the Local AI settings results panel. Also ensure an invalidated in-flight setup check cannot leave the busy state stuck, and clear old results when selecting a model from the library.

## 2026-11-28
- Added a user-triggered local setup file check for the llama.cpp server and GGUF model. The server check validates file presence/type and platform executable permissions without launching it; the model check validates a regular .gguf file, non-empty data and GGUF header. Results appear in first-run onboarding and Local AI settings, run off the UI thread, and are invalidated when paths change. Existing pre-launch SHA-256 verification remains authoritative for registered models.

- Added Windows live GPU monitoring through PDH GPU Engine and Dedicated Usage counters, generic adapter grouping, peak-engine semantics, optional DXGI memory capacity, and explicit partial/unsupported reporting.
- Added the first live GPU monitoring slice: per-adapter AMDGPU utilisation and VRAM allocation on supported Linux systems, explicit unsupported states elsewhere, generic adapter labels, safe bounds, and local-only sampling.
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


- Added cross-platform battery telemetry to the opt-in System Monitor, including energy-weighted charge aggregation and combined battery state. No battery identifiers are collected; no-battery and unavailable states remain explicit.


- Added system-wide disk read/write throughput to the opt-in System Monitor, distinct from midSunday process I/O, with bounded history and no exposed volume or device identifiers.

- Added opt-in, cross-platform launch-at-sign-in registration for Windows, Linux, and macOS. The setting reads the operating-system state, confirms changes, reports failures, and remains disabled unless the user enables it.
- Added separate Pause sampling, Resume sampling, and Stop & clear controls to the local System Monitor. Pausing retains the bounded in-memory history without collecting additional readings; navigating away from Settings still pauses sampling automatically.


## 2026-11-27
- Added five built-in Appearance & Themes choices (Sunday Green, Midnight Blue, Lavender Dream, Rose Blossom, Warm Sand), plus Follow system / Light / Dark modes. Preferences apply immediately and are stored locally on the device; no account, network connection, or paid service is used.


## 2026-11-27
- Added local accessibility preferences: Standard/Large/Largest interface text, Comfortable/Compact settings spacing, and Reduce motion. Existing theme preferences now load before being written, preventing first-launch defaults from overwriting saved choices.


## 2026-11-27
- Added a first-launch setup guide to the empty conversation workspace. It reflects the configured local llama.cpp executable, GGUF model path, and live engine readiness, with direct links to local AI settings and companion selection. The guide explains that compatible files are user-supplied and introduces no downloads, account, cloud dependency, or network check.


- Batch #238: Kept runtime status reads non-blocking during active inference. The UI no longer waits for the session mutex held by streaming generation; idle ready-state refreshes continue checking the managed child, while generation failures flow through the existing result/error path. Added a lock-contention regression test. Physical-device validation remains pending.


- Batch #239: Added Settings → Diagnostics with an explicit read-only local health check for app version/platform, SQLite integrity and schema version, registered model count, and managed engine state. The check does not read conversation or memory content, open model files, expose local paths, repair data, or send telemetry. Added a Rust database-health regression test.
\n\n## 2026-11-28\n- Expanded the local Diagnostics health check to inspect configured llama.cpp executable readiness and the selected GGUF file. Registered models are checked against their saved size and SHA-256 fingerprint; unregistered files receive only a GGUF header check. Slow file verification runs off the UI thread, remains read-only, and never launches binaries or exposes paths.\n

## 2026-11-28 — Batch #241
- Fixed the escaped-newline syntax error in the ModelCatalog declaration that caused #240 to fail on Windows, Linux, and macOS. Added a shared-clone regression test for the background model-verification handle.


## 2026-11-28 — Batch #242
- Added an explicit, user-initiated JSON export for the current sanitized Diagnostics health-check snapshot using the native save dialog. The report contains only displayed health-check fields and a timestamp; it excludes conversations, memories, local paths, model contents, and credentials. No automatic upload or sharing.
- Added Rust regression tests for report contents and file-extension validation.


## 2026-11-28 — Batch #243
- Added a privacy review step before diagnostic report export. Users can inspect every included category and value, see the timestamp behavior, and review explicit exclusions before the save action becomes available. Running a fresh health check resets review approval.


- Added privacy-preserving Diagnostics checks for free and total space on the app-data and configured model volumes, with conservative healthy/low/critical states and no mount-path or drive-identifier disclosure.


- Fixed cross-platform Rust compilation for storage-volume diagnostics and completed the diagnostic export fixture with the new storage fields.

## 2026-11-28 — Batch #248
- Corrected the diagnostic storage-capacity regression fixture to use realistic GiB-scale values. The test still verifies that reported free space is clamped to total capacity, while respecting the absolute low-space thresholds.

## 2026-11-28 — Batch #249
- Corrected the remaining storage diagnostics test expectation: 9 bytes of free space is critical under the documented absolute 2 GiB threshold, not low. GiB-scale fixtures continue to cover the low and percentage boundaries.


## 2026-11-28 — Batch #250
- Added a root-level React error boundary so render/lifecycle failures display a readable, theme-aware recovery screen instead of blanking the desktop window.
- Added an explicit reload action and guidance to revisit Settings → Diagnostics if the issue repeats.
- Kept exception messages and stack traces out of the recovery screen; no error details are persisted or exported. Native crashes, Rust panics, and asynchronous event-handler failures remain separate roadmap work.


## 2026-11-28 — Batch #251
- Fixed the root React entry point to mount App inside AppErrorBoundary. Batch #250 imported the boundary but did not wrap the actual StrictMode root, causing TypeScript's unused-import check to fail on all platforms.


## 2026-11-28 — Batch #252
- Added a collection timestamp to diagnostic snapshots and a versioned JSON export schema (`formatVersion: 1`). The Settings view and reviewed preview distinguish the health-check time from the later report-save time.
- Starting a new health check now immediately invalidates the prior snapshot, review approval, and export status. A failed retry can no longer leave an older report available as though it reflected the latest check.
- Added regression coverage for report version and snapshot timestamp fields.


## 2026-11-28 — Batch #253
- Expanded diagnostic-export privacy regression coverage: tests now assert the exact top-level and snapshot JSON field allowlists.
- Added a regression test using a unique private path marker to ensure server/model readiness details and app-data/model-volume diagnostics do not echo configured paths into serialized diagnostic snapshots.
- Confirmed private conversation and memory categories remain absent from the report schema.


## 2026-11-28 — Batch #254
- Added a plain-language Diagnostics health overview with an overall state and a short list of findings that need attention or are informational.
- Unconfigured server/model settings are presented as setup guidance rather than failures, and a stopped local engine is explicitly treated as normal when idle.
- Kept the overview derived from the existing read-only snapshot; no new collection, automatic repair, telemetry, or network behavior was added.


## 2026-11-28 — Batch #255
- Added contextual navigation shortcuts to Diagnostics findings for the configured local server, GGUF model, and local model catalogue. Users can jump directly to the related settings section without hunting through the Settings page.
- Navigation respects Reduce motion, scrolls the destination into view, and moves keyboard focus to the destination section. The shortcuts are navigation-only; they do not modify files, start the engine, or perform repairs.


## 2026-11-28 — Batch #256
- Improved Diagnostics storage clarity by showing available capacity, total capacity, and percentage used for app-data and model volumes, with compact visual usage meters.
- Unknown or invalid measurements remain explanatory text without a fabricated percentage or meter. The reviewed export preview uses the same safe formatting; collection and report schema are unchanged.


## 2026-11-28 — Batch #257
- Added an explicit local SQLite database backup. SQLite creates a consistent snapshot while database access is serialized, then the app opens the result read-only and verifies integrity plus schema version before reporting success.
- The backup includes conversations, saved memories, and Awareness data stored in SQLite. It stays on the device, refuses to overwrite an existing file, and uses a dedicated .sqlite3 destination.
- Clearly disclosed that this initial database-only backup is unencrypted and does not include model files or separate application settings. Restore, encryption, and cloud backup remain future work.


## 2026-11-28 — Batch #258
- Added a Settings action to verify an existing local SQLite backup. The selected file is opened read-only, checked with SQLite quick_check, checked against the current schema version, and checked for required midSunday database structures.
- Verification reports only integrity status, schema version, and file size. It does not read out personal content, modify the selected file, or restore data.
- Added clear rejection for empty, invalid, incomplete, or unsupported-schema files. Successful verification is not a substitute for end-to-end restore testing.


## 2026-11-28 — Batch #260
- Added regression tests for successful read-only verification of a real SQLite backup, including byte-for-byte confirmation that verification does not modify it.
- Added rejection coverage for empty files, non-SQLite data, unsupported extensions, incomplete midSunday schemas, and unsupported schema versions.
- Updated the roadmap to mark the implemented backup-verification test coverage complete. Physical-device validation remains pending.


## 2026-11-28 — Batch #261
- Extended read-only backup verification to report aggregate counts for conversations, messages, saved memories, and Awareness items, helping users understand a backup before a future restore workflow.
- Kept the summary free of personal text, titles, timestamps, record IDs, and paths. Added regression assertions for all four counts; verification remains non-mutating.


## 2026-11-28 — Batch #262
- Hardened local backup creation by generating and verifying a temporary SQLite snapshot first, then publishing it with an atomic no-overwrite hard link on the destination volume.
- A destination that appears after backup creation begins cannot be silently replaced. Existing files remain byte-for-byte unchanged, and temporary files are cleaned up after success or failure.
- Added regression checks for existing destination preservation and missing-folder failure. Filesystem-specific physical-device validation remains pending.

- Added a versioned metadata manifest to new local SQLite backups, recording creation time, app/schema versions, and privacy-safe aggregate counts. Verification rejects mismatched or unsupported manifests while continuing to recognise older pre-manifest backups as legacy. The manifest contains no personal content and does not add encryption or enable restore.


## 2026-11-28 — Batch #265
- Hardened read-only SQLite backup verification so an absent manifest is treated as legacy only when the manifest object is genuinely absent. A view or other unexpected SQLite object using the manifest name is rejected.
- Required every present manifest table to contain exactly one metadata row before its fields are trusted; malformed lookalikes and extra rows now fail closed.
- Added regression tests for valid legacy backups, manifest views, and multiple metadata rows. Verification remains read-only; this does not add encryption, cryptographic authenticity, or restore.


## 2026-11-28 — Batch #266
- Upgraded newly created local SQLite backup manifests to format v2, adding privacy-safe aggregate counts for Awareness activity/history, relearning exclusions, pending Learning Review suggestions, and app-preference records.
- Extended read-only verification to compare all five additional counts with the database. Existing v1 manifests and pre-manifest legacy backups remain supported.
- Expanded the verification summary with counts only; preference values and all personal content remain hidden. The backup is still unencrypted and restore remains unavailable.


## 2026-11-28 — Batch #300
- Strengthened guarded restore destination verification with an internal SHA-256 fingerprint over all application-owned table rows and typed SQLite values, in addition to schema and aggregate inventory checks.
- Kept fingerprints private and excluded backup-container metadata and SQLite-managed FTS implementation tables.
- Replaced the count-only mismatch test with a same-count/different-content regression test. Restore remains unavailable pending rollback, crash recovery, failure injection, and physical-device validation.


## 2026-11-28 — Batch #302
- Added a guarded rollback-aware candidate operation: if applying or verifying a staged restore candidate fails, the verified pre-restore candidate is copied back into the existing live SQLite connection and checked with full destination verification.
- Added deterministic post-copy verification failure injection proving that rollback restores the original conversation content.
- Critical rollback-verification failures now produce an explicit recovery error directing the caller to preserve the encrypted safety archive and stop using the database. Restore remains unavailable pending persistent recovery blocking, crash/restart recovery, full transaction-boundary failure injection, and physical-device validation.

## 2026-11-28 — Batch #304
- Expanded the guarded restore operation seams so tests can inject failures in candidate copy, candidate verification, rollback copy, and rollback verification.
- Added regression tests proving a failed initial copy rolls back successfully, while rollback-copy and rollback-verification failures return the explicit critical recovery error. The rollback-verification test independently confirms the original records were copied back.
- Restore remains unavailable pending a persistent recovery-blocked state, crash/restart recovery, remaining transaction-boundary tests, and physical-device validation.

## 2026-11-28 — Batch #305
- Added a versioned persistent recovery marker outside SQLite, containing no personal database content or backup credentials.
- Marker creation is exclusive and durable; existing markers are preserved, and malformed, partial, unreadable, or unsupported markers fail closed.
- App startup checks the marker before opening SQLite and blocks normal database startup when recovery is required.
- Added regression tests for clean startup, persistent blocking, marker preservation, malformed/partial markers, and unsupported versions. The marker is not yet wired into rollback failures and has no clearing/bypass command; restore remains unavailable.


## 2026-11-28 — Batch #306

- Connected rollback-copy or rollback-verification failure to the persistent recovery marker.
- The marker is written before returning a critical rollback error; errors distinguish successful marker persistence from marker-write failure.
- Added an integration regression test proving a rollback failure persists a marker that blocks normal startup.
- Restore remains unavailable; crash/restart recovery and physical Windows/Linux/macOS validation are still required.


## 2026-11-28 — Batch #307

- Made recovery-marker writes idempotent when a valid marker already exists, preserving its original contents.
- Added regression coverage for rollback-verification failure and confirmed that the persisted marker blocks normal startup.
- Added regression coverage for marker-write failure and the distinct manual-recovery error path.
- Restore remains unavailable pending recovery-only startup, crash/restart testing, and physical platform validation.


## 2026-11-28 — Batch #308
- Hardened persistent recovery-marker validation with a strict 256-byte maximum and a bounded read, including protection against a file growing after its initial size check.
- Required the marker path to be a regular file and rejected unknown JSON fields; malformed, oversized, and non-regular markers continue to fail closed.
- Added regression tests for oversized marker content, unknown fields, and a directory at the marker path. No marker-clearing command, recovery UI, or restore capability was added.


## 2026-11-28 — Batch #310
- Added a fresh-process regression test that writes a recovery marker, launches a separate test-runner process, and confirms the shared startup guard still blocks the marked data directory.
- Kept the child-process probe inert during ordinary test execution unless its private test-only environment variable is supplied.
- This verifies process-boundary persistence only; it does not claim power-loss durability, SQLite crash recovery, recovery UI, marker clearing, or restore availability.


## 2026-11-28 — Batch #311
- Expanded the fresh-process recovery guard test harness to check both clean startup and fail-closed outcomes.
- Added subprocess regression cases proving valid, malformed/partial, and unsupported-version markers block normal startup in an independent process.
- Kept the child probe inert during ordinary test execution unless its private test-only environment variables are supplied.
- No recovery UI, marker-clearing operation, or restore capability was added.
