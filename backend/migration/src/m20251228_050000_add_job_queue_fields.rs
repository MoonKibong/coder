use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add job_id column (UUID for client polling)
        manager
            .alter_table(
                Table::alter()
                    .table(GenerationLogs::Table)
                    .add_column(
                        ColumnDef::new(GenerationLogs::JobId)
                            .string()
                            .unique_key()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Add request_payload to store the original request for async processing
        manager
            .alter_table(
                Table::alter()
                    .table(GenerationLogs::Table)
                    .add_column(
                        ColumnDef::new(GenerationLogs::RequestPayload)
                            .text()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Add queued_at timestamp
        manager
            .alter_table(
                Table::alter()
                    .table(GenerationLogs::Table)
                    .add_column(
                        ColumnDef::new(GenerationLogs::QueuedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Add started_at timestamp
        manager
            .alter_table(
                Table::alter()
                    .table(GenerationLogs::Table)
                    .add_column(
                        ColumnDef::new(GenerationLogs::StartedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Add completed_at timestamp
        manager
            .alter_table(
                Table::alter()
                    .table(GenerationLogs::Table)
                    .add_column(
                        ColumnDef::new(GenerationLogs::CompletedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Add priority (1=high, 5=low, default=3)
        manager
            .alter_table(
                Table::alter()
                    .table(GenerationLogs::Table)
                    .add_column(
                        ColumnDef::new(GenerationLogs::Priority)
                            .integer()
                            .default(3)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on job_id for fast lookups
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_generation_logs_job_id")
                    .table(GenerationLogs::Table)
                    .col(GenerationLogs::JobId)
                    .to_owned(),
            )
            .await?;

        // Create index on status for queue processing
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_generation_logs_status_priority")
                    .table(GenerationLogs::Table)
                    .col(GenerationLogs::Status)
                    .col(GenerationLogs::Priority)
                    .col(GenerationLogs::QueuedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop indexes
        manager
            .drop_index(
                Index::drop()
                    .name("idx_generation_logs_status_priority")
                    .table(GenerationLogs::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_generation_logs_job_id")
                    .table(GenerationLogs::Table)
                    .to_owned(),
            )
            .await?;

        // Drop columns
        manager
            .alter_table(
                Table::alter()
                    .table(GenerationLogs::Table)
                    .drop_column(GenerationLogs::Priority)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GenerationLogs::Table)
                    .drop_column(GenerationLogs::CompletedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GenerationLogs::Table)
                    .drop_column(GenerationLogs::StartedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GenerationLogs::Table)
                    .drop_column(GenerationLogs::QueuedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GenerationLogs::Table)
                    .drop_column(GenerationLogs::RequestPayload)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GenerationLogs::Table)
                    .drop_column(GenerationLogs::JobId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum GenerationLogs {
    Table,
    JobId,
    RequestPayload,
    QueuedAt,
    StartedAt,
    CompletedAt,
    Priority,
    Status,
}
