use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Mutex, time::{SystemTime, UNIX_EPOCH}};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub preferred_name: String,
    pub use_name: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AwarenessSettings {
    pub enabled: bool,
    pub learn_conversations: bool,
    pub learn_searches: bool,
}

impl Default for AwarenessSettings {
    fn default() -> Self {
        Self { enabled: false, learn_conversations: false, learn_searches: false }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AwarenessItem {
    pub id: i64,
    pub content: String,
    pub category: String,
    pub source_type: String,
    pub knowledge_type: String,
    pub confidence: f64,
    pub source_record_id: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AwarenessSuppression {
    pub category: Option<String>,
    pub source_type: String,
    pub created_at: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AwarenessHistoryEntry {
    pub id: i64,
    pub event_type: String,
    pub awareness_item_id: Option<i64>,
    pub category: Option<String>,
    pub occurred_at: i64,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self { preferred_name: String::new(), use_name: false }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SavedMemory {
    pub id: i64,
    pub content: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MemoryExportFile {
    format: &'static str,
    format_version: u32,
    exported_at: i64,
    memories: Vec<SavedMemory>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConversationSummary {
    pub id: i64,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StoredMessage {
    pub id: i64,
    pub conversation_id: i64,
    pub role: String,
    pub content: String,
    pub created_at: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConversationExport {
    id: i64,
    title: String,
    created_at: i64,
    updated_at: i64,
    messages: Vec<StoredMessage>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConversationExportFile {
    format: &'static str,
    format_version: u32,
    exported_at: i64,
    conversations: Vec<ConversationExport>,
}


#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConversationImport {
    id: i64,
    title: String,
    created_at: i64,
    updated_at: i64,
    messages: Vec<StoredMessage>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConversationImportFile {
    format: String,
    format_version: u32,
    conversations: Vec<ConversationImport>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub imported: usize,
    pub skipped_duplicates: usize,
}

pub struct ConversationStore {
    connection: Mutex<Connection>,
}

impl ConversationStore {
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| format!("Could not create local data directory: {error}"))?;
        }
        let connection = Connection::open(path).map_err(|error| format!("Could not open local conversation database: {error}"))?;
        Self::from_connection(connection)
    }

    fn from_connection(connection: Connection) -> Result<Self, String> {
        connection.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|error| format!("Could not enable local database constraints: {error}"))?;
        let schema_version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(|error| format!("Could not read local database version: {error}"))?;
        if schema_version > 5 {
            return Err(format!("This midSunday database uses a newer schema version ({schema_version}) than this app supports."));
        }
        if schema_version == 0 {
            let transaction = connection.unchecked_transaction()
                .map_err(|error| format!("Could not prepare local database migration: {error}"))?;
            transaction.execute_batch(
                "CREATE TABLE IF NOT EXISTS conversations (
                   id INTEGER PRIMARY KEY AUTOINCREMENT,
                   title TEXT NOT NULL,
                   created_at INTEGER NOT NULL,
                   updated_at INTEGER NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS conversation_messages (
                   id INTEGER PRIMARY KEY AUTOINCREMENT,
                   conversation_id INTEGER NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
                   role TEXT NOT NULL CHECK(role IN ('user', 'assistant')),
                   content TEXT NOT NULL,
                   created_at INTEGER NOT NULL
                 );
                 CREATE INDEX IF NOT EXISTS idx_conversation_messages_order
                   ON conversation_messages(conversation_id, id);
                 CREATE INDEX IF NOT EXISTS idx_conversations_updated
                   ON conversations(updated_at DESC);
                 PRAGMA user_version = 1;"
            ).map_err(|error| format!("Could not apply local database migration 1: {error}"))?;
            transaction.commit().map_err(|error| format!("Could not finish local database migration: {error}"))?;
        }
        let current_version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(|error| format!("Could not recheck local database version: {error}"))?;
        if current_version == 1 {
            let transaction = connection.unchecked_transaction()
                .map_err(|error| format!("Could not prepare local memory migration: {error}"))?;
            transaction.execute_batch(
                "CREATE TABLE IF NOT EXISTS saved_memories (
                   id INTEGER PRIMARY KEY AUTOINCREMENT,
                   content TEXT NOT NULL,
                   created_at INTEGER NOT NULL,
                   updated_at INTEGER NOT NULL
                 );
                 CREATE INDEX IF NOT EXISTS idx_saved_memories_updated
                   ON saved_memories(updated_at DESC, id DESC);
                 CREATE TABLE IF NOT EXISTS app_preferences (
                   key TEXT PRIMARY KEY,
                   value TEXT NOT NULL
                 );
                 INSERT OR IGNORE INTO app_preferences(key, value) VALUES('memory_enabled', 'false');
                 PRAGMA user_version = 2;"
            ).map_err(|error| format!("Could not apply local memory migration: {error}"))?;
            transaction.commit().map_err(|error| format!("Could not finish local memory migration: {error}"))?;
        }
        let current_version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(|error| format!("Could not recheck local database version: {error}"))?;
        if current_version == 2 {
            let transaction = connection.unchecked_transaction()
                .map_err(|error| format!("Could not prepare local Awareness migration: {error}"))?;
            transaction.execute_batch(
                "CREATE TABLE IF NOT EXISTS awareness_items (
                   id INTEGER PRIMARY KEY AUTOINCREMENT,
                   content TEXT NOT NULL,
                   category TEXT NOT NULL,
                   source_type TEXT NOT NULL CHECK(source_type IN ('conversation', 'in_app_search', 'user_entered')),
                   knowledge_type TEXT NOT NULL CHECK(knowledge_type IN ('observed', 'inferred')),
                   confidence REAL NOT NULL CHECK(confidence >= 0.0 AND confidence <= 1.0),
                   source_record_id INTEGER,
                   created_at INTEGER NOT NULL,
                   updated_at INTEGER NOT NULL
                 );
                 CREATE INDEX IF NOT EXISTS idx_awareness_items_updated
                   ON awareness_items(updated_at DESC, id DESC);
                 CREATE TABLE IF NOT EXISTS awareness_activity (
                   id INTEGER PRIMARY KEY AUTOINCREMENT,
                   source_type TEXT NOT NULL CHECK(source_type IN ('conversation', 'in_app_search')),
                   source_record_id INTEGER,
                   activity_type TEXT NOT NULL CHECK(activity_type IN ('observed', 'learned', 'excluded', 'blocked')),
                   awareness_item_id INTEGER REFERENCES awareness_items(id) ON DELETE SET NULL,
                   occurred_at INTEGER NOT NULL
                 );
                 CREATE INDEX IF NOT EXISTS idx_awareness_activity_time
                   ON awareness_activity(occurred_at DESC, id DESC);
                 INSERT OR IGNORE INTO app_preferences(key, value)
                   VALUES('awareness_settings', '{\"enabled\":false,\"learnConversations\":false,\"learnSearches\":false}');
                 PRAGMA user_version = 3;"
            ).map_err(|error| format!("Could not apply local Awareness migration: {error}"))?;
            transaction.commit().map_err(|error| format!("Could not finish local Awareness migration: {error}"))?;
        }
        let current_version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(|error| format!("Could not recheck local database version: {error}"))?;
        if current_version == 3 {
            let transaction = connection.unchecked_transaction()
                .map_err(|error| format!("Could not prepare Awareness history migration: {error}"))?;
            transaction.execute_batch(
                "CREATE TABLE IF NOT EXISTS awareness_history (
                   id INTEGER PRIMARY KEY AUTOINCREMENT,
                   event_type TEXT NOT NULL CHECK(event_type IN ('added', 'updated', 'deleted', 'cleared', 'exported')),
                   awareness_item_id INTEGER,
                   category TEXT,
                   occurred_at INTEGER NOT NULL
                 );
                 CREATE INDEX IF NOT EXISTS idx_awareness_history_time
                   ON awareness_history(occurred_at DESC, id DESC);
                 PRAGMA user_version = 4;"
            ).map_err(|error| format!("Could not apply Awareness history migration: {error}"))?;
            transaction.commit().map_err(|error| format!("Could not finish Awareness history migration: {error}"))?;
        }
        let current_version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(|error| format!("Could not recheck local database version: {error}"))?;
        if current_version == 4 {
            let transaction = connection.unchecked_transaction()
                .map_err(|error| format!("Could not prepare Awareness suppression migration: {error}"))?;
            transaction.execute_batch(
                "CREATE TABLE IF NOT EXISTS awareness_suppressions (
                   content_hash TEXT PRIMARY KEY CHECK(length(content_hash) = 64),
                   category TEXT,
                   source_type TEXT NOT NULL,
                   created_at INTEGER NOT NULL
                 );
                 CREATE INDEX IF NOT EXISTS idx_awareness_suppressions_time
                   ON awareness_suppressions(created_at DESC);
                 PRAGMA user_version = 5;"
            ).map_err(|error| format!("Could not apply Awareness suppression migration: {error}"))?;
            transaction.commit().map_err(|error| format!("Could not finish Awareness suppression migration: {error}"))?;
        }
        Ok(Self { connection: Mutex::new(connection) })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, String> {
        self.connection.lock().map_err(|_| "Local conversation database is unavailable.".to_owned())
    }

    pub fn get_awareness_settings(&self) -> Result<AwarenessSettings, String> {
        let connection = self.lock()?;
        let value: Option<String> = connection.query_row(
            "SELECT value FROM app_preferences WHERE key = 'awareness_settings'", [], |row| row.get(0)
        ).optional().map_err(|error| format!("Could not read Awareness settings: {error}"))?;
        match value {
            Some(json) => serde_json::from_str(&json)
                .map_err(|error| format!("Could not decode Awareness settings: {error}")),
            None => Ok(AwarenessSettings::default()),
        }
    }

    pub fn save_awareness_settings(&self, settings: AwarenessSettings) -> Result<AwarenessSettings, String> {
        let json = serde_json::to_string(&settings)
            .map_err(|error| format!("Could not encode Awareness settings: {error}"))?;
        let connection = self.lock()?;
        connection.execute(
            "INSERT INTO app_preferences(key, value) VALUES('awareness_settings', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![json],
        ).map_err(|error| format!("Could not save Awareness settings: {error}"))?;
        Ok(settings)
    }

    fn record_awareness_history(
        connection: &Connection, event_type: &str, item_id: Option<i64>, category: Option<&str>,
    ) -> Result<(), String> {
        connection.execute(
            "INSERT INTO awareness_history(event_type, awareness_item_id, category, occurred_at)
             VALUES(?1, ?2, ?3, ?4)",
            params![event_type, item_id, category, unix_time()],
        ).map_err(|error| format!("Could not record Awareness history: {error}"))?;
        Ok(())
    }

    pub fn list_awareness_history(&self) -> Result<Vec<AwarenessHistoryEntry>, String> {
        let connection = self.lock()?;
        let mut statement = connection.prepare(
            "SELECT id, event_type, awareness_item_id, category, occurred_at
             FROM awareness_history ORDER BY occurred_at DESC, id DESC LIMIT 500"
        ).map_err(|error| format!("Could not read Awareness history: {error}"))?;
        let rows = statement.query_map([], |row| Ok(AwarenessHistoryEntry {
            id: row.get(0)?, event_type: row.get(1)?, awareness_item_id: row.get(2)?,
            category: row.get(3)?, occurred_at: row.get(4)?,
        })).map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())
    }

    pub fn clear_awareness_history(&self) -> Result<usize, String> {
        let connection = self.lock()?;
        connection.execute("DELETE FROM awareness_history", [])
            .map_err(|error| format!("Could not clear Awareness history: {error}"))
    }

    pub fn list_awareness_suppressions(&self) -> Result<Vec<AwarenessSuppression>, String> {
        let connection = self.lock()?;
        let mut statement = connection.prepare(
            "SELECT category, source_type, created_at FROM awareness_suppressions ORDER BY created_at DESC"
        ).map_err(|error| format!("Could not read Awareness exclusions: {error}"))?;
        let rows = statement.query_map([], |row| Ok(AwarenessSuppression {
            category: row.get(0)?, source_type: row.get(1)?, created_at: row.get(2)?,
        })).map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())
    }

    pub fn clear_awareness_suppressions(&self) -> Result<usize, String> {
        let connection = self.lock()?;
        connection.execute("DELETE FROM awareness_suppressions", [])
            .map_err(|error| format!("Could not clear Awareness exclusions: {error}"))
    }

    pub fn is_awareness_suppressed(&self, content: &str) -> Result<bool, String> {
        let fingerprint = awareness_content_fingerprint(content);
        let connection = self.lock()?;
        let exists: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM awareness_suppressions WHERE content_hash = ?1)",
            params![fingerprint], |row| row.get(0),
        ).map_err(|error| format!("Could not check Awareness exclusion: {error}"))?;
        Ok(exists)
    }

    pub fn list_awareness_items(&self) -> Result<Vec<AwarenessItem>, String> {
        let connection = self.lock()?;
        let mut statement = connection.prepare(
            "SELECT id, content, category, source_type, knowledge_type, confidence, source_record_id, created_at, updated_at
             FROM awareness_items ORDER BY updated_at DESC, id DESC"
        ).map_err(|error| format!("Could not read Awareness items: {error}"))?;
        let rows = statement.query_map([], |row| Ok(AwarenessItem {
            id: row.get(0)?, content: row.get(1)?, category: row.get(2)?,
            source_type: row.get(3)?, knowledge_type: row.get(4)?, confidence: row.get(5)?,
            source_record_id: row.get(6)?, created_at: row.get(7)?, updated_at: row.get(8)?,
        })).map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())
    }

    pub fn create_awareness_item(&self, content: &str, category: &str) -> Result<AwarenessItem, String> {
        let content = clean_awareness_content(content)?;
        let category = clean_awareness_category(category)?;
        let now = unix_time();
        let connection = self.lock()?;
        connection.execute(
            "INSERT INTO awareness_items(content, category, source_type, knowledge_type, confidence, source_record_id, created_at, updated_at)
             VALUES(?1, ?2, 'user_entered', 'observed', 1.0, NULL, ?3, ?3)",
            params![content, category, now],
        ).map_err(|error| format!("Could not save Awareness item: {error}"))?;
        let id = connection.last_insert_rowid();
        Self::record_awareness_history(&connection, "added", Some(id), Some(&category))?;
        Ok(AwarenessItem {
            id, content, category, source_type: "user_entered".to_owned(),
            knowledge_type: "observed".to_owned(), confidence: 1.0, source_record_id: None,
            created_at: now, updated_at: now,
        })
    }

    pub fn update_awareness_item(&self, id: i64, content: &str, category: &str) -> Result<(), String> {
        if id <= 0 { return Err("Invalid Awareness item identifier.".to_owned()); }
        let content = clean_awareness_content(content)?;
        let category = clean_awareness_category(category)?;
        let connection = self.lock()?;
        let transaction = connection.unchecked_transaction()
            .map_err(|error| format!("Could not prepare Awareness update: {error}"))?;
        let previous: Option<(String, String, String)> = transaction.query_row(
            "SELECT content, category, source_type FROM awareness_items WHERE id = ?1",
            params![id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).optional().map_err(|error| error.to_string())?;
        let Some((previous_content, previous_category, previous_source)) = previous else {
            return Err("Awareness item was not found.".to_owned());
        };
        if awareness_content_fingerprint(&previous_content) != awareness_content_fingerprint(&content) {
            insert_awareness_suppression(&transaction, &previous_content, &previous_category, &previous_source)?;
        }
        transaction.execute(
            "UPDATE awareness_items SET content = ?1, category = ?2, updated_at = ?3 WHERE id = ?4",
            params![content, category, unix_time(), id],
        ).map_err(|error| format!("Could not update Awareness item: {error}"))?;
        Self::record_awareness_history(&transaction, "updated", Some(id), Some(&category))?;
        transaction.commit().map_err(|error| format!("Could not finish Awareness update: {error}"))?;
        Ok(())
    }

    pub fn delete_awareness_item(&self, id: i64) -> Result<(), String> {
        if id <= 0 { return Err("Invalid Awareness item identifier.".to_owned()); }
        let connection = self.lock()?;
        let transaction = connection.unchecked_transaction()
            .map_err(|error| format!("Could not prepare Awareness deletion: {error}"))?;
        let previous: Option<(String, String, String)> = transaction.query_row(
            "SELECT content, category, source_type FROM awareness_items WHERE id = ?1",
            params![id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).optional().map_err(|error| error.to_string())?;
        let Some((content, category, source_type)) = previous else {
            return Err("Awareness item was not found.".to_owned());
        };
        insert_awareness_suppression(&transaction, &content, &category, &source_type)?;
        transaction.execute("DELETE FROM awareness_items WHERE id = ?1", params![id])
            .map_err(|error| format!("Could not delete Awareness item: {error}"))?;
        Self::record_awareness_history(&transaction, "deleted", Some(id), Some(&category))?;
        transaction.commit().map_err(|error| format!("Could not finish Awareness deletion: {error}"))?;
        Ok(())
    }

    pub fn clear_awareness_items(&self) -> Result<usize, String> {
        let connection = self.lock()?;
        let transaction = connection.unchecked_transaction()
            .map_err(|error| format!("Could not prepare Awareness clear: {error}"))?;
        let existing: Vec<(String, String, String)> = {
            let mut statement = transaction.prepare(
                "SELECT content, category, source_type FROM awareness_items"
            ).map_err(|error| error.to_string())?;
            let rows = statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                .map_err(|error| error.to_string())?;
            rows.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())?
        };
        for (content, category, source_type) in &existing {
            insert_awareness_suppression(&transaction, content, category, source_type)?;
        }
        let count = transaction.execute("DELETE FROM awareness_items", [])
            .map_err(|error| format!("Could not clear Awareness items: {error}"))?;
        if count > 0 { Self::record_awareness_history(&transaction, "cleared", None, None)?; }
        transaction.commit().map_err(|error| format!("Could not finish Awareness clear: {error}"))?;
        Ok(count)
    }

    pub fn list_awareness_suppressions(&self) -> Result<Vec<AwarenessSuppression>, String> {
        let connection = self.lock()?;
        let mut statement = connection.prepare(
            "SELECT category, source_type, created_at FROM awareness_suppressions ORDER BY created_at DESC"
        ).map_err(|error| format!("Could not read Awareness exclusions: {error}"))?;
        let rows = statement.query_map([], |row| Ok(AwarenessSuppression {
            category: row.get(0)?, source_type: row.get(1)?, created_at: row.get(2)?,
        })).map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())
    }

    pub fn clear_awareness_suppressions(&self) -> Result<usize, String> {
        let connection = self.lock()?;
        connection.execute("DELETE FROM awareness_suppressions", [])
            .map_err(|error| format!("Could not clear Awareness exclusions: {error}"))
    }

    pub fn is_awareness_suppressed(&self, content: &str) -> Result<bool, String> {
        let fingerprint = awareness_content_fingerprint(content);
        let connection = self.lock()?;
        let exists: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM awareness_suppressions WHERE content_hash = ?1)",
            params![fingerprint], |row| row.get(0),
        ).map_err(|error| format!("Could not check Awareness exclusion: {error}"))?;
        Ok(exists)
    }

    pub fn export_awareness_items(&self) -> Result<String, String> {
        let items = self.list_awareness_items()?;
        let json = serde_json::to_string_pretty(&serde_json::json!({
            "format": "midSunday-awareness-export",
            "formatVersion": 1,
            "exportedAt": unix_time(),
            "items": items,
        })).map_err(|error| format!("Could not export Awareness items: {error}"))?;
        let connection = self.lock()?;
        Self::record_awareness_history(&connection, "exported", None, None)?;
        Ok(json)
    }

    pub fn list_memories(&self) -> Result<Vec<SavedMemory>, String> {
        let connection = self.lock()?;
        let mut statement = connection.prepare(
            "SELECT id, content, created_at, updated_at FROM saved_memories ORDER BY updated_at DESC, id DESC"
        ).map_err(|error| error.to_string())?;
        let rows = statement.query_map([], |row| Ok(SavedMemory {
            id: row.get(0)?, content: row.get(1)?, created_at: row.get(2)?, updated_at: row.get(3)?,
        })).map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())
    }

    pub fn create_memory(&self, content: &str) -> Result<SavedMemory, String> {
        let content = clean_memory(content)?;
        let connection = self.lock()?;
        let count: i64 = connection.query_row("SELECT COUNT(*) FROM saved_memories", [], |row| row.get(0))
            .map_err(|error| error.to_string())?;
        if count >= 100 { return Err("You can save up to 100 memories.".to_owned()); }
        let now = unix_time();
        connection.execute(
            "INSERT INTO saved_memories(content, created_at, updated_at) VALUES(?1, ?2, ?2)",
            params![content, now],
        ).map_err(|error| format!("Could not save memory: {error}"))?;
        Ok(SavedMemory { id: connection.last_insert_rowid(), content, created_at: now, updated_at: now })
    }

    pub fn update_memory(&self, id: i64, content: &str) -> Result<(), String> {
        if id <= 0 { return Err("Invalid memory identifier.".to_owned()); }
        let content = clean_memory(content)?;
        let connection = self.lock()?;
        let changed = connection.execute(
            "UPDATE saved_memories SET content = ?1, updated_at = ?2 WHERE id = ?3",
            params![content, unix_time(), id],
        ).map_err(|error| error.to_string())?;
        if changed == 0 { return Err("Saved memory was not found.".to_owned()); }
        Ok(())
    }

    pub fn delete_memory(&self, id: i64) -> Result<(), String> {
        if id <= 0 { return Err("Invalid memory identifier.".to_owned()); }
        let connection = self.lock()?;
        let changed = connection.execute("DELETE FROM saved_memories WHERE id = ?1", params![id])
            .map_err(|error| error.to_string())?;
        if changed == 0 { return Err("Saved memory was not found.".to_owned()); }
        Ok(())
    }

    pub fn clear_memories(&self) -> Result<usize, String> {
        let connection = self.lock()?;
        let count = connection.execute("DELETE FROM saved_memories", [])
            .map_err(|error| error.to_string())?;
        Ok(count)
    }

    pub fn memory_enabled(&self) -> Result<bool, String> {
        let connection = self.lock()?;
        let value: Option<String> = connection.query_row(
            "SELECT value FROM app_preferences WHERE key = 'memory_enabled'", [], |row| row.get(0)
        ).optional().map_err(|error| error.to_string())?;
        Ok(value.as_deref() == Some("true"))
    }

    pub fn set_memory_enabled(&self, enabled: bool) -> Result<bool, String> {
        let connection = self.lock()?;
        connection.execute(
            "INSERT INTO app_preferences(key, value) VALUES('memory_enabled', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![if enabled { "true" } else { "false" }],
        ).map_err(|error| format!("Could not save memory preference: {error}"))?;
        Ok(enabled)
    }

    pub fn get_user_profile(&self) -> Result<UserProfile, String> {
        let connection = self.lock()?;
        let value: Option<String> = connection.query_row(
            "SELECT value FROM app_preferences WHERE key = 'user_profile'", [], |row| row.get(0)
        ).optional().map_err(|error| format!("Could not read user profile: {error}"))?;
        match value {
            Some(json) => serde_json::from_str(&json)
                .map_err(|error| format!("Could not decode saved user profile: {error}")),
            None => Ok(UserProfile::default()),
        }
    }

    pub fn save_user_profile(&self, profile: UserProfile) -> Result<UserProfile, String> {
        let preferred_name = profile.preferred_name.trim().to_owned();
        if preferred_name.chars().count() > 48 {
            return Err("Preferred names must be 48 characters or fewer.".to_owned());
        }
        if preferred_name.contains('\0') {
            return Err("Preferred names cannot contain null characters.".to_owned());
        }
        let profile = UserProfile {
            use_name: profile.use_name && !preferred_name.is_empty(),
            preferred_name,
        };
        let json = serde_json::to_string(&profile)
            .map_err(|error| format!("Could not encode user profile: {error}"))?;
        let connection = self.lock()?;
        connection.execute(
            "INSERT INTO app_preferences(key, value) VALUES('user_profile', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![json],
        ).map_err(|error| format!("Could not save user profile: {error}"))?;
        Ok(profile)
    }

    pub fn export_memories(&self) -> Result<String, String> {
        let memories = self.list_memories()?;
        let export = MemoryExportFile {
            format: "midSunday-memory-export",
            format_version: 1,
            exported_at: unix_time(),
            memories,
        };
        serde_json::to_string_pretty(&export)
            .map_err(|error| format!("Could not create memory export: {error}"))
    }

    pub fn create(&self, title: &str) -> Result<ConversationSummary, String> {
        let title = clean_title(title)?;
        let now = unix_time();
        let connection = self.lock()?;
        connection.execute(
            "INSERT INTO conversations(title, created_at, updated_at) VALUES(?1, ?2, ?2)",
            params![title, now],
        ).map_err(|error| format!("Could not create conversation: {error}"))?;
        let id = connection.last_insert_rowid();
        Ok(ConversationSummary { id, title, created_at: now, updated_at: now })
    }

    pub fn list(&self) -> Result<Vec<ConversationSummary>, String> {
        let connection = self.lock()?;
        let mut statement = connection.prepare(
            "SELECT id, title, created_at, updated_at FROM conversations ORDER BY updated_at DESC, id DESC"
        ).map_err(|error| error.to_string())?;
        let rows = statement.query_map([], |row| Ok(ConversationSummary {
            id: row.get(0)?, title: row.get(1)?, created_at: row.get(2)?, updated_at: row.get(3)?,
        })).map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())
    }

    pub fn messages(&self, conversation_id: i64) -> Result<Vec<StoredMessage>, String> {
        if conversation_id <= 0 { return Err("Invalid conversation identifier.".to_owned()); }
        let connection = self.lock()?;
        let mut statement = connection.prepare(
            "SELECT id, conversation_id, role, content, created_at
             FROM conversation_messages WHERE conversation_id = ?1 ORDER BY id ASC"
        ).map_err(|error| error.to_string())?;
        let rows = statement.query_map(params![conversation_id], |row| Ok(StoredMessage {
            id: row.get(0)?, conversation_id: row.get(1)?, role: row.get(2)?,
            content: row.get(3)?, created_at: row.get(4)?,
        })).map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())
    }

    pub fn append(&self, conversation_id: i64, role: &str, content: &str) -> Result<(), String> {
        if conversation_id <= 0 { return Err("Invalid conversation identifier.".to_owned()); }
        if role != "user" && role != "assistant" { return Err("Only user and assistant messages can be saved.".to_owned()); }
        if content.trim().is_empty() { return Err("Empty messages cannot be saved.".to_owned()); }
        if content.len() > 32_768 { return Err("Messages must be 32 KB or smaller.".to_owned()); }
        let connection = self.lock()?;
        let transaction = connection.unchecked_transaction().map_err(|error| error.to_string())?;
        let now = unix_time();
        let exists = transaction.query_row(
            "SELECT id FROM conversations WHERE id = ?1", params![conversation_id], |row| row.get::<_, i64>(0)
        ).optional().map_err(|error| error.to_string())?;
        if exists.is_none() { return Err("Conversation was not found.".to_owned()); }
        transaction.execute(
            "INSERT INTO conversation_messages(conversation_id, role, content, created_at) VALUES(?1, ?2, ?3, ?4)",
            params![conversation_id, role, content, now],
        ).map_err(|error| format!("Could not save message: {error}"))?;
        transaction.execute(
            "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
            params![now, conversation_id],
        ).map_err(|error| error.to_string())?;
        transaction.commit().map_err(|error| format!("Could not commit conversation message: {error}"))
    }

    pub fn rename(&self, conversation_id: i64, title: &str) -> Result<(), String> {
        if conversation_id <= 0 { return Err("Invalid conversation identifier.".to_owned()); }
        let title = clean_title(title)?;
        let connection = self.lock()?;
        let changed = connection.execute(
            "UPDATE conversations SET title = ?1, updated_at = ?2 WHERE id = ?3",
            params![title, unix_time(), conversation_id],
        ).map_err(|error| error.to_string())?;
        if changed == 0 { return Err("Conversation was not found.".to_owned()); }
        Ok(())
    }

    pub fn delete(&self, conversation_id: i64) -> Result<(), String> {
        if conversation_id <= 0 { return Err("Invalid conversation identifier.".to_owned()); }
        let connection = self.lock()?;
        let changed = connection.execute("DELETE FROM conversations WHERE id = ?1", params![conversation_id])
            .map_err(|error| error.to_string())?;
        if changed == 0 { return Err("Conversation was not found.".to_owned()); }
        Ok(())
    }


    pub fn import_json(&self, json: &str) -> Result<ImportResult, String> {
        const MAX_IMPORT_BYTES: usize = 20 * 1024 * 1024;
        const MAX_CONVERSATIONS: usize = 5_000;
        const MAX_MESSAGES: usize = 100_000;
        if json.len() > MAX_IMPORT_BYTES {
            return Err("The backup is larger than the 20 MB import limit.".to_owned());
        }
        let mut parsed: ConversationImportFile = serde_json::from_str(json)
            .map_err(|error| format!("This is not a valid midSunday backup: {error}"))?;
        if parsed.format != "midSunday-conversation-export" || parsed.format_version != 1 {
            return Err("This backup format or version is not supported. Nothing was imported.".to_owned());
        }
        if parsed.conversations.len() > MAX_CONVERSATIONS {
            return Err("This backup contains too many conversations (maximum 5,000).".to_owned());
        }
        let total_messages: usize = parsed.conversations.iter().map(|item| item.messages.len()).sum();
        if total_messages > MAX_MESSAGES {
            return Err("This backup contains too many messages (maximum 100,000).".to_owned());
        }

        let mut seen_conversation_ids = std::collections::HashSet::new();
        for conversation in &mut parsed.conversations {
            if conversation.id <= 0 || !seen_conversation_ids.insert(conversation.id) {
                return Err("The backup contains an invalid or repeated conversation identifier.".to_owned());
            }
            conversation.title = clean_title(&conversation.title)?;
            if conversation.created_at < 0 || conversation.updated_at < conversation.created_at {
                return Err("The backup contains invalid conversation timestamps.".to_owned());
            }
            let mut seen_message_ids = std::collections::HashSet::new();
            for message in &conversation.messages {
                if message.id <= 0 || !seen_message_ids.insert(message.id)
                    || message.conversation_id != conversation.id
                    || (message.role != "user" && message.role != "assistant")
                    || message.content.trim().is_empty()
                    || message.content.len() > 32_768
                    || message.created_at < 0
                {
                    return Err("The backup contains an invalid message. Nothing was imported.".to_owned());
                }
            }
        }

        let mut connection = self.lock()?;
        let transaction = connection.transaction()
            .map_err(|error| format!("Could not begin conversation import: {error}"))?;
        let mut result = ImportResult { imported: 0, skipped_duplicates: 0 };

        for item in parsed.conversations {
            let mut statement = transaction.prepare(
                "SELECT id FROM conversations WHERE title = ?1 AND created_at = ?2"
            ).map_err(|error| error.to_string())?;
            let candidates = statement.query_map(params![item.title, item.created_at], |row| row.get::<_, i64>(0))
                .map_err(|error| error.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| error.to_string())?;
            drop(statement);

            let mut duplicate = false;
            for existing_id in candidates {
                let existing_messages = {
                    let mut statement = transaction.prepare(
                        "SELECT role, content, created_at FROM conversation_messages
                         WHERE conversation_id = ?1 ORDER BY id ASC"
                    ).map_err(|error| error.to_string())?;
                    let rows = statement.query_map(params![existing_id], |row| Ok((
                        row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?
                    ))).map_err(|error| error.to_string())?;
                    let collected = rows.collect::<Result<Vec<_>, _>>()
                        .map_err(|error| error.to_string())?;
                    collected
                };
                let imported_messages: Vec<(String, String, i64)> = item.messages.iter()
                    .map(|message| (message.role.clone(), message.content.clone(), message.created_at))
                    .collect();
                if existing_messages == imported_messages {
                    duplicate = true;
                    break;
                }
            }
            if duplicate {
                result.skipped_duplicates += 1;
                continue;
            }

            transaction.execute(
                "INSERT INTO conversations(title, created_at, updated_at) VALUES(?1, ?2, ?3)",
                params![item.title, item.created_at, item.updated_at],
            ).map_err(|error| format!("Could not import conversation: {error}"))?;
            let new_id = transaction.last_insert_rowid();
            for message in item.messages {
                transaction.execute(
                    "INSERT INTO conversation_messages(conversation_id, role, content, created_at)
                     VALUES(?1, ?2, ?3, ?4)",
                    params![new_id, message.role, message.content, message.created_at],
                ).map_err(|error| format!("Could not import conversation messages: {error}"))?;
            }
            result.imported += 1;
        }

        transaction.commit().map_err(|error| format!("Could not finish conversation import: {error}"))?;
        Ok(result)
    }

    pub fn clear_all(&self) -> Result<usize, String> {
        let connection = self.lock()?;
        let transaction = connection.unchecked_transaction().map_err(|error| error.to_string())?;
        let count: usize = transaction.query_row("SELECT COUNT(*) FROM conversations", [], |row| row.get(0))
            .map_err(|error| error.to_string())?;
        transaction.execute("DELETE FROM conversations", [])
            .map_err(|error| format!("Could not clear local conversations: {error}"))?;
        transaction.commit().map_err(|error| error.to_string())?;
        Ok(count)
    }

    pub fn export_json(&self) -> Result<String, String> {
        // Hold one read transaction for the entire export so a concurrent write
        // cannot produce a backup with mismatched conversation/message snapshots.
        let mut connection = self.lock()?;
        let transaction = connection.transaction()
            .map_err(|error| format!("Could not begin conversation export: {error}"))?;
        let summaries = {
            let mut statement = transaction.prepare(
                "SELECT id, title, created_at, updated_at
                 FROM conversations ORDER BY updated_at DESC, id DESC"
            ).map_err(|error| error.to_string())?;
            let rows = statement.query_map([], |row| Ok(ConversationSummary {
                id: row.get(0)?, title: row.get(1)?, created_at: row.get(2)?, updated_at: row.get(3)?,
            })).map_err(|error| error.to_string())?;
            rows.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())?
        };
        let mut conversations = Vec::with_capacity(summaries.len());
        for summary in summaries {
            let messages = {
                let mut statement = transaction.prepare(
                    "SELECT id, conversation_id, role, content, created_at
                     FROM conversation_messages WHERE conversation_id = ?1 ORDER BY id ASC"
                ).map_err(|error| error.to_string())?;
                let rows = statement.query_map(params![summary.id], |row| Ok(StoredMessage {
                    id: row.get(0)?, conversation_id: row.get(1)?, role: row.get(2)?,
                    content: row.get(3)?, created_at: row.get(4)?,
                })).map_err(|error| error.to_string())?;
                rows.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())?
            };
            conversations.push(ConversationExport {
                id: summary.id,
                title: summary.title,
                created_at: summary.created_at,
                updated_at: summary.updated_at,
                messages,
            });
        }
        transaction.commit().map_err(|error| format!("Could not finish conversation export: {error}"))?;
        let export = ConversationExportFile {
            format: "midSunday-conversation-export",
            format_version: 1,
            exported_at: unix_time(),
            conversations,
        };
        serde_json::to_string_pretty(&export)
            .map_err(|error| format!("Could not create conversation export: {error}"))
    }
}

fn awareness_content_fingerprint(content: &str) -> String {
    let normalized = content.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
    let mut hasher = Sha256::new();
    hasher.update(b"midSunday-awareness-suppression-v1:");
    hasher.update(normalized.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn insert_awareness_suppression(
    connection: &Connection, content: &str, category: &str, source_type: &str,
) -> Result<(), String> {
    connection.execute(
        "INSERT INTO awareness_suppressions(content_hash, category, source_type, created_at)
         VALUES(?1, ?2, ?3, ?4)
         ON CONFLICT(content_hash) DO UPDATE SET
           category = excluded.category,
           source_type = excluded.source_type,
           created_at = excluded.created_at",
        params![awareness_content_fingerprint(content), category, source_type, unix_time()],
    ).map_err(|error| format!("Could not protect deleted Awareness information: {error}"))?;
    Ok(())
}

fn clean_awareness_category(category: &str) -> Result<String, String> {
    let category = category.trim().to_ascii_lowercase();
    match category.as_str() {
        "interests" | "preferences" | "communication" | "projects" | "goals" | "other" => Ok(category),
        _ => Err("Choose a valid Awareness category.".to_owned()),
    }
}

fn clean_awareness_content(content: &str) -> Result<String, String> {
    let content = content.trim();
    if content.is_empty() { return Err("An Awareness item cannot be empty.".to_owned()); }
    if content.len() > 4_096 || content.chars().count() > 2_000 {
        return Err("Awareness items must be 2,000 characters or 4 KiB or smaller.".to_owned());
    }
    if content.chars().any(|ch| ch == '\0') {
        return Err("Awareness items cannot contain null characters.".to_owned());
    }

    let lower = content.to_ascii_lowercase();
    let sensitive_markers = [
        "password:", "passcode:", "pin code:", "one-time password", "one time password",
        "verification code:", "authentication code:", "security code:", "otp:",
        "api key:", "api_key=", "access token:", "refresh token:", "secret key:",
        "private key:", "seed phrase:", "recovery phrase:", "credit card:", "debit card:",
        "card number:", "cvv:", "cvc:", "bank account:", "routing number:",
        "private message:", "direct message:", "dm from ",
        "-----begin private key-----", "-----begin rsa private key-----",
    ];
    if sensitive_markers.iter().any(|marker| lower.contains(marker)) {
        return Err("This looks like a password, private message, payment detail, or other sensitive data. It was not saved.".to_owned());
    }
    if ["sk-", "ghp_", "github_pat_", "xoxb-", "xoxp-", "bearer "]
        .iter().any(|marker| lower.contains(marker))
    {
        return Err("This looks like an authentication token or secret. It was not saved.".to_owned());
    }
    for part in content.split_whitespace() {
        let candidate = part.trim_matches(|ch: char| !ch.is_alphanumeric() && ch != '@' && ch != '.' && ch != '_' && ch != '-');
        if let Some((local, domain)) = candidate.split_once('@') {
            if !local.is_empty() && domain.contains('.') {
                return Err("Email addresses are treated as private identifiers and are not saved in Awareness.".to_owned());
            }
        }
    }
    let digit_count = content.chars().filter(|ch| ch.is_ascii_digit()).count();
    if (13..=19).contains(&digit_count) {
        return Err("This may contain a payment-card or other long numeric identifier. It was not saved.".to_owned());
    }
    Ok(content.to_owned())
}

fn clean_memory(content: &str) -> Result<String, String> {
    let content = content.trim();
    if content.is_empty() { return Err("A saved memory cannot be empty.".to_owned()); }
    if content.len() > 4_096 || content.chars().count() > 2_000 {
        return Err("Saved memories must be 2,000 characters or 4 KiB or smaller.".to_owned());
    }
    if content.chars().any(|ch| ch == '\0') {
        return Err("Saved memories cannot contain null characters.".to_owned());
    }
    Ok(content.to_owned())
}

fn clean_title(title: &str) -> Result<String, String> {
    let title = title.trim();
    if title.is_empty() { return Err("Conversation title cannot be empty.".to_owned()); }
    if title.chars().count() > 120 { return Err("Conversation titles must be 120 characters or fewer.".to_owned()); }
    Ok(title.to_owned())
}

fn unix_time() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> ConversationStore {
        ConversationStore::from_connection(Connection::open_in_memory().unwrap()).unwrap()
    }

    #[test]
    fn creates_edits_lists_and_deletes_explicit_memories() {
        let store = store();
        let created = store.create_memory("I prefer simple explanations.").unwrap();
        assert_eq!(store.list_memories().unwrap()[0].content, "I prefer simple explanations.");
        store.update_memory(created.id, "I prefer short, simple explanations.").unwrap();
        assert_eq!(store.list_memories().unwrap()[0].content, "I prefer short, simple explanations.");
        store.delete_memory(created.id).unwrap();
        assert!(store.list_memories().unwrap().is_empty());
    }

    #[test]
    fn awareness_items_support_local_crud_export_and_sensitive_data_rejection() {
        let store = store();
        assert!(store.list_awareness_items().unwrap().is_empty());
        let item = store.create_awareness_item("I enjoy technology and anime.", "interests").unwrap();
        assert_eq!(item.source_type, "user_entered");
        assert_eq!(item.knowledge_type, "observed");
        assert_eq!(item.confidence, 1.0);
        assert_eq!(store.list_awareness_items().unwrap()[0].content, "I enjoy technology and anime.");

        store.update_awareness_item(item.id, "I enjoy technology, anime, and learning.", "preferences").unwrap();
        let updated = store.list_awareness_items().unwrap();
        assert_eq!(updated[0].category, "preferences");
        assert_eq!(updated[0].content, "I enjoy technology, anime, and learning.");

        assert!(store.create_awareness_item("My password: hunter2", "other").is_err());
        assert!(store.create_awareness_item("My email is person@example.com", "other").is_err());
        assert!(store.create_awareness_item("My card is 4111 1111 1111 1111", "other").is_err());
        assert!(store.create_awareness_item("Token: ghp_abcdefghijklmnopqrstuvwxyz", "other").is_err());
        assert!(store.create_awareness_item("A harmless fact", "unknown").is_err());

        let export: serde_json::Value = serde_json::from_str(&store.export_awareness_items().unwrap()).unwrap();
        assert_eq!(export["format"], "midSunday-awareness-export");
        assert_eq!(export["formatVersion"], 1);
        assert_eq!(export["items"].as_array().unwrap().len(), 1);

        store.delete_awareness_item(item.id).unwrap();
        assert!(store.list_awareness_items().unwrap().is_empty());
        assert!(store.delete_awareness_item(item.id).is_err());
    }

    #[test]
    fn clear_awareness_items_does_not_clear_saved_memories_or_conversations() {
        let store = store();
        let item = store.create_awareness_item("I like strategy games.", "interests").unwrap();
        store.create_memory("Keep this separate").unwrap();
        let chat = store.create("Keep this chat").unwrap();
        store.append(chat.id, "user", "Hello").unwrap();

        assert_eq!(store.clear_awareness_items().unwrap(), 1);
        assert!(store.list_awareness_items().unwrap().is_empty());
        assert_eq!(store.list_memories().unwrap()[0].content, "Keep this separate");
        assert_eq!(store.list().unwrap()[0].title, "Keep this chat");
        assert_ne!(item.id, 0);
    }

    #[test]
    fn preferred_name_is_local_persistent_and_validated() {
        let store = store();
        assert_eq!(store.get_user_profile().unwrap(), UserProfile::default());
        let saved = store.save_user_profile(UserProfile {
            preferred_name: "  Hasan  ".to_owned(),
            use_name: true,
        }).unwrap();
        assert_eq!(saved.preferred_name, "Hasan");
        assert!(saved.use_name);
        assert_eq!(store.get_user_profile().unwrap(), saved);
        let disabled = store.save_user_profile(UserProfile {
            preferred_name: String::new(),
            use_name: true,
        }).unwrap();
        assert!(!disabled.use_name);
        assert!(store.save_user_profile(UserProfile {
            preferred_name: "H".repeat(49),
            use_name: true,
        }).is_err());
    }

    #[test]
    fn awareness_settings_are_off_by_default_and_each_source_preference_persists() {
        let store = store();
        assert_eq!(store.get_awareness_settings().unwrap(), AwarenessSettings::default());
        let saved = store.save_awareness_settings(AwarenessSettings {
            enabled: true, learn_conversations: true, learn_searches: false,
        }).unwrap();
        assert_eq!(store.get_awareness_settings().unwrap(), saved);
        let paused = store.save_awareness_settings(AwarenessSettings {
            enabled: false, learn_conversations: true, learn_searches: false,
        }).unwrap();
        assert!(!paused.enabled);
        assert!(paused.learn_conversations);
        assert!(!paused.learn_searches);
        assert_eq!(store.get_awareness_settings().unwrap(), paused);
    }

    #[test]
    fn awareness_schema_stores_activity_metadata_without_raw_activity_content() {
        let store = store();
        let connection = store.lock().unwrap();
        let item_columns: Vec<String> = {
            let mut statement = connection.prepare("PRAGMA table_info(awareness_items)").unwrap();
            statement.query_map([], |row| row.get(1)).unwrap().collect::<Result<Vec<_>, _>>().unwrap()
        };
        let activity_columns: Vec<String> = {
            let mut statement = connection.prepare("PRAGMA table_info(awareness_activity)").unwrap();
            statement.query_map([], |row| row.get(1)).unwrap().collect::<Result<Vec<_>, _>>().unwrap()
        };
        assert!(item_columns.contains(&"content".to_owned()));
        assert!(!activity_columns.contains(&"content".to_owned()));
        assert!(!activity_columns.contains(&"raw_text".to_owned()));
    }

    #[test]
    fn migrates_schema_v2_without_losing_existing_memories() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch(
            "CREATE TABLE conversations (id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL);
             CREATE TABLE conversation_messages (id INTEGER PRIMARY KEY AUTOINCREMENT, conversation_id INTEGER NOT NULL REFERENCES conversations(id) ON DELETE CASCADE, role TEXT NOT NULL CHECK(role IN ('user', 'assistant')), content TEXT NOT NULL, created_at INTEGER NOT NULL);
             CREATE INDEX idx_conversation_messages_order ON conversation_messages(conversation_id, id);
             CREATE INDEX idx_conversations_updated ON conversations(updated_at DESC);
             CREATE TABLE saved_memories (id INTEGER PRIMARY KEY AUTOINCREMENT, content TEXT NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL);
             CREATE INDEX idx_saved_memories_updated ON saved_memories(updated_at DESC, id DESC);
             CREATE TABLE app_preferences (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO saved_memories(content, created_at, updated_at) VALUES('Keep my existing fact', 1, 1);
             INSERT INTO app_preferences(key, value) VALUES('memory_enabled', 'true');
             PRAGMA user_version = 2;"
        ).unwrap();
        let store = ConversationStore::from_connection(connection).unwrap();
        assert_eq!(store.list_memories().unwrap()[0].content, "Keep my existing fact");
        assert!(store.memory_enabled().unwrap());
        assert_eq!(store.get_awareness_settings().unwrap(), AwarenessSettings::default());
        let version: i64 = store.lock().unwrap().pragma_query_value(None, "user_version", |row| row.get(0)).unwrap();
        assert_eq!(version, 5);
    }

    #[test]
    fn awareness_history_tracks_management_without_storing_item_text() {
        let store = store();
        let item = store.create_awareness_item("I like strategy games.", "interests").unwrap();
        store.update_awareness_item(item.id, "I like strategy and puzzle games.", "preferences").unwrap();
        store.export_awareness_items().unwrap();
        store.delete_awareness_item(item.id).unwrap();

        let history = store.list_awareness_history().unwrap();
        assert_eq!(history.iter().map(|entry| entry.event_type.as_str()).collect::<Vec<_>>(),
            vec!["deleted", "exported", "updated", "added"]);
        assert_eq!(history[0].awareness_item_id, Some(item.id));
        assert_eq!(history[0].category.as_deref(), Some("preferences"));
        assert_eq!(history[3].category.as_deref(), Some("interests"));

        let connection = store.lock().unwrap();
        let columns: Vec<String> = {
            let mut statement = connection.prepare("PRAGMA table_info(awareness_history)").unwrap();
            statement.query_map([], |row| row.get(1)).unwrap().collect::<Result<Vec<_>, _>>().unwrap()
        };
        assert!(!columns.contains(&"content".to_owned()));
        assert!(!columns.contains(&"raw_text".to_owned()));
    }

    #[test]
    fn clearing_awareness_history_does_not_delete_awareness_items() {
        let store = store();
        store.create_awareness_item("I like science.", "interests").unwrap();
        assert_eq!(store.clear_awareness_history().unwrap(), 1);
        assert!(store.list_awareness_history().unwrap().is_empty());
        assert_eq!(store.list_awareness_items().unwrap().len(), 1);
    }

    #[test]
    fn deleted_awareness_content_is_fingerprint_suppressed_without_retaining_raw_text() {
        let store = store();
        let content = "I enjoy strategy games and science fiction.";
        let item = store.create_awareness_item(content, "interests").unwrap();
        assert!(!store.is_awareness_suppressed(content).unwrap());
        store.delete_awareness_item(item.id).unwrap();
        assert!(store.is_awareness_suppressed("  I ENJOY strategy games and science fiction.  ").unwrap());
        assert!(store.list_awareness_items().unwrap().is_empty());
        assert_eq!(store.list_awareness_suppressions().unwrap().len(), 1);

        let connection = store.lock().unwrap();
        let columns: Vec<String> = {
            let mut statement = connection.prepare("PRAGMA table_info(awareness_suppressions)").unwrap();
            statement.query_map([], |row| row.get(1)).unwrap().collect::<Result<Vec<_>, _>>().unwrap()
        };
        assert!(!columns.contains(&"content".to_owned()));
        assert!(!columns.contains(&"raw_text".to_owned()));
        let stored_hash: String = connection.query_row(
            "SELECT content_hash FROM awareness_suppressions", [], |row| row.get(0)
        ).unwrap();
        assert_eq!(stored_hash.len(), 64);
        assert!(!stored_hash.contains("strategy"));
        drop(connection);

        // Deliberate user entry is always allowed; suppression is for future automatic ingestion.
        store.create_awareness_item(content, "interests").unwrap();
        assert_eq!(store.list_awareness_items().unwrap().len(), 1);
        assert_eq!(store.clear_awareness_suppressions().unwrap(), 1);
        assert!(!store.is_awareness_suppressed(content).unwrap());
    }

    #[test]
    fn editing_and_clearing_awareness_items_suppress_replaced_information() {
        let store = store();
        let item = store.create_awareness_item("I like racing games.", "interests").unwrap();
        store.update_awareness_item(item.id, "I prefer puzzle games.", "preferences").unwrap();
        assert!(store.is_awareness_suppressed("I like racing games.").unwrap());
        assert!(!store.is_awareness_suppressed("I prefer puzzle games.").unwrap());
        assert_eq!(store.clear_awareness_items().unwrap(), 1);
        assert!(store.is_awareness_suppressed("I prefer puzzle games.").unwrap());
        assert_eq!(store.list_awareness_suppressions().unwrap().len(), 2);
    }

    #[test]
    fn memory_is_disabled_by_default_and_preference_persists() {
        let store = store();
        assert!(!store.memory_enabled().unwrap());
        assert!(store.set_memory_enabled(true).unwrap());
        assert!(store.memory_enabled().unwrap());
        assert!(!store.set_memory_enabled(false).unwrap());
        assert!(!store.memory_enabled().unwrap());
    }

    #[test]
    fn validates_memory_size_and_limit_and_exports_separately() {
        let store = store();
        assert!(store.create_memory("  ").is_err());
        assert!(store.create_memory(&"x".repeat(4_097)).is_err());
        for index in 0..100 {
            store.create_memory(&format!("Memory {index}")).unwrap();
        }
        assert!(store.create_memory("One too many").is_err());
        let export: serde_json::Value = serde_json::from_str(&store.export_memories().unwrap()).unwrap();
        assert_eq!(export["format"], "midSunday-memory-export");
        assert_eq!(export["formatVersion"], 1);
        assert_eq!(export["memories"].as_array().unwrap().len(), 100);
        assert!(store.clear_memories().unwrap() == 100);
        assert!(store.list_memories().unwrap().is_empty());
    }

    #[test]
    fn creates_lists_and_renames_conversations() {
        let store = store();
        let created = store.create("A useful conversation").unwrap();
        assert_eq!(store.list().unwrap()[0].title, "A useful conversation");
        store.rename(created.id, "A renamed conversation").unwrap();
        assert_eq!(store.list().unwrap()[0].title, "A renamed conversation");
    }

    #[test]
    fn saves_messages_in_order_and_rejects_invalid_content() {
        let store = store();
        let conversation = store.create("Test").unwrap();
        store.append(conversation.id, "user", "Hello").unwrap();
        store.append(conversation.id, "assistant", "Hi there").unwrap();
        let messages = store.messages(conversation.id).unwrap();
        assert_eq!(messages.iter().map(|message| message.content.as_str()).collect::<Vec<_>>(), vec!["Hello", "Hi there"]);
        assert!(store.append(conversation.id, "system", "not allowed").is_err());
        assert!(store.append(conversation.id, "user", "  ").is_err());
        assert!(store.append(conversation.id, "user", &"x".repeat(32_769)).is_err());
    }

    #[test]
    fn deleting_conversation_cascades_to_messages() {
        let store = store();
        let conversation = store.create("Delete me").unwrap();
        store.append(conversation.id, "user", "Private text").unwrap();
        store.delete(conversation.id).unwrap();
        assert!(store.list().unwrap().is_empty());
        assert!(store.messages(conversation.id).unwrap().is_empty());
    }

    #[test]
    fn rejects_empty_or_overlong_titles() {
        let store = store();
        assert!(store.create("  ").is_err());
        assert!(store.create(&"x".repeat(121)).is_err());
    }

    #[test]
    fn upgrades_existing_unversioned_conversation_database_without_losing_data() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch(
            "PRAGMA foreign_keys = ON;
             CREATE TABLE conversations (
               id INTEGER PRIMARY KEY AUTOINCREMENT,
               title TEXT NOT NULL,
               created_at INTEGER NOT NULL,
               updated_at INTEGER NOT NULL
             );
             CREATE TABLE conversation_messages (
               id INTEGER PRIMARY KEY AUTOINCREMENT,
               conversation_id INTEGER NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
               role TEXT NOT NULL CHECK(role IN ('user', 'assistant')),
               content TEXT NOT NULL,
               created_at INTEGER NOT NULL
             );
             CREATE INDEX idx_conversation_messages_order ON conversation_messages(conversation_id, id);
             CREATE INDEX idx_conversations_updated ON conversations(updated_at DESC);
             INSERT INTO conversations(title, created_at, updated_at) VALUES('Existing chat', 1, 1);
             INSERT INTO conversation_messages(conversation_id, role, content, created_at) VALUES(1, 'user', 'Keep this', 1);"
        ).unwrap();
        let store = ConversationStore::from_connection(connection).unwrap();
        assert_eq!(store.list().unwrap()[0].title, "Existing chat");
        assert_eq!(store.messages(1).unwrap()[0].content, "Keep this");
        let version: i64 = store.lock().unwrap()
            .pragma_query_value(None, "user_version", |row| row.get(0)).unwrap();
        assert_eq!(version, 5);
        assert!(!store.memory_enabled().unwrap());
        assert_eq!(store.get_awareness_settings().unwrap(), AwarenessSettings::default());
        assert!(store.list_memories().unwrap().is_empty());
        assert!(store.list_memories().unwrap().is_empty());
    }

    #[test]
    fn exports_conversations_with_versioned_metadata_and_messages() {
        let store = store();
        let conversation = store.create("Export me").unwrap();
        store.append(conversation.id, "user", "Hello export").unwrap();
        let json = store.export_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["format"], "midSunday-conversation-export");
        assert_eq!(parsed["formatVersion"], 1);
        assert_eq!(parsed["conversations"][0]["title"], "Export me");
        assert_eq!(parsed["conversations"][0]["messages"][0]["content"], "Hello export");
    }


    #[test]
    fn imports_backup_and_remaps_identifiers_without_overwriting_existing_chats() {
        let store = store();
        let existing = store.create("Keep existing").unwrap();
        store.append(existing.id, "user", "Do not replace me").unwrap();
        let backup = serde_json::json!({
            "format": "midSunday-conversation-export",
            "formatVersion": 1,
            "exportedAt": 10,
            "conversations": [{
                "id": 77, "title": "Imported chat", "createdAt": 2, "updatedAt": 3,
                "messages": [{"id": 88, "conversationId": 77, "role": "user", "content": "Imported safely", "createdAt": 3}]
            }]
        }).to_string();
        let result = store.import_json(&backup).unwrap();
        assert_eq!(result.imported, 1);
        assert_eq!(result.skipped_duplicates, 0);
        let items = store.list().unwrap();
        assert_eq!(items.len(), 2);
        let imported = items.iter().find(|item| item.title == "Imported chat").unwrap();
        assert_ne!(imported.id, 77);
        assert_eq!(store.messages(imported.id).unwrap()[0].content, "Imported safely");
        assert_eq!(store.messages(existing.id).unwrap()[0].content, "Do not replace me");
    }

    #[test]
    fn repeated_import_skips_exact_duplicates() {
        let store = store();
        let backup = serde_json::json!({
            "format": "midSunday-conversation-export", "formatVersion": 1, "exportedAt": 10,
            "conversations": [{
                "id": 7, "title": "Same chat", "createdAt": 2, "updatedAt": 3,
                "messages": [{"id": 8, "conversationId": 7, "role": "user", "content": "Same message", "createdAt": 3}]
            }]
        }).to_string();
        assert_eq!(store.import_json(&backup).unwrap().imported, 1);
        let second = store.import_json(&backup).unwrap();
        assert_eq!(second.imported, 0);
        assert_eq!(second.skipped_duplicates, 1);
        assert_eq!(store.list().unwrap().len(), 1);
    }

    #[test]
    fn rejects_invalid_backup_without_partially_importing() {
        let store = store();
        store.create("Already here").unwrap();
        let backup = serde_json::json!({
            "format": "midSunday-conversation-export", "formatVersion": 1, "exportedAt": 10,
            "conversations": [
                {"id": 7, "title": "Would be valid", "createdAt": 2, "updatedAt": 3, "messages": []},
                {"id": 8, "title": "Invalid", "createdAt": 2, "updatedAt": 3,
                 "messages": [{"id": 9, "conversationId": 999, "role": "system", "content": "Bad", "createdAt": 3}]}
            ]
        }).to_string();
        assert!(store.import_json(&backup).is_err());
        assert_eq!(store.list().unwrap().len(), 1);
        assert_eq!(store.list().unwrap()[0].title, "Already here");
    }

    #[test]
    fn import_trims_titles_and_round_trips_exported_backups() {
        let source = store();
        let conversation = source.create("Round trip").unwrap();
        source.append(conversation.id, "user", "Keep this message").unwrap();
        let backup = source.export_json().unwrap();

        let destination = store();
        let result = destination.import_json(&backup).unwrap();
        assert_eq!(result.imported, 1);
        assert_eq!(destination.list().unwrap()[0].title, "Round trip");
        assert_eq!(destination.messages(destination.list().unwrap()[0].id).unwrap()[0].content, "Keep this message");

        let padded = serde_json::json!({
            "format": "midSunday-conversation-export", "formatVersion": 1, "exportedAt": 10,
            "conversations": [{
                "id": 42, "title": "  Trim this title  ", "createdAt": 2, "updatedAt": 3,
                "messages": []
            }]
        }).to_string();
        let imported = destination.import_json(&padded).unwrap();
        assert_eq!(imported.imported, 1);
        assert!(destination.list().unwrap().iter().any(|item| item.title == "Trim this title"));
    }

    #[test]
    fn rejects_invalid_conversation_timestamps_and_titles_before_import() {
        let store = store();
        let invalid_timestamp = serde_json::json!({
            "format": "midSunday-conversation-export", "formatVersion": 1, "exportedAt": 10,
            "conversations": [{
                "id": 1, "title": "Valid", "createdAt": 5, "updatedAt": 4, "messages": []
            }]
        }).to_string();
        assert!(store.import_json(&invalid_timestamp).is_err());

        let invalid_title = serde_json::json!({
            "format": "midSunday-conversation-export", "formatVersion": 1, "exportedAt": 10,
            "conversations": [{
                "id": 1, "title": "   ", "createdAt": 1, "updatedAt": 1, "messages": []
            }]
        }).to_string();
        assert!(store.import_json(&invalid_title).is_err());
        assert!(store.list().unwrap().is_empty());
    }

    #[test]
    fn rejects_unsupported_backup_version() {
        let store = store();
        let backup = r#"{"format":"midSunday-conversation-export","formatVersion":99,"exportedAt":1,"conversations":[]}"#;
        assert!(store.import_json(backup).is_err());
        assert!(store.list().unwrap().is_empty());
    }

    #[test]
    fn clearing_history_removes_all_conversations_and_messages() {
        let store = store();
        let first = store.create("First").unwrap();
        let second = store.create("Second").unwrap();
        store.append(first.id, "user", "Private one").unwrap();
        store.append(second.id, "assistant", "Private two").unwrap();
        assert_eq!(store.clear_all().unwrap(), 2);
        assert!(store.list().unwrap().is_empty());
        assert!(store.messages(first.id).unwrap().is_empty());
    }
}
