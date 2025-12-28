use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(m, "llm_configs",
            &[
            
            ("id", ColType::PkAuto),
            
            ("name", ColType::String),
            ("provider", ColType::String),
            ("endpoint_url", ColType::String),
            ("model_name", ColType::String),
            ("api_key", ColType::StringNull),
            ("temperature", ColType::FloatNull),
            ("max_tokens", ColType::IntegerNull),
            ("is_active", ColType::BooleanNull),
            ],
            &[
            ]
        ).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "llm_configs").await
    }
}
