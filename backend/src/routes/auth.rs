use axum::{routing::post, Router};
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_governor::key_extractor::SmartIpKeyExtractor;
use tower_http::limit::RequestBodyLimitLayer;
use crate::handlers::auth;
use crate::db::DbPool;
use std::sync::Arc;

const LOGIN_BODY_LIMIT: usize = 64 * 1024;

pub fn routes() -> Router<DbPool> {
    let rate_limit_config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(5)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .expect("Failed to build governor config"),
    );

    Router::new()
        // Core Identity Endpoints
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/logout", post(auth::logout))
        
        // System-wide Protections
        .layer(RequestBodyLimitLayer::new(LOGIN_BODY_LIMIT))
        .layer(GovernorLayer::new(rate_limit_config))
}
