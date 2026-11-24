use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::{path::Path, sync::Mutex, time::{SystemTime, UNIX_EPOCH}};

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConversationSummary {
    pub id: i64,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
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
        if schema_version > 1 {
            return Err(format!("This midSunday database uses a newer schema version ({schema_version}) than this app supports."));
        }
        if schema_version == 0 {
            let transaction = connection.unchecked_transaction()
                .map_err(|error| format!("Could not prepare local database migration: {error}"))?;
            transaction.execute_batch(
                "CREATE TABLE conversations (
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
                 CREATE INDEX idx_conversation_messages_order
                   ON conversation_messages(conversation_id, id);
                 CREATE INDEX idx_conversations_updated
                   ON conversations(updated_at DESC);
                 PRAGMA user_version = 1;"
            ).map_err(|error| format!("Could not apply local database migration 1: {error}"))?;
            transaction.commit().map_err(|error| format!("Could not finish local database migration: {error}"))?;
        }
        Ok(Self { connection: Mutex::new(connection) })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, String> {
        self.connection.lock().map_err(|_| "Local conversation database is unavailable.".to_owned())
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
        let summaries = self.list()?;
        let mut conversations = Vec::with_capacity(summaries.len());
        for summary in summaries {
            conversations.push(ConversationExport {
                id: summary.id,
                title: summary.title,
                created_at: summary.created_at,
                updated_at: summary.updated_at,
                messages: self.messages(summary.id)?,
            });
        }
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
