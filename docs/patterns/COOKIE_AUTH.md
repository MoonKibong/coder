# Cookie-Based Authentication Pattern

## Overview

Admin pages use cookie-based JWT authentication that redirects unauthenticated users to the login page instead of returning 401 errors.

## Implementation

### AuthUser Extractor

```rust
// src/middleware/cookie_auth.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub pid: String,
    pub name: String,
    pub email: String,
}

pub struct AuthRedirect {
    pub redirect_to: String,
}

impl IntoResponse for AuthRedirect {
    fn into_response(self) -> Response {
        let login_url = format!("/login?redirect_to={}", urlencoding::encode(&self.redirect_to));
        // HX-Redirect for HTMX, standard redirect for full page loads
        (
            [
                ("HX-Redirect", login_url.as_str()),
                ("HX-Reswap", "none"),
            ],
            Redirect::temporary(&login_url),
        ).into_response()
    }
}

impl FromRequestParts<AppContext> for AuthUser {
    type Rejection = AuthRedirect;

    fn from_request_parts(
        parts: &mut Parts,
        state: &AppContext,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        // Extract token from cookie
        // Validate JWT
        // Return AuthUser or AuthRedirect
    }
}
```

### Usage in Controllers

```rust
#[debug_handler]
pub async fn main(
    _auth_user: AuthUser,  // Automatically redirects if not authenticated
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    // Only reaches here if authenticated
}
```

### Login Flow

1. User submits login form
2. Server validates credentials, generates JWT
3. Server sets HTTP-only cookie with token
4. Server responds with `HX-Redirect` header and `X-User-Data` header
5. Frontend stores user data in sessionStorage
6. Frontend redirects to requested page

```rust
// src/views/auth_pages.rs

pub fn handle_login(_v: &impl ViewRenderer, token: &str, redirect_to: &str, user: &users::Model) -> Result<Response> {
    let cookie = Cookie::build(("token", token))
        .path("/")
        .secure(false)  // TODO: Set to true in production with HTTPS
        .http_only(true)
        .same_site(cookie::SameSite::Lax)
        .into();

    let session_user = SessionUser::from(user);
    let user_json = serde_json::to_string(&session_user).unwrap_or_default();

    format::render()
        .header("HX-Redirect", redirect_to)
        .header("X-User-Data", &user_json)
        .cookies(&[cookie])
        .unwrap()
        .text("Redirecting...")
}
```

### Frontend Session Storage

```javascript
// In login.html
const userData = evt.detail.xhr.getResponseHeader('X-User-Data');
if (userData) {
    sessionStorage.setItem('coder-session-auth', userData);
}
```

## HTMX Considerations

### HX-Redirect Header

HTMX doesn't follow standard 302 redirects for partial updates. The `AuthRedirect` response includes:

- `HX-Redirect`: Tells HTMX to do a full page redirect
- `HX-Reswap`: Prevents HTMX from trying to swap content
- Standard `Redirect::temporary`: Fallback for non-HTMX requests

### 401 Response Handling

```javascript
// In admin/layout.html
document.body.addEventListener('htmx:responseError', function(evt) {
    if (evt.detail.xhr.status === 401) {
        sessionStorage.removeItem('coder-session-auth');
        window.location.href = '/login?redirect_to=' + encodeURIComponent(window.location.pathname);
    }
});
```

## Cookie Settings

| Setting | Development | Production |
|---------|-------------|------------|
| `secure` | `false` | `true` (HTTPS required) |
| `http_only` | `true` | `true` |
| `same_site` | `Lax` | `Lax` |
| `path` | `/` | `/` |

## Logout Flow

1. User clicks logout
2. Clear sessionStorage
3. Server clears cookie (max_age = 0)
4. Redirect to home page

```rust
pub fn handle_logout(_v: &impl ViewRenderer) -> Result<Response> {
    let cookie = Cookie::build(("token", ""))
        .path("/")
        .secure(true)
        .http_only(true)
        .max_age(Duration::ZERO)  // Expires immediately
        .same_site(cookie::SameSite::Lax)
        .into();

    format::render()
        .cookies(&[cookie])
        .unwrap()
        .redirect("/")
}
```

## Security Notes

1. JWT token is stored in HTTP-only cookie (not accessible to JavaScript)
2. User display data is stored in sessionStorage (accessible but not sensitive)
3. Cookie is Lax same-site to prevent CSRF
4. Production must use HTTPS with secure cookies
