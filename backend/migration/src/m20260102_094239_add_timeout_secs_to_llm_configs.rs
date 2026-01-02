use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Add timeout_secs column to llm_configs table
        // NULL means use LLM_TIMEOUT_SECONDS env var (default 120)
        m.alter_table(
            Table::alter()
                .table(LlmConfigs::Table)
                .add_column(
                    ColumnDef::new(LlmConfigs::TimeoutSecs)
                        .integer()
                        .null()
                )
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(LlmConfigs::Table)
                .drop_column(LlmConfigs::TimeoutSecs)
                .to_owned(),
        )
        .await
    }
}

#[derive(Iden)]
enum LlmConfigs {
    Table,
    TimeoutSecs,
}

