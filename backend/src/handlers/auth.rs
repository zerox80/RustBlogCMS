//! Authentication HTTP Handlers
//!
//! This module contains HTTP handlers for authentication-related endpoints.
//! It implements secure login, logout, and user identity verification.
//!
//! # Security Features
//! - Login rate limiting with progressive lockout
//! - Timing-attack resistant password verification
//! - Salted hash-based login attempt tracking
//! - Constant-time dummy hash verification
//! - Input validation
//! - CSRF token issuance on login
//! - Secure cookie management
//!
//! # Endpoints
//! - POST /api/auth/login: Authenticate user and issue tokens
//! - GET /api/auth/me: Get current user information
//! - POST /api/auth/logout: Invalidate session
//!
//! # Rate Limiting
//! Failed login attempts trigger progressive lockout:
//! - 3 failures: 10-second lockout
//! - 5+ failures: 60-second lockout

use crate::{security::{auth, csrf}, db::DbPool, models::*, repositories};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use sha2::{Digest, Sha256};
use sqlx;
use std::{env, sync::OnceLock, time::Duration};

/// Global salt for hashing login attempt identifiers.
/// Initialized once at startup via init_login_attempt_salt().
static LOGIN_ATTEMPT_SALT: OnceLock<String> = OnceLock::new();

/// Initializes the login attempt salt from environment.
///
/// This salt is used to hash usernames before storing them in the
/// login_attempts table, preventing username enumeration attacks.
///
/// # Returns
/// - `Ok(())` if initialization succeeds
/// - `Err(String)` with error message if validation fails
///
/// # Errors
/// - LOGIN_ATTEMPT_SALT environment variable not set
/// - Salt is too short (< 32 characters)
/// - Salt has insufficient entropy (< 10 unique characters)
/// - Salt was already initialized
pub fn init_login_attempt_salt() -> Result<(), String> {
    let raw = env::var("LOGIN_ATTEMPT_SALT")
        .map_err(|_| "LOGIN_ATTEMPT_SALT environment variable not set".to_string())?;
    let trimmed = raw.trim();

    if trimmed.len() < 32 {
        return Err("LOGIN_ATTEMPT_SALT must be at least 32 characters long".to_string());
    }

    let unique_chars = trimmed
        .chars()
        .collect::<std::collections::HashSet<_>>()
        .len();
    if unique_chars < 10 {
        return Err("LOGIN_ATTEMPT_SALT must contain at least 10 unique characters".to_string());
    }

    LOGIN_ATTEMPT_SALT
        .set(trimmed.to_string())
        .map_err(|_| "LOGIN_ATTEMPT_SALT already initialized".to_string())?;

    Ok(())
}

/// Retrieves the initialized login attempt salt.
///
/// # Panics
/// Panics if init_login_attempt_salt() has not been called yet.
fn login_attempt_salt() -> &'static str {
    LOGIN_ATTEMPT_SALT
        .get()
        .expect("LOGIN_ATTEMPT_SALT not initialized. Call init_login_attempt_salt() first.")
        .as_str()
}

/// Hashes a username for login attempt tracking.
///
/// Creates a salted SHA-256 hash of the normalized username.
/// This prevents username enumeration by obscuring which accounts exist.
///
/// # Arguments
/// * `username` - The username to hash
///
/// # Returns
/// Hex-encoded SHA-256 hash
///
/// # Security
/// - Username is trimmed and lowercased for normalization
/// - Salt prevents rainbow table attacks
/// - Hash prevents direct username storage
fn hash_login_identifier(username: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(login_attempt_salt().as_bytes());
    hasher.update(username.trim().to_ascii_lowercase().as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Parses an optional RFC3339 timestamp string into a UTC DateTime.
///
/// # Arguments
/// * `value` - Optional RFC3339 timestamp string
///
/// # Returns
/// - `Some(DateTime<Utc>)` if parsing succeeds
/// - `None` if value is None or parsing fails
fn parse_rfc3339_opt(value: &Option<String>) -> Option<DateTime<Utc>> {
    value
        .as_ref()
        .and_then(|timestamp| chrono::DateTime::parse_from_rfc3339(timestamp).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

/// Returns a precomputed dummy bcrypt hash for timing-attack resistance.
///
/// This hash is used during failed login attempts to ensure password
/// verification takes constant time regardless of whether the user exists.
///
/// # Returns
/// A static bcrypt hash string
///
/// # Security
/// Using a dummy hash when the user doesn't exist prevents timing attacks
/// that could enumerate valid usernames by measuring response times.
fn dummy_bcrypt_hash() -> &'static str {
    static DUMMY_HASH: OnceLock<String> = OnceLock::new();

    DUMMY_HASH.get_or_init(|| match bcrypt::hash("dummy", bcrypt::DEFAULT_COST) {
        Ok(hash) => hash,
        Err(err) => {
            tracing::error!("Failed to generate dummy hash: {}", err);
            "$2b$12$eImiTXuWVxfM37uY4JANjQPzMzXZjQDzqzQpMv0xoGrTplPPNaE3W".to_string()
        }
    })
}

/// Validates a username meets security and format requirements.
///
/// # Arguments
/// * `username` - The username to validate
///
/// # Returns
/// - `Ok(())` if valid
/// - `Err(String)` with error message if invalid
///
/// # Validation Rules
/// - Not empty
/// - Length ≤ 50 characters
/// - Only alphanumeric, underscore, hyphen, and period allowed
fn validate_username(username: &str) -> Result<(), String> {
    if username.is_empty() {
        return Err("Username cannot be empty".to_string());
    }
    if username.len() > 50 {
        return Err("Username too long".to_string());
    }

    if !username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err("Username contains invalid characters".to_string());
    }
    Ok(())
}

/// Validates a password meets security and format requirements.
///
/// # Arguments
/// * `password` - The password to validate
///
/// # Returns
/// - `Ok(())` if valid
/// - `Err(String)` with error message if invalid
///
/// # Validation Rules
/// - Not empty
/// - Length ≤ 128 characters (prevents DoS via bcrypt)
fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 12 {
        return Err("Password must be at least 12 characters long".to_string());
    }
    if password.len() > 128 {
        return Err("Password too long".to_string());
    }

    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if !has_uppercase || !has_lowercase || !has_digit || !has_special {
        return Err(
            "Password must contain at least one uppercase letter, one lowercase letter, one number, and one special character"
                .to_string(),
        );
    }

    Ok(())
}

/// HTTP handler for user login.
///
/// Authenticates a user and issues JWT and CSRF tokens.
/// Implements progressive rate limiting and timing-attack resistance.
///
/// # Endpoint
/// POST /api/auth/login
///
/// # Request
/// JSON body with LoginRequest:
/// ```json
/// {
///   "username": "admin",
///   "password": "secret"
/// }
/// ```
///
/// # Response
/// On success (200 OK):
/// - Sets auth cookie (ltcms_session)
/// - Sets CSRF cookie (ltcms_csrf)
/// - Returns LoginResponse with JWT token and user info
///
/// # Errors
/// - 400 Bad Request: Invalid username/password format
/// - 401 Unauthorized: Invalid credentials
/// - 429 Too Many Requests: Account temporarily locked
/// - 500 Internal Server Error: Database or token generation failure
///
/// # Security Features
/// - Input validation (length, character set)
/// - Progressive lockout (3 failures → 10s, 5+ failures → 60s)
/// - Timing-attack resistance (constant-time verification)
/// - Random jitter (100-300ms) to prevent timing analysis
/// - Username enumeration protection (hashed login tracking)
/// - Automatic lockout reset on successful login
///
/// # Rate Limiting
/// After failed attempts:
/// - 3 failures: 10-second lockout
/// - 5+ failures: 60-second lockout
/// - Lockout countdown shown to user
pub async fn login(
    State(pool): State<DbPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<(HeaderMap, Json<LoginResponse>), (StatusCode, Json<ErrorResponse>)> {
    let username = payload.username.trim().to_string();

    if let Err(e) = validate_username(&username) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })));
    }
    if let Err(e) = validate_password(&payload.password) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })));
    }

    let attempt_key = hash_login_identifier(&username);

    let attempt_record = repositories::users::get_login_attempt(&pool, &attempt_key)
        .await
        .map_err(|e| {
            tracing::error!("Failed to load login attempts for {}: {}", username, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal server error".to_string(),
                }),
            )
        })?;

    if let Some(record) = &attempt_record {
        if let Some(blocked_until) = parse_rfc3339_opt(&record.blocked_until) {
            let now = Utc::now();
            if blocked_until > now {
                let remaining = (blocked_until - now).num_seconds().max(0);
                // Do not sleep here to avoid holding connections (DoS prevention)
                return Err((
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(ErrorResponse {
                        error: format!(
                            "Zu viele fehlgeschlagene Versuche. Bitte warte {} Sekunde{}.",
                            remaining,
                            if remaining == 1 { "" } else { "n" }
                        ),
                    }),
                ));
            }
        }
    }

    let user = repositories::users::get_user_by_username(&pool, &username)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal server error".to_string(),
                }),
            )
        })?;

    let hash_to_verify_owned = user.as_ref().map(|u| u.password_hash.clone());
    let hash_to_verify = hash_to_verify_owned
        .as_deref()
        .unwrap_or(dummy_bcrypt_hash());

    let verification_result = bcrypt::verify(&payload.password, hash_to_verify);

    let (password_valid, user_record) = match (user, verification_result) {
        (Some(user), Ok(true)) => (true, Some(user)),
        (Some(_), Ok(false)) => (false, None),
        (Some(_), Err(e)) => {
            tracing::error!("Password verification error: {}", e);
            (false, None)
        }
        (None, _) => (false, None),
    };

    let jitter = (chrono::Utc::now().timestamp_subsec_millis() % 200) as u64;
    tokio::time::sleep(Duration::from_millis(100 + jitter)).await;

    if !password_valid {
        let now = Utc::now();
        let long_block = (now + ChronoDuration::seconds(60)).to_rfc3339();
        let short_block = (now + ChronoDuration::seconds(10)).to_rfc3339();

        repositories::users::record_failed_login(&pool, &attempt_key, &long_block, &short_block)
            .await
            .map_err(|e| {
                tracing::error!("Failed to record login attempt for hashed key: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Internal server error".to_string(),
                    }),
                )
            })?;

        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Ungültige Anmeldedaten".to_string(),
            }),
        ));
    }

    if attempt_record.is_some() {
        if let Err(e) = repositories::users::clear_login_attempts(&pool, &attempt_key).await {
            tracing::warn!(
                "Failed to clear login attempts for hashed key after successful login: {}",
                e
            );
        }
    }

    let user_record = user_record.expect("Successful login must have user record");
    let token =
        auth::create_jwt(user_record.username.clone(), user_record.role.clone()).map_err(|e| {
            tracing::error!("JWT creation error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to create token".to_string(),
                }),
            )
        })?;

    let mut headers = HeaderMap::new();
    auth::append_auth_cookie(&mut headers, auth::build_auth_cookie(&token));

    if let Ok(csrf_token) = csrf::issue_csrf_token(&user_record.username) {
        csrf::append_csrf_cookie(&mut headers, &csrf_token);
    } else {
        tracing::error!(
            "Failed to issue CSRF token for user {}",
            user_record.username
        );
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to create token".to_string(),
            }),
        ));
    }

    Ok((
        headers,
        Json(LoginResponse {
            token,
            user: UserResponse {
                username: user_record.username,
                role: user_record.role,
            },
        }),
    ))
}

/// HTTP handler for retrieving current user information.
///
/// Returns the authenticated user's identity from their JWT token.
/// Requires a valid authentication token (cookie or Authorization header).
///
/// # Endpoint
/// GET /api/auth/me
///
/// # Authentication
/// Requires valid JWT token via:
/// - Cookie: ltcms_session
/// - Header: Authorization: Bearer <token>
///
/// # Response
/// On success (200 OK):
/// ```json
/// {
///   "username": "admin",
///   "role": "admin"
/// }
/// ```
///
/// # Errors
/// - 401 Unauthorized: Missing or invalid token
///
/// # Security
/// User identity is extracted from the validated JWT token,
/// not from request parameters, preventing impersonation.
pub async fn me(
    claims: auth::Claims,
) -> Result<(HeaderMap, Json<UserResponse>), (StatusCode, Json<ErrorResponse>)> {
    let mut headers = HeaderMap::new();

    // Refresh CSRF token to ensure active sessions always have a valid one
    if let Ok(csrf_token) = csrf::issue_csrf_token(&claims.sub) {
        csrf::append_csrf_cookie(&mut headers, &csrf_token);
    } else {
        tracing::error!("Failed to refresh CSRF token for user {}", claims.sub);
        // We don't fail the request here, as the user is authenticated,
        // but subsequent state-changing requests might fail.
    }

    Ok((
        headers,
        Json(UserResponse {
            username: claims.sub,
            role: claims.role,
        }),
    ))
}

/// HTTP handler for user logout.
///
/// Invalidates the user's session by removing auth and CSRF cookies.
/// Requires CSRF token validation to prevent logout CSRF attacks.
///
/// # Endpoint
/// POST /api/auth/logout
///
/// # Authentication
/// Requires:
/// - Valid JWT token (cookie or header)
/// - Valid CSRF token (header and cookie must match)
///
/// # Response
/// On success (204 No Content):
/// - Sets auth cookie expiration to past (removes session)
/// - Sets CSRF cookie expiration to past (removes token)
/// - Empty response body
///
/// # Errors
/// - 401 Unauthorized: Missing or invalid JWT token
/// - 403 Forbidden: Missing or invalid CSRF token
///
/// # Security
/// - CSRF protection prevents attackers from forcing logout
/// - Logs logout event for audit trail
/// - Client must clear local storage/state separately
pub async fn logout(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    _csrf: csrf::CsrfGuard,
    claims: auth::Claims,
) -> (StatusCode, HeaderMap) {
    // Extract token to blacklist it
    if let Some(token) = auth::extract_token(&headers) {
        if let Err(e) =
            repositories::token_blacklist::blacklist_token(&pool, &token, claims.exp as i64).await
        {
            tracing::error!("Failed to blacklist token on logout: {}", e);
        }
    }

    let mut headers = HeaderMap::new();
    auth::append_auth_cookie(&mut headers, auth::build_cookie_removal());
    csrf::append_csrf_removal(&mut headers);
    tracing::info!(user = %claims.sub, "User logged out");
    (StatusCode::NO_CONTENT, headers)
}
