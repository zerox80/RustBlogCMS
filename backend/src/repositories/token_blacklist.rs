use crate::db::DbPool;
use sqlx;

/// Adds a JWT to the blacklist to invalidate it before its natural expiration.
/// Used during logout or security revocation.
pub async fn blacklist_token(
    pool: &DbPool,
    token: &str,
    expires_at: i64,
) -> Result<(), sqlx::Error> {
    // Convert unix timestamp to RFC3339 for ISO-standard DB storage
    let expires_at_str = chrono::DateTime::<chrono::Utc>::from(
        std::time::UNIX_EPOCH + std::time::Duration::from_secs(expires_at as u64),
    )
    .to_rfc3339();

    sqlx::query("INSERT INTO token_blacklist (token, expires_at) VALUES (?, ?)")
        .bind(token)
        .bind(expires_at_str)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn is_token_blacklisted(pool: &DbPool, token: &str) -> Result<bool, sqlx::Error> {
    let exists: Option<(String,)> =
        sqlx::query_as("SELECT token FROM token_blacklist WHERE token = ?")
            .bind(token)
            .fetch_optional(pool)
            .await?;
    Ok(exists.is_some())
}
