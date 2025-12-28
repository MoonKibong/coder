//! Auth pages view helpers
//!
//! Render functions for authentication UI pages.

use loco_rs::prelude::*;
use ::cookie::Cookie;
use ::time::Duration;
use serde::Serialize;

use crate::controllers::home::RedirectParams;
use crate::models::_entities::users;

/// User data for frontend session storage
#[derive(Debug, Serialize)]
pub struct SessionUser {
    pub pid: String,
    pub name: String,
    pub email: String,
}

impl From<&users::Model> for SessionUser {
    fn from(user: &users::Model) -> Self {
        Self {
            pid: user.pid.to_string(),
            name: user.name.clone(),
            email: user.email.clone(),
        }
    }
}

/// Render login page
pub fn login(v: &impl ViewRenderer, params: &RedirectParams) -> Result<Response> {
    if params.redirect_to.is_some() {
        format::render().view(v, "auth/login.html", data!({"params": params}))
    } else {
        format::render().view(v, "auth/login.html", data!({}))
    }
}

/// Render register page
pub fn register(v: &impl ViewRenderer) -> Result<Response> {
    format::render().view(v, "auth/register.html", data!({}))
}

/// Render forgot password page
pub fn forgot(v: &impl ViewRenderer) -> Result<Response> {
    format::render().view(v, "auth/forgot.html", data!({}))
}

/// Render reset password page
pub fn reset(v: &impl ViewRenderer, token: Option<String>) -> Result<Response> {
    if token.is_none() {
        format::render().view(v, "auth/reset.html", data!({}))
    } else {
        format::render().view(v, "auth/reset.html", data!({"token": token}))
    }
}

/// Render verify email page
pub fn verify(v: &impl ViewRenderer, email: &str) -> Result<Response> {
    format::render()
        .header("HX-Retarget", "closest body")
        .header("HX-Reswap", "innerHTML")
        .view(v, "auth/verify.html", data!({"email": email}))
}

/// Render email verified confirmation page
pub fn verified(v: &impl ViewRenderer) -> Result<Response> {
    format::render().view(v, "auth/verified.html", data!({}))
}

/// Handle successful login - set cookie and redirect
pub fn handle_login(_v: &impl ViewRenderer, token: &str, redirect_to: &str, user: &users::Model) -> Result<Response> {
    // Note: secure(false) for development on HTTP. Set to true in production with HTTPS.
    let cookie = Cookie::build(("token", token))
        .path("/")
        .secure(false) // TODO: Set to true in production with HTTPS
        .http_only(true)
        .same_site(cookie::SameSite::Lax)
        .into();

    // Create session user data for frontend
    let session_user = SessionUser::from(user);
    let user_json = serde_json::to_string(&session_user).unwrap_or_default();

    format::render()
        .header("HX-Redirect", redirect_to)
        .header("X-User-Data", &user_json)
        .cookies(&[cookie])
        .unwrap()
        .text("Redirecting...")
}

/// Handle logout - clear cookie and redirect
pub fn handle_logout(_v: &impl ViewRenderer) -> Result<Response> {
    let cookie = Cookie::build(("token", ""))
        .path("/")
        .secure(true)
        .http_only(true)
        .max_age(Duration::ZERO)
        .same_site(cookie::SameSite::Lax)
        .into();

    format::render()
        .cookies(&[cookie])
        .unwrap()
        .redirect("/")
}
