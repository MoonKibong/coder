//! Add LLM model and provider info to generation_logs table
//!
//! This migration adds columns to track which LLM model and provider was used
//! for each generation request. This is for internal audit/debugging only.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add model_name column
        manager
            .alter_table(
                Table::alter()
                    .table(GenerationLogs::Table)
                    .add_column(
                        ColumnDef::new(GenerationLogs::ModelName)
                            .string_len(100)
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Add provider column
        manager
            .alter_table(
                Table::alter()
                    .table(GenerationLogs::Table)
                    .add_column(
                        ColumnDef::new(GenerationLogs::Provider)
                            .string_len(50)
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove provider column
        manager
            .alter_table(
                Table::alter()
                    .table(GenerationLogs::Table)
                    .drop_column(GenerationLogs::Provider)
                    .to_owned(),
            )
            .await?;

        // Remove model_name column
        manager
            .alter_table(
                Table::alter()
                    .table(GenerationLogs::Table)
                    .drop_column(GenerationLogs::ModelName)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum GenerationLogs {
    Table,
    ModelName,
    Provider,
}
