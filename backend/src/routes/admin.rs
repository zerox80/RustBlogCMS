use crate::handlers::{comments, site_content, site_pages, site_posts, tutorials, upload};
use crate::middleware::auth::auth_middleware;
use crate::security::csrf::enforce_csrf;
use crate::{db::DbPool, middleware::security::TrustedClientIpKeyExtractor};
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use governor::middleware::NoOpMiddleware;
use std::sync::Arc;
use tower_governor::{governor::GovernorConfig, GovernorLayer};
use tower_http::limit::RequestBodyLimitLayer;

const ADMIN_BODY_LIMIT: usize = 11 * 1024 * 1024;

/// Admin Route Module
///
/// Defines all endpoints requiring administrative privileges.
///
/// # Middleware Stacking (Critical)
/// Layers are applied from bottom to top:
/// 1. `GovernorLayer`: Prevents brute force on admin actions.
/// 2. `RequestBodyLimitLayer`: Prevents DoS while allowing 10MB uploads plus multipart overhead.
/// 3. `auth_middleware`: Ensures a valid JWT is present.
/// 4. `enforce_csrf`: Validates session integrity (Double-Submit Cookie).
pub fn routes(
    pool: DbPool,
    rate_limit_config: Arc<GovernorConfig<TrustedClientIpKeyExtractor, NoOpMiddleware>>,
) -> Router<DbPool> {
    // Dashboard initialization loads several independent resources in
    // parallel. Rate-limiting those reads by client IP makes a normal refresh
    // exhaust the small write burst and returns a misleading 429 response.
    // Keep the limiter for state-changing operations only.
    let read_routes = Router::new()
        .route("/api/pages", get(site_pages::list_site_pages))
        .route("/api/pages/{id}", get(site_pages::get_site_page))
        .route(
            "/api/pages/{page_id}/posts",
            get(site_posts::list_posts_for_page),
        )
        .route("/api/posts/{id}", get(site_posts::get_post));

    let write_routes = Router::new()
        .route("/api/tutorials", post(tutorials::create_tutorial))
        .route(
            "/api/tutorials/{id}",
            put(tutorials::update_tutorial).delete(tutorials::delete_tutorial),
        )
        .route("/api/pages", post(site_pages::create_site_page))
        .route(
            "/api/pages/{id}",
            put(site_pages::update_site_page).delete(site_pages::delete_site_page),
        )
        .route("/api/pages/{page_id}/posts", post(site_posts::create_post))
        .route(
            "/api/content/{section}",
            put(site_content::update_site_content),
        )
        .route(
            "/api/posts/{id}",
            put(site_posts::update_post).delete(site_posts::delete_post),
        )
        .route(
            "/api/tutorials/{id}/comments",
            post(comments::create_comment),
        )
        .route("/api/comments/{id}", delete(comments::delete_comment))
        .route("/api/upload", post(upload::upload_image))
        .layer(GovernorLayer::new(rate_limit_config));

    Router::new()
        .merge(read_routes)
        .merge(write_routes)
        .route_layer(axum::middleware::from_fn_with_state(
            pool.clone(),
            enforce_csrf,
        ))
        .route_layer(axum::middleware::from_fn_with_state(
            pool.clone(),
            auth_middleware,
        ))
        .layer(RequestBodyLimitLayer::new(ADMIN_BODY_LIMIT))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    use tower_governor::governor::GovernorConfigBuilder;

    #[tokio::test]
    async fn read_and_write_routes_merge_without_conflicts() {
        let pool = SqlitePool::connect_lazy("sqlite::memory:").expect("valid in-memory SQLite URL");
        let config = Arc::new(
            GovernorConfigBuilder::default()
                .per_second(1)
                .burst_size(3)
                .key_extractor(TrustedClientIpKeyExtractor)
                .finish()
                .expect("valid governor configuration"),
        );

        let _router = routes(pool, config);
    }
}
