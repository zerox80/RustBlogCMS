// Module declarations for organizing the backend codebase
pub mod db; // Database connection and pooling
pub mod handlers; // HTTP request handlers organized by feature
pub mod middleware; // Middleware modules
pub mod models; // Data structures and database models
pub mod repositories; // Repository modules
pub mod routes;
pub mod security; // Authentication, authorization, and CSRF protection // Route definitions

use crate::middleware::{cors, security as security_middleware};

// HTTP-related imports for building the web server
use axum::{extract::DefaultBodyLimit, routing::get, Router};

// External dependencies for configuration, async runtime, and middleware
use dotenv::dotenv;
use std::env;
use std::io::ErrorKind;
use std::net::SocketAddr;
use tokio::signal;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

// Custom HTTP header constants for security policies
use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    Method,
};

/// Main application entry point.
#[tokio::main]
async fn main() {
    // Load environment variables from .env file (if present)
    dotenv().ok();

    // Initialize structured logging
    tracing_subscriber::fmt::init();

    security::auth::init_jwt_secret().expect("Failed to initialize JWT secret");
    tracing::info!("JWT secret initialized successfully");

    security::csrf::init_csrf_secret().expect("Failed to initialize CSRF secret");
    tracing::info!("CSRF secret initialized successfully");

    handlers::auth::init_login_attempt_salt().expect("Failed to initialize login attempt salt");
    tracing::info!("Login attempt salt initialized successfully");

    let pool = db::create_pool()
        .await
        .expect("Failed to create database pool");

    // Ensure uploads directory exists
    let upload_dir = env::var("UPLOAD_DIR").unwrap_or_else(|_| "uploads".to_string());
    if !std::path::Path::new(&upload_dir).exists() {
        tokio::fs::create_dir_all(&upload_dir)
            .await
            .expect("Failed to create uploads directory");
    }

    // Configure CORS (Cross-Origin Resource Sharing)
    let cors_origins = env::var("CORS_ALLOWED_ORIGINS")
        .map(|val| {
            val.split(',')
                .map(|s| s.trim().to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|_| {
            cors::DEV_DEFAULT_FRONTEND_ORIGINS
                .iter()
                .map(|&s| s.to_string())
                .collect()
        });

    let allowed_origins = cors::parse_allowed_origins(cors_origins.iter().map(|s| s.as_str()));

    let cors_layer = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION, ACCEPT])
        .allow_credentials(true)
        .allow_origin(allowed_origins);

    tracing::info!(origins = ?cors_origins, "Configured CORS origins");

    let trust_proxy_ip_headers =
        security_middleware::parse_env_bool("TRUST_PROXY_IP_HEADERS", false);
    if trust_proxy_ip_headers {
        tracing::info!("Trusting X-Forwarded-* headers for client IP extraction");
    } else {
        tracing::info!("Proxy headers will be stripped before rate limiting to prevent spoofing");
    }

    // Create routes
    let app_routes = routes::create_routes(pool.clone(), upload_dir);

    // Define the application router with all routes and middleware
    let app = Router::new()
        .merge(app_routes)
        .route("/api/health", get(|| async { "OK" }))
        // Serve index.html with server-side injection for root and fallback
        .route("/", get(handlers::frontend_proxy::serve_index))
        .route("/{*path}", get(handlers::frontend_proxy::serve_index))
        .layer(axum::middleware::from_fn(
            security_middleware::security_headers,
        ))
        .layer(cors_layer)
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB body limit
        .with_state(pool.clone());

    // Apply trusted proxy middleware if configured
    let app = if trust_proxy_ip_headers {
        app
    } else {
        app.layer(axum::middleware::from_fn(
            security_middleware::strip_untrusted_forwarded_headers,
        ))
    };
    let port_str = env::var("PORT").unwrap_or_else(|_| "8489".to_string());
    let port: u16 = match port_str.parse() {
        Ok(port) => port,
        Err(e) => {
            panic!(
                "Invalid PORT '{}': {}. Please set PORT to a valid u16 value.",
                port_str, e
            );
        }
    };

    if port < 1024 {
        tracing::warn!(
            "PORT {} is in privileged range (< 1024). May require elevated permissions.",
            port
        );
    }

    let addr = format!("0.0.0.0:{}", port);

    tracing::info!("Starting server on {}", addr);

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(err) => {
            if err.kind() == ErrorKind::AddrInUse {
                panic!("Failed to bind to {addr}: port {port} is already in use. Choose a different PORT value.");
            } else {
                panic!("Failed to bind to {addr}: {err}");
            }
        }
    };

    let make_service = app.into_make_service_with_connect_info::<SocketAddr>();

    let server = axum::serve(listener, make_service).with_graceful_shutdown(shutdown_signal());

    tracing::info!("Server is ready to accept connections");

    if let Err(e) = server.await {
        tracing::error!("Server error: {}", e);
    }

    tracing::info!("Server shutdown complete");
}

/// Waits for a shutdown signal and initiates graceful shutdown.
async fn shutdown_signal() {
    // Handle Ctrl+C signal (works on all platforms)
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    // Handle SIGTERM on Unix systems (used by Docker, systemd, etc.)
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    // On non-Unix systems, SIGTERM doesn't exist, so use a pending future
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    // Wait for either Ctrl+C or SIGTERM
    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM signal");
        },
    }

    tracing::info!("Starting graceful shutdown...");
}
