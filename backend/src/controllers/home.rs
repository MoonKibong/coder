//! Home Controller
//!
//! Landing page and auth pages for xFrame5 Code Generator

use axum::debug_handler;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::_entities::users;
use crate::views::auth_pages;

/// Query parameters for redirect
#[derive(Debug, Deserialize, Serialize)]
pub struct RedirectParams {
    pub redirect_to: Option<String>,
}

/// Landing page
#[debug_handler]
pub async fn index(
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    format::render()
        .view(&v, "home/index.html", data!({}))
}

/// Login page
///
/// GET /login
#[debug_handler]
pub async fn login(
    ViewEngine(v): ViewEngine<TeraView>,
    Query(params): Query<RedirectParams>,
) -> Result<Response> {
    auth_pages::login(&v, &params)
}

/// Register page
///
/// GET /register
#[debug_handler]
pub async fn register(
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    auth_pages::register(&v)
}

/// Forgot password page
///
/// GET /forgot
#[debug_handler]
pub async fn forgot(
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    auth_pages::forgot(&v)
}

/// Reset password page (accessed via email link)
///
/// GET /reset/{token}
#[debug_handler]
pub async fn reset(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    Path(token): Path<String>,
) -> Result<Response> {
    // Verify the token exists
    let token_valid = users::Model::find_by_reset_token(&ctx.db, &token).await.is_ok();

    if token_valid {
        auth_pages::reset(&v, Some(token))
    } else {
        auth_pages::reset(&v, None)
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/")
        .add("/", get(index))
        .add("login", get(login))
        .add("register", get(register))
        .add("forgot", get(forgot))
        .add("reset/{token}", get(reset))
}
