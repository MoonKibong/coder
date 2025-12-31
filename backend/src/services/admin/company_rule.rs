//! Company Rule Service
//!
//! Business logic for company rule CRUD operations.

use loco_rs::prelude::*;
use sea_orm::{query::*, DatabaseConnection, PaginatorTrait};
use serde::{Deserialize, Serialize};

use crate::models::_entities::company_rules::{ActiveModel, Column, Entity, Model};
use crate::utils::OptionalField;

const DEFAULT_PAGE_SIZE: u64 = 20;
const MAX_PAGE_SIZE: u64 = 100;

/// Query parameters for search with pagination
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct QueryParams {
    /// Search keyword (matches name)
    pub keyword: Option<String>,

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
    pub naming_convention: Option<String>,
    pub additional_rules: Option<String>,
}

/// Update parameters
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateParams {
    // Required field
    pub name: Option<String>,

    // Optional fields - use OptionalField for proper PATCH semantics
    #[serde(default)]
    pub naming_convention: OptionalField<String>,
    #[serde(default)]
    pub additional_rules: OptionalField<String>,
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

pub struct CompanyRuleService;

impl CompanyRuleService {
    /// Build query with filters and sorting
    fn build_query(params: &QueryParams) -> sea_orm::Select<Entity> {
        let mut condition = Condition::all();

        // Keyword search
        if let Some(keyword) = &params.keyword {
            if !keyword.is_empty() {
                condition = condition.add(Column::Name.contains(keyword));
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

    /// Create new company rule
    pub async fn create(db: &DatabaseConnection, params: CreateParams) -> Result<Model> {
        // Validation
        if params.name.trim().is_empty() {
            return Err(Error::BadRequest("Name is required".to_string()));
        }

        let item = ActiveModel {
            name: Set(params.name.trim().to_string()),
            naming_convention: Set(params.naming_convention),
            additional_rules: Set(params.additional_rules),
            ..Default::default()
        };

        let item = item.insert(db).await?;
        Ok(item)
    }

    /// Update existing company rule
    pub async fn update(
        db: &DatabaseConnection,
        id: i32,
        params: UpdateParams,
    ) -> Result<Model> {
        let item = Self::find_by_id(db, id).await?;
        let mut item: ActiveModel = item.into();

        // Required field
        if let Some(name) = params.name {
            if name.trim().is_empty() {
                return Err(Error::BadRequest("Name cannot be empty".to_string()));
            }
            item.name = Set(name.trim().to_string());
        }

        // Optional fields - only update if Present (not Missing)
        if let OptionalField::Present(opt_value) = params.naming_convention {
            item.naming_convention = Set(opt_value);
        }
        if let OptionalField::Present(opt_value) = params.additional_rules {
            item.additional_rules = Set(opt_value);
        }

        let item = item.update(db).await?;
        Ok(item)
    }

    /// Delete company rule
    pub async fn delete(db: &DatabaseConnection, id: i32) -> Result<()> {
        let item = Self::find_by_id(db, id).await?;
        item.delete(db).await?;
        Ok(())
    }
}
