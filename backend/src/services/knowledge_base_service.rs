//! Knowledge Base Service
//!
//! Manages xFrame5 knowledge base for selective inclusion in prompts.
//! Supports both database storage and file-based fallback.

use loco_rs::prelude::*;
use sea_orm::{query::*, DatabaseConnection, JsonValue};
use serde::{Deserialize, Serialize};

use crate::models::_entities::{knowledge_bases, prelude::*};

#[derive(Debug, Serialize, Deserialize)]
pub struct KnowledgeQuery {
    pub category: Option<String>,
    pub component: Option<String>,
    pub relevance_tags: Option<Vec<String>>,
    pub priority: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub id: i32,
    pub name: String,
    pub category: String,
    pub component: Option<String>,
    pub section: Option<String>,
    pub content: String,
    pub relevance_tags: Option<Vec<String>>,
    pub priority: Option<String>,
    pub token_estimate: Option<i32>,
}

impl From<knowledge_bases::Model> for KnowledgeEntry {
    fn from(model: knowledge_bases::Model) -> Self {
        let relevance_tags = model.relevance_tags.and_then(|json| {
            if let JsonValue::Array(arr) = json {
                Some(
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect(),
                )
            } else {
                None
            }
        });

        Self {
            id: model.id,
            name: model.name,
            category: model.category,
            component: model.component,
            section: model.section,
            content: model.content,
            relevance_tags,
            priority: model.priority,
            token_estimate: model.token_estimate,
        }
    }
}

pub struct KnowledgeBaseService;

impl KnowledgeBaseService {
    /// Query knowledge base entries based on criteria
    pub async fn query(db: &DatabaseConnection, query: &KnowledgeQuery) -> Result<Vec<KnowledgeEntry>> {
        let mut selector = KnowledgeBases::find().filter(knowledge_bases::Column::IsActive.eq(true));

        // Filter by category
        if let Some(category) = &query.category {
            selector = selector.filter(knowledge_bases::Column::Category.eq(category));
        }

        // Filter by component
        if let Some(component) = &query.component {
            selector = selector.filter(knowledge_bases::Column::Component.eq(component));
        }

        // Execute query
        let results = selector
            .order_by_asc(knowledge_bases::Column::Priority)
            .order_by_asc(knowledge_bases::Column::Name)
            .all(db)
            .await
            .map_err(|e| Error::string(&format!("Failed to query knowledge base: {}", e)))?;

        // Filter by relevance tags if provided (post-query since JSONB queries are complex)
        let entries: Vec<KnowledgeEntry> = if let Some(tags) = &query.relevance_tags {
            results
                .into_iter()
                .map(KnowledgeEntry::from)
                .filter(|entry| {
                    if let Some(entry_tags) = &entry.relevance_tags {
                        tags.iter().any(|tag| entry_tags.contains(tag))
                    } else {
                        false
                    }
                })
                .collect()
        } else {
            results.into_iter().map(KnowledgeEntry::from).collect()
        };

        Ok(entries)
    }

    /// Get knowledge for specific screen type
    pub async fn for_screen_type(
        db: &DatabaseConnection,
        screen_type: &str,
    ) -> Result<Vec<KnowledgeEntry>> {
        let query = KnowledgeQuery {
            category: None,
            component: None,
            relevance_tags: Some(vec![screen_type.to_string()]),
            priority: None,
        };

        Self::query(db, &query).await
    }

    /// Get knowledge for specific component
    pub async fn for_component(
        db: &DatabaseConnection,
        component: &str,
    ) -> Result<Vec<KnowledgeEntry>> {
        let query = KnowledgeQuery {
            category: Some("component".to_string()),
            component: Some(component.to_string()),
            relevance_tags: None,
            priority: None,
        };

        Self::query(db, &query).await
    }

    /// Assemble knowledge content into a single string
    pub fn assemble_content(entries: &[KnowledgeEntry]) -> String {
        entries
            .iter()
            .map(|entry| entry.content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n---\n\n")
    }

    /// Estimate total tokens for knowledge entries
    pub fn estimate_tokens(entries: &[KnowledgeEntry]) -> i32 {
        entries
            .iter()
            .filter_map(|e| e.token_estimate)
            .sum()
    }

    /// Find entry by name
    pub async fn find_by_name(
        db: &DatabaseConnection,
        name: &str,
    ) -> Result<Option<KnowledgeEntry>> {
        let result = KnowledgeBases::find()
            .filter(knowledge_bases::Column::Name.eq(name))
            .filter(knowledge_bases::Column::IsActive.eq(true))
            .one(db)
            .await
            .map_err(|e| Error::string(&format!("Failed to find knowledge entry: {}", e)))?;

        Ok(result.map(KnowledgeEntry::from))
    }

    /// Create new knowledge entry
    pub async fn create(
        db: &DatabaseConnection,
        name: String,
        category: String,
        component: Option<String>,
        section: Option<String>,
        content: String,
        relevance_tags: Option<Vec<String>>,
        priority: String,
        token_estimate: Option<i32>,
    ) -> Result<KnowledgeEntry> {
        let relevance_json = relevance_tags.map(|tags| {
            JsonValue::Array(tags.into_iter().map(JsonValue::String).collect())
        });

        let active_model = knowledge_bases::ActiveModel {
            name: Set(name),
            category: Set(category),
            component: Set(component),
            section: Set(section),
            content: Set(content),
            relevance_tags: Set(relevance_json),
            priority: Set(Some(priority)),
            token_estimate: Set(token_estimate),
            version: Set(Some(1)),
            is_active: Set(Some(true)),
            ..Default::default()
        };

        let model = active_model.insert(db).await.map_err(|e| {
            Error::string(&format!("Failed to create knowledge entry: {}", e))
        })?;

        Ok(KnowledgeEntry::from(model))
    }

    /// Update existing knowledge entry
    pub async fn update(
        db: &DatabaseConnection,
        id: i32,
        content: Option<String>,
        relevance_tags: Option<Vec<String>>,
        priority: Option<String>,
        is_active: Option<bool>,
    ) -> Result<KnowledgeEntry> {
        let model = KnowledgeBases::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| Error::string(&format!("Failed to find knowledge entry: {}", e)))?
            .ok_or_else(|| Error::string("Knowledge entry not found"))?;

        let mut active_model: knowledge_bases::ActiveModel = model.into();

        if let Some(content) = content {
            active_model.content = Set(content);
        }

        if let Some(tags) = relevance_tags {
            let relevance_json = JsonValue::Array(tags.into_iter().map(JsonValue::String).collect());
            active_model.relevance_tags = Set(Some(relevance_json));
        }

        if let Some(priority) = priority {
            active_model.priority = Set(Some(priority));
        }

        if let Some(is_active) = is_active {
            active_model.is_active = Set(Some(is_active));
        }

        let updated = active_model.update(db).await.map_err(|e| {
            Error::string(&format!("Failed to update knowledge entry: {}", e))
        })?;

        Ok(KnowledgeEntry::from(updated))
    }

    /// Delete knowledge entry (soft delete by setting is_active = false)
    pub async fn delete(db: &DatabaseConnection, id: i32) -> Result<()> {
        let model = KnowledgeBases::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| Error::string(&format!("Failed to find knowledge entry: {}", e)))?
            .ok_or_else(|| Error::string("Knowledge entry not found"))?;

        let mut active_model: knowledge_bases::ActiveModel = model.into();
        active_model.is_active = Set(Some(false));

        active_model.update(db).await.map_err(|e| {
            Error::string(&format!("Failed to delete knowledge entry: {}", e))
        })?;

        Ok(())
    }

    /// List all entries (for admin)
    pub async fn list_all(db: &DatabaseConnection) -> Result<Vec<KnowledgeEntry>> {
        let results = KnowledgeBases::find()
            .order_by_asc(knowledge_bases::Column::Category)
            .order_by_asc(knowledge_bases::Column::Name)
            .all(db)
            .await
            .map_err(|e| Error::string(&format!("Failed to list knowledge entries: {}", e)))?;

        Ok(results.into_iter().map(KnowledgeEntry::from).collect())
    }

    /// Search knowledge base by keywords (for Q&A)
    /// Returns entries with relevance scores
    pub async fn search_for_qa(
        db: &DatabaseConnection,
        question: &str,
        product: &str,
        max_results: usize,
    ) -> Result<Vec<(KnowledgeEntry, f32)>> {
        // Extract keywords from question
        let keywords = Self::extract_keywords(question);

        // Get all active entries
        let all_entries = KnowledgeBases::find()
            .filter(knowledge_bases::Column::IsActive.eq(true))
            .all(db)
            .await
            .map_err(|e| Error::string(&format!("Failed to search knowledge base: {}", e)))?;

        // Score each entry by relevance
        let mut scored: Vec<(KnowledgeEntry, f32)> = all_entries
            .into_iter()
            .map(KnowledgeEntry::from)
            .map(|entry| {
                let score = Self::calculate_relevance(&entry, &keywords, product);
                (entry, score)
            })
            .filter(|(_, score)| *score > 0.1) // Minimum threshold
            .collect();

        // Sort by score descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Return top N results
        Ok(scored.into_iter().take(max_results).collect())
    }

    /// Extract keywords from a question
    fn extract_keywords(question: &str) -> Vec<String> {
        // Common stop words to filter out
        let stop_words = [
            "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
            "have", "has", "had", "do", "does", "did", "will", "would", "could", "should",
            "may", "might", "must", "shall", "can", "need", "dare", "ought", "used",
            "to", "of", "in", "for", "on", "with", "at", "by", "from", "as", "into",
            "through", "during", "before", "after", "above", "below", "between",
            "under", "again", "further", "then", "once", "here", "there", "when",
            "where", "why", "how", "all", "each", "every", "both", "few", "more",
            "most", "other", "some", "such", "no", "nor", "not", "only", "own",
            "same", "so", "than", "too", "very", "just", "also",
            // Korean stop words
            "은", "는", "이", "가", "을", "를", "에", "에서", "으로", "로",
            "와", "과", "의", "도", "만", "까지", "부터", "하다", "있다", "되다",
            "수", "것", "등", "및", "또는", "그리고", "하지만", "그러나",
            // Common question words to keep context
            "what", "how", "use", "create", "make", "add", "set",
        ];

        question
            .to_lowercase()
            // Split on word boundaries
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|word| {
                let word = word.trim();
                word.len() >= 2 && !stop_words.contains(&word)
            })
            .map(|s| s.to_string())
            .collect()
    }

    /// Calculate relevance score for an entry
    fn calculate_relevance(entry: &KnowledgeEntry, keywords: &[String], product: &str) -> f32 {
        let mut score = 0.0f32;

        // Check product match (boost if matches)
        let product_lower = product.to_lowercase();
        if entry.category.to_lowercase().contains(&product_lower)
            || entry.name.to_lowercase().contains(&product_lower)
        {
            score += 0.3;
        }

        // Check keywords against entry name
        let name_lower = entry.name.to_lowercase();
        for keyword in keywords {
            if name_lower.contains(keyword) {
                score += 0.25;
            }
        }

        // Check keywords against component
        if let Some(ref component) = entry.component {
            let component_lower = component.to_lowercase();
            for keyword in keywords {
                if component_lower.contains(keyword) {
                    score += 0.2;
                }
            }
        }

        // Check keywords against section
        if let Some(ref section) = entry.section {
            let section_lower = section.to_lowercase();
            for keyword in keywords {
                if section_lower.contains(keyword) {
                    score += 0.15;
                }
            }
        }

        // Check keywords against content (lower weight due to length)
        let content_lower = entry.content.to_lowercase();
        for keyword in keywords {
            if content_lower.contains(keyword) {
                // Count occurrences for density scoring
                let count = content_lower.matches(keyword).count();
                score += 0.05 * (count as f32).min(5.0);
            }
        }

        // Check keywords against relevance tags
        if let Some(ref tags) = entry.relevance_tags {
            for tag in tags {
                let tag_lower = tag.to_lowercase();
                for keyword in keywords {
                    if tag_lower.contains(keyword) || keyword.contains(&tag_lower) {
                        score += 0.2;
                    }
                }
            }
        }

        // Priority boost
        if let Some(ref priority) = entry.priority {
            match priority.as_str() {
                "high" => score += 0.1,
                "medium" => score += 0.05,
                _ => {}
            }
        }

        // Normalize to 0-1 range
        score.min(1.0)
    }

    /// Get knowledge entries formatted for Q&A prompt
    pub async fn get_qa_knowledge(
        db: &DatabaseConnection,
        question: &str,
        product: &str,
        max_entries: usize,
    ) -> Result<(String, Vec<(i32, String, String, Option<String>, f32)>)> {
        let scored_entries = Self::search_for_qa(db, question, product, max_entries).await?;

        if scored_entries.is_empty() {
            return Ok((String::new(), vec![]));
        }

        // Assemble content
        let content = scored_entries
            .iter()
            .map(|(entry, _)| entry.content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        // Build references list
        let references: Vec<(i32, String, String, Option<String>, f32)> = scored_entries
            .into_iter()
            .map(|(entry, score)| {
                (entry.id, entry.name, entry.category, entry.section, score)
            })
            .collect();

        Ok((content, references))
    }
}

/// File-based fallback for reading knowledge from markdown files
pub struct KnowledgeFileFallback;

impl KnowledgeFileFallback {
    /// Read knowledge from docs/knowledge directory
    pub fn read_file(filename: &str) -> Result<String> {
        let path = format!("../docs/knowledge/{}", filename);
        std::fs::read_to_string(&path)
            .map_err(|e| Error::string(&format!("Failed to read knowledge file {}: {}", path, e)))
    }

    /// Get knowledge for screen type from files
    pub fn for_screen_type(_screen_type: &str) -> Result<String> {
        let base = Self::read_file("XFRAME5_KNOWLEDGE_BASE.md")?;
        let patterns = Self::read_file("XFRAME5_XML_PATTERNS.md")?;

        // Extract relevant sections based on screen_type
        // This is a simplified version - in production you'd parse and filter sections
        Ok(format!("{}\n\n{}", base, patterns))
    }
}
