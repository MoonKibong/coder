use crate::domain::{
    ReviewContext, ReviewInput, ReviewMeta, ReviewOptions, ReviewResponse, ReviewResult,
    ReviewScore, CategoryScores, ReviewIssue, IssueSeverity, IssueCategory,
};
use crate::llm::create_backend_from_db_or_env;
use crate::models::_entities::generation_logs;
use crate::services::{KnowledgeBaseService, KnowledgeQuery, TemplateService};
use anyhow::{anyhow, Result};
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use serde_json::Value;
use std::time::Instant;

/// Service for code review operations
pub struct ReviewService;

impl ReviewService {
    /// Review code and return structured feedback
    pub async fn review(
        db: &DatabaseConnection,
        input: ReviewInput,
        product: &str,
        options: &ReviewOptions,
        context: &ReviewContext,
        user_id: Option<i32>,
    ) -> Result<ReviewResponse> {
        let start = Instant::now();

        // 1. Detect file type
        let file_type = input.detect_file_type();

        // 2. Load review template from DB
        let template = TemplateService::get_active(db, product, Some("review"))
            .await
            .map_err(|_| anyhow!("Review template not found for product: {}", product))?;

        // 3. Load relevant knowledge base entries
        let knowledge = Self::load_knowledge(db, product, &file_type).await;

        // 4. Load company rules if provided
        let company_rules = if let Some(ref company_id) = options.company_id {
            Self::load_company_rules(db, company_id).await
        } else {
            String::new()
        };

        // 5. Compile prompt
        let (system_prompt, user_prompt) = Self::compile_prompt(
            &template.system_prompt,
            &template.user_prompt_template,
            &input,
            &file_type,
            context,
            options,
            &knowledge,
            &company_rules,
        )?;

        let full_prompt = format!("{}\n\n{}", system_prompt, user_prompt);

        // 6. Generate via LLM
        let llm = create_backend_from_db_or_env(db).await;

        llm.health_check().await.map_err(|e| {
            anyhow!("LLM server not available: {}. Please check your LLM configuration.", e)
        })?;

        let raw_output = llm.generate(&full_prompt).await?;

        // 7. Parse JSON response
        let review_result = Self::parse_review_result(&raw_output)?;

        let review_time_ms = start.elapsed().as_millis() as u64;

        // 8. Log to audit trail (meta only, NO raw code)
        Self::log_review(
            db,
            product,
            &file_type,
            input.code.lines().count(),
            review_result.issues.len(),
            review_result.score.as_ref().map(|s| s.overall as i32),
            review_time_ms as i32,
            user_id,
        )
        .await
        .ok(); // Don't fail on log error

        // 9. Build response
        Ok(ReviewResponse::success(
            review_result,
            ReviewMeta::new(format!("{}-review-v1", product), review_time_ms),
        ))
    }

    /// Load knowledge entries relevant to the file type
    async fn load_knowledge(db: &DatabaseConnection, product: &str, file_type: &str) -> String {
        let query = KnowledgeQuery {
            category: Some(if product.contains("spring") {
                "spring".to_string()
            } else {
                "xframe5".to_string()
            }),
            component: Some(file_type.to_string()),
            relevance_tags: None,
            priority: Some("high".to_string()),
        };

        match KnowledgeBaseService::query(db, &query).await {
            Ok(entries) => {
                if entries.is_empty() {
                    return String::new();
                }
                KnowledgeBaseService::assemble_content(&entries)
            }
            Err(_) => String::new(),
        }
    }

    /// Load company rules
    async fn load_company_rules(db: &DatabaseConnection, company_id: &str) -> String {
        // TODO: Implement company rules loading from company_rules table
        let _ = (db, company_id);
        String::new()
    }

    /// Compile the review prompt using simple string replacement
    fn compile_prompt(
        system_template: &str,
        user_template: &str,
        input: &ReviewInput,
        file_type: &str,
        context: &ReviewContext,
        options: &ReviewOptions,
        knowledge: &str,
        company_rules: &str,
    ) -> Result<(String, String)> {
        // Simple template replacement (handlebars-style)
        let file_name = context.file_name.clone().unwrap_or_default();
        let input_context = input.context.clone().unwrap_or_default();
        let review_focus = options.review_focus.join(", ");

        // Replace system prompt placeholders
        let system_prompt = system_template
            .replace("{{knowledge}}", knowledge)
            .replace("{{company_rules}}", company_rules);

        // Replace user prompt placeholders
        // Handle conditional blocks manually
        let mut user_prompt = user_template
            .replace("{{code}}", &input.code)
            .replace("{{file_type}}", file_type);

        // Handle {{#if file_name}} blocks
        if !file_name.is_empty() {
            user_prompt = user_prompt
                .replace("{{#if file_name}}", "")
                .replace("{{/if}}", "")
                .replace("{{file_name}}", &file_name);
        } else {
            // Remove the entire conditional block
            user_prompt = Self::remove_conditional_block(&user_prompt, "file_name");
        }

        // Handle {{#if context}} blocks
        if !input_context.is_empty() {
            user_prompt = user_prompt
                .replace("{{#if context}}", "")
                .replace("{{context}}", &input_context);
        } else {
            user_prompt = Self::remove_conditional_block(&user_prompt, "context");
        }

        // Handle {{#if review_focus}} blocks
        if !review_focus.is_empty() {
            user_prompt = user_prompt
                .replace("{{#if review_focus}}", "")
                .replace("{{review_focus}}", &review_focus);
        } else {
            user_prompt = Self::remove_conditional_block(&user_prompt, "review_focus");
        }

        // Clean up any remaining {{/if}} tags
        user_prompt = user_prompt.replace("{{/if}}", "");

        Ok((system_prompt, user_prompt))
    }

    /// Remove a conditional block from template
    fn remove_conditional_block(template: &str, var_name: &str) -> String {
        let start_tag = format!("{{{{#if {}}}}}", var_name);
        let end_tag = "{{/if}}";

        if let Some(start_pos) = template.find(&start_tag) {
            if let Some(end_pos) = template[start_pos..].find(end_tag) {
                let before = &template[..start_pos];
                let after = &template[start_pos + end_pos + end_tag.len()..];
                return format!("{}{}", before, after);
            }
        }
        template.to_string()
    }

    /// Parse LLM output into ReviewResult
    fn parse_review_result(raw_output: &str) -> Result<ReviewResult> {
        // Try to extract JSON from the response
        let json_str = Self::extract_json(raw_output)?;

        let json: Value = serde_json::from_str(&json_str)
            .map_err(|e| anyhow!("Failed to parse review response as JSON: {}", e))?;

        // Parse summary
        let summary = json
            .get("summary")
            .and_then(|v| v.as_str())
            .unwrap_or("Code review completed")
            .to_string();

        // Parse issues
        let issues = Self::parse_issues(&json);

        // Parse score
        let score = Self::parse_score(&json);

        // Parse improvements
        let improvements = json
            .get("improvements")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(ReviewResult {
            summary,
            issues,
            score,
            improvements,
        })
    }

    /// Extract JSON from LLM output (handles markdown code blocks)
    fn extract_json(raw: &str) -> Result<String> {
        let trimmed = raw.trim();

        // If it starts with {, assume it's already JSON
        if trimmed.starts_with('{') {
            // Find the matching closing brace
            let mut depth = 0;
            let mut end_idx = 0;
            for (i, c) in trimmed.char_indices() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            end_idx = i + 1;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if end_idx > 0 {
                return Ok(trimmed[..end_idx].to_string());
            }
            return Ok(trimmed.to_string());
        }

        // Try to extract from markdown code block
        if let Some(start) = trimmed.find("```json") {
            let after_marker = &trimmed[start + 7..];
            if let Some(end) = after_marker.find("```") {
                return Ok(after_marker[..end].trim().to_string());
            }
        }

        // Try generic code block
        if let Some(start) = trimmed.find("```") {
            let after_marker = &trimmed[start + 3..];
            // Skip language identifier if present
            let content_start = after_marker.find('\n').unwrap_or(0) + 1;
            let content = &after_marker[content_start..];
            if let Some(end) = content.find("```") {
                let potential_json = content[..end].trim();
                if potential_json.starts_with('{') {
                    return Ok(potential_json.to_string());
                }
            }
        }

        // Try to find JSON object anywhere in the text
        if let Some(start) = trimmed.find('{') {
            let from_brace = &trimmed[start..];
            let mut depth = 0;
            let mut end_idx = 0;
            for (i, c) in from_brace.char_indices() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            end_idx = i + 1;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if end_idx > 0 {
                return Ok(from_brace[..end_idx].to_string());
            }
        }

        Err(anyhow!("Could not extract JSON from LLM response"))
    }

    /// Parse issues array from JSON
    fn parse_issues(json: &Value) -> Vec<ReviewIssue> {
        json.get("issues")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|issue| {
                        let severity = issue
                            .get("severity")
                            .and_then(|v| v.as_str())
                            .map(Self::parse_severity)
                            .unwrap_or(IssueSeverity::Info);

                        let category = issue
                            .get("category")
                            .and_then(|v| v.as_str())
                            .map(Self::parse_category)
                            .unwrap_or(IssueCategory::BestPractice);

                        let line = issue
                            .get("line")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as u32;

                        let message = issue
                            .get("message")
                            .and_then(|v| v.as_str())?
                            .to_string();

                        let suggestion = issue
                            .get("suggestion")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        Some(ReviewIssue {
                            severity,
                            category,
                            line,
                            message,
                            suggestion,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Parse severity string to enum
    fn parse_severity(s: &str) -> IssueSeverity {
        match s.to_lowercase().as_str() {
            "error" => IssueSeverity::Error,
            "warning" => IssueSeverity::Warning,
            "info" => IssueSeverity::Info,
            "suggestion" => IssueSeverity::Suggestion,
            _ => IssueSeverity::Info,
        }
    }

    /// Parse category string to enum
    fn parse_category(s: &str) -> IssueCategory {
        match s.to_lowercase().as_str() {
            "syntax" => IssueCategory::Syntax,
            "pattern" | "patterns" => IssueCategory::Pattern,
            "naming" => IssueCategory::Naming,
            "performance" => IssueCategory::Performance,
            "security" => IssueCategory::Security,
            "best_practice" | "best-practice" | "bestpractice" => IssueCategory::BestPractice,
            _ => IssueCategory::BestPractice,
        }
    }

    /// Parse score from JSON
    fn parse_score(json: &Value) -> Option<ReviewScore> {
        let score_obj = json.get("score")?;

        let overall = score_obj
            .get("overall")
            .and_then(|v| v.as_u64())
            .unwrap_or(50) as u8;

        let categories = score_obj.get("categories").map(|cats| CategoryScores {
            syntax: cats.get("syntax").and_then(|v| v.as_u64()).map(|n| n as u8),
            patterns: cats.get("patterns").and_then(|v| v.as_u64()).map(|n| n as u8),
            naming: cats.get("naming").and_then(|v| v.as_u64()).map(|n| n as u8),
            performance: cats.get("performance").and_then(|v| v.as_u64()).map(|n| n as u8),
            security: cats.get("security").and_then(|v| v.as_u64()).map(|n| n as u8),
        }).unwrap_or_default();

        Some(ReviewScore { overall, categories })
    }

    /// Log review to audit trail (meta only, NO raw code)
    async fn log_review(
        db: &DatabaseConnection,
        product: &str,
        file_type: &str,
        line_count: usize,
        issue_count: usize,
        score: Option<i32>,
        review_time_ms: i32,
        user_id: Option<i32>,
    ) -> Result<()> {
        // Store meta information about the review
        let ui_intent_json = serde_json::to_string(&serde_json::json!({
            "type": "code_review",
            "file_type": file_type,
            "line_count": line_count,
            "issue_count": issue_count,
            "score": score,
        }))?;

        let log = generation_logs::ActiveModel {
            product: Set(product.to_string()),
            input_type: Set("code-review".to_string()),
            ui_intent: Set(ui_intent_json),
            template_version: Set(1),
            status: Set("success".to_string()),
            artifacts: Set(None),
            warnings: Set(None),
            error_message: Set(None),
            generation_time_ms: Set(Some(review_time_ms)),
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

    #[test]
    fn test_extract_json_plain() {
        let input = r#"{"summary": "test", "issues": []}"#;
        let result = ReviewService::extract_json(input).unwrap();
        assert!(result.contains("summary"));
    }

    #[test]
    fn test_extract_json_markdown() {
        let input = r#"Here is the review:
```json
{"summary": "test", "issues": []}
```"#;
        let result = ReviewService::extract_json(input).unwrap();
        assert!(result.contains("summary"));
    }

    #[test]
    fn test_parse_severity() {
        assert!(matches!(ReviewService::parse_severity("error"), IssueSeverity::Error));
        assert!(matches!(ReviewService::parse_severity("WARNING"), IssueSeverity::Warning));
        assert!(matches!(ReviewService::parse_severity("info"), IssueSeverity::Info));
    }

    #[test]
    fn test_file_type_detection() {
        let xml_input = ReviewInput::new("<?xml version=\"1.0\"?><screen/>");
        assert_eq!(xml_input.detect_file_type(), "xml");

        let java_input = ReviewInput::new("public class Test {}");
        assert_eq!(java_input.detect_file_type(), "java");

        let js_input = ReviewInput::new("function test() {}");
        assert_eq!(js_input.detect_file_type(), "javascript");
    }
}
