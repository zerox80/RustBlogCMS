use sqlx::{self, Sqlite};

/// Retrieves a specific metadata value by key.
///
/// Accepts any type that implements `Executor`, allowing calls within
/// transactions or from a standard connection pool.
pub async fn get_metadata<'e, E>(executor: E, key: &str) -> Result<Option<String>, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    let result: Option<(String,)> = sqlx::query_as("SELECT value FROM app_metadata WHERE key = ?")
        .bind(key)
        .fetch_optional(executor)
        .await?;

    Ok(result.map(|(v,)| v))
}

/// Persists or updates a metadata key-value pair.
/// Uses an UPSERT pattern to ensure key uniqueness.
pub async fn set_metadata<'e, E>(executor: E, key: &str, value: &str) -> Result<(), sqlx::Error>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    sqlx::query(
        "INSERT INTO app_metadata (key, value) VALUES (?, ?) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
    .execute(executor)
    .await?;

    Ok(())
}
