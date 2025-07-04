use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use crate::persistence::entities::conversation;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "generations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub conversation_id: i32,
    pub role: String,
    #[sea_orm(column_type = "Text")]
    pub content: String,
    #[sea_orm(column_type = "Integer")]
    pub created_at: i64,
    pub mcp_server: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::persistence::entities::conversation::Entity",
        from = "Column::ConversationId",
        to = "crate::persistence::entities::conversation::Column::Id"
    )]
    Conversation,
}

impl Related<conversation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Conversation.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}