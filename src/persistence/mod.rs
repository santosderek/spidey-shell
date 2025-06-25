pub mod config;

use crate::openai::Message;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Conversation {
    pub id: i64,
    pub title: String,
    pub created_at: i64,
    pub mcp_servers: Vec<String>, // List of enabled MCP servers for this conversation
}

pub struct ConversationStore {
    pub conn: Connection,
}

impl ConversationStore {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        // First create the tables if they don't exist
        conn.execute_batch(
            r#"
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
            CREATE TABLE IF NOT EXISTS mcp_activation (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_id INTEGER NOT NULL,
                mcp_server TEXT NOT NULL,
                is_active BOOLEAN DEFAULT 1 NOT NULL,
                activated_at INTEGER NOT NULL,
                FOREIGN KEY(conversation_id) REFERENCES conversation(id)
            );
        "#,
        )?;

        // Now add the columns if they don't exist
        // We need to check if the columns exist before adding them
        let has_mcp_servers = conn.query_row::<bool, _, _>(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('conversation') WHERE name = 'mcp_servers'",
            [],
            |row| row.get(0),
        ).unwrap_or(false);

        let has_mcp_server = conn
            .query_row::<bool, _, _>(
                "SELECT COUNT(*) > 0 FROM pragma_table_info('message') WHERE name = 'mcp_server'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_mcp_servers {
            conn.execute(
                "ALTER TABLE conversation ADD COLUMN mcp_servers TEXT DEFAULT '[]' NOT NULL",
                [],
            )?;
        }

        if !has_mcp_server {
            conn.execute(
                "ALTER TABLE message ADD COLUMN mcp_server TEXT DEFAULT NULL",
                [],
            )?;
        }
        Ok(Self { conn })
    }

    pub fn create_conversation(&self, title: &str) -> Result<i64> {
        let now = chrono::Utc::now().timestamp();
        self.conn.execute(
            "INSERT INTO conversation (title, created_at, mcp_servers) VALUES (?, ?, '[]')",
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

    /// Add a message with a specific MCP server
    pub fn add_message_with_server(
        &self,
        conversation_id: i64,
        message: &Message,
        mcp_server: &str,
    ) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        self.conn.execute(
            "INSERT INTO message (conversation_id, role, content, created_at, mcp_server) VALUES (?, ?, ?, ?, ?)",
            params![conversation_id, &message.role, &message.content, now, mcp_server],
        )?;
        Ok(())
    }

    /// Get a conversation by ID
    pub fn get_conversation(&self, id: i64) -> Result<Option<Conversation>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, created_at, mcp_servers FROM conversation WHERE id = ?")?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            let mcp_servers_json: String = row.get(3)?;
            let mcp_servers: Vec<String> =
                serde_json::from_str(&mcp_servers_json).unwrap_or_else(|_| Vec::new());

            Ok(Some(Conversation {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
                mcp_servers,
            }))
        } else {
            Ok(None)
        }
    }

    /// Enable an MCP server for a conversation
    pub fn enable_mcp_server(&self, conversation_id: i64, server_name: &str) -> Result<()> {
        // First retrieve the current list
        if let Some(mut conversation) = self.get_conversation(conversation_id)? {
            if !conversation.mcp_servers.contains(&server_name.to_string()) {
                conversation.mcp_servers.push(server_name.to_string());
                let mcp_servers_json = serde_json::to_string(&conversation.mcp_servers)
                    .unwrap_or_else(|_| "[]".to_string());

                // Update the conversation record
                self.conn.execute(
                    "UPDATE conversation SET mcp_servers = ? WHERE id = ?",
                    params![mcp_servers_json, conversation_id],
                )?;

                // Add an activation record
                let now = chrono::Utc::now().timestamp();
                self.conn.execute(
                    "INSERT INTO mcp_activation (conversation_id, mcp_server, is_active, activated_at) VALUES (?, ?, 1, ?)",
                    params![conversation_id, server_name, now],
                )?;
            }
        }

        Ok(())
    }

    /// Disable an MCP server for a conversation
    pub fn disable_mcp_server(&self, conversation_id: i64, server_name: &str) -> Result<()> {
        // First retrieve the current list
        if let Some(mut conversation) = self.get_conversation(conversation_id)? {
            conversation.mcp_servers.retain(|s| s != server_name);
            let mcp_servers_json = serde_json::to_string(&conversation.mcp_servers)
                .unwrap_or_else(|_| "[]".to_string());

            // Update the conversation record
            self.conn.execute(
                "UPDATE conversation SET mcp_servers = ? WHERE id = ?",
                params![mcp_servers_json, conversation_id],
            )?;

            // Add a deactivation record
            let now = chrono::Utc::now().timestamp();
            self.conn.execute(
                "INSERT INTO mcp_activation (conversation_id, mcp_server, is_active, activated_at) VALUES (?, ?, 0, ?)",
                params![conversation_id, server_name, now],
            )?;
        }

        Ok(())
    }

    /// Get all enabled MCP servers for a conversation
    pub fn get_enabled_servers(&self, conversation_id: i64) -> Result<Vec<String>> {
        if let Some(conversation) = self.get_conversation(conversation_id)? {
            Ok(conversation.mcp_servers)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get all conversations from the database
    pub fn get_all_conversations(&self) -> Result<Vec<Conversation>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, created_at, mcp_servers FROM conversation ORDER BY created_at DESC",
        )?;

        let conversations = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let title: String = row.get(1)?;
            let created_at: i64 = row.get(2)?;
            let mcp_servers_json: String = row.get(3)?;
            let mcp_servers: Vec<String> =
                serde_json::from_str(&mcp_servers_json).unwrap_or_else(|_| Vec::new());

            Ok(Conversation {
                id,
                title,
                created_at,
                mcp_servers,
            })
        })?;

        let mut result = Vec::new();
        for conversation in conversations {
            if let Ok(conv) = conversation {
                result.push(conv);
            }
        }

        Ok(result)
    }

    /// Get all messages for a specific conversation
    pub fn get_conversation_messages(&self, conversation_id: i64) -> Result<Vec<Message>> {
        let mut stmt = self.conn.prepare(
            "SELECT role, content FROM message WHERE conversation_id = ? ORDER BY created_at ASC",
        )?;

        let messages = stmt.query_map(params![conversation_id], |row| {
            let role: String = row.get(0)?;
            let content: String = row.get(1)?;

            Ok(Message { role, content })
        })?;

        let mut result = Vec::new();
        for message in messages {
            if let Ok(msg) = message {
                result.push(msg);
            }
        }

        Ok(result)
    }
}
