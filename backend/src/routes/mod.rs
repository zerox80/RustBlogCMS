pub mod admin;
pub mod api;
pub mod auth;

use axum::Router;
use crate::db::DbPool;
use tower_governor::{governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor};
use std::sync::Arc;

pub fn create_routes(pool: DbPool, upload_dir: String) -> Router<DbPool> {
    let admin_rate_limit_config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(3)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .expect("Failed to build governor config for write routes"),
    );

    let public_rate_limit_config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(5)
            .burst_size(10)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .expect("Failed to build governor config for public routes"),
    );

    let login_router = auth::routes();
    let admin_router = admin::routes(pool.clone(), admin_rate_limit_config.clone());
    let api_router = api::routes(upload_dir, admin_rate_limit_config, public_rate_limit_config);

    Router::new()
        .merge(login_router)
        .merge(admin_router)
        .merge(api_router)
}
