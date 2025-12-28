use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(m, "knowledge_bases",
            &[
            
            ("id", ColType::PkAuto),
            
            ("name", ColType::String),
            ("category", ColType::String),
            ("component", ColType::StringNull),
            ("section", ColType::StringNull),
            ("content", ColType::Text),
            ("relevance_tags", ColType::JsonNull),
            ("priority", ColType::StringNull),
            ("token_estimate", ColType::IntegerNull),
            ("version", ColType::IntegerNull),
            ("is_active", ColType::BooleanNull),
            ],
            &[
            ]
        ).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "knowledge_bases").await
    }
}
