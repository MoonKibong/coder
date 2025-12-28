//! LLM Config Service
//!
//! Business logic for LLM configuration CRUD operations.

use loco_rs::prelude::*;
use sea_orm::{query::*, DatabaseConnection, PaginatorTrait};
use serde::{Deserialize, Serialize};

use crate::models::_entities::llm_configs::{ActiveModel, Column, Entity, Model};

const DEFAULT_PAGE_SIZE: u64 = 20;
const MAX_PAGE_SIZE: u64 = 100;

/// Query parameters for search with pagination
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct QueryParams {
    /// Search keyword (matches name or provider)
    pub keyword: Option<String>,

    /// Filter by provider
    #[serde(default)]
    pub provider: Vec<String>,

    /// Filter by active status
    pub is_active: Option<bool>,

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
    pub provider: String,
    pub model_name: String,
    pub endpoint_url: String,
    pub api_key: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<i32>,
    pub is_active: Option<bool>,
}

/// Update parameters
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateParams {
    pub name: Option<String>,
    pub provider: Option<String>,
    pub model_name: Option<String>,
    pub endpoint_url: Option<String>,
    pub api_key: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<i32>,
    pub is_active: Option<bool>,
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

pub struct LlmConfigService;

impl LlmConfigService {
    /// Build query with filters and sorting
    fn build_query(params: &QueryParams) -> sea_orm::Select<Entity> {
        let mut condition = Condition::all();

        // Keyword search (name or provider)
        if let Some(keyword) = &params.keyword {
            if !keyword.is_empty() {
                condition = condition.add(
                    Condition::any()
                        .add(Column::Name.contains(keyword))
                        .add(Column::Provider.contains(keyword)),
                );
            }
        }

        // Multi-select provider filter
        if !params.provider.is_empty() {
            condition = condition.add(Column::Provider.is_in(params.provider.clone()));
        }

        // Active status filter
        if let Some(is_active) = params.is_active {
            condition = condition.add(Column::IsActive.eq(Some(is_active)));
        }

        let mut query = Entity::find().filter(condition);

        // Apply sorting
        let order = match params.sort_order.as_deref() {
            Some("asc") => Order::Asc,
            _ => Order::Desc,
        };

        query = match params.sort_by.as_deref() {
            Some("name") => query.order_by(Column::Name, order),
            Some("provider") => query.order_by(Column::Provider, order),
            Some("created_at") => query.order_by(Column::CreatedAt, order),
            _ => query.order_by(Column::UpdatedAt, Order::Desc), // Default
        };

        query
    }

    /// Search with pagination, filters, and sorting
    pub async fn search(
        db: &DatabaseConnection,
        params: &QueryParams,
    ) -> Result<PageResponse<Model>> {
        let page = params.page.unwrap_or(1).max(1);
        let page_size = params.page_size.unwrap_or(DEFAULT_PAGE_SIZE).min(MAX_PAGE_SIZE);

        let query = Self::build_query(params);
        let paginator = query.paginate(db, page_size);

        let total_items = paginator.num_items().await?;
        let total_pages = paginator.num_pages().await?;
        let items = paginator.fetch_page(page - 1).await?;

        Ok(PageResponse {
            items,
            page,
            page_size,
            total_pages,
            total_items,
        })
    }

    /// Find by ID
    pub async fn find_by_id(db: &DatabaseConnection, id: i32) -> Result<Model> {
        Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| Error::NotFound)
    }

    /// Create new LLM config
    pub async fn create(db: &DatabaseConnection, params: CreateParams) -> Result<Model> {
        // Validation
        if params.name.trim().is_empty() {
            return Err(Error::BadRequest("Name is required".to_string()));
        }
        if params.provider.trim().is_empty() {
            return Err(Error::BadRequest("Provider is required".to_string()));
        }
        if params.model_name.trim().is_empty() {
            return Err(Error::BadRequest("Model name is required".to_string()));
        }
        if params.endpoint_url.trim().is_empty() {
            return Err(Error::BadRequest("Endpoint URL is required".to_string()));
        }

        // Validate temperature range
        if let Some(temp) = params.temperature {
            if !(0.0..=2.0).contains(&temp) {
                return Err(Error::BadRequest("Temperature must be between 0.0 and 2.0".to_string()));
            }
        }

        // Validate max_tokens
        if let Some(tokens) = params.max_tokens {
            if tokens <= 0 {
                return Err(Error::BadRequest("Max tokens must be positive".to_string()));
            }
        }

        let item = ActiveModel {
            name: Set(params.name.trim().to_string()),
            provider: Set(params.provider.trim().to_string()),
            model_name: Set(params.model_name.trim().to_string()),
            endpoint_url: Set(params.endpoint_url.trim().to_string()),
            api_key: Set(params.api_key),
            temperature: Set(params.temperature),
            max_tokens: Set(params.max_tokens),
            is_active: Set(params.is_active),
            ..Default::default()
        };

        let item = item.insert(db).await?;
        Ok(item)
    }

    /// Update existing LLM config
    pub async fn update(
        db: &DatabaseConnection,
        id: i32,
        params: UpdateParams,
    ) -> Result<Model> {
        let item = Self::find_by_id(db, id).await?;
        let mut item: ActiveModel = item.into();

        if let Some(name) = params.name {
            if name.trim().is_empty() {
                return Err(Error::BadRequest("Name cannot be empty".to_string()));
            }
            item.name = Set(name.trim().to_string());
        }
        if let Some(provider) = params.provider {
            item.provider = Set(provider);
        }
        if let Some(model_name) = params.model_name {
            item.model_name = Set(model_name);
        }
        if let Some(endpoint_url) = params.endpoint_url {
            item.endpoint_url = Set(endpoint_url);
        }
        if params.api_key.is_some() {
            item.api_key = Set(params.api_key);
        }
        if params.temperature.is_some() {
            if let Some(temp) = params.temperature {
                if !(0.0..=2.0).contains(&temp) {
                    return Err(Error::BadRequest("Temperature must be between 0.0 and 2.0".to_string()));
                }
            }
            item.temperature = Set(params.temperature);
        }
        if params.max_tokens.is_some() {
            if let Some(tokens) = params.max_tokens {
                if tokens <= 0 {
                    return Err(Error::BadRequest("Max tokens must be positive".to_string()));
                }
            }
            item.max_tokens = Set(params.max_tokens);
        }
        if params.is_active.is_some() {
            item.is_active = Set(params.is_active);
        }

        let item = item.update(db).await?;
        Ok(item)
    }

    /// Delete LLM config
    pub async fn delete(db: &DatabaseConnection, id: i32) -> Result<()> {
        let item = Self::find_by_id(db, id).await?;
        item.delete(db).await?;
        Ok(())
    }
}
