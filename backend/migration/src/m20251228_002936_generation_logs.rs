use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(m, "generation_logs",
            &[
            
            ("id", ColType::PkAuto),
            
            ("product", ColType::String),
            ("input_type", ColType::String),
            ("ui_intent", ColType::Text),
            ("template_version", ColType::Integer),
            ("status", ColType::String),
            ("artifacts", ColType::TextNull),
            ("warnings", ColType::TextNull),
            ("error_message", ColType::TextNull),
            ("generation_time_ms", ColType::IntegerNull),
            ],
            &[
            ("user", ""),
            ]
        ).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "generation_logs").await
    }
}
