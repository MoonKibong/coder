//! Admin Users Controller
//!
//! HTMX-based CRUD for user management.
//! Thin controller - delegates to UserService.

use axum::http::HeaderMap;
use loco_rs::prelude::*;

use crate::middleware::cookie_auth::AuthUser;
use crate::services::admin::user::{
    CreateParams, QueryParams, UpdateParams, UserService,
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
    let response = UserService::search(&ctx.db, &params).await?;

    // Check if this is an HTMX request
    let is_htmx = headers.get("HX-Request").is_some();
    let template = if is_htmx {
        "admin/user/main.html"
    } else {
        "admin/user/index.html"
    };

    format::render().view(
        &v,
        template,
        data!({
            "current_page": "users",
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
    let response = UserService::search(&ctx.db, &params).await?;

    format::render().view(
        &v,
        "admin/user/list.html",
        data!({
            "items": response.items,
            "page": response.page,
            "page_size": response.page_size,
            "total_pages": response.total_pages,
            "total_items": response.total_items,
        }),
    )
}

/// Show user details
#[debug_handler]
pub async fn show(
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = UserService::find_by_id(&ctx.db, id).await?;

    format::render().view(
        &v,
        "admin/user/show.html",
        data!({
            "item": item,
        }),
    )
}

/// New form
#[debug_handler]
pub async fn new_form(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(&v, "admin/user/create.html", data!({}))
}

/// Edit form
#[debug_handler]
pub async fn edit_form(
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = UserService::find_by_id(&ctx.db, id).await?;

    format::render().view(
        &v,
        "admin/user/edit.html",
        data!({
            "item": item,
        }),
    )
}

/// Create new user
#[debug_handler]
pub async fn create(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateParams>,
) -> Result<Response> {
    let item = UserService::create(&ctx.db, params).await?;

    format::render().view(
        &v,
        "admin/user/row.html",
        data!({
            "item": item,
        }),
    )
}

/// Update existing user
#[debug_handler]
pub async fn update(
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<UpdateParams>,
) -> Result<Response> {
    let item = UserService::update(&ctx.db, id, params).await?;

    format::render().view(
        &v,
        "admin/user/row.html",
        data!({
            "item": item,
        }),
    )
}

/// Delete user
#[debug_handler]
pub async fn delete(
    auth_user: AuthUser,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    UserService::delete(&ctx.db, id, &auth_user.pid).await?;
    format::html("")
}
