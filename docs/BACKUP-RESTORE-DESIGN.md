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

The database mutex remains the final serialization boundary for all `ConversationStore` methods. The two gates have separate responsibilities: the runtime gate prevents new model work from racing the restore, while the database mutex prevents reads/writes from interleaving with the database copy.

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
