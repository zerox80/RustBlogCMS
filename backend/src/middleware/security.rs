//! Security Headers and Request Sanitization Middleware
//!
//! This module implements defense-in-depth security measures at the HTTP layer.
//! It ensures all responses carry strict security headers (CSP, HSTS, etc.)
//! and sanitizes incoming requests to prevent header-based spoofing attacks.

use axum::{
    extract::{ConnectInfo, Request},
    http::{
        header::{
            CACHE_CONTROL, CONTENT_SECURITY_POLICY, EXPIRES, PRAGMA, STRICT_TRANSPORT_SECURITY,
            X_CONTENT_TYPE_OPTIONS, X_FRAME_OPTIONS,
        },
        HeaderMap, HeaderName, HeaderValue, Method,
    },
    middleware::Next,
    response::Response,
};
use std::{
    env,
    net::{IpAddr, SocketAddr},
    sync::OnceLock,
};
use tower_governor::{errors::GovernorError, key_extractor::KeyExtractor};

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

/// SECURITY: This function must only ever trust IP data that a proxy we
/// control *overwrites* or *appends*, never data a client can freely set.
///
/// - `X-Real-IP` is safe first choice: the bundled nginx config always sets
///   it via `proxy_set_header X-Real-IP $remote_addr;`, which *overwrites*
///   any value the client sent -- a client-supplied `X-Real-IP` never
///   survives the hop.
/// - `X-Forwarded-For` is only safe if we read the *last* (rightmost) entry.
///   nginx's `$proxy_add_x_forwarded_for` *appends* its own view of
///   `$remote_addr` to whatever the client already supplied, so the first
///   entry is fully attacker-controlled (e.g. `X-Forwarded-For: 1.2.3.4`
///   arrives as `1.2.3.4, <real-ip>`). Trusting the first hop let an
///   attacker forge an arbitrary IP and defeat IP-keyed rate limiting
///   (login lockout, guest comment throttling) by rotating fake values.
fn trusted_client_ip_from_headers(headers: &HeaderMap) -> Option<IpAddr> {
    headers
        .get(X_REAL_IP_HEADER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().parse::<IpAddr>().ok())
        .or_else(|| {
            headers
                .get(X_FORWARDED_FOR_HEADER)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| {
                    value
                        .split(',')
                        .map(str::trim)
                        .rfind(|segment| !segment.is_empty())
                        .and_then(|segment| segment.parse::<IpAddr>().ok())
                })
        })
}

fn resolve_client_ip(headers: &HeaderMap, fallback: IpAddr, trust_proxy_headers: bool) -> IpAddr {
    if trust_proxy_headers {
        trusted_client_ip_from_headers(headers).unwrap_or(fallback)
    } else {
        fallback
    }
}

/// Whether proxy-supplied IP headers are trusted for this process.
///
/// The environment variable cannot change for the lifetime of the process, so
/// it is read once and cached instead of hitting the environment on every
/// request that resolves a client IP.
pub fn trust_proxy_ip_headers() -> bool {
    static TRUST_PROXY_IP_HEADERS: OnceLock<bool> = OnceLock::new();
    *TRUST_PROXY_IP_HEADERS.get_or_init(|| parse_env_bool("TRUST_PROXY_IP_HEADERS", false))
}

/// Resolves the effective client IP for rate limiting and audit purposes.
///
/// When proxy headers are not explicitly trusted, the socket peer address is used.
/// When they are trusted, `X-Real-IP` is preferred (always overwritten by our
/// nginx, never client-controlled), falling back to the last (rightmost) hop
/// of `X-Forwarded-For` -- the entry appended by our own proxy.
pub fn extract_client_ip(headers: &HeaderMap, fallback: IpAddr) -> IpAddr {
    resolve_client_ip(headers, fallback, trust_proxy_ip_headers())
}

/// Rate-limit key extractor that shares the application's hardened client-IP
/// policy. Unlike `SmartIpKeyExtractor`, it never trusts the first
/// `X-Forwarded-For` value, which is client-controlled with nginx's
/// `$proxy_add_x_forwarded_for` configuration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TrustedClientIpKeyExtractor;

impl KeyExtractor for TrustedClientIpKeyExtractor {
    type Key = IpAddr;

    fn extract<T>(&self, request: &axum::http::Request<T>) -> Result<Self::Key, GovernorError> {
        let fallback = request
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|connect_info| connect_info.0.ip())
            .or_else(|| request.extensions().get::<SocketAddr>().map(SocketAddr::ip))
            .ok_or(GovernorError::UnableToExtractKey)?;

        Ok(resolve_client_ip(
            request.headers(),
            fallback,
            trust_proxy_ip_headers(),
        ))
    }
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
/// - **HSTS**: Enforce HTTPS for a year (only if ENABLE_HSTS=true is set explicitly).
/// - **X-Content-Type-Options**: Prevent MIME-sniffing.
/// - **X-Frame-Options**: Prevent clickjacking.
/// - **Referrer-Policy**: Protect user privacy during navigation.
/// - **Permissions-Policy**: Disable unused browser features (geolocation, etc.).
pub async fn security_headers(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();

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
    // SECURITY: Do not auto-detect HTTPS via x-forwarded-proto. When
    // TRUST_PROXY_IP_HEADERS=true, that header is not stripped before this
    // middleware runs, so a client with direct access to the backend port
    // could set it themselves. Require deployments behind a TLS-terminating
    // proxy to opt in explicitly via ENABLE_HSTS instead.
    let hsts_enabled = parse_env_bool("ENABLE_HSTS", false);
    if hsts_enabled {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_trusted_client_ip_prefers_x_real_ip() {
        let mut headers = HeaderMap::new();
        headers.insert(X_REAL_IP_HEADER, HeaderValue::from_static("203.0.113.7"));
        headers.insert(
            X_FORWARDED_FOR_HEADER,
            HeaderValue::from_static("9.9.9.9, 203.0.113.7"),
        );

        assert_eq!(
            trusted_client_ip_from_headers(&headers),
            Some(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 7)))
        );
    }

    #[test]
    fn test_trusted_client_ip_falls_back_to_x_forwarded_for_last_hop() {
        let mut headers = HeaderMap::new();
        headers.insert(
            X_FORWARDED_FOR_HEADER,
            HeaderValue::from_static("198.51.100.24, 10.0.0.10"),
        );

        // The last (rightmost) entry is the one our own proxy appended --
        // the first entry is fully attacker-controlled and must NOT be trusted.
        assert_eq!(
            trusted_client_ip_from_headers(&headers),
            Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 10)))
        );
    }

    #[test]
    fn test_trusted_client_ip_ignores_spoofed_first_hop() {
        // Attacker sends a forged X-Forwarded-For; our nginx appends the real
        // peer address as the last entry via $proxy_add_x_forwarded_for.
        let mut headers = HeaderMap::new();
        headers.insert(
            X_FORWARDED_FOR_HEADER,
            HeaderValue::from_static("6.6.6.6, 198.51.100.1"),
        );

        assert_ne!(
            trusted_client_ip_from_headers(&headers),
            Some(IpAddr::V4(Ipv4Addr::new(6, 6, 6, 6)))
        );
        assert_eq!(
            trusted_client_ip_from_headers(&headers),
            Some(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 1)))
        );
    }

    #[test]
    fn resolve_client_ip_uses_the_proxy_overwritten_ip_only_when_enabled() {
        let mut headers = HeaderMap::new();
        headers.insert(X_REAL_IP_HEADER, HeaderValue::from_static("203.0.113.7"));
        let fallback = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));

        assert_eq!(
            resolve_client_ip(&headers, fallback, true),
            IpAddr::V4(Ipv4Addr::new(203, 0, 113, 7))
        );
        assert_eq!(resolve_client_ip(&headers, fallback, false), fallback);
    }
}
