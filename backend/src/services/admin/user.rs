//! User Service
//!
//! Business logic for user CRUD operations in admin panel.

use loco_rs::prelude::*;
use sea_orm::{query::*, DatabaseConnection, PaginatorTrait};
use serde::{Deserialize, Serialize};

use crate::models::_entities::users::{ActiveModel, Column, Entity, Model};

const DEFAULT_PAGE_SIZE: u64 = 20;
const MAX_PAGE_SIZE: u64 = 100;

/// Query parameters for search with pagination
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct QueryParams {
    /// Search keyword (matches name or email)
    pub keyword: Option<String>,

    /// Filter by email verification status
    pub email_verified: Option<bool>,

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
    pub email: String,
    pub password: String,
}

/// Update parameters
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateParams {
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}

/// User response without sensitive fields
#[derive(Debug, Clone, Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub pid: String,
    pub name: String,
    pub email: String,
    pub email_verified: bool,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

impl From<Model> for UserResponse {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            pid: model.pid.to_string(),
            name: model.name,
            email: model.email,
            email_verified: model.email_verified_at.is_some(),
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
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

pub struct UserService;

impl UserService {
    /// Build query with filters and sorting
    fn build_query(params: &QueryParams) -> sea_orm::Select<Entity> {
        let mut condition = Condition::all();

        // Keyword search (name or email)
        if let Some(keyword) = &params.keyword {
            if !keyword.is_empty() {
                condition = condition.add(
                    Condition::any()
                        .add(Column::Name.contains(keyword))
                        .add(Column::Email.contains(keyword)),
                );
            }
        }

        // Email verification status filter
        if let Some(email_verified) = params.email_verified {
            if email_verified {
                condition = condition.add(Column::EmailVerifiedAt.is_not_null());
            } else {
                condition = condition.add(Column::EmailVerifiedAt.is_null());
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
            Some("email") => query.order_by(Column::Email, order),
            Some("created_at") => query.order_by(Column::CreatedAt, order),
            _ => query.order_by(Column::UpdatedAt, Order::Desc), // Default
        };

        query
    }

    /// Search with pagination, filters, and sorting
    pub async fn search(
        db: &DatabaseConnection,
        params: &QueryParams,
    ) -> Result<PageResponse<UserResponse>> {
        let page = params.page.unwrap_or(1).max(1);
        let page_size = params.page_size.unwrap_or(DEFAULT_PAGE_SIZE).min(MAX_PAGE_SIZE);

        let query = Self::build_query(params);
        let paginator = query.paginate(db, page_size);

        let total_items = paginator.num_items().await?;
        let total_pages = paginator.num_pages().await?;
        let items: Vec<UserResponse> = paginator
            .fetch_page(page - 1)
            .await?
            .into_iter()
            .map(UserResponse::from)
            .collect();

        Ok(PageResponse {
            items,
            page,
            page_size,
            total_pages,
            total_items,
        })
    }

    /// Find by ID
    pub async fn find_by_id(db: &DatabaseConnection, id: i32) -> Result<UserResponse> {
        let user = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| Error::NotFound)?;
        Ok(UserResponse::from(user))
    }

    /// Create new user
    pub async fn create(db: &DatabaseConnection, params: CreateParams) -> Result<UserResponse> {
        use loco_rs::hash;

        // Validation
        if params.name.trim().is_empty() {
            return Err(Error::BadRequest("Name is required".to_string()));
        }
        if params.email.trim().is_empty() {
            return Err(Error::BadRequest("Email is required".to_string()));
        }
        if params.password.is_empty() {
            return Err(Error::BadRequest("Password is required".to_string()));
        }
        if params.password.len() < 8 {
            return Err(Error::BadRequest("Password must be at least 8 characters".to_string()));
        }

        // Check if email already exists
        let existing = Entity::find()
            .filter(Column::Email.eq(params.email.trim()))
            .one(db)
            .await?;
        if existing.is_some() {
            return Err(Error::BadRequest("Email already exists".to_string()));
        }

        // Hash password
        let password_hash = hash::hash_password(&params.password)
            .map_err(|e| Error::string(&e.to_string()))?;

        let user = ActiveModel {
            name: Set(params.name.trim().to_string()),
            email: Set(params.email.trim().to_lowercase()),
            password: Set(password_hash),
            ..Default::default()
        };

        let user = user.insert(db).await?;
        Ok(UserResponse::from(user))
    }

    /// Update existing user
    pub async fn update(
        db: &DatabaseConnection,
        id: i32,
        params: UpdateParams,
    ) -> Result<UserResponse> {
        use loco_rs::hash;

        let user = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| Error::NotFound)?;
        let mut user: ActiveModel = user.into();

        if let Some(name) = params.name {
            if name.trim().is_empty() {
                return Err(Error::BadRequest("Name cannot be empty".to_string()));
            }
            user.name = Set(name.trim().to_string());
        }

        if let Some(email) = params.email {
            if email.trim().is_empty() {
                return Err(Error::BadRequest("Email cannot be empty".to_string()));
            }
            // Check if email is taken by another user
            let existing = Entity::find()
                .filter(Column::Email.eq(email.trim()))
                .filter(Column::Id.ne(id))
                .one(db)
                .await?;
            if existing.is_some() {
                return Err(Error::BadRequest("Email already exists".to_string()));
            }
            user.email = Set(email.trim().to_lowercase());
        }

        if let Some(password) = params.password {
            if !password.is_empty() {
                if password.len() < 8 {
                    return Err(Error::BadRequest("Password must be at least 8 characters".to_string()));
                }
                let password_hash = hash::hash_password(&password)
                    .map_err(|e| Error::string(&e.to_string()))?;
                user.password = Set(password_hash);
            }
        }

        let user = user.update(db).await?;
        Ok(UserResponse::from(user))
    }

    /// Delete user
    pub async fn delete(db: &DatabaseConnection, id: i32, current_user_pid: &str) -> Result<()> {
        // Find the user to delete
        let user = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| Error::NotFound)?;

        // Prevent self-deletion (compare pids)
        if user.pid.to_string() == current_user_pid {
            return Err(Error::BadRequest("Cannot delete your own account".to_string()));
        }

        // Ensure at least one user remains
        let count = Entity::find().count(db).await?;
        if count <= 1 {
            return Err(Error::BadRequest("Cannot delete the last user".to_string()));
        }

        user.delete(db).await?;
        Ok(())
    }
}
