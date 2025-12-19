use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use std::env;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use super::migrations::run_migrations;

/// Type alias for the SQLite connection pool.
/// Used throughout the application for database access.
pub type DbPool = SqlitePool;

/// Creates and initializes the database connection pool.
///
/// This is the main entry point for database initialization. It:
/// 1. Loads database URL from environment (defaults to ./database.db)
/// 2. Ensures the database directory exists
/// 3. Configures SQLite connection options
/// 4. Creates connection pool (1-5 connections)
/// 5. Runs all migrations
///
/// # Database Configuration
/// - **WAL Mode**: Write-Ahead Logging for better concurrency
/// - **Foreign Keys**: Enabled for referential integrity
/// - **Synchronous**: Normal mode (balanced safety/performance)
/// - **Busy Timeout**: 60 seconds to handle lock contention
/// - **Auto-create**: Database file created if missing
///
/// # Connection Pool
/// - Min connections: 1 (always ready)
/// - Max connections: 5 (prevents resource exhaustion)
/// - Acquire timeout: 30 seconds
/// - No idle timeout (connections persist)
/// - No max lifetime (connections don't expire)
///
/// # Returns
/// - `Ok(DbPool)` on success
/// - `Err(sqlx::Error)` if initialization fails
///
/// # Errors
/// - Invalid DATABASE_URL format
/// - Database directory creation failure
/// - Connection establishment failure
/// - Migration failure
///
/// # Environment Variables
/// - `DATABASE_URL`: SQLite database path (default: "sqlite:./database.db")
pub async fn create_pool() -> Result<DbPool, sqlx::Error> {
    // Load database URL from environment or use default
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        tracing::warn!("DATABASE_URL not set, defaulting to sqlite:./database.db");
        "sqlite:./database.db".to_string()
    });

    // Ensure parent directory exists
    ensure_sqlite_directory(&database_url)?;

    // Configure SQLite connection options
    let connect_options = SqliteConnectOptions::from_str(&database_url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .foreign_keys(true)
        .busy_timeout(std::time::Duration::from_secs(60));

    // Create connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .idle_timeout(None)
        .max_lifetime(None)
        .connect_with(connect_options)
        .await?;

    // Run all database migrations
    run_migrations(&pool).await?;

    tracing::info!("Database pool created successfully");
    Ok(pool)
}

fn ensure_sqlite_directory(database_url: &str) -> Result<(), sqlx::Error> {
    // Step 1: Extract file path from connection string
    if let Some(db_path) = sqlite_file_path(database_url) {
        // Step 2: Get parent directory
        if let Some(parent) = db_path.parent() {
            // Step 3: Create directory if not current working dir
            if parent != Path::new("") && parent != Path::new(".") {
                if let Err(err) = std::fs::create_dir_all(parent) {
                    tracing::error!(error = %err, path = ?parent, "Failed to create SQLite directory");
                    return Err(sqlx::Error::Io(err));
                }
                tracing::info!(path = ?parent, "Ensured SQLite directory exists");
            }
        }
    }

    Ok(())
}

fn sqlite_file_path(database_url: &str) -> Option<PathBuf> {
    const PREFIX: &str = "sqlite:";

    // Verify scheme
    if !database_url.starts_with(PREFIX) {
        return None;
    }

    // Extract path part after prefix
    let mut remainder = &database_url[PREFIX.len()..];

    // Reject memory-only databases or empty paths for directory creation
    if remainder.starts_with(':') || remainder.is_empty() {
        return None;
    }

    // Strip optional query parameters (e.g., ?mode=rwc)
    if let Some((path_part, _)) = remainder.split_once('?') {
        remainder = path_part;
    }

    // Normalize slashes for mixed OS environments
    let normalized = if remainder.starts_with("///") {
        &remainder[2..]
    } else if remainder.starts_with("//") {
        &remainder[1..]
    } else {
        remainder
    };

    if normalized.trim().is_empty() {
        return None;
    }

    Some(PathBuf::from(normalized))
}
