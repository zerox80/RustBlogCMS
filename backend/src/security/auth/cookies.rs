use super::*;

/// Builds a secure authentication cookie containing the JWT token.
///
/// Creates an HttpOnly cookie with appropriate security flags for
/// storing the JWT token in the client's browser.
///
/// # Arguments
/// * `token` - The JWT token to store in the cookie
///
/// # Returns
/// A Cookie configured for secure authentication token storage
///
/// # Security Features
/// - HttpOnly: Prevents JavaScript access (XSS protection)
/// - SameSite=Lax: CSRF protection while allowing navigation
/// - Secure flag: HTTPS-only (when AUTH_COOKIE_SECURE is not false)
/// - 24-hour expiration: Matches JWT expiration
/// - Path=/: Available to all routes
pub fn build_auth_cookie(token: &str) -> Cookie<'static> {
    // Build cookie with security flags
    let mut builder = Cookie::build((AUTH_COOKIE_NAME, token.to_owned()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(TimeDuration::seconds(AUTH_COOKIE_TTL_SECONDS));

    // Add Secure flag in production (HTTPS only)
    if cookies_should_be_secure() {
        builder = builder.secure(true);
    }

    builder.build()
}

/// Builds a cookie that removes the authentication cookie.
///
/// Creates a cookie with expired timestamp to instruct the browser
/// to delete the authentication cookie (used for logout).
///
/// # Returns
/// A Cookie configured to remove the authentication cookie
///
/// # Mechanism
/// - Empty value
/// - Expiration set to Unix epoch (Jan 1, 1970)
/// - Max-age of 0
/// - Same path and security flags as the auth cookie
pub fn build_cookie_removal() -> Cookie<'static> {
    // Build cookie with expiration in the past to trigger removal
    let mut builder = Cookie::build((AUTH_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .expires(OffsetDateTime::UNIX_EPOCH)
        .max_age(TimeDuration::seconds(0));

    // Match security settings of auth cookie
    if cookies_should_be_secure() {
        builder = builder.secure(true);
    }

    builder.build()
}

/// AXUM extractor implementation for Claims.
///
/// This allows Claims to be used as a function parameter in route handlers,
/// automatically extracting and validating the JWT token from the request.
///
/// # Extraction order
/// 1. Check if claims already in request extensions (from middleware)
/// 2. Extract token from Authorization header or cookie
/// 3. Validate token and decode claims
///
/// # Errors
/// Returns 401 Unauthorized if:
/// - No token found in headers or cookies
/// - Token is invalid or expired
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
    DbPool: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Step 1: Check cache. If auth middleware already ran, claims are in extensions.
        if let Some(claims) = parts.extensions.get::<Claims>() {
            return Ok(claims.clone());
        }

        // Step 2: Extract raw token from standard locations (Header/Cookie).
        let token = extract_token(&parts.headers).ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                "Missing authentication token".to_string(),
            )
        })?;

        // Step 3: Verify cryptographic signature and expiration.
        let claims = verify_jwt(&token)
            .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Invalid token: {}", e)))?;

        // Step 4: Check if token has been revoked (Logout/Blacklist).
        let pool = DbPool::from_ref(state);
        let is_blacklisted =
            crate::repositories::token_blacklist::is_token_blacklisted(&pool, &token)
                .await
                .map_err(|e| {
                    tracing::error!("Database error checking token blacklist: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Internal server error".to_string(),
                    )
                })?;

        if is_blacklisted {
            return Err((
                StatusCode::UNAUTHORIZED,
                "Token has been revoked".to_string(),
            ));
        }

        // Cache result for downstream handlers
        parts.extensions.insert(claims.clone());
        Ok(claims)
    }
}

/// Appends an authentication cookie to the response headers.
///
/// # Arguments
/// * `headers` - Mutable reference to the response HeaderMap
/// * `cookie` - The cookie to append
///
/// # Error Handling
/// Logs an error if the cookie cannot be serialized (should never happen)
pub fn append_auth_cookie(headers: &mut HeaderMap, cookie: Cookie<'static>) {
    // Convert cookie to header value
    if let Ok(value) = HeaderValue::from_str(&cookie.to_string()) {
        headers.append(SET_COOKIE, value);
    } else {
        // This should never happen with valid cookie values
        tracing::error!("Failed to serialize auth cookie for Set-Cookie header");
    }
}
