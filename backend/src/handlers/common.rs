//! Helpers shared across handler modules.
//!
//! These were previously duplicated verbatim in `site_pages` and
//! `site_posts`; keeping a single copy here guarantees the two admin APIs
//! stay consistent in how they authorize and how they translate database
//! failures into HTTP responses.

use crate::models::{api_error, bad_request, forbidden, internal_error_plain, not_found, ApiError};
use crate::security::auth;
use axum::http::StatusCode;

/// Ensures the current user has administrative privileges.
pub fn ensure_admin(claims: &auth::Claims) -> Result<(), ApiError> {
    if claims.role != "admin" {
        Err(forbidden("Insufficient permissions"))
    } else {
        Ok(())
    }
}

/// Maps SQLx database errors to user-facing HTTP responses.
///
/// - `RowNotFound` → 404 with the given context ("Site page not found").
/// - `Protocol` → 400; the repository layer raises these for validation-style
///   failures with messages written for end users (e.g. slug length).
/// - Unique constraint violations → 409 Conflict.
/// - Everything else → 500 with a generic message; the real error is logged
///   but never sent to the client.
pub fn map_sqlx_error(err: sqlx::Error, context: &str) -> ApiError {
    match err {
        sqlx::Error::RowNotFound => not_found(format!("{context} not found")),
        sqlx::Error::Protocol(e) => bad_request(e.to_string()),
        sqlx::Error::Database(db_err) => {
            if db_err.is_unique_violation() {
                api_error(
                    StatusCode::CONFLICT,
                    db_err
                        .constraint()
                        .map(|c| format!("Duplicate value violates unique constraint '{c}'"))
                        .unwrap_or_else(|| {
                            "Duplicate value violates unique constraint".to_string()
                        }),
                )
            } else {
                tracing::error!("{context}: database error: {db_err}");
                internal_error_plain("Database error")
            }
        }
        other => {
            tracing::error!("{context}: unexpected database error: {other}");
            internal_error_plain("Database error")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claims(role: &str) -> auth::Claims {
        auth::Claims {
            sub: "someone".to_string(),
            role: role.to_string(),
            exp: usize::MAX,
        }
    }

    #[test]
    fn ensure_admin_accepts_admin_and_rejects_others() {
        assert!(ensure_admin(&claims("admin")).is_ok());

        let (status, _) = ensure_admin(&claims("user")).unwrap_err();
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[test]
    fn map_sqlx_error_never_leaks_unexpected_error_details() {
        let (status, axum::Json(body)) = map_sqlx_error(sqlx::Error::PoolTimedOut, "Site page");
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body.error, "Database error");
    }

    #[test]
    fn map_sqlx_error_translates_row_not_found() {
        let (status, axum::Json(body)) = map_sqlx_error(sqlx::Error::RowNotFound, "Site post");
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body.error, "Site post not found");
    }
}
