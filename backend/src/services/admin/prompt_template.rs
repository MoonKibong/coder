//! Prompt Template Service
//!
//! Business logic for prompt template CRUD operations.

use loco_rs::prelude::*;
use sea_orm::{query::*, DatabaseConnection, PaginatorTrait};
use serde::{Deserialize, Serialize};

use crate::models::_entities::prompt_templates::{ActiveModel, Column, Entity, Model};
use crate::utils::{bool_from_str_or_bool, optional_bool_from_str_or_bool, OptionalField};

const DEFAULT_PAGE_SIZE: u64 = 20;
const MAX_PAGE_SIZE: u64 = 100;

/// Query parameters for search with pagination
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct QueryParams {
    /// Search keyword (matches name or product)
    pub keyword: Option<String>,

    /// Filter by product
    #[serde(default)]
    pub product: Vec<String>,

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
    pub product: String,
    pub screen_type: Option<String>,
    pub system_prompt: String,
    pub user_prompt_template: String,
    #[serde(default, deserialize_with = "bool_from_str_or_bool")]
    pub is_active: Option<bool>,
}

/// Update parameters
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateParams {
    // Required fields that can be updated
    pub name: Option<String>,
    pub product: Option<String>,
    pub system_prompt: Option<String>,
    pub user_prompt_template: Option<String>,

    // Optional fields - use OptionalField for proper PATCH semantics
    #[serde(default)]
    pub screen_type: OptionalField<String>,
    #[serde(default, deserialize_with = "optional_bool_from_str_or_bool")]
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

pub struct PromptTemplateService;

impl PromptTemplateService {
    /// Build query with filters and sorting
    fn build_query(params: &QueryParams) -> sea_orm::Select<Entity> {
        let mut condition = Condition::all();

        // Keyword search (name or product)
        if let Some(keyword) = &params.keyword {
            if !keyword.is_empty() {
                condition = condition.add(
                    Condition::any()
                        .add(Column::Name.contains(keyword))
                        .add(Column::Product.contains(keyword)),
                );
            }
        }

        // Multi-select product filter
        if !params.product.is_empty() {
            condition = condition.add(Column::Product.is_in(params.product.clone()));
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
            Some("product") => query.order_by(Column::Product, order),
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

    /// Create new prompt template
    pub async fn create(db: &DatabaseConnection, params: CreateParams) -> Result<Model> {
        // Validation
        if params.name.trim().is_empty() {
            return Err(Error::BadRequest("Name is required".to_string()));
        }
        if params.product.trim().is_empty() {
            return Err(Error::BadRequest("Product is required".to_string()));
        }
        if params.system_prompt.trim().is_empty() {
            return Err(Error::BadRequest("System prompt is required".to_string()));
        }

        let item = ActiveModel {
            name: Set(params.name.trim().to_string()),
            product: Set(params.product.trim().to_string()),
            screen_type: Set(params.screen_type),
            system_prompt: Set(params.system_prompt),
            user_prompt_template: Set(params.user_prompt_template),
            version: Set(1),
            is_active: Set(params.is_active),
            ..Default::default()
        };

        let item = item.insert(db).await?;
        Ok(item)
    }

    /// Update existing prompt template
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
        if let Some(product) = params.product {
            item.product = Set(product);
        }
        if let Some(system_prompt) = params.system_prompt {
            item.system_prompt = Set(system_prompt);
        }
        if let Some(user_prompt_template) = params.user_prompt_template {
            item.user_prompt_template = Set(user_prompt_template);
        }

        // Optional fields - only update if Present (not Missing)
        if let OptionalField::Present(opt_value) = params.screen_type {
            item.screen_type = Set(opt_value);
        }
        if let OptionalField::Present(opt_value) = params.is_active {
            item.is_active = Set(opt_value);
        }

        // Increment version
        let current_version = item.version.clone().unwrap();
        item.version = Set(current_version + 1);

        let item = item.update(db).await?;
        Ok(item)
    }

    /// Delete prompt template
    pub async fn delete(db: &DatabaseConnection, id: i32) -> Result<()> {
        let item = Self::find_by_id(db, id).await?;
        item.delete(db).await?;
        Ok(())
    }
}
