use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use crate::persistence::entities::generation;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "conversations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    #[sea_orm(column_type = "Integer")]
    pub created_at: i64,
    #[sea_orm(column_type = "Text")]
    pub mcp_servers: Option<String>, // JSON string storing Vec<String> of MCP servers
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::persistence::entities::generation::Entity")]
    Generation,
}

impl Related<generation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Generation.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}