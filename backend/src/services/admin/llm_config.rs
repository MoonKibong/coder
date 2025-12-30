//! LLM Config Service
//!
//! Business logic for LLM configuration CRUD operations.

use loco_rs::prelude::*;
use sea_orm::{query::*, DatabaseConnection, PaginatorTrait};
use serde::{Deserialize, Serialize};

use crate::models::_entities::llm_configs::{ActiveModel, Column, Entity, Model};
use crate::utils::{
    bool_from_str_or_bool, f32_from_str_or_number, i32_from_str_or_number,
    optional_bool_from_str_or_bool, optional_f32_from_str_or_number, optional_i32_from_str_or_number,
    OptionalField,
};

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
    /// Optional for local-llama-cpp provider (which uses model_path instead)
    pub endpoint_url: Option<String>,
    pub api_key: Option<String>,
    #[serde(default, deserialize_with = "f32_from_str_or_number")]
    pub temperature: Option<f32>,
    #[serde(default, deserialize_with = "i32_from_str_or_number")]
    pub max_tokens: Option<i32>,
    #[serde(default, deserialize_with = "bool_from_str_or_bool")]
    pub is_active: Option<bool>,

    // Local LLM fields (for local-llama-cpp provider)
    /// Path to GGUF model file
    pub model_path: Option<String>,
    /// Context window size
    #[serde(default, deserialize_with = "i32_from_str_or_number")]
    pub n_ctx: Option<i32>,
    /// Number of CPU threads
    #[serde(default, deserialize_with = "i32_from_str_or_number")]
    pub n_threads: Option<i32>,
}

/// Update parameters
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateParams {
    // Required fields
    pub name: Option<String>,
    pub provider: Option<String>,
    pub model_name: Option<String>,
    pub endpoint_url: Option<String>,

    // Optional fields - use OptionalField for proper PATCH semantics
    // with string-to-type conversion for HTML form compatibility
    #[serde(default)]
    pub api_key: OptionalField<String>,
    #[serde(default, deserialize_with = "optional_f32_from_str_or_number")]
    pub temperature: OptionalField<f32>,
    #[serde(default, deserialize_with = "optional_i32_from_str_or_number")]
    pub max_tokens: OptionalField<i32>,
    #[serde(default, deserialize_with = "optional_bool_from_str_or_bool")]
    pub is_active: OptionalField<bool>,

    // Local LLM fields (for local-llama-cpp provider)
    #[serde(default)]
    pub model_path: OptionalField<String>,
    #[serde(default, deserialize_with = "optional_i32_from_str_or_number")]
    pub n_ctx: OptionalField<i32>,
    #[serde(default, deserialize_with = "optional_i32_from_str_or_number")]
    pub n_threads: OptionalField<i32>,
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

        let is_local_llm = params.provider == "local-llama-cpp";

        // Endpoint URL is required for non-local providers
        if !is_local_llm {
            if params.endpoint_url.as_ref().map_or(true, |url| url.trim().is_empty()) {
                return Err(Error::BadRequest("Endpoint URL is required for this provider".to_string()));
            }
        }

        // Model path is recommended for local-llama-cpp
        if is_local_llm && params.model_path.as_ref().map_or(true, |p| p.trim().is_empty()) {
            return Err(Error::BadRequest("Model path is required for local-llama-cpp provider".to_string()));
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

        // Validate n_ctx
        if let Some(n_ctx) = params.n_ctx {
            if n_ctx < 512 || n_ctx > 32768 {
                return Err(Error::BadRequest("Context size must be between 512 and 32768".to_string()));
            }
        }

        // Validate n_threads
        if let Some(n_threads) = params.n_threads {
            if n_threads < 1 || n_threads > 64 {
                return Err(Error::BadRequest("CPU threads must be between 1 and 64".to_string()));
            }
        }

        let item = ActiveModel {
            name: Set(params.name.trim().to_string()),
            provider: Set(params.provider.trim().to_string()),
            model_name: Set(params.model_name.trim().to_string()),
            endpoint_url: Set(params.endpoint_url.map(|url| url.trim().to_string())),
            api_key: Set(params.api_key),
            temperature: Set(params.temperature),
            max_tokens: Set(params.max_tokens),
            is_active: Set(params.is_active),
            model_path: Set(params.model_path.map(|p| p.trim().to_string())),
            n_ctx: Set(params.n_ctx),
            n_threads: Set(params.n_threads),
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

        // Required fields
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
            // endpoint_url is now Option<String> in entity
            let url = if endpoint_url.trim().is_empty() {
                None
            } else {
                Some(endpoint_url.trim().to_string())
            };
            item.endpoint_url = Set(url);
        }

        // Optional fields - only update if Present (not Missing)
        if let OptionalField::Present(opt_value) = params.api_key {
            item.api_key = Set(opt_value);
        }
        if let OptionalField::Present(opt_value) = params.temperature {
            if let Some(temp) = opt_value {
                if !(0.0..=2.0).contains(&temp) {
                    return Err(Error::BadRequest("Temperature must be between 0.0 and 2.0".to_string()));
                }
            }
            item.temperature = Set(opt_value);
        }
        if let OptionalField::Present(opt_value) = params.max_tokens {
            if let Some(tokens) = opt_value {
                if tokens <= 0 {
                    return Err(Error::BadRequest("Max tokens must be positive".to_string()));
                }
            }
            item.max_tokens = Set(opt_value);
        }
        if let OptionalField::Present(opt_value) = params.is_active {
            item.is_active = Set(opt_value);
        }

        // Local LLM fields
        if let OptionalField::Present(opt_value) = params.model_path {
            item.model_path = Set(opt_value.map(|p| p.trim().to_string()));
        }
        if let OptionalField::Present(opt_value) = params.n_ctx {
            if let Some(n_ctx) = opt_value {
                if n_ctx < 512 || n_ctx > 32768 {
                    return Err(Error::BadRequest("Context size must be between 512 and 32768".to_string()));
                }
            }
            item.n_ctx = Set(opt_value);
        }
        if let OptionalField::Present(opt_value) = params.n_threads {
            if let Some(n_threads) = opt_value {
                if n_threads < 1 || n_threads > 64 {
                    return Err(Error::BadRequest("CPU threads must be between 1 and 64".to_string()));
                }
            }
            item.n_threads = Set(opt_value);
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

    /// Activate a specific LLM config and deactivate all others
    ///
    /// This ensures only one LLM config is active at a time.
    pub async fn activate(db: &DatabaseConnection, id: i32) -> Result<Model> {
        // First, verify the item exists
        let _item = Self::find_by_id(db, id).await?;

        // Deactivate all currently active configs
        use sea_orm::QueryFilter;
        let active_configs = Entity::find()
            .filter(Column::IsActive.eq(Some(true)))
            .all(db)
            .await?;

        for config in active_configs {
            let mut active_model: ActiveModel = config.into();
            active_model.is_active = Set(Some(false));
            active_model.update(db).await?;
        }

        // Now activate the specified config
        let item = Self::find_by_id(db, id).await?;
        let mut item: ActiveModel = item.into();
        item.is_active = Set(Some(true));
        let item = item.update(db).await?;

        Ok(item)
    }
}
