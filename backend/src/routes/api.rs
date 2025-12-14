use axum::{routing::{get, post}, Router};
use tower_governor::{governor::GovernorConfig, key_extractor::SmartIpKeyExtractor, GovernorLayer};
use tower_http::services::ServeDir;
use crate::handlers::{auth, tutorials, search, comments, site_content, site_pages};
use crate::db::DbPool;
use std::sync::Arc;
use governor::middleware::NoOpMiddleware;

pub fn routes(
    upload_dir: String,
    admin_rate_limit_config: Arc<GovernorConfig<SmartIpKeyExtractor, NoOpMiddleware>>,
    public_rate_limit_config: Arc<GovernorConfig<SmartIpKeyExtractor, NoOpMiddleware>>,
) -> Router<DbPool> {
    Router::new()
        .route("/api/auth/me", get(auth::me))
        .route("/api/tutorials", get(tutorials::list_tutorials))
        .route(
            "/api/tutorials/{id}",
            get(tutorials::get_tutorial),
        )
        .route(
            "/api/search/tutorials",
            get(search::search_tutorials),
        )
        .route("/api/search/topics", get(search::get_all_topics))
        .route(
            "/api/tutorials/{id}/comments",
            get(comments::list_comments),
        )
        .route(
            "/api/content",
            get(site_content::list_site_content),
        )
        .route(
            "/api/content/{section}",
            get(site_content::get_site_content).put(site_content::update_site_content),
        )
        .route(
            "/api/posts/{id}/comments",
            get(comments::list_post_comments)
                .post(comments::create_post_comment)
                .route_layer(GovernorLayer::new(public_rate_limit_config)),
        )
        .route(
            "/api/comments/{id}/vote",
            post(comments::vote_comment),
        )
        .route(
            "/api/public/pages/{slug}",
            get(site_pages::get_published_page_by_slug),
        )
        .route(
            "/api/public/pages/{slug}/posts/{post_slug}",
            get(site_pages::get_published_post_by_slug),
        )
        .route(
            "/api/public/navigation",
            get(site_pages::get_navigation),
        )
        .route(
            "/api/public/published-pages",
            get(site_pages::list_published_page_slugs),
        )
        .nest_service("/uploads", ServeDir::new(upload_dir))
}
