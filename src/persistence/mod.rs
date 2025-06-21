pub mod config;
use rusqlite::{params, Connection, Result};
use std::path::PathBuf;
use crate::openai::Message;

// pub struct Conversation {
//     pub id: i64,
//     pub title: String,
//     pub created_at: i64,
// }

pub struct ConversationStore {
    pub conn: Connection,
}

impl ConversationStore {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS conversation (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS message (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_id INTEGER NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY(conversation_id) REFERENCES conversation(id)
            );
        "#)?;
        Ok(Self { conn })
    }

    pub fn create_conversation(&self, title: &str) -> Result<i64> {
        let now = chrono::Utc::now().timestamp();
        self.conn.execute(
            "INSERT INTO conversation (title, created_at) VALUES (?, ?)",
            params![title, now],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn add_message(&self, conversation_id: i64, message: &Message) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        self.conn.execute(
            "INSERT INTO message (conversation_id, role, content, created_at) VALUES (?, ?, ?, ?)",
            params![conversation_id, &message.role, &message.content, now],
        )?;
        Ok(())
    }
}

