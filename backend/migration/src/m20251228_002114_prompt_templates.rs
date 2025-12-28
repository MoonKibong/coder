use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(m, "prompt_templates",
            &[
            
            ("id", ColType::PkAuto),
            
            ("name", ColType::String),
            ("product", ColType::String),
            ("screen_type", ColType::StringNull),
            ("system_prompt", ColType::Text),
            ("user_prompt_template", ColType::Text),
            ("version", ColType::Integer),
            ("is_active", ColType::BooleanNull),
            ],
            &[
            ]
        ).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "prompt_templates").await
    }
}
