//! Cross-Site Request Forgery (CSRF) Protection Module
//!
//! This module provides CSRF protection for state-changing HTTP operations.
//! It implements a double-submit cookie pattern with additional security features.
//!
//! # Security Features
//! - HMAC-SHA256 signed tokens (prevents forgery)
//! - Per-user token binding (prevents token theft across accounts)
//! - Time-based expiration (6-hour TTL)
//! - Random nonce for uniqueness
//! - Version support for token format evolution
//! - Constant-time signature comparison (prevents timing attacks)
//! - Double-submit cookie pattern (cookie + header validation)
//!
//! # Token Format
//! `v1|base64url(username)|expiry|nonce|base64url(signature)`
//!
//! # Usage
//! Tokens are automatically validated by the CsrfGuard extractor for
//! state-changing HTTP methods (POST, PUT, DELETE, PATCH).
//!
//! ## Initialization
//! ```rust,no_run
//! use rust_blog_backend::security::csrf;
//! csrf::init_csrf_secret().expect("Failed to initialize CSRF secret");
//! ```
//!
//! ## Protection
//! ```rust,no_run
//! use axum::{Router, routing::post, middleware};
//! use rust_blog_backend::security::csrf::CsrfGuard;
//! async fn handler() {}
//!
//! let app = Router::new()
//!     .route("/api/resource", post(handler))
//!     .route_layer(middleware::from_extractor::<CsrfGuard>());
//! ```

use axum::{
    extract::FromRequestParts,
    http::{
        header::{HeaderName, SET_COOKIE},
        request::Parts,
        HeaderMap, HeaderValue, Method, StatusCode,
    },
    Json,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use base64ct::{Base64UrlUnpadded, Encoding};
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::{collections::HashSet, env, sync::OnceLock};
use time::{Duration as TimeDuration, OffsetDateTime};
use uuid::Uuid;

use crate::{security::auth, models::ErrorResponse};

/// HMAC-SHA256 type alias for token signing
type HmacSha256 = Hmac<Sha256>;

/// Environment variable name for the CSRF secret
const CSRF_SECRET_ENV: &str = "CSRF_SECRET";

/// Name of the CSRF cookie
const CSRF_COOKIE_NAME: &str = "ltcms_csrf";

/// Name of the CSRF HTTP header
const CSRF_HEADER_NAME: &str = "x-csrf-token";

/// CSRF token time-to-live in seconds (6 hours)
const CSRF_TOKEN_TTL_SECONDS: i64 = 6 * 60 * 60;

/// Minimum length for CSRF secret (256 bits recommended)
const CSRF_MIN_SECRET_LENGTH: usize = 32;

/// Current CSRF token format version
const CSRF_VERSION: &str = "v1";

/// Global storage for the CSRF secret key
static CSRF_SECRET: OnceLock<Vec<u8>> = OnceLock::new();

/// Initializes the CSRF secret from the environment variable.
///
/// This function must be called once at application startup before any
/// CSRF operations. It validates the secret for security and stores it
/// in global state.
///
/// # Security Validation
/// The secret is checked for:
/// - Presence (not missing)
/// - Minimum length (32 bytes for adequate entropy)
/// - Character diversity (at least 10 unique characters)
///
/// # Returns
/// - `Ok(())` if the secret was successfully initialized
/// - `Err(String)` with a descriptive error message if validation fails
///
/// # Errors
/// - CSRF_SECRET environment variable not set
/// - Secret is too short (< 32 characters)
/// - Secret has insufficient entropy (< 10 unique characters)
/// - Secret was already initialized (can only be called once)
///
/// # Example
/// ```rust,no_run
/// use rust_blog_backend::security::csrf;
/// csrf::init_csrf_secret().expect("Failed to initialize CSRF secret");
/// ```
pub fn init_csrf_secret() -> Result<(), String> {
    // Load secret from environment variable
    let secret = env::var(CSRF_SECRET_ENV)
        .map_err(|_| format!("{CSRF_SECRET_ENV} environment variable not set"))?;
    let trimmed = secret.trim();

    // Validate minimum length requirement
    if trimmed.len() < CSRF_MIN_SECRET_LENGTH {
        return Err(format!(
            "{CSRF_SECRET_ENV} must be at least {CSRF_MIN_SECRET_LENGTH} characters long"
        ));
    }

    // Validate entropy requirement (unique characters)
    let unique_chars = trimmed.chars().collect::<HashSet<_>>().len();
    if unique_chars < 10 {
        return Err(format!(
            "{CSRF_SECRET_ENV} must contain at least 10 unique characters"
        ));
    }

    // Store secret in thread-safe static storage
    CSRF_SECRET
        .set(trimmed.as_bytes().to_vec())
        .map_err(|_| "CSRF secret already initialized".to_string())?;

    Ok(())
}

/// Retrieves the CSRF secret from global state.
///
/// # Panics
/// Panics if init_csrf_secret() has not been called yet.
///
/// # Returns
/// A reference to the CSRF secret bytes.
fn get_secret() -> &'static [u8] {
    CSRF_SECRET
        .get()
        .expect("CSRF secret not initialized. Call init_csrf_secret() first.")
        .as_slice()
}

/// Issues a new CSRF token for a user.
///
/// Creates a cryptographically signed token bound to the user's identity.
/// The token is valid for 6 hours and includes a random nonce for uniqueness.
///
/// # Arguments
/// * `username` - The username to bind the token to
///
/// # Returns
/// - `Ok(String)` - The complete CSRF token (v1 format)
/// - `Err(String)` - If token generation fails
///
/// # Token Structure
/// The token consists of:
/// 1. Version identifier ("v1")
/// 2. Base64URL-encoded username
/// 3. Unix timestamp expiration
/// 4. Random UUID nonce
/// 5. Base64URL-encoded HMAC-SHA256 signature
///
/// All components are pipe-separated.
///
/// # Security
/// - HMAC signature prevents token forgery
/// - Username binding prevents token theft across accounts
/// - Nonce prevents token reuse
/// - Expiration limits token lifetime
///
/// # Errors
/// - Username is empty
/// - Failed to compute expiration timestamp
/// - HMAC initialization fails
pub fn issue_csrf_token(username: &str) -> Result<String, String> {
    // Validate input
    if username.is_empty() {
        return Err("Username required for CSRF token".to_string());
    }

    // Calculate token expiration
    let expiry = Utc::now()
        .checked_add_signed(Duration::seconds(CSRF_TOKEN_TTL_SECONDS))
        .ok_or_else(|| "Failed to compute CSRF expiry".to_string())?
        .timestamp();

    // Generate random nonce for uniqueness
    let nonce = Uuid::new_v4().to_string();

    // Encode username for safe transport
    let username_b64 = Base64UrlUnpadded::encode_string(username.as_bytes());

    // Build token payload
    let payload = format!("{username_b64}|{expiry}|{nonce}");
    let versioned_payload = format!("{CSRF_VERSION}|{payload}");

    // Create HMAC signature
    let mut mac = HmacSha256::new_from_slice(get_secret())
        .map_err(|_| "Failed to initialize CSRF HMAC".to_string())?;
    mac.update(versioned_payload.as_bytes());
    let signature = Base64UrlUnpadded::encode_string(&mac.finalize().into_bytes());

    // Return complete token
    Ok(format!("{versioned_payload}|{signature}"))
}

/// Validates a CSRF token against an expected username.
///
/// This performs comprehensive validation including:
/// - Token format and structure
/// - Version compatibility
/// - Username binding
/// - Expiration check
/// - Signature verification (constant-time)
///
/// # Arguments
/// * `token` - The CSRF token to validate
/// * `expected_username` - The username the token should be bound to
///
/// # Returns
/// - `Ok(())` if the token is valid for the user
/// - `Err(String)` with a descriptive error message if validation fails
///
/// # Security
/// - Constant-time signature comparison (prevents timing attacks)
/// - Strict format validation (prevents malformed tokens)
/// - Username binding check (prevents cross-account token use)
/// - Expiration enforcement (limits token lifetime)
///
/// # Errors
/// - Malformed token structure
/// - Unsupported version
/// - Username mismatch
/// - Token expired
/// - Invalid signature
/// - Nonce too short
fn validate_csrf_token(token: &str, expected_username: &str) -> Result<(), String> {
    // Parse token into components
    let mut parts = token.split('|');

    // Extract and validate version
    let version = parts
        .next()
        .ok_or_else(|| "Malformed CSRF token".to_string())?;
    if version != CSRF_VERSION {
        return Err("Unsupported CSRF token version".to_string());
    }

    // Extract required components
    let username_b64 = parts
        .next()
        .ok_or_else(|| "Malformed CSRF token".to_string())?;
    let expiry_str = parts
        .next()
        .ok_or_else(|| "Malformed CSRF token".to_string())?;
    let nonce = parts
        .next()
        .ok_or_else(|| "Malformed CSRF token".to_string())?;
    let signature = parts
        .next()
        .ok_or_else(|| "Malformed CSRF token".to_string())?;

    // Ensure no extra components
    if parts.next().is_some() {
        return Err("Malformed CSRF token".to_string());
    }

    // Decode and verify username binding
    let username_bytes = Base64UrlUnpadded::decode_vec(username_b64)
        .map_err(|_| "Malformed CSRF username segment".to_string())?;
    let username = String::from_utf8(username_bytes)
        .map_err(|_| "Invalid CSRF username encoding".to_string())?;

    if username != expected_username {
        return Err("CSRF token not issued for this account".to_string());
    }

    // Check token expiration
    let expiry: i64 = expiry_str
        .parse()
        .map_err(|_| "Invalid CSRF expiry".to_string())?;

    if expiry < Utc::now().timestamp() {
        return Err("CSRF token expired".to_string());
    }

    // Validate nonce length
    if nonce.len() < 16 {
        return Err("CSRF nonce too short".to_string());
    }

    // Verify HMAC signature
    let versioned_payload = format!("{version}|{username_b64}|{expiry}|{nonce}");

    let mut mac = HmacSha256::new_from_slice(get_secret())
        .map_err(|_| "Failed to initialize CSRF HMAC".to_string())?;
    mac.update(versioned_payload.as_bytes());
    let expected_signature = mac.finalize().into_bytes();

    let provided_signature = Base64UrlUnpadded::decode_vec(signature)
        .map_err(|_| "Invalid CSRF signature".to_string())?;

    // Constant-time signature comparison
    if expected_signature.len() != provided_signature.len()
        || !subtle_equals(&expected_signature, &provided_signature)
    {
        return Err("CSRF signature mismatch".to_string());
    }

    Ok(())
}

/// Performs constant-time equality comparison on byte slices.
///
/// This prevents timing side-channel attacks by ensuring the comparison
/// takes the same time regardless of where differences occur.
///
/// # Arguments
/// * `a` - First byte slice
/// * `b` - Second byte slice
///
/// # Returns
/// `true` if the slices are equal, `false` otherwise
///
/// # Security
/// Uses the `subtle` crate for constant-time comparison, preventing
/// attackers from learning about signature bytes through timing analysis.
fn subtle_equals(a: &[u8], b: &[u8]) -> bool {
    use subtle::ConstantTimeEq;
    a.ct_eq(b).into()
}

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
fn build_csrf_cookie(token: &str) -> Cookie<'static> {
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
fn build_csrf_removal() -> Cookie<'static> {
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

/// AXUM extractor for CSRF protection.
///
/// This extractor validates CSRF tokens for state-changing HTTP methods.
/// Safe methods (GET, HEAD, OPTIONS, TRACE) are automatically allowed.
///
/// # Validation Process
/// 1. Skip validation for safe HTTP methods
/// 2. Ensure user is authenticated (extract Claims)
/// 3. Extract token from x-csrf-token header
/// 4. Extract token from cookie
/// 5. Verify header and cookie tokens match (double-submit pattern)
/// 6. Validate token signature and binding to user
///
/// # Usage
/// ```rust,no_run
/// use axum::{Router, routing::post, middleware};
/// use rust_blog_backend::security::csrf::CsrfGuard;
/// async fn handler() {}
///
/// let app = Router::new()
///     .route("/api/resource", post(handler))
///     .route_layer(middleware::from_extractor::<CsrfGuard>());
/// ```
///
/// # Security
/// - Double-submit cookie pattern (cookie + header)
/// - Per-user token binding
/// - HMAC signature verification
/// - Expiration enforcement
///
/// # Errors
/// Returns 403 Forbidden if:
/// - CSRF token header is missing
/// - CSRF cookie is missing
/// - Header and cookie tokens don't match
/// - Token validation fails (expired, wrong user, invalid signature)
pub struct CsrfGuard;

impl<S> FromRequestParts<S> for CsrfGuard
where
    S: Send + Sync,
    crate::db::DbPool: axum::extract::FromRef<S>,
{
    type Rejection = (StatusCode, Json<ErrorResponse>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Step 1: Method Filter. CSRF is only required for state-changing operations.
        if matches!(
            parts.method,
            Method::GET | Method::HEAD | Method::OPTIONS | Method::TRACE
        ) {
            return Ok(Self);
        }

        // Step 2: Authenticated Check. CSRF protects sessions, so we first check the user's identity.
        let claims_result = if let Some(existing) = parts.extensions.get::<auth::Claims>() {
            Ok(existing.clone())
        } else {
            auth::Claims::from_request_parts(parts, _state).await
        };

        let claims = match claims_result {
            Ok(claims) => {
                // User is logged in -> Enforce strict CSRF checks.
                parts.extensions.insert(claims.clone());
                claims
            }
            Err(_) => {
                // Anonymous user -> No session to hijack via CSRF.
                return Ok(Self);
            }
        };

        // Step 3: Extract tokens from both submission channels.
        let header_value = parts
            .headers
            .get(HeaderName::from_static(CSRF_HEADER_NAME))
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| {
                (
                    StatusCode::FORBIDDEN,
                    Json(ErrorResponse {
                        error: "Missing CSRF token header".to_string(),
                    }),
                )
            })?;

        let jar = CookieJar::from_headers(&parts.headers);
        let cookie = jar.get(CSRF_COOKIE_NAME).ok_or_else(|| {
            (
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    error: "Missing CSRF cookie".to_string(),
                }),
            )
        })?;

        // Step 4: Double-Submit Validation. Ensure the tokens match.
        if cookie.value() != header_value {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    error: "CSRF token mismatch".to_string(),
                }),
            ));
        }

        // Step 5: Master Validation. Verify signature, expiration, and user binding.
        validate_csrf_token(header_value, &claims.sub)
            .map_err(|err| (StatusCode::FORBIDDEN, Json(ErrorResponse { error: err })))?;

        Ok(Self)
    }
}

/// Returns the name of the CSRF cookie.
///
/// # Returns
/// The constant CSRF cookie name: "ltcms_csrf"
pub fn csrf_cookie_name() -> &'static str {
    CSRF_COOKIE_NAME
}

/// Returns the name of the CSRF HTTP header.
///
/// # Returns
/// The constant CSRF header name: "x-csrf-token"
pub fn csrf_header_name() -> &'static str {
    CSRF_HEADER_NAME
}

/// Middleware to enforce CSRF protection.
///
/// This middleware extracts the `CsrfGuard` which performs the validation.
/// It is designed to be used with `axum::middleware::from_fn_with_state`
/// to ensure the database pool state is available for extraction.
pub async fn enforce_csrf(
    axum::extract::State(_pool): axum::extract::State<crate::db::DbPool>,
    _guard: CsrfGuard,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    next.run(req).await
}
