use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const MARKER_FILE: &str = "midsunday-database-recovery-required.json";
const MARKER_VERSION: u32 = 1;
const MAX_MARKER_BYTES: u64 = 256;
const RECOVERY_REASON: &str = "restore_rollback_unverified";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RecoveryMarker {
    format_version: u32,
    reason_code: String,
    created_at: u64,
}

/// A durable fail-closed marker kept outside SQLite so it remains readable even
/// when the database itself cannot be trusted.
pub(crate) struct RecoveryState {
    marker_path: PathBuf,
}

impl RecoveryState {
    pub(crate) fn in_data_dir(data_dir: &Path) -> Self {
        Self { marker_path: data_dir.join(MARKER_FILE) }
    }

    /// Persist a minimal marker without storing database content, backup paths,
    /// passwords, or arbitrary error strings. create_new prevents overwriting
    /// an earlier recovery record. A partial marker is intentionally fail-closed.
    pub(crate) fn mark_required(&self) -> Result<(), String> {
        fs::create_dir_all(self.marker_path.parent().ok_or_else(|| {
            "The database recovery marker location is invalid.".to_owned()
        })?).map_err(|_| "The database recovery marker directory could not be prepared.".to_owned())?;

        let created_at = SystemTime::now().duration_since(UNIX_EPOCH)
            .map_err(|_| "The system clock is invalid; the database recovery marker was not written.".to_owned())?
            .as_secs();
        let marker = RecoveryMarker {
            format_version: MARKER_VERSION,
            reason_code: RECOVERY_REASON.to_owned(),
            created_at,
        };
        let encoded = serde_json::to_vec(&marker)
            .map_err(|_| "The database recovery marker could not be encoded.".to_owned())?;

        let mut file = match OpenOptions::new().write(true).create_new(true).open(&self.marker_path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                // An already-valid marker is an idempotent success for the
                // writer, even though it continues to block normal startup.
                return self.validate_existing_marker();
            }
            Err(_) => return Err("The database recovery marker could not be created. Preserve the encrypted safety archive and do not continue using this database.".to_owned()),
        };
        file.write_all(&encoded)
            .and_then(|_| file.sync_all())
            .map_err(|_| "The database recovery marker could not be durably written. Preserve the encrypted safety archive and do not continue using this database.".to_owned())
    }

    fn validate_existing_marker(&self) -> Result<(), String> {
        let metadata = fs::symlink_metadata(&self.marker_path)
            .map_err(|_| "A database recovery marker exists but could not be inspected. Preserve the encrypted safety archive and do not continue using this database.".to_owned())?;
        if !metadata.file_type().is_file() {
            return Err("The database recovery marker is not a regular file. Preserve the encrypted safety archive; normal database startup is blocked.".to_owned());
        }
        if metadata.len() > MAX_MARKER_BYTES {
            return Err("The database recovery marker exceeds the supported size. Preserve the encrypted safety archive; normal database startup is blocked.".to_owned());
        }

        let file = OpenOptions::new().read(true).open(&self.marker_path)
            .map_err(|_| "A database recovery marker exists but could not be read. Preserve the encrypted safety archive and do not continue using this database.".to_owned())?;
        let mut encoded = Vec::new();
        file.take(MAX_MARKER_BYTES + 1).read_to_end(&mut encoded)
            .map_err(|_| "A database recovery marker exists but could not be read. Preserve the encrypted safety archive and do not continue using this database.".to_owned())?;
        if encoded.len() as u64 > MAX_MARKER_BYTES {
            return Err("The database recovery marker exceeds the supported size. Preserve the encrypted safety archive; normal database startup is blocked.".to_owned());
        }

        let marker: RecoveryMarker = serde_json::from_slice(&encoded)
            .map_err(|_| "A database recovery marker is damaged, incomplete, or contains unsupported fields. Preserve the encrypted safety archive; normal database startup is blocked.".to_owned())?;
        if marker.format_version != MARKER_VERSION
            || marker.reason_code != RECOVERY_REASON
            || marker.created_at == 0
        {
            return Err("A database recovery marker has an unsupported or invalid format. Preserve the encrypted safety archive; normal database startup is blocked.".to_owned());
        }
        Ok(())
    }

    fn ensure_required(&self) -> Result<(), String> {
        self.validate_existing_marker()?;
        Err("Database recovery is required because a previous restore could not verify rollback. Preserve the encrypted pre-restore safety archive. midSunday has not opened the database; do not delete the recovery marker or continue using this database.".to_owned())
    }
}

/// Call before opening the SQLite database. Any present marker, including a
/// malformed or partially written one, prevents normal database startup.
pub(crate) fn ensure_database_not_blocked(data_dir: &Path) -> Result<(), String> {
    let state = RecoveryState::in_data_dir(data_dir);
    match fs::symlink_metadata(&state.marker_path) {
        Ok(_) => state.ensure_required(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("The database recovery state could not be inspected. Normal database startup is blocked to protect local data.".to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_is_allowed_when_no_recovery_marker_exists() {
        let directory = tempfile::tempdir().unwrap();
        ensure_database_not_blocked(directory.path()).unwrap();
    }

    #[test]
    fn persistent_marker_blocks_startup_and_is_not_overwritten() {
        let directory = tempfile::tempdir().unwrap();
        let state = RecoveryState::in_data_dir(directory.path());
        state.mark_required().unwrap();
        let path = directory.path().join(MARKER_FILE);
        let original = fs::read(&path).unwrap();
        assert!(ensure_database_not_blocked(directory.path()).unwrap_err().contains("recovery is required"));
        state.mark_required().unwrap();
        assert_eq!(fs::read(path).unwrap(), original);
    }

    #[test]
    fn malformed_or_partial_marker_fails_closed() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join(MARKER_FILE);
        fs::write(&path, br#"{"formatVersion":1"#).unwrap();
        let error = ensure_database_not_blocked(directory.path()).unwrap_err();
        assert!(error.contains("damaged, incomplete, or contains unsupported fields"));
    }

    #[test]
    fn rollback_failure_persists_marker_before_critical_error() {
        use crate::conversation_store::ConversationStore;
        use std::cell::Cell;

        let directory = tempfile::tempdir().unwrap();
        let source_store = ConversationStore::open(&directory.path().join("source-live.sqlite3")).unwrap();
        let source_conversation = source_store.create("Replacement source").unwrap();
        source_store.append(source_conversation.id, "user", "Replacement content").unwrap();
        let source_backup = directory.path().join("source.sqlite3");
        source_store.backup_to(&source_backup).unwrap();
        let source_candidate = ConversationStore::prepare_verified_restore_candidate(&source_backup).unwrap();

        let target_store = ConversationStore::open(&directory.path().join("target-live.sqlite3")).unwrap();
        let original_conversation = target_store.create("Original database").unwrap();
        target_store.append(original_conversation.id, "user", "Original content").unwrap();
        let rollback_backup = directory.path().join("rollback.sqlite3");
        target_store.backup_to(&rollback_backup).unwrap();
        let rollback_candidate = ConversationStore::prepare_verified_restore_candidate(&rollback_backup).unwrap();

        let app_data_dir = directory.path().join("app-data");
        let state = RecoveryState::in_data_dir(&app_data_dir);
        let mut guard = target_store.try_acquire_restore_lock().unwrap();
        let copy_calls = Cell::new(0usize);
        let error = guard.copy_from_candidate_with_operations_and_recovery(
            &source_candidate,
            &rollback_candidate,
            |current, candidate| {
                let call = copy_calls.get();
                copy_calls.set(call + 1);
                if call == 1 {
                    Err("injected rollback copy failure".to_owned())
                } else {
                    current.copy_from_candidate(candidate)
                }
            },
            |_current, _expected| Err("injected requested-candidate verification failure".to_owned()),
            || state.mark_required(),
        ).unwrap_err();

        assert!(error.contains("persistent recovery block was saved"));
        assert!(ensure_database_not_blocked(&app_data_dir)
            .unwrap_err().contains("recovery is required"));
        drop(guard);
    }

    #[test]
    fn rollback_verification_failure_persists_marker() {
        use crate::conversation_store::ConversationStore;
        use std::cell::Cell;

        let directory = tempfile::tempdir().unwrap();
        let source_store = ConversationStore::open(&directory.path().join("source-live.sqlite3")).unwrap();
        let source_conversation = source_store.create("Replacement source").unwrap();
        source_store.append(source_conversation.id, "user", "Replacement content").unwrap();
        let source_backup = directory.path().join("source.sqlite3");
        source_store.backup_to(&source_backup).unwrap();
        let source_candidate = ConversationStore::prepare_verified_restore_candidate(&source_backup).unwrap();

        let target_store = ConversationStore::open(&directory.path().join("target-live.sqlite3")).unwrap();
        let original_conversation = target_store.create("Original database").unwrap();
        target_store.append(original_conversation.id, "user", "Original content").unwrap();
        let rollback_backup = directory.path().join("rollback.sqlite3");
        target_store.backup_to(&rollback_backup).unwrap();
        let rollback_candidate = ConversationStore::prepare_verified_restore_candidate(&rollback_backup).unwrap();

        let app_data_dir = directory.path().join("app-data");
        let state = RecoveryState::in_data_dir(&app_data_dir);
        let mut guard = target_store.try_acquire_restore_lock().unwrap();
        let verify_calls = Cell::new(0usize);
        let error = guard.copy_from_candidate_with_operations_and_recovery(
            &source_candidate,
            &rollback_candidate,
            |current, candidate| current.copy_from_candidate(candidate),
            |_current, _expected| {
                let call = verify_calls.get();
                verify_calls.set(call + 1);
                Err(if call == 0 {
                    "injected requested-candidate verification failure".to_owned()
                } else {
                    "injected rollback verification failure".to_owned()
                })
            },
            || state.mark_required(),
        ).unwrap_err();

        assert!(error.contains("persistent recovery block was saved"));
        assert!(ensure_database_not_blocked(&app_data_dir).unwrap_err().contains("recovery is required"));
        drop(guard);
    }

    #[test]
    fn marker_write_failure_is_reported_as_manual_recovery() {
        use crate::conversation_store::ConversationStore;

        let directory = tempfile::tempdir().unwrap();
        let source_store = ConversationStore::open(&directory.path().join("source-live.sqlite3")).unwrap();
        let source_conversation = source_store.create("Replacement source").unwrap();
        source_store.append(source_conversation.id, "user", "Replacement content").unwrap();
        let source_backup = directory.path().join("source.sqlite3");
        source_store.backup_to(&source_backup).unwrap();
        let source_candidate = ConversationStore::prepare_verified_restore_candidate(&source_backup).unwrap();

        let target_store = ConversationStore::open(&directory.path().join("target-live.sqlite3")).unwrap();
        let original_conversation = target_store.create("Original database").unwrap();
        target_store.append(original_conversation.id, "user", "Original content").unwrap();
        let rollback_backup = directory.path().join("rollback.sqlite3");
        target_store.backup_to(&rollback_backup).unwrap();
        let rollback_candidate = ConversationStore::prepare_verified_restore_candidate(&rollback_backup).unwrap();

        let app_data_dir = directory.path().join("app-data");
        let mut guard = target_store.try_acquire_restore_lock().unwrap();
        let error = guard.copy_from_candidate_with_operations_and_recovery(
            &source_candidate,
            &rollback_candidate,
            |_current, _candidate| Err("injected copy failure".to_owned()),
            |_current, _expected| Err("injected verification failure".to_owned()),
            || Err("injected marker persistence failure".to_owned()),
        ).unwrap_err();

        assert!(error.contains("persistent recovery block could not be saved"));
        assert!(error.contains("injected marker persistence failure"));
        assert!(!app_data_dir.join(MARKER_FILE).exists());
        drop(guard);
    }

    #[test]
    fn oversized_marker_fails_closed_before_unbounded_read() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join(MARKER_FILE);
        fs::write(&path, vec![b'x'; MAX_MARKER_BYTES as usize + 1]).unwrap();
        let error = ensure_database_not_blocked(directory.path()).unwrap_err();
        assert!(error.contains("exceeds the supported size"));
    }

    #[test]
    fn marker_with_unknown_fields_fails_closed() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join(MARKER_FILE);
        fs::write(&path, br#"{"formatVersion":1,"reasonCode":"restore_rollback_unverified","createdAt":1,"unexpected":"value"}"#).unwrap();
        let error = ensure_database_not_blocked(directory.path()).unwrap_err();
        assert!(error.contains("unsupported fields"));
    }

    #[test]
    fn non_regular_marker_path_fails_closed() {
        let directory = tempfile::tempdir().unwrap();
        fs::create_dir(directory.path().join(MARKER_FILE)).unwrap();
        let error = ensure_database_not_blocked(directory.path()).unwrap_err();
        assert!(error.contains("not a regular file"));
    }

    #[test]
    fn unsupported_marker_version_fails_closed() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join(MARKER_FILE);
        fs::write(&path, br#"{"formatVersion":99,"reasonCode":"restore_rollback_unverified","createdAt":1}"#).unwrap();
        let error = ensure_database_not_blocked(directory.path()).unwrap_err();
        assert!(error.contains("unsupported or invalid format"));
    }

    // Invoked by the parent test in a fresh test-runner process. Keeping this
    // probe as a normal test also makes it harmless during the full test suite.
    #[test]
    fn recovery_marker_child_process_probe() {
        let Ok(data_dir) = std::env::var("MIDSUNDAY_RECOVERY_MARKER_CHILD_DATA_DIR") else {
            return;
        };
        let error = ensure_database_not_blocked(Path::new(&data_dir))
            .expect_err("a persisted recovery marker must block a fresh process");
        assert!(error.contains("recovery is required"), "unexpected startup guard result: {error}");
    }

    #[test]
    fn persisted_marker_blocks_a_fresh_process() {
        use std::process::Command;

        let directory = tempfile::tempdir().unwrap();
        RecoveryState::in_data_dir(directory.path()).mark_required().unwrap();

        let output = Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("recovery_state::tests::recovery_marker_child_process_probe")
            .arg("--nocapture")
            .env("MIDSUNDAY_RECOVERY_MARKER_CHILD_DATA_DIR", directory.path())
            .output()
            .expect("the recovery-marker child test process should start");

        assert!(
            output.status.success(),
            "fresh process did not enforce the persisted recovery block. stdout: {} stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

}
