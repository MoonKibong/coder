//! Generation Log Service
//!
//! Business logic for generation log viewing (audit trail).
//! Read-only - no create/update/delete operations.

use chrono::{DateTime, FixedOffset};
use loco_rs::prelude::*;
use sea_orm::{query::*, DatabaseConnection, PaginatorTrait};
use serde::{Deserialize, Serialize};

use crate::models::_entities::generation_logs::{Column, Entity, Model};
use crate::models::_entities::users;

const DEFAULT_PAGE_SIZE: u64 = 50;
const MAX_PAGE_SIZE: u64 = 100;

/// Generation log with user info for display
#[derive(Debug, Clone, Serialize)]
pub struct GenerationLogWithUser {
    pub id: i32,
    pub created_at: DateTime<FixedOffset>,
    pub product: String,
    pub input_type: String,
    pub status: String,
    pub generation_time_ms: Option<i32>,
    pub user_id: i32,
    pub user_name: String,
    pub user_email: String,
    pub ui_intent: String,
    pub artifacts: Option<String>,
    pub warnings: Option<String>,
    pub error_message: Option<String>,
}

impl GenerationLogWithUser {
    fn from_models(log: Model, user: Option<users::Model>) -> Self {
        let (user_name, user_email) = match user {
            Some(u) => (u.name, u.email),
            None => ("Unknown".to_string(), "".to_string()),
        };

        Self {
            id: log.id,
            created_at: log.created_at,
            product: log.product,
            input_type: log.input_type,
            status: log.status,
            generation_time_ms: log.generation_time_ms,
            user_id: log.user_id,
            user_name,
            user_email,
            ui_intent: log.ui_intent,
            artifacts: log.artifacts,
            warnings: log.warnings,
            error_message: log.error_message,
        }
    }
}

/// Query parameters for search with pagination
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct QueryParams {
    /// Filter by status
    #[serde(default)]
    pub status: Vec<String>,

    /// Filter by product
    #[serde(default)]
    pub product: Vec<String>,

    /// Filter by input type
    pub input_type: Option<String>,

    /// Sort column
    pub sort_by: Option<String>,

    /// Sort order: "asc" or "desc"
    pub sort_order: Option<String>,

    /// Page number (1-indexed)
    pub page: Option<u64>,

    /// Page size
    pub page_size: Option<u64>,
}

/// Paginated response with total count
#[derive(Debug, Serialize)]
pub struct PageResponse<T> {
    pub items: Vec<T>,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
    pub total_items: u64,
}

pub struct GenerationLogService;

impl GenerationLogService {
    /// Build query with filters and sorting
    fn build_query(params: &QueryParams) -> sea_orm::Select<Entity> {
        let mut condition = Condition::all();

        // Multi-select status filter
        if !params.status.is_empty() {
            condition = condition.add(Column::Status.is_in(params.status.clone()));
        }

        // Multi-select product filter
        if !params.product.is_empty() {
            condition = condition.add(Column::Product.is_in(params.product.clone()));
        }

        // Input type filter
        if let Some(input_type) = &params.input_type {
            if !input_type.is_empty() {
                condition = condition.add(Column::InputType.eq(input_type.as_str()));
            }
        }

        let mut query = Entity::find().filter(condition);

        // Apply sorting
        let order = match params.sort_order.as_deref() {
            Some("asc") => Order::Asc,
            _ => Order::Desc,
        };

        query = match params.sort_by.as_deref() {
            Some("status") => query.order_by(Column::Status, order),
            Some("product") => query.order_by(Column::Product, order),
            Some("generation_time_ms") => query.order_by(Column::GenerationTimeMs, order),
            _ => query.order_by(Column::CreatedAt, Order::Desc), // Default
        };

        query
    }

    /// Search with pagination, filters, and sorting
    pub async fn search(
        db: &DatabaseConnection,
        params: &QueryParams,
    ) -> Result<PageResponse<GenerationLogWithUser>> {
        let page = params.page.unwrap_or(1).max(1);
        let page_size = params.page_size.unwrap_or(DEFAULT_PAGE_SIZE).min(MAX_PAGE_SIZE);

        let query = Self::build_query(params);
        let paginator = query.paginate(db, page_size);

        let total_items = paginator.num_items().await?;
        let total_pages = paginator.num_pages().await?;
        let logs = paginator.fetch_page(page - 1).await?;

        // Fetch user info for each log
        let mut items = Vec::with_capacity(logs.len());
        for log in logs {
            let user = users::Entity::find_by_id(log.user_id).one(db).await.ok().flatten();
            items.push(GenerationLogWithUser::from_models(log, user));
        }

        Ok(PageResponse {
            items,
            page,
            page_size,
            total_pages,
            total_items,
        })
    }

    /// Find by ID with user info
    pub async fn find_by_id(db: &DatabaseConnection, id: i32) -> Result<GenerationLogWithUser> {
        let log = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| Error::NotFound)?;

        let user = users::Entity::find_by_id(log.user_id).one(db).await.ok().flatten();
        Ok(GenerationLogWithUser::from_models(log, user))
    }
}
