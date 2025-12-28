//! Admin Prompt Templates Controller
//!
//! HTMX-based CRUD for prompt templates.
//! Thin controller - delegates to PromptTemplateService.

use loco_rs::prelude::*;

use crate::middleware::cookie_auth::AuthUser;
use crate::services::admin::prompt_template::{
    CreateParams, PromptTemplateService, QueryParams, UpdateParams,
};

/// Main page - renders full layout with list
#[debug_handler]
pub async fn main(
    _auth_user: AuthUser,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let params = QueryParams::default();
    let response = PromptTemplateService::search(&ctx.db, &params).await?;

    format::render().view(
        &v,
        "admin/prompt_template/main.html",
        data!({
            "current_page": "prompt_templates",
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
    let response = PromptTemplateService::search(&ctx.db, &params).await?;

    format::render().view(
        &v,
        "admin/prompt_template/list.html",
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
    let item = PromptTemplateService::find_by_id(&ctx.db, id).await?;

    format::render().view(
        &v,
        "admin/prompt_template/show.html",
        data!({
            "item": item,
        }),
    )
}

/// New form
#[debug_handler]
pub async fn new_form(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(&v, "admin/prompt_template/create.html", data!({}))
}

/// Edit form
#[debug_handler]
pub async fn edit_form(
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = PromptTemplateService::find_by_id(&ctx.db, id).await?;

    format::render().view(
        &v,
        "admin/prompt_template/edit.html",
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
    let item = PromptTemplateService::create(&ctx.db, params).await?;

    format::render().view(
        &v,
        "admin/prompt_template/row.html",
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
    let item = PromptTemplateService::update(&ctx.db, id, params).await?;

    format::render().view(
        &v,
        "admin/prompt_template/row.html",
        data!({
            "item": item,
        }),
    )
}

/// Delete item
#[debug_handler]
pub async fn delete(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    PromptTemplateService::delete(&ctx.db, id).await?;
    format::html("")
}
