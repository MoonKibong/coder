use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Seed LLM configurations for fresh installations
        // Default active model: Qwen3-coder:30b (Ollama)

        // ===========================================
        // Ollama Models (localhost:11434)
        // ===========================================

        // 1. Qwen3 Coder 30B (Default - is_active: true)
        let insert = Query::insert()
            .into_table(Alias::new("llm_configs"))
            .columns([
                Alias::new("name"),
                Alias::new("provider"),
                Alias::new("endpoint_url"),
                Alias::new("model_name"),
                Alias::new("temperature"),
                Alias::new("max_tokens"),
                Alias::new("is_active"),
            ])
            .values_panic([
                "Qwen3 Coder 30B (Default)".into(),
                "ollama".into(),
                "http://localhost:11434".into(),
                "qwen3-coder:30b".into(),
                0.7f32.into(),
                8192.into(),
                true.into(),
            ])
            .to_owned();
        m.exec_stmt(insert).await?;

        // 2. DeepSeek Coder V2 16B
        let insert = Query::insert()
            .into_table(Alias::new("llm_configs"))
            .columns([
                Alias::new("name"),
                Alias::new("provider"),
                Alias::new("endpoint_url"),
                Alias::new("model_name"),
                Alias::new("temperature"),
                Alias::new("max_tokens"),
                Alias::new("is_active"),
            ])
            .values_panic([
                "DeepSeek Coder V2 16B".into(),
                "ollama".into(),
                "http://localhost:11434".into(),
                "deepseek-coder-v2:16b".into(),
                0.7f32.into(),
                8192.into(),
                false.into(),
            ])
            .to_owned();
        m.exec_stmt(insert).await?;

        // 3. Codestral 22B
        let insert = Query::insert()
            .into_table(Alias::new("llm_configs"))
            .columns([
                Alias::new("name"),
                Alias::new("provider"),
                Alias::new("endpoint_url"),
                Alias::new("model_name"),
                Alias::new("temperature"),
                Alias::new("max_tokens"),
                Alias::new("is_active"),
            ])
            .values_panic([
                "Codestral 22B".into(),
                "ollama".into(),
                "http://localhost:11434".into(),
                "codestral:22b".into(),
                0.7f32.into(),
                8192.into(),
                false.into(),
            ])
            .to_owned();
        m.exec_stmt(insert).await?;

        // ===========================================
        // Local llama.cpp Models (In-Process, GGUF)
        // ===========================================

        // 4. MiniMax M2 Q5_K_M
        let insert = Query::insert()
            .into_table(Alias::new("llm_configs"))
            .columns([
                Alias::new("name"),
                Alias::new("provider"),
                Alias::new("model_name"),
                Alias::new("model_path"),
                Alias::new("n_ctx"),
                Alias::new("n_threads"),
                Alias::new("temperature"),
                Alias::new("max_tokens"),
                Alias::new("is_active"),
            ])
            .values_panic([
                "MiniMax M2 Q5_K_M".into(),
                "local-llama-cpp".into(),
                "MiniMax-M2-Q5_K_M".into(),
                "llm-models/MiniMax-M2-Q5_K_M.gguf".into(),
                8192.into(),
                8.into(),
                0.7f32.into(),
                4096.into(),
                false.into(),
            ])
            .to_owned();
        m.exec_stmt(insert).await?;

        // 5. Qwen3 Next 80B
        let insert = Query::insert()
            .into_table(Alias::new("llm_configs"))
            .columns([
                Alias::new("name"),
                Alias::new("provider"),
                Alias::new("model_name"),
                Alias::new("model_path"),
                Alias::new("n_ctx"),
                Alias::new("n_threads"),
                Alias::new("temperature"),
                Alias::new("max_tokens"),
                Alias::new("is_active"),
            ])
            .values_panic([
                "Qwen3 Next 80B".into(),
                "local-llama-cpp".into(),
                "Qwen3-Next-80B".into(),
                "llm-models/Qwen3-Next-80B.gguf".into(),
                16384.into(),
                16.into(),
                0.7f32.into(),
                8192.into(),
                false.into(),
            ])
            .to_owned();
        m.exec_stmt(insert).await?;

        // ===========================================
        // Anthropic Models (Cloud API)
        // ===========================================

        // 6. Claude Opus 4.5
        let insert = Query::insert()
            .into_table(Alias::new("llm_configs"))
            .columns([
                Alias::new("name"),
                Alias::new("provider"),
                Alias::new("endpoint_url"),
                Alias::new("model_name"),
                Alias::new("temperature"),
                Alias::new("max_tokens"),
                Alias::new("is_active"),
            ])
            .values_panic([
                "Claude Opus 4.5".into(),
                "anthropic".into(),
                "https://api.anthropic.com/v1".into(),
                "claude-opus-4-5-20251101".into(),
                0.7f32.into(),
                8192.into(),
                false.into(),
            ])
            .to_owned();
        m.exec_stmt(insert).await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Delete all seeded LLM configs
        let delete = Query::delete()
            .from_table(Alias::new("llm_configs"))
            .and_where(Expr::col(Alias::new("name")).is_in([
                "Qwen3 Coder 30B (Default)",
                "DeepSeek Coder V2 16B",
                "Codestral 22B",
                "MiniMax M2 Q5_K_M",
                "Qwen3 Next 80B",
                "Claude Opus 4.5",
            ]))
            .to_owned();

        m.exec_stmt(delete).await?;

        Ok(())
    }
}
