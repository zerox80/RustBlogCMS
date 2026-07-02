//! CORS Configuration and Origin Validation
//!
//! This module provides utilities for configuring Cross-Origin Resource Sharing (CORS).
//! It ensures that only trusted domains can interact with the API from a browser,
//! protecting against CSRF and cross-site data leakage.

use axum::http::HeaderValue;
use std::sync::OnceLock;

/// Default origins permitted during local development.
pub const DEV_DEFAULT_FRONTEND_ORIGINS: &[&str] =
    &["http://localhost:5173", "http://localhost:3000"];

/// Origins (normalized, lowercase, no trailing slash) that browsers are
/// allowed to send state-changing requests from. Shared with the CSRF layer
/// so its anonymous-request Origin check accepts exactly the same set of
/// cross-origin frontends that CORS does, instead of maintaining two lists.
static ALLOWED_BROWSER_ORIGINS: OnceLock<Vec<String>> = OnceLock::new();

/// Normalizes an origin string for comparison: trimmed, lowercased, and
/// without a trailing slash (browsers never send one in `Origin`, but a
/// human editing `CORS_ALLOWED_ORIGINS` easily adds one).
pub fn normalize_origin(origin: &str) -> String {
    origin.trim().trim_end_matches('/').to_ascii_lowercase()
}

/// Stores the configured allowed origins for process-wide lookup.
/// Called once at startup; later calls are ignored (first write wins).
pub fn init_allowed_browser_origins<'a, I>(origins: I)
where
    I: IntoIterator<Item = &'a str>,
{
    let normalized = origins.into_iter().map(normalize_origin).collect();
    let _ = ALLOWED_BROWSER_ORIGINS.set(normalized);
}

/// Returns the configured allowed origins, or an empty slice if
/// `init_allowed_browser_origins` has not run (e.g. in unit tests).
pub fn allowed_browser_origins() -> &'static [String] {
    ALLOWED_BROWSER_ORIGINS
        .get()
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

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
                None
            } else if let Err(e) = url::Url::parse(trimmed) {
                // ENFORCEMENT: Verify the string is at least a valid URL to avoid corruption.
                tracing::warn!("Ignoring malformed origin URL '{trimmed}': {e}");
                None
            } else {
                // Final conversion to Axum-compatible HeaderValue
                match HeaderValue::from_str(trimmed) {
                    Ok(value) => Some(value),
                    Err(err) => {
                        tracing::warn!("Ignoring invalid origin header value '{trimmed}': {err}");
                        None
                    }
                }
            }
        })
        .collect()
}
