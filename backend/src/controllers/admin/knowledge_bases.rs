//! Admin Knowledge Bases Controller
//!
//! HTMX-based CRUD for knowledge base entries.
//! Thin controller - delegates to AdminKnowledgeBaseService.

use loco_rs::prelude::*;

use crate::middleware::cookie_auth::AuthUser;
use crate::services::admin::{
    AdminKnowledgeBaseService,
};
use crate::services::admin::knowledge_base::{
    CreateParams, QueryParams, UpdateParams,
};

/// Main page - renders full layout with list
#[debug_handler]
pub async fn main(
    _auth_user: AuthUser,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let params = QueryParams::default();
    let response = AdminKnowledgeBaseService::search(&ctx.db, &params).await?;

    format::render().view(
        &v,
        "admin/knowledge_base/main.html",
        data!({
            "current_page": "knowledge_bases",
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
    let response = AdminKnowledgeBaseService::search(&ctx.db, &params).await?;

    format::render().view(
        &v,
        "admin/knowledge_base/list.html",
        data!({
            "items": response.items,
            "page": response.page,
            "page_size": response.page_size,
            "total_pages": response.total_pages,
            "total_items": response.total_items,
        }),
    )
}

/// Show single item
#[debug_handler]
pub async fn show(
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = AdminKnowledgeBaseService::find_by_id(&ctx.db, id).await?;

    format::render().view(
        &v,
        "admin/knowledge_base/show.html",
        data!({
            "item": item,
        }),
    )
}

/// New form
#[debug_handler]
pub async fn new_form(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(&v, "admin/knowledge_base/create.html", data!({}))
}

/// Edit form
#[debug_handler]
pub async fn edit_form(
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = AdminKnowledgeBaseService::find_by_id(&ctx.db, id).await?;

    format::render().view(
        &v,
        "admin/knowledge_base/edit.html",
        data!({
            "item": item,
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
    AdminKnowledgeBaseService::create(&ctx.db, params).await?;

    // Return updated list
    let query_params = QueryParams::default();
    let response = AdminKnowledgeBaseService::search(&ctx.db, &query_params).await?;

    format::render().view(
        &v,
        "admin/knowledge_base/list.html",
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
    AdminKnowledgeBaseService::update(&ctx.db, id, params).await?;

    // Return updated list
    let query_params = QueryParams::default();
    let response = AdminKnowledgeBaseService::search(&ctx.db, &query_params).await?;

    format::render().view(
        &v,
        "admin/knowledge_base/list.html",
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
pub async fn delete(
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    AdminKnowledgeBaseService::delete(&ctx.db, id).await?;

    // Return updated list
    let query_params = QueryParams::default();
    let response = AdminKnowledgeBaseService::search(&ctx.db, &query_params).await?;

    format::render().view(
        &v,
        "admin/knowledge_base/list.html",
        data!({
            "items": response.items,
            "page": response.page,
            "page_size": response.page_size,
            "total_pages": response.total_pages,
            "total_items": response.total_items,
        }),
    )
}
