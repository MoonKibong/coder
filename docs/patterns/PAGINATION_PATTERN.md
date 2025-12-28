# Pagination Pattern

## Overview

This document describes the pagination pattern used in this project, adapted from the HWS project's established patterns.

## Pattern Structure

### Service Layer (Business Logic)

All pagination logic is in the service layer with a consistent structure:

```rust
// src/services/admin/prompt_template.rs

/// Query parameters for search with pagination
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct QueryParams {
    // Search/filter parameters
    pub keyword: Option<String>,
    #[serde(default)]
    pub product: Vec<String>,  // Multi-select with Vec<T>
    pub is_active: Option<bool>,

    // Sorting
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,  // "asc" or "desc"

    // Pagination
    pub page: Option<u64>,
    pub page_size: Option<u64>,
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

        // Apply filters...
        // Apply sorting...

        query
    }

    /// Search with pagination
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
}
```

### Controller Layer (HTTP Handling)

Controllers are thin - only handle HTTP concerns:

```rust
// src/controllers/admin/prompt_templates.rs

/// Main page - renders full layout with list
#[debug_handler]
pub async fn main(
    _auth_user: AuthUser,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let params = QueryParams::default();
    let response = PromptTemplateService::search(&ctx.db, &params).await?;

    format::render().view(&v, "admin/prompt_template/main.html", data!({
        "items": response.items,
        "page": response.page,
        "total_pages": response.total_pages,
        "total_items": response.total_items,
    }))
}

/// List view - for HTMX partial updates
#[debug_handler]
pub async fn list(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    Query(params): Query<QueryParams>,
) -> Result<Response> {
    let response = PromptTemplateService::search(&ctx.db, &params).await?;
    // Render partial template...
}
```

## Key Features

### Multi-Select Filters

Use `Vec<T>` for multi-select filters:

```rust
#[serde(default)]
pub status: Vec<String>,

// In build_query:
if !params.status.is_empty() {
    condition = condition.add(Column::Status.is_in(params.status.clone()));
}
```

### Page Size Limits

Always enforce max page size:

```rust
const DEFAULT_PAGE_SIZE: u64 = 20;
const MAX_PAGE_SIZE: u64 = 100;

let page_size = params.page_size.unwrap_or(DEFAULT_PAGE_SIZE).min(MAX_PAGE_SIZE);
```

### Consistent Response Format

All paginated responses include:
- `items` - The data for the current page
- `page` - Current page number (1-indexed)
- `page_size` - Items per page
- `total_pages` - Total number of pages
- `total_items` - Total count of all matching items

## Template Integration

### HTMX Pagination

```html
{% if total_pages > 1 %}
<div class="flex items-center justify-between px-4 py-3 border-t">
    <div class="text-sm text-muted-foreground">
        Page {{ page }} of {{ total_pages }} ({{ total_items }} total)
    </div>
    <div class="flex items-center gap-2">
        <input type="number" value="{{ page }}" min="1" max="{{ total_pages }}"
               name="page" form="search-form"
               hx-get="/admin/items/list" hx-target="#search-result"
               hx-trigger="input changed delay:500ms" />
    </div>
</div>
{% endif %}
```

## File Organization

```
backend/
├── src/
│   ├── controllers/admin/    # Thin - HTTP handling only
│   │   ├── prompt_templates.rs
│   │   ├── company_rules.rs
│   │   └── ...
│   └── services/admin/       # Fat - business logic
│       ├── mod.rs
│       ├── prompt_template.rs
│       ├── company_rule.rs
│       └── ...
```

## Reference

Based on: `../HWS/docs/patterns/PAGINATION_PATTERN.md`
