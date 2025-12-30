//! Admin Company Rules Controller
//!
//! HTMX-based CRUD for company rules.
//! Thin controller - delegates to CompanyRuleService.

use axum::http::HeaderMap;
use loco_rs::prelude::*;

use crate::middleware::cookie_auth::AuthUser;
use crate::services::admin::company_rule::{
    CompanyRuleService, CreateParams, QueryParams, UpdateParams,
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
    let response = CompanyRuleService::search(&ctx.db, &params).await?;

    // Check if this is an HTMX request
    let is_htmx = headers.get("HX-Request").is_some();
    let template = if is_htmx {
        "admin/company_rule/main.html"
    } else {
        "admin/company_rule/index.html"
    };

    format::render().view(
        &v,
        template,
        data!({
            "current_page": "company_rules",
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
    let response = CompanyRuleService::search(&ctx.db, &params).await?;

    format::render().view(
        &v,
        "admin/company_rule/list.html",
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
    let item = CompanyRuleService::find_by_id(&ctx.db, id).await?;

    format::render().view(
        &v,
        "admin/company_rule/show.html",
        data!({
            "item": item,
        }),
    )
}

/// New form
#[debug_handler]
pub async fn new_form(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(&v, "admin/company_rule/create.html", data!({}))
}

/// Edit form
#[debug_handler]
pub async fn edit_form(
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = CompanyRuleService::find_by_id(&ctx.db, id).await?;

    format::render().view(
        &v,
        "admin/company_rule/edit.html",
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
    CompanyRuleService::create(&ctx.db, params).await?;

    // Return the full list to replace #search-result
    let query_params = QueryParams::default();
    let response = CompanyRuleService::search(&ctx.db, &query_params).await?;

    format::render().view(
        &v,
        "admin/company_rule/list.html",
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
    CompanyRuleService::update(&ctx.db, id, params).await?;

    // Return the full list to replace #search-result
    let query_params = QueryParams::default();
    let response = CompanyRuleService::search(&ctx.db, &query_params).await?;

    format::render().view(
        &v,
        "admin/company_rule/list.html",
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
    CompanyRuleService::delete(&ctx.db, id).await?;
    format::html("")
}
