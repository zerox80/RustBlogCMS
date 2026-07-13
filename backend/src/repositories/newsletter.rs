use crate::db::DbPool;

/// Stores a normalized subscription without revealing whether it already existed.
pub async fn subscribe(pool: &DbPool, email: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO newsletter_subscriptions (id, email) VALUES (?, ?) \
         ON CONFLICT(email) DO NOTHING",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(email)
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::subscribe;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn repeated_subscriptions_are_idempotent() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("create sqlite pool");
        sqlx::query(
            "CREATE TABLE newsletter_subscriptions (\
                id TEXT PRIMARY KEY, \
                email TEXT NOT NULL COLLATE NOCASE UNIQUE, \
                created_at TEXT NOT NULL DEFAULT (datetime('now'))\
            )",
        )
        .execute(&pool)
        .await
        .expect("create newsletter table");

        subscribe(&pool, "reader@example.com")
            .await
            .expect("first subscription");
        subscribe(&pool, "reader@example.com")
            .await
            .expect("repeated subscription");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM newsletter_subscriptions")
            .fetch_one(&pool)
            .await
            .expect("count subscriptions");
        assert_eq!(count, 1);
    }
}
