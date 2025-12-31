//! Admin LLM Configs Controller
//!
//! HTMX-based CRUD for LLM configurations.
//! Thin controller - delegates to LlmConfigService.

use axum::http::{header, HeaderMap, StatusCode};
use loco_rs::prelude::*;
use serde::Deserialize;
use tracing::debug;

use crate::llm::OllamaBackend;
use crate::middleware::cookie_auth::AuthUser;
use crate::services::admin::llm_config::{
    CreateParams, LlmConfigService, QueryParams, UpdateParams,
};

/// Query parameters for fetching models from a remote endpoint
#[derive(Debug, Deserialize)]
pub struct FetchModelsParams {
    pub endpoint_url: Option<String>,
    pub current_model: Option<String>,
}

/// Helper to check if request is from HTMX
fn is_htmx_request(headers: &HeaderMap) -> bool {
    headers.get("HX-Request").is_some()
}

/// Redirect response for non-HTMX requests to modal endpoints
fn redirect_to_main_page() -> Result<Response> {
    Ok(Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/admin/llm-configs")
        .body(axum::body::Body::empty())?
        .into_response())
}

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
pub async fn new_form(
    headers: HeaderMap,
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    // Redirect to main page if not an HTMX request (direct URL access)
    if !is_htmx_request(&headers) {
        return redirect_to_main_page();
    }

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
    headers: HeaderMap,
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    // Redirect to main page if not an HTMX request (direct URL access)
    if !is_htmx_request(&headers) {
        return redirect_to_main_page();
    }

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
    headers: HeaderMap,
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    // Redirect to main page if not an HTMX request (direct URL access)
    if !is_htmx_request(&headers) {
        return redirect_to_main_page();
    }

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
    let item = LlmConfigService::create(&ctx.db, params).await?;

    // Return just the row to insert at the beginning of tbody
    format::render().view(
        &v,
        "admin/llm_config/row.html",
        data!({
            "item": item,
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
    let item = LlmConfigService::update(&ctx.db, id, params).await?;

    // Return just the updated row to replace the specific row
    format::render().view(
        &v,
        "admin/llm_config/row.html",
        data!({
            "item": item,
        }),
    )
}

/// Delete item
#[debug_handler]
pub async fn delete(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    LlmConfigService::delete(&ctx.db, id).await?;
    format::html("")
}

/// Activate item (deactivates all others)
#[debug_handler]
pub async fn activate(
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let _item = LlmConfigService::activate(&ctx.db, id).await?;

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

/// Fetch available models from a remote Ollama endpoint
/// Returns HTML options for the model dropdown
#[debug_handler]
pub async fn fetch_models(
    ViewEngine(v): ViewEngine<TeraView>,
    Query(params): Query<FetchModelsParams>,
) -> Result<Response> {
    debug!("llm_configs::fetch_models - params: {:?}", params);

    let endpoint_url = params.endpoint_url.unwrap_or_default();

    if endpoint_url.is_empty() {
        debug!("llm_configs::fetch_models - empty endpoint_url, returning empty options");
        return format::render().view(
            &v,
            "admin/llm_config/model_options.html",
            data!({
                "available_models": Vec::<String>::new(),
                "current_model": params.current_model,
                "error": null,
            }),
        );
    }

    // Create an OllamaBackend with the specified endpoint
    let ollama = OllamaBackend::new(endpoint_url.clone(), String::new(), 10);

    // Try to fetch models with a short timeout
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        ollama.list_models(),
    )
    .await;

    let (available_models, error) = match result {
        Ok(Ok(models)) => {
            debug!("llm_configs::fetch_models - found {} models", models.len());
            (models, None)
        }
        Ok(Err(e)) => {
            debug!("llm_configs::fetch_models - error fetching models: {:?}", e);
            (Vec::new(), Some(format!("Failed to connect: {}", e)))
        }
        Err(_) => {
            debug!("llm_configs::fetch_models - timeout fetching models");
            (Vec::new(), Some("Connection timeout".to_string()))
        }
    };

    format::render().view(
        &v,
        "admin/llm_config/model_options.html",
        data!({
            "available_models": available_models,
            "current_model": params.current_model,
            "error": error,
        }),
    )
}
