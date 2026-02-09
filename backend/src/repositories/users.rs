use crate::db::DbPool;
use crate::models::User;
use sqlx::{self, FromRow};

/// Represents a snapshot of failed login attempts for a specific user.
/// Used by the auth handler to enforce temporary account lockouts.
#[derive(Debug, FromRow, Clone)]
pub struct LoginAttempt {
    /// Number of consecutive failed attempts.
    pub fail_count: i64,
    /// ISO 8601 timestamp string indicating when the block expires.
    pub blocked_until: Option<String>,
}

/// Retrieves a full user record by username, including the secure password hash.
pub async fn get_user_by_username(
    pool: &DbPool,
    username: &str,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await
}

pub async fn get_login_attempt(
    pool: &DbPool,
    username_hash: &str,
) -> Result<Option<LoginAttempt>, sqlx::Error> {
    sqlx::query_as::<_, LoginAttempt>(
        "SELECT fail_count, blocked_until FROM login_attempts WHERE username = ?",
    )
    .bind(username_hash)
    .fetch_optional(pool)
    .await
}

/// Atomically increments the failure count and applies tiered blocking logic.
///
/// Blocking Strategy:
/// - 3-4 Failures: Applies `short_block` duration.
/// - 5+ Failures: Applies `long_block` duration.
/// - Uses SQLite's UPSERT pattern for thread-safe counters.
pub async fn record_failed_login(
    pool: &DbPool,
    username_hash: &str,
    long_block: &str,
    short_block: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO login_attempts (username, fail_count, blocked_until) VALUES (?, 1, NULL) \
         ON CONFLICT(username) DO UPDATE SET fail_count = login_attempts.fail_count + 1, \
         blocked_until = CASE \
             WHEN login_attempts.fail_count + 1 >= 5 THEN ? \
             WHEN login_attempts.fail_count + 1 >= 3 THEN ? \
             ELSE NULL \
         END",
    )
    .bind(username_hash)
    .bind(long_block)
    .bind(short_block)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn clear_login_attempts(pool: &DbPool, username_hash: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM login_attempts WHERE username = ?")
        .bind(username_hash)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn check_user_exists_by_name(pool: &DbPool, username: &str) -> Result<bool, sqlx::Error> {
    let exists: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await?;
    Ok(exists.is_some())
}
