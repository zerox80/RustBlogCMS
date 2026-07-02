use crate::db::DbPool;
use sha2::{Digest, Sha256};
use sqlx;

/// Hashes a raw JWT before it is stored in or queried against the blacklist.
///
/// The blacklist must never persist raw, reusable session tokens: anyone who
/// reads the SQLite file (backup, restore leak, export) would otherwise gain
/// directly reusable session artifacts. Storing only the SHA-256 hash keeps
/// the revocation check working (equality is preserved under hashing) while
/// making the stored value useless for session hijacking.
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

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
        .bind(hash_token(token))
        .bind(expires_at_str)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn is_token_blacklisted(pool: &DbPool, token: &str) -> Result<bool, sqlx::Error> {
    let exists: Option<(String,)> =
        sqlx::query_as("SELECT token FROM token_blacklist WHERE token = ?")
            .bind(hash_token(token))
            .fetch_optional(pool)
            .await?;
    Ok(exists.is_some())
}

/// Deletes expired tokens from the blacklist to prevent unbounded table growth.
pub async fn cleanup_expired(pool: &DbPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM token_blacklist WHERE expires_at < datetime('now')")
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    async fn setup_test_db() -> DbPool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        crate::db::migrations::run_migrations(&pool)
            .await
            .expect("Failed to run migrations");
        pool
    }

    #[tokio::test]
    async fn blacklisted_token_is_detected_after_hashing() {
        let pool = setup_test_db().await;
        let token = "some.raw.jwt.value";

        assert!(!is_token_blacklisted(&pool, token).await.unwrap());

        let expires_at = chrono::Utc::now().timestamp() + 3600;
        blacklist_token(&pool, token, expires_at).await.unwrap();

        assert!(is_token_blacklisted(&pool, token).await.unwrap());
    }

    #[tokio::test]
    async fn stored_value_is_hashed_not_plaintext() {
        let pool = setup_test_db().await;
        let token = "another.raw.jwt.value";
        let expires_at = chrono::Utc::now().timestamp() + 3600;

        blacklist_token(&pool, token, expires_at).await.unwrap();

        let stored: (String,) = sqlx::query_as("SELECT token FROM token_blacklist WHERE token = ?")
            .bind(hash_token(token))
            .fetch_one(&pool)
            .await
            .expect("hashed row must exist");

        assert_ne!(stored.0, token, "raw token must never be stored");
        assert_eq!(stored.0, hash_token(token));
    }
}
