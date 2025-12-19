//! Security Headers and Request Sanitization Middleware
//!
//! This module implements defense-in-depth security measures at the HTTP layer.
//! It ensures all responses carry strict security headers (CSP, HSTS, etc.)
//! and sanitizes incoming requests to prevent header-based spoofing attacks.

use axum::{
    extract::Request,
    http::{
        header::{
            CACHE_CONTROL, CONTENT_SECURITY_POLICY, EXPIRES, PRAGMA, STRICT_TRANSPORT_SECURITY,
            X_CONTENT_TYPE_OPTIONS, X_FRAME_OPTIONS,
        },
        HeaderName, HeaderValue, Method,
    },
    middleware::Next,
    response::Response,
};
use std::env;

// Custom HTTP header constants for security policies
const PERMISSIONS_POLICY: HeaderName = HeaderName::from_static("permissions-policy");
const REFERRER_POLICY: HeaderName = HeaderName::from_static("referrer-policy");
const X_XSS_PROTECTION: HeaderName = HeaderName::from_static("x-xss-protection");

// Forwarded header constants for proxy handling
const FORWARDED_HEADER: HeaderName = HeaderName::from_static("forwarded");
const X_FORWARDED_FOR_HEADER: HeaderName = HeaderName::from_static("x-forwarded-for");
const X_FORWARDED_PROTO_HEADER: HeaderName = HeaderName::from_static("x-forwarded-proto");
const X_FORWARDED_HOST_HEADER: HeaderName = HeaderName::from_static("x-forwarded-host");
const X_REAL_IP_HEADER: HeaderName = HeaderName::from_static("x-real-ip");

/// Helper to parse environment variables as boolean flags.
/// Supports common truthy/falsy strings like '1', 'true', 'yes', 'on', '0', 'false', etc.
pub fn parse_env_bool(key: &str, default: bool) -> bool {
    env::var(key)
        .ok()
        .and_then(|value| {
            match value.trim().to_ascii_lowercase().as_str() {
                "1" | "true" | "yes" | "on" => Some(true),
                "0" | "false" | "no" | "off" => Some(false),
                _ => {
                    tracing::warn!(key = %key, value = %value, "Invalid boolean env value; using default");
                    None
                }
            }
        })
        .unwrap_or(default)
}

/// Middleware to strip potentially spoofable forwarded headers from incoming requests.
///
/// SECURITY: This prevents "Client IP Spoofing" by removing headers like `X-Forwarded-For`
/// before the request reaches handlers or rate-limiters. In a production environment,
/// these should be re-injected ONLY by a trusted reverse proxy (like Nginx).
pub async fn strip_untrusted_forwarded_headers(mut request: Request, next: Next) -> Response {
    {
        let headers = request.headers_mut();

        // Remove all potentially spoofable forwarded headers to establish a clean slate
        headers.remove(FORWARDED_HEADER);
        headers.remove(X_FORWARDED_FOR_HEADER);
        headers.remove(X_FORWARDED_PROTO_HEADER);
        headers.remove(X_FORWARDED_HOST_HEADER);
        headers.remove(X_REAL_IP_HEADER);
    }

    next.run(request).await
}

/// Middleware to add security and privacy headers to all HTTP responses.
///
/// Implementations:
/// - **Cache-Control**: Dynamic based on path (public vs sensitive).
/// - **CSP**: Strict policy to prevent XSS and data injection.
/// - **HSTS**: Enforce HTTPS for a year (only if request arrived via HTTPS).
/// - **X-Content-Type-Options**: Prevent MIME-sniffing.
/// - **X-Frame-Options**: Prevent clickjacking.
/// - **Referrer-Policy**: Protect user privacy during navigation.
/// - **Permissions-Policy**: Disable unused browser features (geolocation, etc.).
pub async fn security_headers(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();

    // Detect if request is over HTTPS for HSTS header
    // We check the protocol usually injected by a trusted proxy
    let is_https = request
        .headers()
        .get("x-forwarded-proto") // Note: This assumes strip_untrusted was ALREADY run and proxy injected it
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "https")
        .unwrap_or(false);

    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Step 1: Configure cache control based on endpoint type
    // Public endpoints can be cached to improve performance, sensitive endpoints cannot.
    let cacheable = method == Method::GET
        && (path == "/api/tutorials"
            || path.starts_with("/api/tutorials/")
            || path.starts_with("/api/public/"));

    if cacheable {
        // Optimized caching for public read-only endpoints (5 minute TTL)
        headers.insert(
            CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=300, stale-while-revalidate=60"),
        );
        headers.remove(PRAGMA);
        headers.remove(EXPIRES);
    } else {
        // Strict no-cache for sensitive endpoints (auth, admin, comments, etc.)
        headers.insert(
            CACHE_CONTROL,
            HeaderValue::from_static("no-store, no-cache, must-revalidate"),
        );
        headers.insert(PRAGMA, HeaderValue::from_static("no-cache"));
        headers.insert(EXPIRES, HeaderValue::from_static("0"));
    }

    // Step 2: Content Security Policy (CSP)
    // Note: 'unsafe-inline' for style-src is currently required for syntax highlighting and math rendering.
    let csp = if cfg!(debug_assertions) {
        // Development CSP - allows local hot reloading ws/wss
        "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com data:; img-src 'self' data: blob:; connect-src 'self' ws: wss:; object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'; upgrade-insecure-requests;"
    } else {
        // Production CSP - restricted connections
        "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com data:; img-src 'self' data: blob:; connect-src 'self'; object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'; upgrade-insecure-requests;"
    };

    headers.insert(CONTENT_SECURITY_POLICY, HeaderValue::from_static(csp));

    // Step 3: Transport Security (HSTS)
    if is_https {
        headers.insert(
            STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
        );
    }

    // Step 4: Defense-in-depth headers
    // Prevent MIME sniffing
    headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));

    // Prevent Clickjacking (framing)
    headers.insert(X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));

    // Privacy logic
    headers.insert(REFERRER_POLICY, HeaderValue::from_static("no-referrer"));

    // Opt-out of browser privacy-invasive features
    headers.insert(
        PERMISSIONS_POLICY,
        HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
    );

    // Legacy XSS filter (redundant with CSP but kept as '0' to avoid browser interference)
    headers.insert(X_XSS_PROTECTION, HeaderValue::from_static("0"));

    response
}
