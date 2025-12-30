use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add model_path column for local-llama-cpp provider
        manager
            .alter_table(
                Table::alter()
                    .table(LlmConfigs::Table)
                    .add_column(
                        ColumnDef::new(LlmConfigs::ModelPath)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Add n_ctx (context size) for local-llama-cpp
        manager
            .alter_table(
                Table::alter()
                    .table(LlmConfigs::Table)
                    .add_column(
                        ColumnDef::new(LlmConfigs::NCtx)
                            .integer()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Add n_threads for local-llama-cpp
        manager
            .alter_table(
                Table::alter()
                    .table(LlmConfigs::Table)
                    .add_column(
                        ColumnDef::new(LlmConfigs::NThreads)
                            .integer()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Make endpoint_url nullable (not needed for local-llama-cpp)
        manager
            .alter_table(
                Table::alter()
                    .table(LlmConfigs::Table)
                    .modify_column(
                        ColumnDef::new(LlmConfigs::EndpointUrl)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(LlmConfigs::Table)
                    .drop_column(LlmConfigs::ModelPath)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(LlmConfigs::Table)
                    .drop_column(LlmConfigs::NCtx)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(LlmConfigs::Table)
                    .drop_column(LlmConfigs::NThreads)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum LlmConfigs {
    Table,
    EndpointUrl,
    ModelPath,
    NCtx,
    NThreads,
}
