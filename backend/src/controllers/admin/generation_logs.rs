//! Admin Generation Logs Controller
//!
//! HTMX-based view-only for generation logs (audit trail).
//! Thin controller - delegates to GenerationLogService.

use axum::http::{header, HeaderMap, StatusCode};
use loco_rs::prelude::*;

/// Helper to check if request is from HTMX
fn is_htmx_request(headers: &HeaderMap) -> bool {
    headers.get("HX-Request").is_some()
}

/// Redirect response for non-HTMX requests to modal endpoints
fn redirect_to_main_page() -> Result<Response> {
    Ok(Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/admin/generation-logs")
        .body(axum::body::Body::empty())?
        .into_response())
}

use crate::middleware::cookie_auth::AuthUser;
use crate::services::admin::generation_log::{GenerationLogService, QueryParams};

/// Main page - renders full layout for direct access, partial for HTMX
#[debug_handler]
pub async fn main(
    auth_user: AuthUser,
    headers: HeaderMap,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let params = QueryParams::default();
    let response = GenerationLogService::search(&ctx.db, &params).await?;

    // Check if this is an HTMX request
    let is_htmx = headers.get("HX-Request").is_some();
    let template = if is_htmx {
        "admin/generation_log/main.html"
    } else {
        "admin/generation_log/index.html"
    };

    format::render().view(
        &v,
        template,
        data!({
            "current_page": "generation_logs",
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
    let response = GenerationLogService::search(&ctx.db, &params).await?;

    format::render().view(
        &v,
        "admin/generation_log/list.html",
        data!({
            "items": response.items,
            "page": response.page,
            "page_size": response.page_size,
            "total_pages": response.total_pages,
            "total_items": response.total_items,
        }),
    )
}

/// Show single log entry
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

    let item = GenerationLogService::find_by_id(&ctx.db, id).await?;

    format::render().view(
        &v,
        "admin/generation_log/show.html",
        data!({
            "item": item,
        }),
    )
}
