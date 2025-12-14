//! Authentication and Authorization Module
//!
//! This module provides JWT-based authentication and authorization for the application.
//! It includes token generation, validation, cookie management, and middleware for
//! protecting routes.
//!
//! # Security Features
//! - HS256 JWT tokens with configurable expiration
//! - Secure, HttpOnly session cookies
//! - High-entropy secret validation
//! - Bearer token and cookie-based authentication
//! - Automatic token expiration handling
//!
//! # Usage
//! Before using any authentication functions, initialize the JWT secret:
//! ```rust,no_run
//! use linux_tutorial_cms::auth;
//! auth::init_jwt_secret().expect("Failed to initialize JWT secret");
//! ```

use axum::{
    extract::FromRef,
    extract::FromRequestParts,
    http::{
        header::{AUTHORIZATION, SET_COOKIE},
        request::Parts,
        HeaderMap, HeaderValue, StatusCode,
    },

};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::sync::LazyLock;
use std::sync::OnceLock;
use time::{Duration as TimeDuration, OffsetDateTime};

use crate::db::DbPool;

/// Global storage for the JWT secret key.
/// Initialized once at application startup via init_jwt_secret().
pub static JWT_SECRET: OnceLock<String> = OnceLock::new();

/// Global storage for the JWT decoding key.
/// Derived from JWT_SECRET once it's initialized.
pub static DECODING_KEY: LazyLock<DecodingKey> =
    LazyLock::new(|| DecodingKey::from_secret(get_jwt_secret().as_bytes()));

/// List of known placeholder secrets that must not be used in production.
/// These are common defaults found in example configurations.
const SECRET_BLACKLIST: &[&str] = &[
    "CHANGE_ME_OR_APP_WILL_FAIL",
    "your-super-secret-jwt-key-min-32-chars-change-me-in-production",
    "PLEASE-SET-THIS-VIA-DOCKER-COMPOSE-ENV",
];

/// Minimum length for JWT secret to ensure adequate entropy (~256 bits).
/// Base64-encoded secrets should be at least this long.
const MIN_SECRET_LENGTH: usize = 43;

/// Minimum number of unique characters required in the secret.
/// Helps detect low-entropy secrets like repeated characters.
const MIN_UNIQUE_CHARS: usize = 10;

/// Minimum number of character classes (lowercase, uppercase, digits, symbols).
/// Ensures the secret has good diversity.
const MIN_CHAR_CLASSES: usize = 3;

/// Name of the HTTP-only authentication cookie.
pub const AUTH_COOKIE_NAME: &str = "ltcms_session";

/// Authentication cookie time-to-live in seconds (24 hours).
const AUTH_COOKIE_TTL_SECONDS: i64 = 24 * 60 * 60;

/// Initializes the JWT secret from the environment variable.
///
/// This function must be called once at application startup before any
/// authentication operations. It validates the secret for security and
/// stores it in global state.
///
/// # Security Validation
/// The secret is checked for:
/// - Presence (not missing or empty)
/// - Blacklisted placeholder values
/// - Minimum length (43 characters for ~256 bits of entropy)
/// - Character diversity (at least 3 character classes)
/// - Uniqueness (at least 10 unique characters)
///
/// # Returns
/// - `Ok(())` if the secret was successfully initialized
/// - `Err(String)` with a descriptive error message if validation fails
///
/// # Errors
/// - JWT_SECRET environment variable not set
/// - Secret is empty or whitespace only
/// - Secret uses a known placeholder value
/// - Secret has insufficient entropy
/// - Secret was already initialized (can only be called once)
///
/// # Example
/// ```rust,no_run
/// use linux_tutorial_cms::auth;
/// auth::init_jwt_secret().expect("Failed to initialize JWT secret");
/// ```
pub fn init_jwt_secret() -> Result<(), String> {
    // Load secret from environment
    let secret = env::var("JWT_SECRET")
        .map_err(|_| "JWT_SECRET environment variable not set".to_string())?;
    let trimmed = secret.trim();

    // Check for empty secret
    if trimmed.is_empty() {
        return Err("JWT_SECRET cannot be empty or whitespace".to_string());
    }

    // Check against known placeholder values
    if SECRET_BLACKLIST
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(trimmed))
    {
        return Err(
            "JWT_SECRET uses a known placeholder value. Generate a fresh random secret (e.g. `openssl rand -base64 48)`."
                .to_string(),
        );
    }

    // Validate entropy
    if !secret_has_min_entropy(trimmed) {
        return Err(
            "JWT_SECRET must be a high-entropy value (~256 bits). Use a cryptographically random string of at least 43 characters mixing upper, lower, digits, and symbols."
                .to_string(),
        );
    }

    // Store secret in global state (can only be done once)
    JWT_SECRET
        .set(trimmed.to_string())
        .map_err(|_| "JWT_SECRET already initialized".to_string())?;

    Ok(())
}

/// Retrieves the JWT secret from global state.
///
/// # Panics
/// Panics if init_jwt_secret() has not been called yet.
///
/// # Returns
/// A reference to the JWT secret string.
fn get_jwt_secret() -> &'static str {
    JWT_SECRET
        .get()
        .expect("JWT_SECRET not initialized. Call init_jwt_secret() first.")
        .as_str()
}

/// JWT claims structure containing user identity and authorization information.
///
/// These claims are encoded into the JWT token and can be extracted when
/// validating authenticated requests.
///
/// # Fields
/// - `sub`: Subject (username) - identifies the user
/// - `role`: User role (e.g., "admin", "user") - for authorization
/// - `exp`: Expiration timestamp (Unix epoch) - prevents token reuse
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject: the username of the authenticated user
    pub sub: String,

    /// User role for authorization (e.g., "admin")
    pub role: String,

    /// Expiration time as Unix timestamp (seconds since epoch)
    pub exp: usize,
}

impl Claims {
    /// Creates new JWT claims with a 24-hour expiration.
    ///
    /// # Arguments
    /// * `username` - The username to include in the token
    /// * `role` - The user's role for authorization
    ///
    /// # Returns
    /// A new Claims instance with expiration set to 24 hours from now
    ///
    /// # Panics
    /// Panics if the system time is severely misconfigured
    pub fn new(username: String, role: String) -> Self {
        // Calculate expiration time (24 hours from now)
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .and_then(|dt| usize::try_from(dt.timestamp()).ok())
            .expect(
                "Failed to calculate JWT expiration timestamp. System time may be misconfigured.",
            );

        Claims {
            sub: username,
            role,
            exp: expiration,
        }
    }
}

/// Creates a signed JWT token for a user.
///
/// This generates a new JWT token with the user's identity and role,
/// signed with the application's secret key.
///
/// # Arguments
/// * `username` - The username to encode in the token
/// * `role` - The user's role for authorization
///
/// # Returns
/// - `Ok(String)` - The encoded JWT token
/// - `Err(jsonwebtoken::errors::Error)` - If token generation fails
///
/// # Security
/// The token is signed using HS256 with the JWT secret, ensuring it
/// cannot be forged without knowledge of the secret key.
///
/// # Example
/// ```rust,no_run
/// use linux_tutorial_cms::auth;
/// let token = auth::create_jwt("admin".to_string(), "admin".to_string())?;
/// # Ok::<(), jsonwebtoken::errors::Error>(())
/// ```
pub fn create_jwt(username: String, role: String) -> Result<String, jsonwebtoken::errors::Error> {
    // Create claims with 24-hour expiration
    let claims = Claims::new(username, role);

    // Get the initialized JWT secret
    let secret = get_jwt_secret();

    // Encode and sign the token
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// Verifies a JWT token and extracts its claims.
///
/// This validates the token's signature, checks expiration, and returns
/// the decoded claims if valid.
///
/// # Arguments
/// * `token` - The JWT token string to verify
///
/// # Returns
/// - `Ok(Claims)` - The decoded and validated claims
/// - `Err(jsonwebtoken::errors::Error)` - If validation fails
///
/// # Validation
/// - Signature must match (prevents tampering)
/// - Token must not be expired (with 60-second leeway for clock skew)
/// - Token must be well-formed
///
/// # Security
/// This function prevents:
/// - Token forgery (signature validation)
/// - Token replay after expiration (expiration check)
/// - Malformed tokens (parsing validation)
pub fn verify_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    // Get the initialized JWT secret
    let secret = get_jwt_secret();

    // Configure validation rules
    let mut validation = Validation::default();
    validation.leeway = 60; // Allow 60 seconds of clock skew
    validation.validate_exp = true; // Ensure token hasn't expired

    // Decode and validate the token
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;

    Ok(token_data.claims)
}

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
        // Check if claims already extracted by middleware
        if let Some(claims) = parts.extensions.get::<Claims>() {
            return Ok(claims.clone());
        }

        // Extract token from Authorization header or cookie
        let token = extract_token(&parts.headers).ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                "Missing authentication token".to_string(),
            )
        })?;

        // Verify and decode the token
        let claims = verify_jwt(&token)
            .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Invalid token: {}", e)))?;

        // Check if token is blacklisted
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

/// Validates that a secret has minimum entropy requirements.
///
/// This function checks if a secret meets security requirements to prevent
/// use of weak or predictable secrets.
///
/// # Arguments
/// * `secret` - The secret string to validate
///
/// # Returns
/// - `true` if the secret meets all entropy requirements
/// - `false` if the secret is too weak
///
/// # Requirements
/// - At least 43 characters long (~256 bits base64-encoded)
/// - At least 3 character classes (lowercase, uppercase, digits, symbols)
/// - At least 10 unique characters
///
/// # Purpose
/// Prevents use of weak secrets like:
/// - Short secrets
/// - Repeated characters
/// - Dictionary words
/// - Sequential patterns
fn secret_has_min_entropy(secret: &str) -> bool {
    // Check minimum length
    if secret.len() < MIN_SECRET_LENGTH {
        return false;
    }

    // Count character classes present
    let mut classes = 0;
    if secret.chars().any(|c| c.is_ascii_lowercase()) {
        classes += 1;
    }
    if secret.chars().any(|c| c.is_ascii_uppercase()) {
        classes += 1;
    }
    if secret.chars().any(|c| c.is_ascii_digit()) {
        classes += 1;
    }
    if secret.chars().any(|c| !c.is_ascii_alphanumeric()) {
        classes += 1;
    }

    // Require at least 3 different character classes
    if classes < MIN_CHAR_CLASSES {
        return false;
    }

    // Check for sufficient character diversity
    let unique_chars: HashSet<char> = secret.chars().collect();
    unique_chars.len() >= MIN_UNIQUE_CHARS
}

/// Determines whether authentication cookies should use the Secure flag.
///
/// # Returns
/// - `true` by default (cookies only sent over HTTPS)
/// - `false` if AUTH_COOKIE_SECURE is explicitly set to "false"
///
/// # Security Warning
/// Setting this to false allows cookies over HTTP, which exposes tokens
/// to network sniffing. Only use this in trusted development environments.
///
/// # Environment Variable
/// Set AUTH_COOKIE_SECURE=false to disable (logs a warning)
pub fn cookies_should_be_secure() -> bool {
    match env::var("AUTH_COOKIE_SECURE") {
        // Only disable if explicitly set to false
        Ok(value) if value.trim().eq_ignore_ascii_case("false") => {
            tracing::warn!(
                "AUTH_COOKIE_SECURE explicitly set to false. Cookies will be sent over HTTP; only use this in trusted development environments."
            );
            false
        }
        // Default to secure cookies
        _ => true,
    }
}

/// Extracts the JWT token from request headers.
///
/// Supports two authentication methods:
/// 1. Authorization header with Bearer scheme
/// 2. Session cookie
///
/// # Arguments
/// * `headers` - The request headers to search
///
/// # Returns
/// - `Some(String)` if a token was found
/// - `None` if no token was found in either location
///
/// # Priority
/// Authorization header is checked first, falling back to cookies
pub fn extract_token(headers: &HeaderMap) -> Option<String> {
    // First check Authorization header
    if let Some(header_value) = headers.get(AUTHORIZATION) {
        if let Ok(value_str) = header_value.to_str() {
            if let Some(token) = parse_bearer_token(value_str) {
                return Some(token);
            }
        }
    }

    // Fall back to cookie
    let jar = CookieJar::from_headers(headers);
    jar.get(AUTH_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string())
}

/// Parses a Bearer token from an Authorization header value.
///
/// # Arguments
/// * `value` - The Authorization header value (e.g., "Bearer eyJhbGci...")
///
/// # Returns
/// - `Some(String)` if the value is a valid Bearer token
/// - `None` if the format is invalid or not a Bearer token
///
/// # Format
/// Expected format: "Bearer <token>"
/// - Scheme must be "Bearer" (case-insensitive)
/// - Token must not be empty after trimming
fn parse_bearer_token(value: &str) -> Option<String> {
    // Split into scheme and token
    let trimmed = value.trim();
    let (scheme, token) = trimmed.split_once(' ')?;

    // Verify Bearer scheme and non-empty token
    if scheme.eq_ignore_ascii_case("Bearer") && !token.trim().is_empty() {
        return Some(token.trim().to_string());
    }
    None
}

/// Optional Claims extractor for endpoints that support both authenticated and anonymous access.
///
/// If a valid token is provided, it extracts the claims.
/// If no token is provided, it returns `None`.
/// If an invalid token is provided, it returns an error (401).
pub struct OptionalClaims(pub Option<Claims>);

impl<S> FromRequestParts<S> for OptionalClaims
where
    S: Send + Sync,
    DbPool: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Check if claims already extracted by middleware
        if let Some(claims) = parts.extensions.get::<Claims>() {
            return Ok(OptionalClaims(Some(claims.clone())));
        }

        // Extract token from Authorization header or cookie
        let token = match extract_token(&parts.headers) {
            Some(token) => token,
            None => return Ok(OptionalClaims(None)),
        };

        // Verify and decode the token
        let claims = verify_jwt(&token)
            .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Invalid token: {}", e)))?;

        // Check if token is blacklisted
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

        Ok(OptionalClaims(Some(claims)))
    }
}
