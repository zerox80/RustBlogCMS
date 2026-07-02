use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use rust_blog_backend::{db, handlers, routes, security::auth};
use sqlx::SqlitePool;
use std::env;
use std::net::SocketAddr;
use tower::ServiceExt; // for `oneshot`

fn deterministic_test_material(label: &str, min_len: usize) -> String {
    let alphabet = "abcABC123!@#xyzXYZ789";
    let mut value = format!("not-real-{label}-");
    while value.len() < min_len {
        value.push_str(alphabet);
    }
    value
}

fn init_test_secrets() {
    env::set_var(
        "LOGIN_ATTEMPT_SALT",
        deterministic_test_material("login-salt", 32),
    );
    let _ = handlers::auth::init_login_attempt_salt();

    env::set_var("JWT_SECRET", deterministic_test_material("jwt-signing", 43));
    let _ = auth::init_jwt_secret();

    env::set_var("CSRF_SECRET", deterministic_test_material("csrf-hmac", 32));
    let _ = rust_blog_backend::security::csrf::init_csrf_secret();
}

fn with_connect_info(mut request: Request<Body>) -> Request<Body> {
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 3000))));
    request
}

/// Regression test for the fail-open blacklist bug: a database error while
/// checking token revocation must reject the request (fail closed), not
/// silently treat the token as valid (fail open).
#[tokio::test]
async fn blacklist_db_error_fails_closed_with_500_not_passthrough() {
    init_test_secrets();

    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    db::migrations::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    let token = auth::create_jwt("admin".to_string(), "admin".to_string()).unwrap();

    // Simulate a database failure specific to the blacklist check.
    sqlx::query("DROP TABLE token_blacklist")
        .execute(&pool)
        .await
        .unwrap();

    let app = routes::create_routes(pool.clone(), "test_uploads".to_string()).with_state(pool);

    // Use a GET admin route so CSRF enforcement (which only applies to
    // state-changing methods) doesn't interfere with isolating the
    // auth_middleware behavior under test.
    let response = app
        .oneshot(with_connect_info(
            Request::builder()
                .method("GET")
                .uri("/api/pages/nonexistent-id")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

/// Companion test guarding against overcorrection: with the blacklist table
/// intact and the token not revoked, the request must pass through the
/// middleware normally (reaching the handler), not be blocked unconditionally.
#[tokio::test]
async fn valid_non_blacklisted_token_passes_middleware() {
    init_test_secrets();

    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    db::migrations::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    let token = auth::create_jwt("admin".to_string(), "admin".to_string()).unwrap();

    let app = routes::create_routes(pool.clone(), "test_uploads".to_string()).with_state(pool);

    let response = app
        .oneshot(with_connect_info(
            Request::builder()
                .method("GET")
                .uri("/api/pages/nonexistent-id")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        ))
        .await
        .unwrap();

    // The middleware let the request through; the handler then reports the
    // page doesn't exist. This must NOT be 401/500 from the middleware.
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
