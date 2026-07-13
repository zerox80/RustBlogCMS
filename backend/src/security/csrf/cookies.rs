use super::*;

/// Appends a CSRF token cookie to the response headers.
///
/// # Arguments
/// * `headers` - Mutable reference to the response HeaderMap
/// * `token` - The CSRF token to include in the cookie
///
/// # Error Handling
/// Logs an error if cookie serialization fails (should never happen)
pub fn append_csrf_cookie(headers: &mut HeaderMap, token: &str) {
    // Build cookie with security flags
    let cookie = build_csrf_cookie(token);

    // Append to Set-Cookie header
    if let Ok(value) = HeaderValue::from_str(&cookie.to_string()) {
        headers.append(SET_COOKIE, value);
    } else {
        tracing::error!("Failed to serialize CSRF cookie");
    }
}

/// Appends a cookie that removes the CSRF cookie (for logout).
///
/// # Arguments
/// * `headers` - Mutable reference to the response HeaderMap
///
/// # Error Handling
/// Logs an error if cookie serialization fails (should never happen)
pub fn append_csrf_removal(headers: &mut HeaderMap) {
    // Build removal cookie (expired)
    let cookie = build_csrf_removal();

    // Append to Set-Cookie header
    if let Ok(value) = HeaderValue::from_str(&cookie.to_string()) {
        headers.append(SET_COOKIE, value);
    } else {
        tracing::error!("Failed to serialize CSRF removal cookie");
    }
}

/// Builds a CSRF cookie with appropriate security flags.
///
/// # Arguments
/// * `token` - The CSRF token to store in the cookie
///
/// # Returns
/// A Cookie configured for CSRF protection
///
/// # Security Flags
/// - SameSite=Strict: Prevents cross-site cookie sending (strict CSRF protection)
/// - HttpOnly=false: Allows JavaScript read access (needed for header submission)
/// - Secure: HTTPS-only (when AUTH_COOKIE_SECURE is not false)
/// - Path=/: Available to all routes
/// - Max-Age: 6 hours (matches token expiration)
pub(super) fn build_csrf_cookie(token: &str) -> Cookie<'static> {
    // Build cookie with security settings
    let mut builder = Cookie::build((CSRF_COOKIE_NAME, token.to_owned()))
        .path("/")
        .same_site(SameSite::Strict)
        .max_age(TimeDuration::seconds(CSRF_TOKEN_TTL_SECONDS))
        .http_only(false); // Must be false for JavaScript to read and submit in header

    // Add Secure flag in production (HTTPS only)
    if auth::cookies_should_be_secure() {
        builder = builder.secure(true);
    }

    builder.build()
}

/// Builds a cookie that removes the CSRF cookie.
///
/// # Returns
/// A Cookie configured to remove the CSRF cookie
///
/// # Mechanism
/// - Empty value
/// - Expiration set to Unix epoch (Jan 1, 1970)
/// - Max-age of 0
/// - Same path and security flags as the CSRF cookie
pub(super) fn build_csrf_removal() -> Cookie<'static> {
    // Build cookie with expiration in the past to trigger removal
    let mut builder = Cookie::build((CSRF_COOKIE_NAME, ""))
        .path("/")
        .same_site(SameSite::Strict)
        .expires(OffsetDateTime::UNIX_EPOCH)
        .max_age(TimeDuration::seconds(0))
        .http_only(false);

    // Match security settings of CSRF cookie
    if auth::cookies_should_be_secure() {
        builder = builder.secure(true);
    }

    builder.build()
}
