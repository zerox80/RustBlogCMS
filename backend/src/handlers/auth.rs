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
//! Failed login attempts trigger progressive lockout on two keys:
//! - Per (IP + username): 3 failures → 10s lockout, 5+ failures → 60s lockout
//! - Per IP across all usernames (anti password-spraying):
//!   10 failures → 60s lockout, 20+ failures → 300s lockout

use crate::{
    db::DbPool,
    middleware::security as security_middleware,
    models::*,
    repositories,
    security::{auth, csrf},
};
use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::{DateTime, Duration as ChronoDuration, Utc};

use axum_extra::extract::cookie::CookieJar;
use rand::RngExt;
use std::net::SocketAddr;
use std::{env, sync::OnceLock, time::Duration};

mod support;
pub use support::init_login_attempt_salt;
use support::*;

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
/// - Returns LoginResponse with user info
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
/// - Per (IP + username): 3 failures → 10s lockout, 5+ → 60s lockout
/// - Per IP across all usernames: 10 failures → 60s, 20+ → 300s lockout
/// - Lockout countdown shown to user
pub async fn login(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<LoginRequest>,
) -> Result<(HeaderMap, Json<LoginResponse>), ApiError> {
    let username = payload.username.trim().to_string();

    validate_username(&username).map_err(bad_request)?;
    validate_login_password(&payload.password).map_err(bad_request)?;

    // Probabilistic cleanup (1% of requests) of tables that otherwise grow
    // unbounded. Runs before the auth outcome is known on purpose: attack
    // traffic is almost entirely *failed* logins, so tying cleanup to
    // successful logins would never fire under the load that fills these
    // tables. Spawned so it never adds latency to the login itself.
    let should_cleanup = {
        let mut rng = rand::rng();
        rng.random_bool(0.01)
    };
    if should_cleanup {
        let pool_clone = pool.clone();
        tokio::spawn(async move {
            if let Err(e) = repositories::token_blacklist::cleanup_expired(&pool_clone).await {
                tracing::error!("Failed to cleanup expired blacklist tokens: {}", e);
            }
            if let Err(e) = repositories::users::cleanup_stale_login_attempts(&pool_clone).await {
                tracing::error!("Failed to cleanup stale login attempts: {}", e);
            }
        });
    }

    // Rate limit on two independent keys:
    //
    // 1. (IP + Username): tight thresholds. Using the pair (not username
    //    alone) stops an attacker from locking out 'admin' remotely, and
    //    (not IP alone at these tight thresholds) keeps NAT'd users from
    //    locking each other out.
    // 2. IP only: looser thresholds. Closes the gap the pair key leaves
    //    open -- an attacker rotating *usernames* from one address gets a
    //    fresh pair key on every attempt and would otherwise never hit any
    //    limit (password spraying).
    let client_ip = security_middleware::extract_client_ip(&headers, addr.ip());
    let attempt_key = hash_login_identifier(&format!("{}:{}", client_ip, username));
    let ip_attempt_key = hash_login_identifier(&format!("ip-wide:{}", client_ip));

    let attempt_record = repositories::users::get_login_attempt(&pool, &attempt_key)
        .await
        .map_err(internal_error("Failed to load login attempts"))?;
    let ip_attempt_record = repositories::users::get_login_attempt(&pool, &ip_attempt_key)
        .await
        .map_err(internal_error("Failed to load login attempts"))?;

    let blocked_until = [&attempt_record, &ip_attempt_record]
        .into_iter()
        .flatten()
        .filter_map(|record| parse_rfc3339_opt(&record.blocked_until))
        .max();
    if let Some(blocked_until) = blocked_until {
        let now = Utc::now();
        if blocked_until > now {
            let remaining = (blocked_until - now).num_seconds().max(0);
            // Do not sleep here to avoid holding connections (DoS prevention)
            return Err(api_error(
                StatusCode::TOO_MANY_REQUESTS,
                format!(
                    "Too many failed attempts. Please wait {} second{}.",
                    remaining,
                    if remaining == 1 { "" } else { "s" }
                ),
            ));
        }
    }

    let user = repositories::users::get_user_by_username(&pool, &username)
        .await
        .map_err(internal_error("Failed to load user"))?;

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

    let jitter = {
        let mut rng = rand::rng();
        rng.random_range(100..300)
    };
    tokio::time::sleep(Duration::from_millis(jitter)).await;

    if !password_valid {
        let now = Utc::now();
        let long_block = (now + ChronoDuration::seconds(60)).to_rfc3339();
        let short_block = (now + ChronoDuration::seconds(10)).to_rfc3339();

        repositories::users::record_failed_login(
            &pool,
            &attempt_key,
            &long_block,
            &short_block,
            PAIR_LONG_THRESHOLD,
            PAIR_SHORT_THRESHOLD,
        )
        .await
        .map_err(internal_error("Failed to record login attempt"))?;

        let ip_long_block =
            (now + ChronoDuration::seconds(IP_WIDE_LONG_BLOCK_SECONDS)).to_rfc3339();
        let ip_short_block =
            (now + ChronoDuration::seconds(IP_WIDE_SHORT_BLOCK_SECONDS)).to_rfc3339();
        repositories::users::record_failed_login(
            &pool,
            &ip_attempt_key,
            &ip_long_block,
            &ip_short_block,
            IP_WIDE_LONG_THRESHOLD,
            IP_WIDE_SHORT_THRESHOLD,
        )
        .await
        .map_err(internal_error("Failed to record login attempt"))?;

        return Err(api_error(StatusCode::UNAUTHORIZED, "Invalid credentials"));
    }

    // Only the pair key is cleared on success. The IP-wide counter must
    // survive: an attacker who knows one valid credential could otherwise
    // reset the spray counter at will by logging into that account.
    if attempt_record.is_some() {
        if let Err(e) = repositories::users::clear_login_attempts(&pool, &attempt_key).await {
            tracing::warn!(
                "Failed to clear login attempts for hashed key after successful login: {}",
                e
            );
        }
    }

    let user_record = user_record.expect("Successful login must have user record");
    let token = auth::create_jwt(user_record.username.clone(), user_record.role.clone())
        .map_err(internal_error("Failed to create token"))?;

    let mut headers = HeaderMap::new();
    auth::append_auth_cookie(&mut headers, auth::build_auth_cookie(&token));

    if let Ok(csrf_token) = csrf::issue_csrf_token(&user_record.username) {
        csrf::append_csrf_cookie(&mut headers, &csrf_token);
    } else {
        tracing::error!(
            "Failed to issue CSRF token for user {}",
            user_record.username
        );
        return Err(internal_error_plain("Failed to create token"));
    }

    Ok((
        headers,
        Json(LoginResponse {
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
pub async fn me(claims: auth::Claims) -> Result<(HeaderMap, Json<UserResponse>), ApiError> {
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
    jar: CookieJar,
    _csrf: csrf::CsrfGuard,
    claims: auth::Claims,
) -> (StatusCode, HeaderMap) {
    let mut tokens_to_blacklist = Vec::new();

    // 1. Extract from Header
    if let Some(auth_header) = headers.get(axum::http::header::AUTHORIZATION) {
        if let Ok(value) = auth_header.to_str() {
            if let Some(token) = value.strip_prefix("Bearer ").map(|t| t.trim()) {
                if !token.is_empty() {
                    tokens_to_blacklist.push(token.to_string());
                }
            }
        }
    }

    // 2. Extract from Cookie
    if let Some(cookie) = jar.get(auth::AUTH_COOKIE_NAME) {
        let token = cookie.value();
        // Avoid duplicates
        if !tokens_to_blacklist.contains(&token.to_string()) {
            tokens_to_blacklist.push(token.to_string());
        }
    }

    // 3. Blacklist all found tokens
    for token in tokens_to_blacklist {
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

#[cfg(test)]
mod tests;
