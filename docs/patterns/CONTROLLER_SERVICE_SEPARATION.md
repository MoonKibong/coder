# Controller/Service Separation Pattern

## Overview

This project follows a strict separation between controllers and services:

- **Controllers**: Thin - handle HTTP concerns only (request parsing, response formatting, authentication)
- **Services**: Fat - contain all business logic, validation, and database operations

## Pattern Structure

### Service Layer

Services contain:
- Business logic and validation
- Database operations
- Parameter structs for each operation
- Response types

```rust
// src/services/admin/prompt_template.rs

/// Create parameters
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateParams {
    pub name: String,
    pub product: String,
    pub system_prompt: String,
    // ...
}

/// Update parameters
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateParams {
    pub name: Option<String>,
    pub product: Option<String>,
    // ...
}

pub struct PromptTemplateService;

impl PromptTemplateService {
    /// Create with validation
    pub async fn create(db: &DatabaseConnection, params: CreateParams) -> Result<Model> {
        // Validation
        if params.name.trim().is_empty() {
            return Err(Error::BadRequest("Name is required".to_string()));
        }

        // Database operation
        let item = ActiveModel {
            name: Set(params.name.trim().to_string()),
            // ...
        };
        let item = item.insert(db).await?;
        Ok(item)
    }

    /// Find by ID
    pub async fn find_by_id(db: &DatabaseConnection, id: i32) -> Result<Model> {
        Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| Error::NotFound)
    }

    /// Update with validation
    pub async fn update(db: &DatabaseConnection, id: i32, params: UpdateParams) -> Result<Model> {
        let item = Self::find_by_id(db, id).await?;
        let mut item: ActiveModel = item.into();

        if let Some(name) = params.name {
            if name.trim().is_empty() {
                return Err(Error::BadRequest("Name cannot be empty".to_string()));
            }
            item.name = Set(name.trim().to_string());
        }

        let item = item.update(db).await?;
        Ok(item)
    }

    /// Delete
    pub async fn delete(db: &DatabaseConnection, id: i32) -> Result<()> {
        let item = Self::find_by_id(db, id).await?;
        item.delete(db).await?;
        Ok(())
    }
}
```

### Controller Layer

Controllers handle:
- Authentication extraction
- Request parsing (Path, Query, Json)
- Response formatting (HTML, JSON)
- HTTP-specific concerns

```rust
// src/controllers/admin/prompt_templates.rs

use crate::services::admin::prompt_template::{
    CreateParams, PromptTemplateService, QueryParams, UpdateParams,
};

/// Create new item
#[debug_handler]
pub async fn create(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateParams>,  // Parse request body
) -> Result<Response> {
    // Delegate to service
    let item = PromptTemplateService::create(&ctx.db, params).await?;

    // Format response
    format::render().view(&v, "admin/prompt_template/row.html", data!({
        "item": item,
    }))
}

/// Update existing item
#[debug_handler]
pub async fn update(
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,             // Extract path parameter
    State(ctx): State<AppContext>,
    Json(params): Json<UpdateParams>, // Parse request body
) -> Result<Response> {
    let item = PromptTemplateService::update(&ctx.db, id, params).await?;

    format::render().view(&v, "admin/prompt_template/row.html", data!({
        "item": item,
    }))
}
```

## Benefits

1. **Testability**: Services can be unit tested without HTTP concerns
2. **Reusability**: Services can be called from multiple controllers or background jobs
3. **Maintainability**: Clear separation of concerns
4. **Validation**: Centralized in service layer, not duplicated

## File Organization

```
backend/src/
├── controllers/
│   └── admin/
│       ├── mod.rs
│       ├── prompt_templates.rs  # Thin - HTTP only
│       ├── company_rules.rs
│       ├── llm_configs.rs
│       └── generation_logs.rs
└── services/
    └── admin/
        ├── mod.rs
        ├── prompt_template.rs   # Fat - business logic
        ├── company_rule.rs
        ├── llm_config.rs
        └── generation_log.rs
```

## Guidelines

### Controllers Should NOT:
- Contain business logic
- Validate data (beyond basic HTTP parsing)
- Access database directly (except through services)
- Build complex queries

### Services Should NOT:
- Know about HTTP (no Request/Response types)
- Format responses (return domain types, not HTML/JSON)
- Handle authentication (receive authenticated context)

## Reference

Based on: HWS project's `src/controllers/` and `src/services/` structure
