//! Knowledge Base Admin Service
//!
//! Business logic for knowledge base CRUD operations in admin panel.

use loco_rs::prelude::*;
use sea_orm::{query::*, DatabaseConnection, JsonValue, PaginatorTrait};
use serde::{Deserialize, Serialize};

use crate::models::_entities::knowledge_bases::{ActiveModel, Column, Entity, Model};
use crate::utils::{bool_from_str_or_bool, i32_from_str_or_number, OptionalField};

const DEFAULT_PAGE_SIZE: u64 = 20;
const MAX_PAGE_SIZE: u64 = 100;

/// Query parameters for search with pagination
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct QueryParams {
    /// Search keyword (matches name or content)
    pub keyword: Option<String>,

    /// Filter by category
    pub category: Option<String>,

    /// Filter by component
    pub component: Option<String>,

    /// Sort column
    pub sort_by: Option<String>,

    /// Sort order: "asc" or "desc"
    pub sort_order: Option<String>,

    /// Page number (1-indexed)
    pub page: Option<u64>,

    /// Page size
    pub page_size: Option<u64>,
}

/// Create parameters
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateParams {
    pub name: String,
    pub category: String,
    pub component: Option<String>,
    pub section: Option<String>,
    pub content: String,
    pub relevance_tags: Option<String>, // Comma-separated string
    pub priority: Option<String>,
    #[serde(default, deserialize_with = "i32_from_str_or_number")]
    pub token_estimate: Option<i32>,
    #[serde(default, deserialize_with = "bool_from_str_or_bool")]
    pub is_active: Option<bool>,
}

/// Update parameters
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateParams {
    // Required fields
    pub name: Option<String>,
    pub category: Option<String>,
    pub content: Option<String>,

    // Optional fields - use OptionalField for proper PATCH semantics
    #[serde(default)]
    pub component: OptionalField<String>,
    #[serde(default)]
    pub section: OptionalField<String>,
    #[serde(default)]
    pub relevance_tags: OptionalField<String>, // Comma-separated string
    #[serde(default)]
    pub priority: OptionalField<String>,
    #[serde(default)]
    pub token_estimate: OptionalField<i32>,
    #[serde(default)]
    pub is_active: OptionalField<bool>,
}

/// Paginated response
#[derive(Debug, Serialize)]
pub struct PageResponse<T> {
    pub items: Vec<T>,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
    pub total_items: u64,
}

/// Knowledge Entry DTO for admin views
#[derive(Debug, Serialize)]
pub struct KnowledgeEntryDto {
    pub id: i32,
    pub name: String,
    pub category: String,
    pub component: Option<String>,
    pub section: Option<String>,
    pub content: String,
    pub relevance_tags: Option<Vec<String>>,
    pub priority: Option<String>,
    pub token_estimate: Option<i32>,
    pub version: Option<i32>,
    pub is_active: Option<bool>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Model> for KnowledgeEntryDto {
    fn from(model: Model) -> Self {
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
            version: model.version,
            is_active: model.is_active,
            created_at: model.created_at.to_string(),
            updated_at: model.updated_at.to_string(),
        }
    }
}

pub struct KnowledgeBaseService;

impl KnowledgeBaseService {
    /// Build query with filters and sorting
    fn build_query(params: &QueryParams) -> sea_orm::Select<Entity> {
        let mut condition = Condition::all();

        // Keyword search (name or content)
        if let Some(keyword) = &params.keyword {
            if !keyword.is_empty() {
                let keyword_condition = Condition::any()
                    .add(Column::Name.contains(keyword))
                    .add(Column::Content.contains(keyword));
                condition = condition.add(keyword_condition);
            }
        }

        // Category filter
        if let Some(category) = &params.category {
            if !category.is_empty() {
                condition = condition.add(Column::Category.eq(category));
            }
        }

        // Component filter
        if let Some(component) = &params.component {
            if !component.is_empty() {
                condition = condition.add(Column::Component.eq(component));
            }
        }

        let mut query = Entity::find().filter(condition);

        // Apply sorting
        let order = match params.sort_order.as_deref() {
            Some("asc") => Order::Asc,
            _ => Order::Desc,
        };

        query = match params.sort_by.as_deref() {
            Some("name") => query.order_by(Column::Name, order),
            Some("category") => query.order_by(Column::Category, order),
            Some("priority") => query.order_by(Column::Priority, order),
            _ => query.order_by(Column::UpdatedAt, order),
        };

        query
    }

    /// Search with pagination
    pub async fn search(
        db: &DatabaseConnection,
        params: &QueryParams,
    ) -> Result<PageResponse<KnowledgeEntryDto>> {
        let page = params.page.unwrap_or(1).max(1);
        let page_size = params
            .page_size
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .min(MAX_PAGE_SIZE)
            .max(1);

        let query = Self::build_query(params);
        let paginator = query.paginate(db, page_size);

        let total_items = paginator.num_items().await?;
        let total_pages = paginator.num_pages().await?;

        let models = paginator.fetch_page(page - 1).await?;
        let items = models.into_iter().map(KnowledgeEntryDto::from).collect();

        Ok(PageResponse {
            items,
            page,
            page_size,
            total_pages,
            total_items,
        })
    }

    /// Find by ID
    pub async fn find_by_id(db: &DatabaseConnection, id: i32) -> Result<KnowledgeEntryDto> {
        let model = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| Error::NotFound)?;

        Ok(KnowledgeEntryDto::from(model))
    }

    /// Create new entry
    pub async fn create(db: &DatabaseConnection, params: CreateParams) -> Result<KnowledgeEntryDto> {
        // Parse comma-separated tags
        let relevance_tags = params.relevance_tags.and_then(|tags_str| {
            let tags: Vec<String> = tags_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            if tags.is_empty() {
                None
            } else {
                Some(JsonValue::Array(
                    tags.into_iter().map(JsonValue::String).collect(),
                ))
            }
        });

        let active_model = ActiveModel {
            name: Set(params.name),
            category: Set(params.category),
            component: Set(params.component),
            section: Set(params.section),
            content: Set(params.content),
            relevance_tags: Set(relevance_tags),
            priority: Set(params.priority),
            token_estimate: Set(params.token_estimate),
            version: Set(Some(1)),
            is_active: Set(params.is_active.or(Some(true))),
            ..Default::default()
        };

        let model = active_model.insert(db).await?;
        Ok(KnowledgeEntryDto::from(model))
    }

    /// Update existing entry
    pub async fn update(
        db: &DatabaseConnection,
        id: i32,
        params: UpdateParams,
    ) -> Result<KnowledgeEntryDto> {
        let model = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| Error::NotFound)?;

        let mut active_model: ActiveModel = model.into();

        // Required fields
        if let Some(name) = params.name {
            active_model.name = Set(name);
        }

        if let Some(category) = params.category {
            active_model.category = Set(category);
        }

        if let Some(content) = params.content {
            active_model.content = Set(content);
        }

        // Optional fields - only update if Present (not Missing)
        if let OptionalField::Present(opt_value) = params.component {
            active_model.component = Set(opt_value);
        }

        if let OptionalField::Present(opt_value) = params.section {
            active_model.section = Set(opt_value);
        }

        if let OptionalField::Present(opt_tags_str) = params.relevance_tags {
            let relevance_tags = opt_tags_str.map(|tags_str| {
                let tags: Vec<String> = tags_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                if tags.is_empty() {
                    None
                } else {
                    Some(JsonValue::Array(
                        tags.into_iter().map(JsonValue::String).collect(),
                    ))
                }
            }).flatten();

            active_model.relevance_tags = Set(relevance_tags);
        }

        if let OptionalField::Present(opt_value) = params.priority {
            active_model.priority = Set(opt_value);
        }

        if let OptionalField::Present(opt_value) = params.token_estimate {
            active_model.token_estimate = Set(opt_value);
        }

        if let OptionalField::Present(opt_value) = params.is_active {
            active_model.is_active = Set(opt_value);
        }

        let updated = active_model.update(db).await?;
        Ok(KnowledgeEntryDto::from(updated))
    }

    /// Delete entry (soft delete)
    pub async fn delete(db: &DatabaseConnection, id: i32) -> Result<()> {
        let model = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| Error::NotFound)?;

        let mut active_model: ActiveModel = model.into();
        active_model.is_active = Set(Some(false));
        active_model.update(db).await?;

        Ok(())
    }
}
