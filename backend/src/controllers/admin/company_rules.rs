//! Admin Company Rules Controller
//!
//! HTMX-based CRUD for company rules.
//! Thin controller - delegates to CompanyRuleService.

use axum::http::{header, HeaderMap, StatusCode};
use loco_rs::prelude::*;
use tracing::{debug, error};

/// Helper to check if request is from HTMX
fn is_htmx_request(headers: &HeaderMap) -> bool {
    headers.get("HX-Request").is_some()
}

/// Redirect response for non-HTMX requests to modal endpoints
fn redirect_to_main_page() -> Result<Response> {
    Ok(Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/admin/company-rules")
        .body(axum::body::Body::empty())?
        .into_response())
}

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
    println!(">>> company_rules::main - ENTERING (println)");
    debug!("company_rules::main - entering");
    let params = QueryParams::default();

    let response = match CompanyRuleService::search(&ctx.db, &params).await {
        Ok(r) => {
            debug!("company_rules::main - search returned {} items", r.items.len());
            r
        }
        Err(e) => {
            error!("company_rules::main - search failed: {:?}", e);
            return Err(e);
        }
    };

    // Check if this is an HTMX request
    let is_htmx = headers.get("HX-Request").is_some();
    let template = if is_htmx {
        "admin/company_rule/main.html"
    } else {
        "admin/company_rule/index.html"
    };
    debug!("company_rules::main - rendering template: {}", template);

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
    debug!("company_rules::list - params: {:?}", params);

    let response = match CompanyRuleService::search(&ctx.db, &params).await {
        Ok(r) => {
            debug!("company_rules::list - search returned {} items", r.items.len());
            r
        }
        Err(e) => {
            error!("company_rules::list - search failed: {:?}", e);
            return Err(e);
        }
    };

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
    headers: HeaderMap,
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    // Redirect to main page if not an HTMX request (direct URL access)
    if !is_htmx_request(&headers) {
        return redirect_to_main_page();
    }

    debug!("company_rules::show - id: {}", id);

    let item = match CompanyRuleService::find_by_id(&ctx.db, id).await {
        Ok(i) => {
            debug!("company_rules::show - found item: {:?}", i.name);
            i
        }
        Err(e) => {
            error!("company_rules::show - find_by_id failed: {:?}", e);
            return Err(e);
        }
    };

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
pub async fn new_form(
    headers: HeaderMap,
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    // Redirect to main page if not an HTMX request (direct URL access)
    if !is_htmx_request(&headers) {
        return redirect_to_main_page();
    }

    debug!("company_rules::new_form - rendering create form");
    format::render().view(&v, "admin/company_rule/create.html", data!({}))
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

    debug!("company_rules::edit_form - id: {}", id);

    let item = match CompanyRuleService::find_by_id(&ctx.db, id).await {
        Ok(i) => {
            debug!("company_rules::edit_form - found item: {:?}", i.name);
            i
        }
        Err(e) => {
            error!("company_rules::edit_form - find_by_id failed: {:?}", e);
            return Err(e);
        }
    };

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
    debug!("company_rules::create - params: {:?}", params);

    let item = match CompanyRuleService::create(&ctx.db, params).await {
        Ok(i) => {
            debug!("company_rules::create - created item id: {}", i.id);
            i
        }
        Err(e) => {
            error!("company_rules::create - failed: {:?}", e);
            return Err(e);
        }
    };

    // Return just the row to insert at the beginning of tbody
    format::render().view(
        &v,
        "admin/company_rule/row.html",
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
    debug!("company_rules::update - id: {}, params: {:?}", id, params);

    let item = match CompanyRuleService::update(&ctx.db, id, params).await {
        Ok(i) => {
            debug!("company_rules::update - updated item id: {}", i.id);
            i
        }
        Err(e) => {
            error!("company_rules::update - failed: {:?}", e);
            return Err(e);
        }
    };

    // Return just the updated row to replace the specific row
    format::render().view(
        &v,
        "admin/company_rule/row.html",
        data!({
            "item": item,
        }),
    )
}

/// Delete item
#[debug_handler]
pub async fn delete(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    debug!("company_rules::delete - id: {}", id);

    match CompanyRuleService::delete(&ctx.db, id).await {
        Ok(_) => debug!("company_rules::delete - deleted id: {}", id),
        Err(e) => {
            error!("company_rules::delete - failed: {:?}", e);
            return Err(e);
        }
    };

    format::html("")
}
