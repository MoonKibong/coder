//! Admin Generation Logs Controller
//!
//! HTMX-based view-only for generation logs (audit trail).
//! Thin controller - delegates to GenerationLogService.

use axum::http::HeaderMap;
use loco_rs::prelude::*;

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
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = GenerationLogService::find_by_id(&ctx.db, id).await?;

    format::render().view(
        &v,
        "admin/generation_log/show.html",
        data!({
            "item": item,
        }),
    )
}
