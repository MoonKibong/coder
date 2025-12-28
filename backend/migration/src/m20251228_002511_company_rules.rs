use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(m, "company_rules",
            &[
            
            ("id", ColType::PkAuto),
            
            ("company_id", ColType::String),
            ("naming_convention", ColType::TextNull),
            ("additional_rules", ColType::TextNull),
            ],
            &[
            ]
        ).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "company_rules").await
    }
}
