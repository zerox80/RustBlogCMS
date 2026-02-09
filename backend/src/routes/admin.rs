use crate::db::DbPool;
use crate::handlers::{comments, site_content, site_pages, site_posts, tutorials, upload};
use crate::middleware::auth::auth_middleware;
use crate::security::csrf::enforce_csrf;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use governor::middleware::NoOpMiddleware;
use std::sync::Arc;
use tower_governor::{governor::GovernorConfig, key_extractor::SmartIpKeyExtractor, GovernorLayer};
use tower_http::limit::RequestBodyLimitLayer;

const ADMIN_BODY_LIMIT: usize = 8 * 1024 * 1024;

/// Admin Route Module
///
/// Defines all endpoints requiring administrative privileges.
///
/// # Middleware Stacking (Critical)
/// Layers are applied from bottom to top:
/// 1. `GovernorLayer`: Prevents brute force on admin actions.
/// 2. `RequestBodyLimitLayer`: Prevents DoS via large payloads (8MB cap for uploads).
/// 3. `auth_middleware`: Ensures a valid JWT is present.
/// 4. `enforce_csrf`: Validates session integrity (Double-Submit Cookie).
pub fn routes(
    pool: DbPool,
    rate_limit_config: Arc<GovernorConfig<SmartIpKeyExtractor, NoOpMiddleware>>,
) -> Router<DbPool> {
    Router::new()
        .route("/api/tutorials", post(tutorials::create_tutorial))
        .route(
            "/api/tutorials/{id}",
            put(tutorials::update_tutorial).delete(tutorials::delete_tutorial),
        )
        .route(
            "/api/pages",
            get(site_pages::list_site_pages).post(site_pages::create_site_page),
        )
        .route(
            "/api/pages/{id}",
            get(site_pages::get_site_page)
                .put(site_pages::update_site_page)
                .delete(site_pages::delete_site_page),
        )
        .route(
            "/api/pages/{page_id}/posts",
            get(site_posts::list_posts_for_page).post(site_posts::create_post),
        )
        .route(
            "/api/posts/{id}",
            get(site_posts::get_post)
                .put(site_posts::update_post)
                .delete(site_posts::delete_post),
        )
        .route(
            "/api/tutorials/{id}/comments",
            post(comments::create_comment),
        )
        .route("/api/comments/{id}", delete(comments::delete_comment))
        .route("/api/upload", post(upload::upload_image))
        .route_layer(axum::middleware::from_fn_with_state(
            pool.clone(),
            enforce_csrf,
        ))
        .route_layer(axum::middleware::from_fn_with_state(
            pool.clone(),
            auth_middleware,
        ))
        .layer(RequestBodyLimitLayer::new(ADMIN_BODY_LIMIT))
        .layer(GovernorLayer::new(rate_limit_config))
}
