//! Auth Controller
//!
//! Handles authentication API endpoints. Returns HTML responses for HTMX forms.

use axum::debug_handler;
use loco_rs::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

use crate::{
    mailers::auth::AuthMailer,
    models::{
        _entities::users,
        users::{LoginParams, RegisterParams},
    },
    views::{auth::CurrentResponse, auth_pages, common},
};

pub static EMAIL_DOMAIN_RE: OnceLock<Regex> = OnceLock::new();

fn get_allow_email_domain_re() -> &'static Regex {
    EMAIL_DOMAIN_RE.get_or_init(|| {
        Regex::new(r"@example\.com$|@gmail\.com$").expect("Failed to compile regex")
    })
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ForgotParams {
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginQueryParams {
    pub redirect_to: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResetParams {
    pub token: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MagicLinkParams {
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResendVerificationParams {
    pub email: String,
}

/// Register function creates a new user with the given parameters and sends a
/// welcome email to the user
#[debug_handler]
async fn register(
    State(ctx): State<AppContext>,
    ViewEngine(v): ViewEngine<TeraView>,
    Json(params): Json<RegisterParams>,
) -> Result<Response> {
    let res = users::Model::create_with_password(&ctx.db, &params).await;

    let user = match res {
        Ok(user) => user,
        Err(err) => {
            tracing::info!(
                message = err.to_string(),
                user_email = &params.email,
                "could not register user",
            );
            return common::error(&v, "Could not register user. Email may already be in use.");
        }
    };

    let user = user
        .into_active_model()
        .set_email_verification_sent(&ctx.db)
        .await?;

    AuthMailer::send_welcome(&ctx, &user).await?;

    auth_pages::verify(&v, &user.email)
}

/// Verify register user. if the user not verified his email, he can't login to
/// the system.
#[debug_handler]
async fn verify(
    State(ctx): State<AppContext>,
    ViewEngine(v): ViewEngine<TeraView>,
    Path(token): Path<String>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_verification_token(&ctx.db, &token).await else {
        return common::unauthorized(&v, "Invalid verification token");
    };

    if user.email_verified_at.is_some() {
        tracing::info!(pid = user.pid.to_string(), "user already verified");
    } else {
        let active_model = user.into_active_model();
        let user = active_model.verified(&ctx.db).await?;
        tracing::info!(pid = user.pid.to_string(), "user verified");
    }

    auth_pages::verified(&v)
}

/// In case the user forgot his password this endpoints generate a forgot token
/// and send email to the user. In case the email not found in our DB, we are
/// returning a valid request for security reasons (not exposing users DB list).
#[debug_handler]
async fn forgot(
    State(ctx): State<AppContext>,
    ViewEngine(v): ViewEngine<TeraView>,
    Json(params): Json<ForgotParams>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_email(&ctx.db, &params.email).await else {
        // we don't want to expose our users email. if the email is invalid we still
        // returning success to the caller
        return common::message(&v, "If an account exists with this email, you will receive a password reset link.");
    };

    let user = user
        .into_active_model()
        .set_forgot_password_sent(&ctx.db)
        .await?;

    AuthMailer::forgot_password(&ctx, &user).await?;

    common::message(&v, "Password reset email sent. Please check your inbox.")
}

/// reset user password by the given parameters
#[debug_handler]
async fn reset(
    State(ctx): State<AppContext>,
    ViewEngine(v): ViewEngine<TeraView>,
    Json(params): Json<ResetParams>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_reset_token(&ctx.db, &params.token).await else {
        tracing::info!("reset token not found");
        return common::error(&v, "Invalid or expired reset token. Please request a new link.");
    };

    user.into_active_model()
        .reset_password(&ctx.db, &params.password)
        .await?;

    common::message(&v, "Password reset successfully! You can now log in with your new password.")
}

/// Creates a user login and returns a token
#[debug_handler]
async fn login(
    State(ctx): State<AppContext>,
    ViewEngine(v): ViewEngine<TeraView>,
    Query(query): Query<LoginQueryParams>,
    Json(params): Json<LoginParams>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_email(&ctx.db, &params.email).await else {
        tracing::debug!(
            email = params.email,
            "login attempt with non-existent email"
        );
        return common::unauthorized(&v, "Invalid email or password");
    };

    // Check if email is verified
    if user.email_verified_at.is_none() {
        return common::unauthorized(&v, "Please verify your email before logging in. Check your inbox for the verification link.");
    }

    let valid = user.verify_password(&params.password);

    if !valid {
        return common::unauthorized(&v, "Incorrect password. Please try again or reset your password.");
    }

    let jwt_secret = ctx.config.get_jwt_config()?;

    let token = user
        .generate_jwt(&jwt_secret.secret, jwt_secret.expiration)
        .or_else(|_| unauthorized("unauthorized!"))?;

    // Determine redirect URL
    let redirect_to = query.redirect_to.unwrap_or_else(|| "/admin".to_string());

    auth_pages::handle_login(&v, &token, &redirect_to, &user)
}

/// Returns the current user information (JSON for API calls)
#[debug_handler]
async fn current(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    format::json(CurrentResponse::new(&user))
}

/// Handle logout
#[debug_handler]
async fn logout(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    auth_pages::handle_logout(&v)
}

/// Magic link authentication provides a secure and passwordless way to log in to the application.
async fn magic_link(
    State(ctx): State<AppContext>,
    ViewEngine(v): ViewEngine<TeraView>,
    Json(params): Json<MagicLinkParams>,
) -> Result<Response> {
    let email_regex = get_allow_email_domain_re();
    if !email_regex.is_match(&params.email) {
        tracing::debug!(
            email = params.email,
            "The provided email is invalid or does not match the allowed domains"
        );
        return common::bad_request(&v, "Invalid email domain");
    }

    let Ok(user) = users::Model::find_by_email(&ctx.db, &params.email).await else {
        // we don't want to expose our users email. if the email is invalid we still
        // returning success to the caller
        tracing::debug!(email = params.email, "user not found by email");
        return common::message(&v, "Magic link sent to your email");
    };

    let user = user.into_active_model().create_magic_link(&ctx.db).await?;
    AuthMailer::send_magic_link(&ctx, &user).await?;

    common::message(&v, "Magic link sent to your email")
}

/// Verifies a magic link token and authenticates the user.
async fn magic_link_verify(
    Path(token): Path<String>,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_magic_token(&ctx.db, &token).await else {
        return common::unauthorized(&v, "Invalid or expired magic link");
    };

    let user = user.into_active_model().clear_magic_link(&ctx.db).await?;

    let jwt_secret = ctx.config.get_jwt_config()?;

    let token = user
        .generate_jwt(&jwt_secret.secret, jwt_secret.expiration)
        .or_else(|_| unauthorized("unauthorized!"))?;

    auth_pages::handle_login(&v, &token, "/admin", &user)
}

#[debug_handler]
async fn resend_verification_email(
    State(ctx): State<AppContext>,
    ViewEngine(v): ViewEngine<TeraView>,
    Json(params): Json<ResendVerificationParams>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_email(&ctx.db, &params.email).await else {
        tracing::info!(
            email = params.email,
            "User not found for resend verification"
        );
        return common::message(&v, "Verification email sent");
    };

    if user.email_verified_at.is_some() {
        tracing::info!(
            pid = user.pid.to_string(),
            "User already verified, skipping resend"
        );
        return common::message(&v, "Your email is already verified. You can log in.");
    }

    let user = user
        .into_active_model()
        .set_email_verification_sent(&ctx.db)
        .await?;

    AuthMailer::send_welcome(&ctx, &user).await?;
    tracing::info!(pid = user.pid.to_string(), "Verification email re-sent");

    common::message(&v, "Verification email sent! Please check your inbox.")
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/auth/")
        .add("register", post(register))
        .add("verify/{token}", get(verify))
        .add("login", post(login))
        .add("logout", get(logout))
        .add("forgot", post(forgot))
        .add("reset", post(reset))
        .add("current", get(current))
        .add("magic-link", post(magic_link))
        .add("magic-link/{token}", get(magic_link_verify))
        .add("resend-verification-mail", post(resend_verification_email))
}
