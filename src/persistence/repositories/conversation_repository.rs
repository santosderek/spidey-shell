use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, Set,
};
use chrono::Utc;
use serde_json;
use std::error::Error;

use crate::persistence::entities::conversation::{self, ActiveModel, Entity as Conversation};

pub struct ConversationRepository {
    db: DatabaseConnection,
}

impl ConversationRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(&self, title: &str) -> Result<i32, Box<dyn Error>> {
        let now = Utc::now().timestamp();
        
        let conversation = ActiveModel {
            id: Default::default(), // Auto-increment
            title: Set(title.to_string()),
            created_at: Set(now),
            mcp_servers: Set(Some("[]".to_string())), // Empty JSON array
        };

        let result = conversation.insert(&self.db).await?;
        
        Ok(result.id)
    }

    pub async fn get_by_id(&self, id: i32) -> Result<Option<conversation::Model>, Box<dyn Error>> {
        let result = Conversation::find_by_id(id)
            .one(&self.db)
            .await?;
            
        Ok(result)
    }

    pub async fn get_all(&self) -> Result<Vec<conversation::Model>, Box<dyn Error>> {
        let conversations = Conversation::find()
            .order_by_desc(conversation::Column::CreatedAt)
            .all(&self.db)
            .await?;
            
        Ok(conversations)
    }

    pub async fn enable_mcp_server(&self, conversation_id: i32, server_name: &str) -> Result<(), Box<dyn Error>> {
        if let Some(conversation) = self.get_by_id(conversation_id).await? {
            // Parse the current MCP servers list
            let mcp_servers_json = conversation.mcp_servers.unwrap_or_else(|| "[]".to_string());
            let mut mcp_servers: Vec<String> = serde_json::from_str(&mcp_servers_json)?;
            
            // Add the server if it's not already in the list
            if !mcp_servers.contains(&server_name.to_string()) {
                mcp_servers.push(server_name.to_string());
                let updated_servers_json = serde_json::to_string(&mcp_servers)?;
                
                // Update the conversation record
                let mut conversation_model: ActiveModel = conversation.into();
                conversation_model.mcp_servers = Set(Some(updated_servers_json));
                conversation_model.update(&self.db).await?;
                
                // Add an activation record (we'll need to add this to the MCP activation repository)
                // For now, we're not implementing the MCP activation table
            }
        }
        
        Ok(())
    }

    pub async fn disable_mcp_server(&self, conversation_id: i32, server_name: &str) -> Result<(), Box<dyn Error>> {
        if let Some(conversation) = self.get_by_id(conversation_id).await? {
            // Parse the current MCP servers list
            let mcp_servers_json = conversation.mcp_servers.unwrap_or_else(|| "[]".to_string());
            let mut mcp_servers: Vec<String> = serde_json::from_str(&mcp_servers_json)?;
            
            // Remove the server from the list
            mcp_servers.retain(|s| s != server_name);
            let updated_servers_json = serde_json::to_string(&mcp_servers)?;
            
            // Update the conversation record
            let mut conversation_model: ActiveModel = conversation.into();
            conversation_model.mcp_servers = Set(Some(updated_servers_json));
            conversation_model.update(&self.db).await?;
            
            // Add a deactivation record (we'll need to add this to the MCP activation repository)
            // For now, we're not implementing the MCP activation table
        }
        
        Ok(())
    }

    pub async fn get_enabled_servers(&self, conversation_id: i32) -> Result<Vec<String>, Box<dyn Error>> {
        if let Some(conversation) = self.get_by_id(conversation_id).await? {
            let mcp_servers_json = conversation.mcp_servers.unwrap_or_else(|| "[]".to_string());
            let mcp_servers: Vec<String> = serde_json::from_str(&mcp_servers_json)?;
            
            Ok(mcp_servers)
        } else {
            Ok(Vec::new())
        }
    }
}