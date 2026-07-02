//! Shared HTTP error helpers.
//!
//! Every handler returns errors as `(StatusCode, Json<ErrorResponse>)`. These
//! helpers centralize the construction of that tuple so handlers don't repeat
//! the same eight-line `map_err` blocks for logging and response shaping.
//!
//! # Usage
//! ```rust,ignore
//! repositories::comments::get_comment(&pool, &id)
//!     .await
//!     .map_err(internal_error("Failed to fetch comment"))?;
//!
//! return Err(not_found("Comment not found"));
//! ```

use super::ErrorResponse;
use axum::{http::StatusCode, Json};
use std::fmt::Display;

/// The uniform error type returned by all HTTP handlers.
pub type ApiError = (StatusCode, Json<ErrorResponse>);

/// Builds an [`ApiError`] with an arbitrary status code and public message.
pub fn api_error(status: StatusCode, message: impl Into<String>) -> ApiError {
    (
        status,
        Json(ErrorResponse {
            error: message.into(),
        }),
    )
}

/// 400 Bad Request with the given public message.
pub fn bad_request(message: impl Into<String>) -> ApiError {
    api_error(StatusCode::BAD_REQUEST, message)
}

/// 403 Forbidden with the given public message.
pub fn forbidden(message: impl Into<String>) -> ApiError {
    api_error(StatusCode::FORBIDDEN, message)
}

/// 404 Not Found with the given public message.
pub fn not_found(message: impl Into<String>) -> ApiError {
    api_error(StatusCode::NOT_FOUND, message)
}

/// 500 Internal Server Error that logs the underlying cause.
///
/// Returns a closure for use with `map_err`: the real error is logged with
/// the given context while the client only ever sees the generic public
/// message, so internal details (SQL errors, file paths) never leak.
pub fn internal_error<E: Display>(public_message: &'static str) -> impl FnOnce(E) -> ApiError {
    move |err| {
        tracing::error!("{public_message}: {err}");
        api_error(StatusCode::INTERNAL_SERVER_ERROR, public_message)
    }
}

/// 500 Internal Server Error without an underlying error value to log.
pub fn internal_error_plain(public_message: impl Into<String>) -> ApiError {
    api_error(StatusCode::INTERNAL_SERVER_ERROR, public_message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helpers_produce_expected_status_and_body() {
        let (status, Json(body)) = bad_request("bad input");
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body.error, "bad input");

        let (status, Json(body)) = not_found("missing");
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body.error, "missing");

        let (status, Json(body)) = forbidden("nope");
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(body.error, "nope");
    }

    #[test]
    fn internal_error_hides_cause_from_client() {
        let map = internal_error("Failed to do the thing");
        let (status, Json(body)) = map(std::io::Error::other("secret detail"));
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body.error, "Failed to do the thing");
        assert!(!body.error.contains("secret detail"));
    }
}
