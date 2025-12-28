use crate::domain::{
    GenerateInput, GenerateOptions, GenerateStatus, RequestContext, ResponseMeta, SpringArtifacts,
};
use crate::llm::create_backend_from_env;
use crate::models::_entities::generation_logs;
use crate::services::{SpringNormalizerService, SpringValidator, TemplateService};
use crate::services::spring_prompt_compiler::SpringPromptCompiler;
use anyhow::{anyhow, Result};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Response for Spring code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpringGenerateResponse {
    /// Status of generation
    pub status: GenerateStatus,

    /// Generated Spring artifacts
    pub artifacts: Option<SpringArtifacts>,

    /// Warnings during generation
    #[serde(default)]
    pub warnings: Vec<String>,

    /// Error message (if status is error)
    pub error: Option<String>,

    /// Response metadata
    pub meta: ResponseMeta,
}

/// Service for orchestrating Spring code generation
pub struct SpringGenerationService;

impl SpringGenerationService {
    /// Main generation entry point for Spring backend code
    pub async fn generate(
        db: &DatabaseConnection,
        input: GenerateInput,
        options: &GenerateOptions,
        context: &RequestContext,
        user_id: Option<i32>,
    ) -> Result<SpringGenerateResponse> {
        let start = Instant::now();

        // Get package base from context or use default
        let package_base = context.project.as_deref().unwrap_or("com.company.project");

        // 1. Normalize input to SpringIntent
        let intent = SpringNormalizerService::normalize(&input, package_base)?;

        // 2. Get template version for logging
        let template = TemplateService::get_active(db, "spring-backend", Some("crud"))
            .await
            .ok();
        let template_version = template.as_ref().map(|t| t.version).unwrap_or(0);

        // 3. Compile prompt
        let prompt = SpringPromptCompiler::compile(
            db,
            &intent,
            options.company_id.as_deref(),
        )
        .await?;

        // 4. Generate via LLM
        let llm = create_backend_from_env();

        // Health check
        llm.health_check().await.map_err(|e| {
            anyhow!("LLM server not available: {}. Please check your LLM configuration.", e)
        })?;

        let raw_output = llm.generate(&prompt.full()).await?;

        // 5. Parse and validate
        let validation_result = SpringValidator::parse_and_validate(&raw_output, &intent);

        let (artifacts, warnings, status, error_message) = match validation_result {
            Ok(mut validated) => {
                // Post-process to fix common issues
                SpringValidator::post_process(&mut validated, &intent);

                let warnings = validated.warnings.clone();
                let status = if warnings.iter().any(|w| w.starts_with("Warning:")) {
                    GenerateStatus::PartialSuccess
                } else {
                    GenerateStatus::Success
                };

                (Some(validated), warnings, status, None)
            }
            Err(e) => {
                // Validation failed - try retry once
                tracing::warn!("First Spring generation failed validation: {}", e);

                // Retry with more explicit instructions
                let retry_prompt = format!(
                    "{}\n\nIMPORTANT: Your previous response could not be parsed. \
                    Please ensure you output exactly 6 sections with these markers:\n\
                    --- CONTROLLER ---\n--- SERVICE ---\n--- SERVICE_IMPL ---\n\
                    --- DTO ---\n--- MAPPER ---\n--- MAPPER_XML ---",
                    prompt.full()
                );

                match llm.generate(&retry_prompt).await {
                    Ok(retry_output) => {
                        match SpringValidator::parse_and_validate(&retry_output, &intent) {
                            Ok(mut validated) => {
                                SpringValidator::post_process(&mut validated, &intent);
                                let mut warnings = validated.warnings.clone();
                                warnings.push("Note: Generation required retry".to_string());
                                (Some(validated), warnings, GenerateStatus::PartialSuccess, None)
                            }
                            Err(retry_err) => {
                                (None, vec![], GenerateStatus::Error, Some(format!("Validation failed after retry: {}", retry_err)))
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
            tracing::error!("Failed to log Spring generation: {}", e);
        }

        // 7. Build response (NO LLM details exposed)
        Ok(SpringGenerateResponse {
            status,
            artifacts,
            warnings,
            error: error_message,
            meta: ResponseMeta {
                generator: "spring-backend-v1".to_string(),
                timestamp: Utc::now(),
                generation_time_ms,
            },
        })
    }

    /// Generate using only default templates (no DB)
    pub async fn generate_with_defaults(
        input: GenerateInput,
        package_base: &str,
        company_rules: Option<&str>,
    ) -> Result<SpringGenerateResponse> {
        let start = Instant::now();

        // 1. Normalize input
        let intent = SpringNormalizerService::normalize(&input, package_base)?;

        // 2. Compile prompt with defaults
        let prompt = SpringPromptCompiler::compile_with_defaults(&intent, company_rules);

        // 3. Generate via LLM
        let llm = create_backend_from_env();
        llm.health_check().await?;

        let raw_output = llm.generate(&prompt.full()).await?;

        // 4. Parse and validate
        let mut validated = SpringValidator::parse_and_validate(&raw_output, &intent)?;
        SpringValidator::post_process(&mut validated, &intent);

        let generation_time_ms = start.elapsed().as_millis() as u64;

        let status = if validated.warnings.iter().any(|w| w.starts_with("Warning:")) {
            GenerateStatus::PartialSuccess
        } else {
            GenerateStatus::Success
        };

        let warnings = validated.warnings.clone();

        Ok(SpringGenerateResponse {
            status,
            artifacts: Some(validated),
            warnings,
            error: None,
            meta: ResponseMeta {
                generator: "spring-backend-v1".to_string(),
                timestamp: Utc::now(),
                generation_time_ms,
            },
        })
    }

    /// Log generation to audit trail
    async fn log_generation(
        db: &DatabaseConnection,
        input: &GenerateInput,
        intent: &crate::domain::SpringIntent,
        template_version: i32,
        status: &GenerateStatus,
        artifacts: &Option<SpringArtifacts>,
        warnings: &[String],
        error_message: Option<&str>,
        generation_time_ms: i32,
        user_id: Option<i32>,
    ) -> Result<()> {
        // Determine input type (without storing actual input data)
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

        // Store Spring intent (meta model) instead of raw input
        let spring_intent_json = serde_json::to_string(intent)?;

        // Store artifacts
        let artifacts_json = artifacts.as_ref().map(|a| serde_json::to_string(a).ok()).flatten();

        // Store warnings
        let warnings_json = if warnings.is_empty() {
            None
        } else {
            Some(serde_json::to_string(warnings)?)
        };

        let log = generation_logs::ActiveModel {
            product: Set("spring-backend".to_string()),
            input_type: Set(input_type.to_string()),
            ui_intent: Set(spring_intent_json), // Reuse column for SpringIntent
            template_version: Set(template_version),
            status: Set(status_str.to_string()),
            artifacts: Set(artifacts_json),
            warnings: Set(warnings_json),
            error_message: Set(error_message.map(|s| s.to_string())),
            generation_time_ms: Set(Some(generation_time_ms)),
            user_id: Set(user_id.unwrap_or(1)),
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
        let schema = GenerateInput::DbSchema(SchemaInput::new("TB_MEMBER"));
        let input_type = match &schema {
            GenerateInput::DbSchema(_) => "db-schema",
            GenerateInput::QuerySample(_) => "query-sample",
            GenerateInput::NaturalLanguage(_) => "natural-language",
        };
        assert_eq!(input_type, "db-schema");
    }
}
