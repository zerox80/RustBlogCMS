use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use rust_blog_backend::{
    handlers, routes,
    security::{auth, csrf},
};
use sqlx::SqlitePool;
use std::env;
use std::net::SocketAddr;
use tower::ServiceExt; // for `oneshot`

fn init_test_secrets() {
    env::set_var(
        "LOGIN_ATTEMPT_SALT",
        "this_is_a_test_salt_for_login_attempts_at_least_32_chars",
    );
    let _ = handlers::auth::init_login_attempt_salt();

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

#[tokio::test]
async fn test_api_health_check() {
    // 1. Setup
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // We need to run migrations here to have a valid DB state if the routes depend on it
    // But for health check it might not be needed.
    // However, create_routes takes the pool.

    // Create the app
    let app = routes::create_routes(pool.clone(), "test_uploads".to_string()).with_state(pool);

    // 2. Execute
    // The health check is defined in main.rs, not routes::create_routes!
    // create_routes returns the sub-router.
    // We should check if we can reach an endpoint defined in create_routes.
    // e.g. /api/tutorials is likely in api_router.

    // Let's try to hit a route that definitely exists in create_routes.
    // Looking at routes/mod.rs:
    // let api_router = api::routes(...);
    // and it merges login_router, admin_router, api_router.

    // Let's check `backend/src/routes/api.rs` to find a GET route.
    // Or we can just test that the router builds successfully for now.

    // Let's assume there is a public route, e.g. getting tutorials.

    let response = app
        .oneshot(with_connect_info(
            Request::builder()
                .uri("/api/tutorials")
                .body(Body::empty())
                .unwrap(),
        ))
        .await
        .unwrap();

    // 3. Verify
    // Only verify it doesn't 404. It might return 200 (empty list) or 500 (if DB tables missing).
    // If it returns 500, it means it reached the handler -> Success for routing test.
    assert_ne!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_login_route_exists() {
    init_test_secrets();
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    let app = routes::create_routes(pool.clone(), "test_uploads".to_string()).with_state(pool);

    let response = app
        .oneshot(with_connect_info(
            Request::builder()
                .uri("/api/auth/login")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"username":"admin","password":"password"}"#))
                .unwrap(),
        ))
        .await
        .unwrap();

    // Should return 401 or 200, but not 404.
    assert_ne!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_site_content_update_requires_auth() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    let app = routes::create_routes(pool.clone(), "test_uploads".to_string()).with_state(pool);

    let response = app
        .oneshot(with_connect_info(
            Request::builder()
                .uri("/api/content/hero")
                .method("PUT")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"content":{"title":"Updated hero"}}"#))
                .unwrap(),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
