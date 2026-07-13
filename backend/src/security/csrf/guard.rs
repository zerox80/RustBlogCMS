use super::*;

/// AXUM extractor for CSRF protection.
///
/// This extractor validates CSRF tokens for state-changing HTTP methods.
/// Safe methods (GET, HEAD, OPTIONS, TRACE) are automatically allowed.
///
/// # Validation Process
/// 1. Skip validation for safe HTTP methods
/// 2. Ensure user is authenticated (extract Claims)
/// 3. Extract token from x-csrf-token header
/// 4. Extract token from cookie
/// 5. Verify header and cookie tokens match (double-submit pattern)
/// 6. Validate token signature and binding to user
///
/// # Usage
/// ```rust,ignore
/// use axum::{Router, routing::post, middleware};
/// use rust_blog_backend::security::csrf::CsrfGuard;
/// async fn handler() {}
///
/// let app = Router::new()
///     .route("/api/resource", post(handler))
///     .route_layer(middleware::from_extractor::<CsrfGuard>());
/// ```
///
/// # Security
/// - Double-submit cookie pattern (cookie + header)
/// - Per-user token binding
/// - HMAC signature verification
/// - Expiration enforcement
///
/// # Errors
/// Returns 403 Forbidden if:
/// - CSRF token header is missing
/// - CSRF cookie is missing
/// - Header and cookie tokens don't match
/// - Token validation fails (expired, wrong user, invalid signature)
/// - Anonymous request carries a cross-site Origin/Referer
pub struct CsrfGuard;

/// Validates that a state-changing request from an anonymous client was not
/// issued cross-site by a hostile page in the victim's browser.
///
/// Browsers attach an `Origin` header to every cross-origin (and same-origin
/// non-GET) fetch, and it cannot be forged or suppressed from a web page.
///
/// The policy:
/// - No `Origin` and no `Referer`: allow. This is a non-browser client
///   (curl, mobile app, integration test); those carry no ambient browser
///   state that CSRF could abuse.
/// - Origin present: its hostname must match the request's `Host` header
///   (hostname comparison -- an attacker cannot serve content under the
///   victim host's name), or the full origin must be in the configured
///   CORS allowlist (separate-frontend deployments).
/// - Anything else (mismatch, unparseable, the literal "null"): reject.
pub(super) fn validate_browser_origin(headers: &HeaderMap) -> Result<(), String> {
    // Prefer Origin; fall back to the origin part of Referer for the rare
    // legitimate clients that send only the latter.
    let origin_value = headers
        .get(axum::http::header::ORIGIN)
        .or_else(|| headers.get(axum::http::header::REFERER))
        .and_then(|value| value.to_str().ok());

    let origin_raw = match origin_value {
        Some(value) => value.trim(),
        None => return Ok(()),
    };

    let parsed = url::Url::parse(origin_raw)
        .map_err(|_| "Cross-origin request blocked: invalid Origin".to_string())?;
    let origin_host = parsed
        .host_str()
        .ok_or_else(|| "Cross-origin request blocked: invalid Origin".to_string())?
        .to_ascii_lowercase();

    // Same-host check against the Host header (hostname without port).
    if let Some(request_host) = headers
        .get(axum::http::header::HOST)
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            let value = value.trim();
            // Bracketed IPv6 literals ("[::1]:8080") keep their brackets in
            // url::Url::host_str too, so compare including brackets.
            if let Some(end) = value.strip_prefix('[').and_then(|_| value.find(']')) {
                value[..=end].to_ascii_lowercase()
            } else {
                value
                    .rsplit_once(':')
                    .map_or(value, |(host, _port)| host)
                    .to_ascii_lowercase()
            }
        })
    {
        if !request_host.is_empty() && origin_host == request_host {
            return Ok(());
        }
    }

    // Allowlist check: the exact origin (scheme://host[:port]) must be one
    // of the configured cross-origin frontends.
    let normalized_origin =
        crate::middleware::cors::normalize_origin(parsed.origin().ascii_serialization().as_str());
    if crate::middleware::cors::allowed_browser_origins().contains(&normalized_origin) {
        return Ok(());
    }

    Err("Cross-origin request blocked".to_string())
}

impl<S> FromRequestParts<S> for CsrfGuard
where
    S: Send + Sync,
    crate::db::DbPool: axum::extract::FromRef<S>,
{
    type Rejection = (StatusCode, Json<ErrorResponse>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Step 1: Method Filter. CSRF is only required for state-changing operations.
        if matches!(
            parts.method,
            Method::GET | Method::HEAD | Method::OPTIONS | Method::TRACE
        ) {
            return Ok(Self);
        }

        // Step 2: Authenticated Check. CSRF protects sessions, so we first check the user's identity.
        let claims_result = if let Some(existing) = parts.extensions.get::<auth::Claims>() {
            Ok(existing.clone())
        } else {
            auth::Claims::from_request_parts(parts, _state).await
        };

        let claims = match claims_result {
            Ok(claims) => {
                // User is logged in -> Enforce strict CSRF checks.
                parts.extensions.insert(claims.clone());
                claims
            }
            Err(_) => {
                // Anonymous user -> no session cookie to ride, so the full
                // double-submit token check does not apply. But anonymous
                // endpoints (e.g. guest comments) can still be driven from a
                // third-party page in the victim's browser, so enforce a
                // browser-origin check: if the request carries an Origin (or
                // Referer), it must be same-host or a configured frontend.
                if let Err(reason) = validate_browser_origin(&parts.headers) {
                    return Err((StatusCode::FORBIDDEN, Json(ErrorResponse { error: reason })));
                }
                return Ok(Self);
            }
        };

        // Step 3: Extract tokens from both submission channels.
        let header_value = parts
            .headers
            .get(HeaderName::from_static(CSRF_HEADER_NAME))
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| {
                (
                    StatusCode::FORBIDDEN,
                    Json(ErrorResponse {
                        error: "Missing CSRF token header".to_string(),
                    }),
                )
            })?;

        let jar = CookieJar::from_headers(&parts.headers);
        let cookie = jar.get(CSRF_COOKIE_NAME).ok_or_else(|| {
            (
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    error: "Missing CSRF cookie".to_string(),
                }),
            )
        })?;

        // Step 4: Double-Submit Validation. Ensure the tokens match.
        if cookie.value() != header_value {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    error: "CSRF token mismatch".to_string(),
                }),
            ));
        }

        // Step 5: Master Validation. Verify signature, expiration, and user binding.
        validate_csrf_token(header_value, &claims.sub)
            .map_err(|err| (StatusCode::FORBIDDEN, Json(ErrorResponse { error: err })))?;

        Ok(Self)
    }
}
