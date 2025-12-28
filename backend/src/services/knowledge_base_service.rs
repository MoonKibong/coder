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
