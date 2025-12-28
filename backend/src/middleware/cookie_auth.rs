//! Cookie-based JWT Authentication
//!
//! Extracts JWT token from cookies and validates it.
//! For page routes, redirects to login on failure instead of returning 401.

use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
    response::{IntoResponse, Redirect, Response},
};
use loco_rs::{app::AppContext, auth};
use serde::{Deserialize, Serialize};
use std::future::Future;

use crate::models::_entities::users;

/// Authenticated user extracted from cookie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub pid: String,
    pub name: String,
    pub email: String,
}

/// Error that redirects to login page
pub struct AuthRedirect {
    pub redirect_to: String,
}

impl IntoResponse for AuthRedirect {
    fn into_response(self) -> Response {
        let login_url = format!("/login?redirect_to={}", urlencoding::encode(&self.redirect_to));
        // Use HX-Redirect header for HTMX requests, or standard redirect for full page loads
        // HTMX doesn't follow 302 redirects properly for partial updates
        (
            [
                ("HX-Redirect", login_url.as_str()),
                ("HX-Reswap", "none"),
            ],
            Redirect::temporary(&login_url),
        ).into_response()
    }
}

/// Extract JWT token from cookie header
fn extract_token_from_cookies(parts: &Parts) -> Option<String> {
    let cookie_header = parts.headers.get(header::COOKIE)?;
    let cookie_str = cookie_header.to_str().ok()?;

    // Parse cookies and find "token"
    for cookie in cookie_str.split(';') {
        let cookie = cookie.trim();
        if let Some(value) = cookie.strip_prefix("token=") {
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Cookie-based auth extractor for page routes
///
/// Redirects to login page if user is not authenticated.
/// Use this for admin pages that require authentication.
impl FromRequestParts<AppContext> for AuthUser {
    type Rejection = AuthRedirect;

    fn from_request_parts(
        parts: &mut Parts,
        state: &AppContext,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        // Get the request URI for redirect
        let redirect_to = parts.uri.path().to_string();

        // Extract token from cookie (sync operation)
        let token_result = extract_token_from_cookies(parts).ok_or_else(|| AuthRedirect {
            redirect_to: redirect_to.clone(),
        });

        // Clone state data needed for async block
        let db = state.db.clone();
        let config = state.config.clone();

        async move {
            let token = token_result?;

            // Get JWT config
            let jwt_config = config.get_jwt_config().map_err(|_| AuthRedirect {
                redirect_to: redirect_to.clone(),
            })?;

            // Validate token
            let claims = auth::jwt::JWT::new(&jwt_config.secret)
                .validate(&token)
                .map_err(|_| AuthRedirect {
                    redirect_to: redirect_to.clone(),
                })?;

            // Find user by PID
            let user = users::Model::find_by_pid(&db, &claims.claims.pid)
                .await
                .map_err(|_| AuthRedirect {
                    redirect_to: redirect_to.clone(),
                })?;

            Ok(AuthUser {
                pid: user.pid.to_string(),
                name: user.name,
                email: user.email,
            })
        }
    }
}
