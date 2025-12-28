//! Common view helpers
//!
//! Shared view functions for messages, errors, etc.

use axum::http::StatusCode;
use loco_rs::prelude::*;

/// Render a success/info message partial
pub fn message(v: &impl ViewRenderer, msg: &str) -> Result<Response> {
    format::render().view(v, "partials/message.html", data!({"message": msg}))
}

/// Render an error message partial (200 status for soft errors)
pub fn error(v: &impl ViewRenderer, msg: &str) -> Result<Response> {
    format::render().view(v, "partials/error.html", data!({"message": msg}))
}

/// Render an unauthorized error (401 status)
pub fn unauthorized(v: &impl ViewRenderer, msg: &str) -> Result<Response> {
    format::render()
        .status(StatusCode::UNAUTHORIZED)
        .view(v, "partials/error.html", data!({"message": msg}))
}

/// Render a bad request error (400 status)
pub fn bad_request(v: &impl ViewRenderer, msg: &str) -> Result<Response> {
    format::render()
        .status(StatusCode::BAD_REQUEST)
        .view(v, "partials/error.html", data!({"message": msg}))
}
