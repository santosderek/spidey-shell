use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, Set,
};
use chrono::Utc;
use std::error::Error;

use crate::openai::Message;
use crate::persistence::entities::generation::{self, ActiveModel, Entity as Generation};

pub struct GenerationRepository {
    db: DatabaseConnection,
}

impl GenerationRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn add_message(&self, conversation_id: i32, message: &Message) -> Result<(), Box<dyn Error>> {
        let now = Utc::now().timestamp();
        
        let generation = ActiveModel {
            id: Default::default(), // Auto-increment
            conversation_id: Set(conversation_id),
            role: Set(message.role.clone()),
            content: Set(message.content.clone()),
            created_at: Set(now),
            mcp_server: Set(None), // No MCP server by default
        };

        generation.insert(&self.db).await?;
        
        Ok(())
    }

    pub async fn add_message_with_server(&self, conversation_id: i32, message: &Message, mcp_server: &str) -> Result<(), Box<dyn Error>> {
        let now = Utc::now().timestamp();
        
        let generation = ActiveModel {
            id: Default::default(), // Auto-increment
            conversation_id: Set(conversation_id),
            role: Set(message.role.clone()),
            content: Set(message.content.clone()),
            created_at: Set(now),
            mcp_server: Set(Some(mcp_server.to_string())),
        };

        generation.insert(&self.db).await?;
        
        Ok(())
    }

    pub async fn get_conversation_messages(&self, conversation_id: i32) -> Result<Vec<Message>, Box<dyn Error>> {
        let generations = Generation::find()
            .filter(generation::Column::ConversationId.eq(conversation_id))
            .order_by_asc(generation::Column::CreatedAt)
            .all(&self.db)
            .await?;
            
        let messages = generations.into_iter()
            .map(|gen| Message {
                role: gen.role,
                content: gen.content,
            })
            .collect();
            
        Ok(messages)
    }
}