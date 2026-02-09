//! Authentication Middleware Layer
//!
//! This module provides the primary filter for protecting API routes.
//! It acts as a gatekeeper, ensuring every request to a protected resource
//! carries a valid, non-revoked JWT token.
//!
//! # Extractor Logic
//! The middleware doesn't just block unauthorized requests; it also
//! extracts identity information (Claims) and places it into Axum's
//! request extensions. This allows downstream handlers to simply
//! use the `Claims` extractor to identify the user and their role.

use crate::{repositories, security::auth};
use axum::{http::StatusCode, Json};

/// Middleware to enforce authentication on a per-route or per-router basis.
///
/// Process Flow:
/// 1. **Extraction**: Checks both Authorization header and ltcms_session cookie.
/// 2. **Verification**: Validates the JWT signature and expiration.
/// 3. **Revocation Check**: Queries the database to ensure the token isn't blacklisted (e.g., after logout).
/// 4. **Injection**: Places the verified Claims into the request lifecycle.
pub async fn auth_middleware(
    axum::extract::State(pool): axum::extract::State<crate::db::DbPool>,
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, (StatusCode, Json<crate::models::ErrorResponse>)> {
    // Step 1: Token Extraction
    // Checks for 'Bearer' token or 'ltcms_session' fallback cookie.
    let token = auth::extract_token(request.headers()).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(crate::models::ErrorResponse {
                error: "Missing authentication token".to_string(),
            }),
        )
    })?;

    // Step 2: Cryptographic Verification
    // Validates the HMAC signature and ensured the token has not expired.
    let claims = auth::verify_jwt(&token).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(crate::models::ErrorResponse {
                error: format!("Invalid token: {}", e),
            }),
        )
    })?;

    // Step 3: Revocation Check (Blacklist)
    // Even a cryptographically valid token is rejected if the user has logged out.
    if let Ok(true) = repositories::token_blacklist::is_token_blacklisted(&pool, &token).await {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(crate::models::ErrorResponse {
                error: "Token has been revoked".to_string(),
            }),
        ));
    }

    // Step 4: Extension Injection
    // Makes the user's role and identity available to all subsequent middleware/handlers.
    request.extensions_mut().insert(claims);

    // Call the next item in the middleware chain
    Ok(next.run(request).await)
}
