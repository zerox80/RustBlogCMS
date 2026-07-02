//! Regression tests locking down `CsrfGuard`'s "anonymous users have no
//! session to hijack" short-circuit (backend/src/security/csrf.rs).
//!
//! That short-circuit is only safe because every mutating route requiring
//! real authorization also independently verifies identity (either via the
//! `auth_middleware` stack on admin routes, or via the `Claims`/`OptionalClaims`
//! extractor inside the handler). If a future refactor ever let an
//! authenticated request past `CsrfGuard` without a matching double-submit
//! token, session-riding CSRF would silently reopen. These tests exercise
//! the full router (not just the extractor in isolation) so a regression
//! here fails at the same layer a real attacker would hit.

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use rust_blog_backend::{
    db::migrations::run_migrations,
    routes,
    security::{auth, csrf},
};
use sqlx::SqlitePool;
use std::env;
use std::net::SocketAddr;
use tower::ServiceExt;

fn init_test_secrets() {
    env::set_var(
        "LOGIN_ATTEMPT_SALT",
        "this_is_a_test_salt_for_login_attempts_at_least_32_chars",
    );
    let _ = rust_blog_backend::handlers::auth::init_login_attempt_salt();

    env::set_var(
        "JWT_SECRET",
        "this_is_a_test_jwt_secret_with_adequate_entropy_123_ABC_!!!",
    );
    let _ = auth::init_jwt_secret();

    env::set_var(
        "CSRF_SECRET",
        "this_is_a_very_long_secret_key_for_testing_purposes_only_at_least_32_bytes",
    );
    let _ = csrf::init_csrf_secret();
}

fn with_connect_info(mut request: Request<Body>) -> Request<Body> {
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 3000))));
    request
}

async fn setup_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    run_migrations(&pool).await.expect("run migrations");
    pool
}

/// An authenticated request (valid session cookie) that omits the CSRF
/// header/cookie entirely must be rejected with 403, never silently
/// processed as if it were anonymous.
#[tokio::test]
async fn authenticated_comment_post_without_csrf_token_is_rejected() {
    init_test_secrets();
    let pool = setup_pool().await;
    let app = routes::create_routes(pool.clone(), "test_uploads".to_string()).with_state(pool);

    let session_token = auth::create_jwt("alice".to_string(), "user".to_string())
        .expect("failed to create session token");

    let response = app
        .oneshot(with_connect_info(
            Request::builder()
                .uri("/api/posts/some-post/comments")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", format!("ltcms_session={session_token}"))
                .body(Body::from(r#"{"content":"hello"}"#))
                .unwrap(),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// An authenticated request whose CSRF header doesn't match its CSRF cookie
/// (double-submit mismatch) must be rejected with 403.
#[tokio::test]
async fn authenticated_comment_post_with_mismatched_csrf_header_is_rejected() {
    init_test_secrets();
    let pool = setup_pool().await;
    let app = routes::create_routes(pool.clone(), "test_uploads".to_string()).with_state(pool);

    let session_token = auth::create_jwt("alice".to_string(), "user".to_string())
        .expect("failed to create session token");
    let csrf_cookie_token = csrf::issue_csrf_token("alice").expect("failed to issue csrf token");

    let response = app
        .oneshot(with_connect_info(
            Request::builder()
                .uri("/api/posts/some-post/comments")
                .method("POST")
                .header("Content-Type", "application/json")
                .header(
                    "Cookie",
                    format!("ltcms_session={session_token}; ltcms_csrf={csrf_cookie_token}"),
                )
                .header("x-csrf-token", "attacker-supplied-value")
                .body(Body::from(r#"{"content":"hello"}"#))
                .unwrap(),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// A CSRF token issued for one user must not validate a request
/// authenticated as a different user, even if header and cookie match
/// each other (double-submit alone is not sufficient -- the token must also
/// be bound to the authenticated identity).
#[tokio::test]
async fn authenticated_comment_post_with_csrf_token_for_different_user_is_rejected() {
    init_test_secrets();
    let pool = setup_pool().await;
    let app = routes::create_routes(pool.clone(), "test_uploads".to_string()).with_state(pool);

    let session_token = auth::create_jwt("alice".to_string(), "user".to_string())
        .expect("failed to create session token");
    // Token issued for a different account than the one authenticating.
    let csrf_token_for_bob = csrf::issue_csrf_token("bob").expect("failed to issue csrf token");

    let response = app
        .oneshot(with_connect_info(
            Request::builder()
                .uri("/api/posts/some-post/comments")
                .method("POST")
                .header("Content-Type", "application/json")
                .header(
                    "Cookie",
                    format!("ltcms_session={session_token}; ltcms_csrf={csrf_token_for_bob}"),
                )
                .header("x-csrf-token", csrf_token_for_bob.as_str())
                .body(Body::from(r#"{"content":"hello"}"#))
                .unwrap(),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// Sanity check for the positive path: a valid, matching CSRF token for the
/// authenticated user must pass the guard and reach the handler body (which
/// then 404s because the referenced post doesn't exist -- proving the guard,
/// not the router, was the thing rejecting the earlier requests).
#[tokio::test]
async fn authenticated_comment_post_with_valid_csrf_token_passes_guard() {
    init_test_secrets();
    let pool = setup_pool().await;
    let app = routes::create_routes(pool.clone(), "test_uploads".to_string()).with_state(pool);

    let session_token = auth::create_jwt("alice".to_string(), "user".to_string())
        .expect("failed to create session token");
    let csrf_token = csrf::issue_csrf_token("alice").expect("failed to issue csrf token");

    let response = app
        .oneshot(with_connect_info(
            Request::builder()
                .uri("/api/posts/nonexistent-post/comments")
                .method("POST")
                .header("Content-Type", "application/json")
                .header(
                    "Cookie",
                    format!("ltcms_session={session_token}; ltcms_csrf={csrf_token}"),
                )
                .header("x-csrf-token", csrf_token.as_str())
                .body(Body::from(r#"{"content":"hello"}"#))
                .unwrap(),
        ))
        .await
        .unwrap();

    // Guard passed (not 401/403); handler ran and found no such post.
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
