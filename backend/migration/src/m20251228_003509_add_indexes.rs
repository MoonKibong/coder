use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // prompt_templates: index on (product, screen_type, is_active)
        m.create_index(
            Index::create()
                .name("idx_prompt_templates_product_screen_active")
                .table(PromptTemplates::Table)
                .col(PromptTemplates::Product)
                .col(PromptTemplates::ScreenType)
                .col(PromptTemplates::IsActive)
                .to_owned(),
        )
        .await?;

        // generation_logs: index on (user_id, created_at) for user audit queries
        m.create_index(
            Index::create()
                .name("idx_generation_logs_user_created")
                .table(GenerationLogs::Table)
                .col(GenerationLogs::UserId)
                .col(GenerationLogs::CreatedAt)
                .to_owned(),
        )
        .await?;

        // generation_logs: index on status for filtering
        m.create_index(
            Index::create()
                .name("idx_generation_logs_status")
                .table(GenerationLogs::Table)
                .col(GenerationLogs::Status)
                .to_owned(),
        )
        .await?;

        // llm_configs: index on (provider, is_active)
        m.create_index(
            Index::create()
                .name("idx_llm_configs_provider_active")
                .table(LlmConfigs::Table)
                .col(LlmConfigs::Provider)
                .col(LlmConfigs::IsActive)
                .to_owned(),
        )
        .await?;

        // company_rules: index on company_id
        m.create_index(
            Index::create()
                .name("idx_company_rules_company_id")
                .table(CompanyRules::Table)
                .col(CompanyRules::CompanyId)
                .to_owned(),
        )
        .await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_index(Index::drop().name("idx_prompt_templates_product_screen_active").to_owned()).await?;
        m.drop_index(Index::drop().name("idx_generation_logs_user_created").to_owned()).await?;
        m.drop_index(Index::drop().name("idx_generation_logs_status").to_owned()).await?;
        m.drop_index(Index::drop().name("idx_llm_configs_provider_active").to_owned()).await?;
        m.drop_index(Index::drop().name("idx_company_rules_company_id").to_owned()).await?;
        Ok(())
    }
}

#[derive(Iden)]
enum PromptTemplates {
    Table,
    Product,
    ScreenType,
    IsActive,
}

#[derive(Iden)]
enum GenerationLogs {
    Table,
    UserId,
    CreatedAt,
    Status,
}

#[derive(Iden)]
enum LlmConfigs {
    Table,
    Provider,
    IsActive,
}

#[derive(Iden)]
enum CompanyRules {
    Table,
    CompanyId,
}

