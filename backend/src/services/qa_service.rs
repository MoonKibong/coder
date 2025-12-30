//! Q&A Service
//!
//! Handles Q&A operations using knowledge base and LLM.

use crate::domain::{
    CodeExample, KnowledgeReference, QAAnswer, QAInput, QAMeta, QAOptions, QAResponse,
};
use crate::llm::create_backend_from_db_or_env;
use crate::models::_entities::generation_logs;
use crate::services::{KnowledgeBaseService, TemplateService};
use anyhow::{anyhow, Result};
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use serde_json::Value;
use std::time::Instant;

/// Service for Q&A operations
pub struct QAService;

impl QAService {
    /// Answer a question using knowledge base and LLM
    pub async fn answer(
        db: &DatabaseConnection,
        input: QAInput,
        product: &str,
        options: &QAOptions,
        user_id: Option<i32>,
    ) -> Result<QAResponse> {
        let start = Instant::now();

        // 1. Load QA template from DB
        let template = TemplateService::get_active(db, product, Some("qa"))
            .await
            .map_err(|_| anyhow!("QA template not found for product: {}", product))?;

        // 2. Query knowledge base for relevant entries
        let (knowledge_content, knowledge_refs) = KnowledgeBaseService::get_qa_knowledge(
            db,
            &input.question,
            product,
            options.max_references,
        )
        .await
        .unwrap_or_else(|_| (String::new(), vec![]));

        // 3. Compile prompt
        let (system_prompt, user_prompt) = Self::compile_prompt(
            &template.system_prompt,
            &template.user_prompt_template,
            &input,
            &knowledge_content,
        )?;

        let full_prompt = format!("{}\n\n{}", system_prompt, user_prompt);

        // 4. Generate via LLM
        let llm = create_backend_from_db_or_env(db).await;

        llm.health_check().await.map_err(|e| {
            anyhow!(
                "LLM server not available: {}. Please check your LLM configuration.",
                e
            )
        })?;

        let raw_output = llm.generate(&full_prompt).await?;

        // 5. Parse JSON response
        let qa_answer = Self::parse_qa_answer(&raw_output)?;

        let answer_time_ms = start.elapsed().as_millis() as u64;

        // 6. Build knowledge references for response
        let references: Vec<KnowledgeReference> = knowledge_refs
            .into_iter()
            .map(|(id, name, category, section, relevance)| KnowledgeReference {
                knowledge_id: id,
                name,
                category,
                section,
                relevance,
            })
            .collect();

        // 7. Log to audit trail (meta only, NO question content)
        Self::log_qa(
            db,
            product,
            input.question.len(),
            references.len(),
            answer_time_ms as i32,
            user_id,
        )
        .await
        .ok(); // Don't fail on log error

        // 8. Build response
        Ok(QAResponse::success(
            qa_answer,
            references,
            QAMeta::new(format!("{}-qa-v1", product), answer_time_ms),
        ))
    }

    /// Compile the QA prompt using simple string replacement
    fn compile_prompt(
        system_template: &str,
        user_template: &str,
        input: &QAInput,
        knowledge: &str,
    ) -> Result<(String, String)> {
        // Replace system prompt placeholders
        let system_prompt = system_template.replace("{{knowledge}}", knowledge);

        // Replace user prompt placeholders
        let mut user_prompt = user_template.replace("{{question}}", &input.question);

        // Handle {{#if context}} blocks
        if let Some(ref context) = input.context {
            if !context.is_empty() {
                user_prompt = user_prompt
                    .replace("{{#if context}}", "")
                    .replace("{{/if}}", "")
                    .replace("{{context}}", context);
            } else {
                user_prompt = Self::remove_conditional_block(&user_prompt, "context");
            }
        } else {
            user_prompt = Self::remove_conditional_block(&user_prompt, "context");
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

    /// Parse LLM output into QAAnswer
    fn parse_qa_answer(raw_output: &str) -> Result<QAAnswer> {
        // Try to extract JSON from the response
        let json_str = Self::extract_json(raw_output)?;

        let json: Value = serde_json::from_str(&json_str)
            .map_err(|e| anyhow!("Failed to parse QA response as JSON: {}", e))?;

        // Parse text (required)
        let text = json
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'text' field in QA response"))?
            .to_string();

        // Parse code examples (optional)
        let code_examples = Self::parse_code_examples(&json);

        // Parse related topics (optional)
        let related_topics = json
            .get("related_topics")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(QAAnswer {
            text,
            code_examples,
            related_topics,
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

    /// Parse code examples array from JSON
    fn parse_code_examples(json: &Value) -> Vec<CodeExample> {
        json.get("code_examples")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|example| {
                        let language = example
                            .get("language")
                            .and_then(|v| v.as_str())
                            .unwrap_or("text")
                            .to_string();

                        let code = example.get("code").and_then(|v| v.as_str())?.to_string();

                        let description = example
                            .get("description")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        Some(CodeExample {
                            language,
                            code,
                            description,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Log Q&A to audit trail (meta only, NO question content)
    async fn log_qa(
        db: &DatabaseConnection,
        product: &str,
        question_length: usize,
        reference_count: usize,
        answer_time_ms: i32,
        user_id: Option<i32>,
    ) -> Result<()> {
        // Store meta information about the Q&A
        let ui_intent_json = serde_json::to_string(&serde_json::json!({
            "type": "qa",
            "question_length": question_length,
            "reference_count": reference_count,
        }))?;

        let log = generation_logs::ActiveModel {
            product: Set(product.to_string()),
            input_type: Set("qa".to_string()),
            ui_intent: Set(ui_intent_json),
            template_version: Set(1),
            status: Set("success".to_string()),
            artifacts: Set(None),
            warnings: Set(None),
            error_message: Set(None),
            generation_time_ms: Set(Some(answer_time_ms)),
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
        let input = r#"{"text": "This is the answer", "code_examples": [], "related_topics": []}"#;
        let result = QAService::extract_json(input).unwrap();
        assert!(result.contains("text"));
    }

    #[test]
    fn test_extract_json_markdown() {
        let input = r#"Here is the answer:
```json
{"text": "This is the answer", "code_examples": [], "related_topics": []}
```"#;
        let result = QAService::extract_json(input).unwrap();
        assert!(result.contains("text"));
    }

    #[test]
    fn test_parse_qa_answer() {
        let json_str = r#"{
            "text": "Dataset은 xFrame5에서 데이터를 관리하는 핵심 컴포넌트입니다.",
            "code_examples": [
                {
                    "language": "xml",
                    "code": "<Dataset id=\"ds_member\" />",
                    "description": "Dataset definition example"
                }
            ],
            "related_topics": ["Grid Component", "Data Binding"]
        }"#;

        let answer = QAService::parse_qa_answer(json_str).unwrap();
        assert!(answer.text.contains("Dataset"));
        assert_eq!(answer.code_examples.len(), 1);
        assert_eq!(answer.code_examples[0].language, "xml");
        assert_eq!(answer.related_topics.len(), 2);
    }

    #[test]
    fn test_remove_conditional_block() {
        let template = "Question: {{question}}\n\n{{#if context}}Context: {{context}}{{/if}}\n\nPlease answer.";
        let result = QAService::remove_conditional_block(template, "context");
        assert!(!result.contains("Context:"));
        assert!(result.contains("Question:"));
        assert!(result.contains("Please answer."));
    }
}
