use crate::domain::{
    GenerateInput, GenerateOptions, GenerateResponse, GenerateStatus, GeneratedArtifacts,
    RequestContext, ResponseMeta,
};
use crate::llm::{create_backend_from_db_or_env, create_backend_from_env};
use crate::models::_entities::generation_logs;
use crate::services::{NormalizerService, PromptCompiler, TemplateService};
use crate::services::pipeline::{PostProcessingPipeline, ExecutionMode};
use anyhow::{anyhow, Result};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use std::time::Instant;

/// Service for orchestrating the generation flow
pub struct GenerationService;

impl GenerationService {
    /// Main generation entry point
    pub async fn generate(
        db: &DatabaseConnection,
        input: GenerateInput,
        product: &str,
        options: &GenerateOptions,
        _context: &RequestContext,
        user_id: Option<i32>,
    ) -> Result<GenerateResponse> {
        let start = Instant::now();

        // 1. Normalize input to UiIntent
        let intent = NormalizerService::normalize(&input)?;

        // 2. Get template version for logging
        let template = TemplateService::get_active(db, product, Some(intent.screen_type.as_str()))
            .await
            .ok();
        let template_version = template.as_ref().map(|t| t.version).unwrap_or(0);

        // 3. Compile prompt
        let prompt = PromptCompiler::compile(
            db,
            &intent,
            product,
            options.company_id.as_deref(),
        )
        .await?;

        // 4. Generate via LLM (DB config takes priority, falls back to env)
        let llm = create_backend_from_db_or_env(db).await;

        // Health check
        llm.health_check().await.map_err(|e| {
            anyhow!("LLM server not available: {}. Please check your LLM configuration.", e)
        })?;

        let raw_output = llm.generate(&prompt.full()).await?;

        // Log raw output for debugging (truncated)
        let output_preview = if raw_output.len() > 500 {
            format!("{}...[truncated, total {} chars]", &raw_output[..500], raw_output.len())
        } else {
            raw_output.clone()
        };
        tracing::debug!("LLM raw output preview:\n{}", output_preview);

        // 5. Run through post-processing pipeline
        // Execution mode is derived from strictMode option
        let execution_mode = ExecutionMode::from_strict_mode(options.strict_mode);

        let pipeline_result = PostProcessingPipeline::run(
            raw_output.clone(),
            &intent,
            execution_mode,
        );

        let (artifacts, warnings, status, error_message) = match pipeline_result {
            Ok(result) => {
                // Convert pipeline result to GeneratedArtifacts
                let artifacts = GeneratedArtifacts {
                    xml: Some(result.xml),
                    javascript: Some(result.javascript),
                    xml_filename: Some(format!("{}.xml", intent.screen_name.to_lowercase().replace(' ', "_"))),
                    js_filename: Some(format!("{}.js", intent.screen_name.to_lowercase().replace(' ', "_"))),
                };

                let status = if result.warnings.iter().any(|w| w.contains("Warning") || w.contains("Error")) {
                    GenerateStatus::PartialSuccess
                } else {
                    GenerateStatus::Success
                };

                (Some(artifacts), result.warnings, status, None)
            }
            Err(e) => {
                // Pipeline failed - try retry once
                tracing::warn!("First generation failed pipeline: {}", e);

                // Retry with more explicit instructions
                let retry_prompt = format!(
                    "{}\n\nIMPORTANT: Your previous response could not be parsed. \
                    Please ensure you output exactly two sections:\n\
                    --- XML ---\n<your XML here>\n\n--- JS ---\n<your JavaScript here>",
                    prompt.full()
                );

                match llm.generate(&retry_prompt).await {
                    Ok(retry_output) => {
                        // Use Relaxed mode for retry to be more permissive
                        match PostProcessingPipeline::run(retry_output, &intent, ExecutionMode::Relaxed) {
                            Ok(result) => {
                                let artifacts = GeneratedArtifacts {
                                    xml: Some(result.xml),
                                    javascript: Some(result.javascript),
                                    xml_filename: Some(format!("{}.xml", intent.screen_name.to_lowercase().replace(' ', "_"))),
                                    js_filename: Some(format!("{}.js", intent.screen_name.to_lowercase().replace(' ', "_"))),
                                };
                                let mut warnings = result.warnings;
                                warnings.push("Note: Generation required retry".to_string());
                                (Some(artifacts), warnings, GenerateStatus::PartialSuccess, None)
                            }
                            Err(retry_err) => {
                                (None, vec![], GenerateStatus::Error, Some(format!("Pipeline failed after retry: {}", retry_err)))
                            }
                        }
                    }
                    Err(retry_err) => {
                        (None, vec![], GenerateStatus::Error, Some(format!("Retry failed: {}", retry_err)))
                    }
                }
            }
        };

        let generation_time_ms = start.elapsed().as_millis() as u64;

        // 6. Log to audit trail (NO input data stored)
        let log_result = Self::log_generation(
            db,
            product,
            &input,
            &intent,
            template_version,
            &status,
            &artifacts,
            &warnings,
            error_message.as_deref(),
            generation_time_ms as i32,
            user_id,
        )
        .await;

        if let Err(e) = log_result {
            tracing::error!("Failed to log generation: {}", e);
        }

        // 7. Build response (NO LLM details exposed)
        Ok(GenerateResponse {
            status,
            artifacts,
            warnings,
            error: error_message,
            meta: ResponseMeta {
                generator: format!("{}-v1", product),
                timestamp: Utc::now(),
                generation_time_ms,
            },
        })
    }

    /// Generate using only default templates (no DB)
    pub async fn generate_with_defaults(
        input: GenerateInput,
        product: &str,
        company_rules: Option<&str>,
    ) -> Result<GenerateResponse> {
        let start = Instant::now();

        // 1. Normalize input
        let intent = NormalizerService::normalize(&input)?;

        // 2. Compile prompt with defaults
        let prompt = PromptCompiler::compile_with_defaults(&intent, company_rules);

        // 3. Generate via LLM
        let llm = create_backend_from_env();
        llm.health_check().await?;

        let raw_output = llm.generate(&prompt.full()).await?;

        // 4. Run through post-processing pipeline (Relaxed mode for defaults)
        let result = PostProcessingPipeline::run(
            raw_output,
            &intent,
            ExecutionMode::Relaxed,
        )?;

        let generation_time_ms = start.elapsed().as_millis() as u64;

        let status = if result.warnings.iter().any(|w| w.contains("Warning") || w.contains("Error")) {
            GenerateStatus::PartialSuccess
        } else {
            GenerateStatus::Success
        };

        let artifacts = GeneratedArtifacts {
            xml: Some(result.xml),
            javascript: Some(result.javascript),
            xml_filename: Some(format!("{}.xml", intent.screen_name.to_lowercase().replace(' ', "_"))),
            js_filename: Some(format!("{}.js", intent.screen_name.to_lowercase().replace(' ', "_"))),
        };

        Ok(GenerateResponse {
            status,
            artifacts: Some(artifacts),
            warnings: result.warnings,
            error: None,
            meta: ResponseMeta {
                generator: format!("{}-v1", product),
                timestamp: Utc::now(),
                generation_time_ms,
            },
        })
    }

    /// Log generation to audit trail
    async fn log_generation(
        db: &DatabaseConnection,
        product: &str,
        input: &GenerateInput,
        intent: &crate::domain::UiIntent,
        template_version: i32,
        status: &GenerateStatus,
        artifacts: &Option<GeneratedArtifacts>,
        warnings: &[String],
        error_message: Option<&str>,
        generation_time_ms: i32,
        user_id: Option<i32>,
    ) -> Result<()> {
        // Determine input type (without storing actual input data - 개인정보 보호)
        let input_type = match input {
            GenerateInput::DbSchema(_) => "db-schema",
            GenerateInput::QuerySample(_) => "query-sample",
            GenerateInput::NaturalLanguage(_) => "natural-language",
        };

        let status_str = match status {
            GenerateStatus::Success => "success",
            GenerateStatus::PartialSuccess => "partial_success",
            GenerateStatus::Error => "error",
        };

        // Store UI intent (meta model) instead of raw input
        let ui_intent_json = serde_json::to_string(intent)?;

        // Store artifacts
        let artifacts_json = artifacts.as_ref().map(|a| serde_json::to_string(a).ok()).flatten();

        // Store warnings
        let warnings_json = if warnings.is_empty() {
            None
        } else {
            Some(serde_json::to_string(warnings)?)
        };

        let log = generation_logs::ActiveModel {
            product: Set(product.to_string()),
            input_type: Set(input_type.to_string()),
            ui_intent: Set(ui_intent_json),
            template_version: Set(template_version),
            status: Set(status_str.to_string()),
            artifacts: Set(artifacts_json),
            warnings: Set(warnings_json),
            error_message: Set(error_message.map(|s| s.to_string())),
            generation_time_ms: Set(Some(generation_time_ms)),
            user_id: Set(user_id.unwrap_or(1)), // Default to system user
            ..Default::default()
        };

        log.insert(db).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::SchemaInput;

    #[test]
    fn test_input_type_detection() {
        let schema = GenerateInput::DbSchema(SchemaInput::new("test"));
        let input_type = match &schema {
            GenerateInput::DbSchema(_) => "db-schema",
            GenerateInput::QuerySample(_) => "query-sample",
            GenerateInput::NaturalLanguage(_) => "natural-language",
        };
        assert_eq!(input_type, "db-schema");
    }
}
