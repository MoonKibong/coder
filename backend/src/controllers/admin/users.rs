//! Admin Users Controller
//!
//! HTMX-based CRUD for user management.
//! Thin controller - delegates to UserService.

use axum::http::{header, HeaderMap, StatusCode};
use loco_rs::prelude::*;

use crate::middleware::cookie_auth::AuthUser;
use crate::services::admin::user::{
    CreateParams, QueryParams, UpdateParams, UserService,
};

/// Helper to check if request is from HTMX
fn is_htmx_request(headers: &HeaderMap) -> bool {
    headers.get("HX-Request").is_some()
}

/// Redirect response for non-HTMX requests to modal endpoints
fn redirect_to_users_page() -> Result<Response> {
    Ok(Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/admin/users")
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
    headers: HeaderMap,
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    // Redirect to main page if not an HTMX request (direct URL access)
    if !is_htmx_request(&headers) {
        return redirect_to_users_page();
    }

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
pub async fn new_form(
    headers: HeaderMap,
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    // Redirect to main page if not an HTMX request (direct URL access)
    if !is_htmx_request(&headers) {
        return redirect_to_users_page();
    }

    format::render().view(&v, "admin/user/create.html", data!({}))
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
        return redirect_to_users_page();
    }

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
    UserService::create(&ctx.db, params).await?;

    // Return the full list to replace #search-result
    let query_params = QueryParams::default();
    let response = UserService::search(&ctx.db, &query_params).await?;

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

/// Update existing user
#[debug_handler]
pub async fn update(
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<UpdateParams>,
) -> Result<Response> {
    UserService::update(&ctx.db, id, params).await?;

    // Return the full list to replace #search-result
    let query_params = QueryParams::default();
    let response = UserService::search(&ctx.db, &query_params).await?;

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
