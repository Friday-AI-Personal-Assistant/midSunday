# midSunday Backup & Restore — Safety Design

**Status:** Restore design and implementation gates  
**Last updated:** 2026-11-28  
**Scope:** Local SQLite database backups in `.sqlite3` and authenticated encrypted `.msbackup` containers. This document does not enable restore.

## 1. Current verified behaviour

The desktop app can create a consistent local SQLite snapshot, add and verify its versioned manifest, run full SQLite integrity and foreign-key checks, and report aggregate content counts. It can also create password-protected `.msbackup` files using Argon2id key derivation and XChaCha20-Poly1305 authenticated encryption, decrypt them into a temporary workspace for verification, and show a read-only restore-impact preview.

The encrypted backup password is not stored. Encrypted backups remain local-only. The backup contains the SQLite database; model files and separate JSON settings are not included. The app currently has no restore command or restore button.

## 2. Non-negotiable safety requirements

1. **No blind file replacement.** The live SQLite file must never be renamed, deleted, or overwritten while its connection may still be open.
2. **Explicit consent.** Restore must require a clear final confirmation that current conversations, memories, Awareness data, and database preferences will be replaced by the selected backup.
3. **Verified source.** Authenticate and decrypt the encrypted container, then run the complete SQLite integrity check, foreign-key check, schema check, required-object checks, and manifest/inventory comparison before touching live data.
4. **Pre-restore safety copy.** Create and verify a separate encrypted copy of the current database before restore begins. The user must choose where it is saved. Never overwrite an existing file.
5. **Preserve the live connection.** The current `ConversationStore` owns a single `Arc<Mutex<Connection>>`. The preferred implementation candidate is SQLite's online backup API to copy a verified source into that existing connection while holding the store lock. Do not replace the database file underneath the connection.
6. **Serialize database activity.** All normal reads and writes use the same connection mutex. Restore must hold the same lock throughout the critical operation so no application database operation can interleave with the replacement.
7. **Coordinate the local model runtime.** Restore must be refused while the runtime is starting, ready, or generating. It must also prevent a new runtime operation from starting during restore; a status check alone is not sufficient to close this race.
8. **Verify the destination.** After copying, run full integrity and foreign-key checks against the live connection and compare its aggregate inventory with the verified source backup.
9. **Rollback on failure.** If post-copy validation fails, restore the verified pre-restore safety copy before releasing the database lock. If rollback also fails, preserve the safety archive, stop further database mutations, and present a clear recovery error. Never silently continue.
10. **Crash recovery must be demonstrated.** SQLite transaction/journal behaviour must be validated with failure injection and restart tests before claiming that interrupted restores leave a usable old or new database.
11. **Temporary plaintext is short-lived.** Decrypted source and safety work files must be created in private temporary storage, never logged or uploaded, and explicitly cleaned up. Cleanup failures must be reported.
12. **No silent compatibility conversion.** The first restore implementation should accept only the exact supported schema and manifest versions. No automatic downgrade or destructive migration.

## 3. Proposed restore sequence

1. User selects an encrypted `.msbackup` archive and enters its password.
2. App authenticates/decrypts to a private temporary directory and performs the same full verifier used by the existing verification and preview actions.
3. App displays the backup inventory and the current-device inventory. The preview clearly states that restore replaces the current database.
4. User explicitly chooses a destination for a new encrypted pre-restore safety archive. Existing destinations are rejected. The safety archive is encrypted with the password the user has supplied for this restore operation, and the UI must explain this before proceeding.
5. User confirms the destructive restore in a separate confirmation step.
6. App acquires the restore/runtime coordination gate, refuses if the model runtime is active, then acquires the database connection lock.
7. While holding the lock, app creates and verifies the pre-restore safety snapshot. It must represent the exact current database state immediately before replacement.
8. App copies the verified source database into the existing live connection using the SQLite online backup API. The operation must complete before any success state is shown.
9. App validates live database integrity, foreign keys, schema version, required structures, and all aggregate inventory counts against the selected source.
10. If validation fails, app restores the safety snapshot into the same live connection and validates the rollback. The safety archive is retained regardless of outcome.
11. Only after successful validation does the app release the lock and report success. The UI refreshes conversation, memory, and Awareness views from the restored database.
12. The user retains the encrypted pre-restore safety archive and can later verify it independently or delete it manually.

## 4. Runtime coordination design

A simple `RuntimeManager::snapshot()` check is not sufficient: another command could start the model between the check and the database replacement. Before implementation, introduce a shared operation gate used by runtime start/recovery/generation and restore. Restore takes exclusive access; normal runtime operations must either wait or return a clear busy result. Stop/status operations must not deadlock behind a long-running restore.

The database mutex remains the final serialization boundary for all `ConversationStore` methods. Batch #285 adds a non-blocking `ConversationStore::try_acquire_restore_lock` that returns an exclusive guard over this same mutex. Batch #286 gives that guard a dedicated safety-snapshot method: it creates the same manifest-bearing SQLite snapshot directly from the already-locked connection, without trying to reacquire the mutex. The ordinary backup path and guarded path share one implementation, and the guarded snapshot is fully verified before publication. Restore must hold the guard throughout the critical operation so no application database operation can interleave with the database copy. If another database operation is already active, restore fails fast with a user-safe busy message. The runtime gate prevents new model work from racing the restore; the database mutex prevents reads/writes from interleaving with the database copy. The guard API does not itself copy, validate, or roll back database contents.

## 5. Required automated tests

- Valid encrypted archive restores the expected conversations, messages, memories, Awareness records, settings, and manifest counts.
- Wrong password, truncated ciphertext, altered header, trailing data, invalid SQLite, broken foreign keys, unsupported schema, and manifest mismatch all fail before live data changes.
- Existing safety-archive destinations are never overwritten.
- Safety archive is verified and decryptable before the first live-database write.
- Database operations are serialized for the entire restore.
- Restore is refused while runtime is starting, ready, or generating; a concurrent start cannot pass the restore gate.
- Injected failure before copy leaves the current database unchanged.
- Injected copy failure leaves the old database usable.
- Injected post-copy validation failure triggers rollback and restores the exact pre-restore inventory.
- Injected rollback failure preserves the safety archive and blocks further writes with a clear recovery state.
- Interruption/restart tests demonstrate the actual SQLite journal behaviour at each critical boundary.
- Successful restore refreshes frontend data and does not leave stale conversation or memory selections.
- Temporary plaintext is removed on success and every ordinary error path; cleanup failure is surfaced.
- Tests prove no password, plaintext content, user file path, or personal record content is written to logs or status messages.

## 6. Physical validation gate

CI compilation and unit tests are necessary but not sufficient. Before enabling restore in a release, test on physical Windows 10/11, Linux, and macOS systems with representative database sizes. Include low disk space, read-only folders, interrupted process, forced shutdown during copy, restart recovery, active chat/runtime, and repeated restore/rollback cycles.

Until these tests pass, the UI must continue to label restore as unavailable. A successful preview is not a restore test and must never be described as proof of recoverability.

## 7. Explicit exclusions

This milestone does not restore model files, separate JSON settings, application binaries, or in-memory System Monitor history. It does not upload backups to Google Drive or any other cloud service. Cloud backup remains a separate future feature requiring least-privilege provider permissions and explicit consent.

## 8. Encrypted safety-archive preparation milestone (Batch #290)

The backend now has a separately callable operation that prepares the user-selected encrypted pre-restore safety archive without performing a restore. It acquires the shared runtime operation gate first, then the same database mutex used by all ConversationStore operations. While both guards are held, it creates the guarded SQLite snapshot, encrypts it using the existing authenticated .msbackup format, decrypts it into a private verification workspace, and requires full SQLite integrity, foreign-key, schema, and v2 manifest verification before publishing with an atomic no-overwrite hard link. Temporary plaintext cleanup failures are surfaced. Regression tests cover successful verification, existing-destination preservation, and fail-fast database/runtime contention.

This is a backend preparation primitive only. It is not exposed in the UI, is not part of the restore preview, and does not modify the live database. The actual restore transaction, rollback, crash recovery, and physical-device validation remain incomplete; restore stays unavailable.


## 9. Isolated restore-candidate staging milestone (Batch #291)

The backend now has a private staging primitive that first verifies a v2 SQLite backup, copies it into a separate temporary SQLite database with rusqlite's SQLite online-backup API, and then runs the full backup verifier against the copied candidate. It compares schema version, every aggregate content count, and all available manifest metadata with the original. The candidate owns its temporary workspace and removes it when dropped. The helper does not open or mutate the live ConversationStore connection and is not registered as a restore command. This validates the SQLite copy mechanism in isolation; it does not prove live restore, rollback, or crash recovery.


## 10. Encrypted restore-candidate staging milestone (Batch #292)

A user-selected encrypted `.msbackup` can now be decrypted into a private temporary workspace and passed through the isolated SQLite candidate staging path. The source is fully verified, copied using SQLite's online-backup API, and the candidate is fully re-verified against its source schema, manifest, and aggregate inventory. The decrypted source workspace is explicitly removed before the candidate is returned; cleanup failure aborts staging and discards the candidate. Wrong passwords and invalid encrypted sources fail without modifying the encrypted source or live database. This is preparation only: no live restore, rollback, crash recovery, or UI enablement is included.


## 11. Guarded live-connection copy and verification primitives (Batch #293)

The restore guard now exposes two private backend primitives. The first copies a previously staged and verified candidate into the existing live SQLite connection using SQLite's online-backup API while the exclusive database mutex is held. The second snapshots that guarded connection into a private temporary workspace, runs the full backup verifier, and compares schema version and every aggregate inventory count against the candidate. Temporary verification storage is explicitly closed and cleanup failures are surfaced.

Automated tests exercise the copy while holding the runtime gate and database guard, confirm the target database reflects the candidate inventory after the guard is released, and prove that a mismatched live inventory is rejected without mutation. These primitives are not registered as a command and are not connected to UI. They do not yet provide the complete encrypted safety-archive + rollback transaction, crash recovery, or physical-device validation; restore remains unavailable.


## 12. Batch #293 regression correction

The v2 `midsunday_backup_manifest` is backup-container metadata and must not remain in the live application database after a candidate is copied into the existing connection. The guarded copy now explicitly closes the SQLite online-backup handle and removes that backup-only table before any destination verification snapshot is generated. This keeps the live schema clean and allows the normal backup writer to create a fresh manifest in the verification snapshot.

The aggregate mismatch test now deliberately uses a target database with two conversations against a one-conversation candidate. The earlier version used different text but identical aggregate counts, which could not be expected to fail an inventory-only comparison. Restore remains disabled pending the complete safety archive, rollback, and crash-recovery transaction.


## 13. Exact application-record destination verification (Batch #300)

The guarded destination check now compares a SHA-256 fingerprint of every row and typed column value across the application-owned tables, in addition to the full SQLite verifier, schema version, and aggregate inventory. Table and row ordering is stable, values are encoded with explicit SQLite type tags and lengths, and the digest remains internal: no conversation text, memory content, identifiers, or fingerprint is exposed in UI or logs. Backup manifest metadata and SQLite-managed FTS implementation tables are excluded because they are container/derived data rather than source application records.

The regression test now deliberately gives the candidate and live database equal aggregate counts but different conversation content. Verification must reject this same-count mismatch without mutating the target. This strengthens only the private verification primitive; it does not add rollback, crash recovery, a restore command, or UI enablement. Restore remains unavailable.


## 14. Guarded rollback primitive and injected verification failure (Batch #302)

The guarded database layer now provides a rollback-aware candidate application primitive. It attempts the requested candidate copy and exact destination verification while the caller retains the runtime-operation gate and database mutex. If copying or verification fails, it copies the separately staged pre-restore candidate back into the existing live connection and verifies that rollback with the full integrity, schema, inventory, and application-record fingerprint checks.

A deterministic regression test injects a post-copy verification failure, confirms that the pre-restore candidate is copied back, and verifies that the original conversation content survives. If rollback itself cannot be verified, the helper returns an explicit critical recovery error and instructs the caller to preserve the encrypted safety archive and stop using the database.

This is still a private transaction building block, not a complete restore workflow. The caller must create and retain the encrypted safety archive and both verified candidates, keep the runtime/database guards for the entire operation, and handle a critical rollback failure with a persistent recovery-blocked state. Crash/restart recovery, transaction-boundary failure injection, and physical-device validation remain outstanding. No restore command or UI is enabled.
