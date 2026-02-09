//! CORS Configuration and Origin Validation
//!
//! This module provides utilities for configuring Cross-Origin Resource Sharing (CORS).
//! It ensures that only trusted domains can interact with the API from a browser,
//! protecting against CSRF and cross-site data leakage.

use axum::http::HeaderValue;

/// Default origins permitted during local development.
pub const DEV_DEFAULT_FRONTEND_ORIGINS: &[&str] =
    &["http://localhost:5173", "http://localhost:3000"];

/// Parses and validates a collection of allowed CORS origins.
///
/// This function converts raw strings into Axum-compatible `HeaderValue` objects while:
/// 1. **Filtering**: Removing empty or malformed strings.
/// 2. **Protocol Enforcement**: Ensuring all origins use `http://` or `https://`.
/// 3. **Validation**: verifies the origin is a valid URL to prevent header injection.
pub fn parse_allowed_origins<'a, I>(origins: I) -> Vec<HeaderValue>
where
    I: IntoIterator<Item = &'a str>,
{
    origins
        .into_iter()
        .filter_map(|origin| {
            let trimmed = origin.trim();

            if trimmed.is_empty() {
                return None;
            }

            // SECURITY: Only allow explicit web protocols.
            // Prevents file:// or other unexpected protocols from being whitelisted.
            if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
                tracing::warn!(
                    "Ignoring invalid origin (must start with http:// or https://): '{trimmed}'"
                );
                return None;
            }

            // ENFORCEMENT: Verify the string is at least a valid URL to avoid corruption.
            if let Err(e) = url::Url::parse(trimmed) {
                tracing::warn!("Ignoring malformed origin URL '{trimmed}': {e}");
                return None;
            }

            // Final conversion to Axum-compatible HeaderValue
            match HeaderValue::from_str(trimmed) {
                Ok(value) => Some(value),
                Err(err) => {
                    tracing::warn!("Ignoring invalid origin header value '{trimmed}': {err}");
                    return None;
                }
            }
        })
        .collect()
}
