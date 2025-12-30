//! Admin LLM Configs Controller
//!
//! HTMX-based CRUD for LLM configurations.
//! Thin controller - delegates to LlmConfigService.

use axum::http::HeaderMap;
use loco_rs::prelude::*;

use crate::llm::OllamaBackend;
use crate::middleware::cookie_auth::AuthUser;
use crate::services::admin::llm_config::{
    CreateParams, LlmConfigService, QueryParams, UpdateParams,
};

/// Main page - renders full layout for direct access, partial for HTMX
#[debug_handler]
pub async fn main(
    auth_user: AuthUser,
    headers: HeaderMap,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let params = QueryParams::default();
    let response = LlmConfigService::search(&ctx.db, &params).await?;

    // Check if this is an HTMX request
    let is_htmx = headers.get("HX-Request").is_some();
    let template = if is_htmx {
        "admin/llm_config/main.html"
    } else {
        "admin/llm_config/index.html"
    };

    format::render().view(
        &v,
        template,
        data!({
            "current_page": "llm_configs",
            "user": auth_user,
            "items": response.items,
            "page": response.page,
            "page_size": response.page_size,
            "total_pages": response.total_pages,
            "total_items": response.total_items,
        }),
    )
}

/// List view - for HTMX partial updates
#[debug_handler]
pub async fn list(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    Query(params): Query<QueryParams>,
) -> Result<Response> {
    let response = LlmConfigService::search(&ctx.db, &params).await?;

    format::render().view(
        &v,
        "admin/llm_config/list.html",
        data!({
            "items": response.items,
            "page": response.page,
            "page_size": response.page_size,
            "total_pages": response.total_pages,
            "total_items": response.total_items,
        }),
    )
}

/// New form
#[debug_handler]
pub async fn new_form(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    // Try to fetch available models from Ollama with a short timeout
    // If server is not reachable, show empty list (user can still enter manually)
    let ollama = OllamaBackend::from_env();
    let available_models = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        ollama.list_models(),
    )
    .await
    .ok()
    .and_then(|r| r.ok())
    .unwrap_or_default();

    format::render().view(
        &v,
        "admin/llm_config/create.html",
        data!({
            "available_models": available_models,
        }),
    )
}

/// Show item details
#[debug_handler]
pub async fn show(
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = LlmConfigService::find_by_id(&ctx.db, id).await?;

    format::render().view(
        &v,
        "admin/llm_config/show.html",
        data!({
            "item": item,
        }),
    )
}

/// Edit form
#[debug_handler]
pub async fn edit_form(
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = LlmConfigService::find_by_id(&ctx.db, id).await?;

    // Try to fetch available models from Ollama with a short timeout
    let ollama = OllamaBackend::from_env();
    let available_models = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        ollama.list_models(),
    )
    .await
    .ok()
    .and_then(|r| r.ok())
    .unwrap_or_default();

    format::render().view(
        &v,
        "admin/llm_config/edit.html",
        data!({
            "item": item,
            "available_models": available_models,
        }),
    )
}

/// Create new item
#[debug_handler]
pub async fn create(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateParams>,
) -> Result<Response> {
    let _item = LlmConfigService::create(&ctx.db, params).await?;

    // Return the full list to replace #search-result
    let query_params = QueryParams::default();
    let response = LlmConfigService::search(&ctx.db, &query_params).await?;

    format::render().view(
        &v,
        "admin/llm_config/list.html",
        data!({
            "items": response.items,
            "page": response.page,
            "page_size": response.page_size,
            "total_pages": response.total_pages,
            "total_items": response.total_items,
        }),
    )
}

/// Update existing item
#[debug_handler]
pub async fn update(
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<UpdateParams>,
) -> Result<Response> {
    let _item = LlmConfigService::update(&ctx.db, id, params).await?;

    // Return the full list to replace #search-result
    let query_params = QueryParams::default();
    let response = LlmConfigService::search(&ctx.db, &query_params).await?;

    format::render().view(
        &v,
        "admin/llm_config/list.html",
        data!({
            "items": response.items,
            "page": response.page,
            "page_size": response.page_size,
            "total_pages": response.total_pages,
            "total_items": response.total_items,
        }),
    )
}

/// Delete item
#[debug_handler]
pub async fn delete(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    LlmConfigService::delete(&ctx.db, id).await?;
    format::html("")
}
