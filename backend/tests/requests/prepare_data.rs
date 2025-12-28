use axum::http::{HeaderName, HeaderValue};
use coder::models::users;
use loco_rs::{app::AppContext, TestServer};

const USER_EMAIL: &str = "test@loco.com";
const USER_PASSWORD: &str = "1234";

pub struct LoggedInUser {
    pub user: users::Model,
    pub token: String,
}

pub async fn init_user_login(request: &TestServer, ctx: &AppContext) -> LoggedInUser {
    let register_payload = serde_json::json!({
        "name": "loco",
        "email": USER_EMAIL,
        "password": USER_PASSWORD
    });

    // Creating a new user
    request
        .post("/api/auth/register")
        .json(&register_payload)
        .await;
    let user = users::Model::find_by_email(&ctx.db, USER_EMAIL)
        .await
        .unwrap();

    // Verify user email via GET endpoint with token in path
    let token = user.email_verification_token.as_ref().unwrap();
    request
        .get(&format!("/api/auth/verify/{}", token))
        .await;

    // Login and extract token from Set-Cookie header
    let response = request
        .post("/api/auth/login")
        .json(&serde_json::json!({
            "email": USER_EMAIL,
            "password": USER_PASSWORD
        }))
        .await;

    // Extract token from Set-Cookie header
    let set_cookie = response
        .headers()
        .get("set-cookie")
        .expect("Expected Set-Cookie header")
        .to_str()
        .unwrap();

    // Parse "token=<value>; ..." format
    let token = set_cookie
        .split(';')
        .next()
        .unwrap()
        .trim_start_matches("token=")
        .to_string();

    LoggedInUser {
        user: users::Model::find_by_email(&ctx.db, USER_EMAIL)
            .await
            .unwrap(),
        token,
    }
}

pub fn auth_header(token: &str) -> (HeaderName, HeaderValue) {
    let auth_header_value = HeaderValue::from_str(&format!("Bearer {}", &token)).unwrap();

    (HeaderName::from_static("authorization"), auth_header_value)
}
