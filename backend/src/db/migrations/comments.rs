use super::*;

pub(super) async fn apply_comment_migrations(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<(), sqlx::Error> {
    // Check if post_id column exists
    let has_post_id: bool = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('comments') WHERE name='post_id'",
    )
    .fetch_one(&mut **tx)
    .await
    .map(|count: i64| count > 0)?;

    if !has_post_id {
        tracing::info!("Adding post_id column to comments table");
        sqlx::query("ALTER TABLE comments ADD COLUMN post_id TEXT")
            .execute(&mut **tx)
            .await?;

        // Add index for post_id
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_comments_post ON comments(post_id)")
            .execute(&mut **tx)
            .await?;
    }

    let has_rate_limit_key: bool = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('comments') WHERE name='rate_limit_key'",
    )
    .fetch_one(&mut **tx)
    .await
    .map(|count: i64| count > 0)?;

    if !has_rate_limit_key {
        tracing::info!("Adding rate_limit_key column to comments table");
        sqlx::query("ALTER TABLE comments ADD COLUMN rate_limit_key TEXT NOT NULL DEFAULT ''")
            .execute(&mut **tx)
            .await?;
    }

    sqlx::query("UPDATE comments SET rate_limit_key = author WHERE rate_limit_key = ''")
        .execute(&mut **tx)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_comments_rate_limit ON comments(rate_limit_key)")
        .execute(&mut **tx)
        .await?;

    Ok(())
}

pub(super) async fn apply_vote_migration(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<(), sqlx::Error> {
    // Create comment_votes table
    sqlx::query(include_str!(
        "../../../migrations/20241119_create_comment_votes.sql"
    ))
    .execute(&mut **tx)
    .await?;

    // Add votes column to comments if missing
    let has_votes: bool =
        sqlx::query_scalar("SELECT COUNT(*) FROM pragma_table_info('comments') WHERE name='votes'")
            .fetch_one(&mut **tx)
            .await
            .map(|count: i64| count > 0)?;

    if !has_votes {
        tracing::info!("Adding votes column to comments table");
        sqlx::query("ALTER TABLE comments ADD COLUMN votes INTEGER NOT NULL DEFAULT 0")
            .execute(&mut **tx)
            .await?;
    }

    // Add is_admin column to comments if missing
    let has_is_admin: bool = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('comments') WHERE name='is_admin'",
    )
    .fetch_one(&mut **tx)
    .await
    .map(|count: i64| count > 0)?;

    if !has_is_admin {
        tracing::info!("Adding is_admin column to comments table");
        sqlx::query("ALTER TABLE comments ADD COLUMN is_admin BOOLEAN NOT NULL DEFAULT FALSE")
            .execute(&mut **tx)
            .await?;
    }

    Ok(())
}

pub(super) async fn fix_comment_schema(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<(), sqlx::Error> {
    // Check whether the schema fix has already run. SQLite does not expose
    // nullability conveniently without parsing the table definition.
    // Instead, we'll check if we've already run this fix by checking app_metadata.
    let fixed: Option<(String,)> =
        sqlx::query_as("SELECT value FROM app_metadata WHERE key = 'comment_schema_fixed_v1'")
            .fetch_optional(&mut **tx)
            .await?;

    if fixed.is_some() {
        return Ok(());
    }

    tracing::info!("Fixing comment schema: Making tutorial_id nullable");

    // 1. Rename existing table to avoid name collision during schema swap
    sqlx::query("ALTER TABLE comments RENAME TO comments_old")
        .execute(&mut **tx)
        .await?;

    // 2. Create new table with nullable tutorial_id and post_id (the fix)
    sqlx::query(
        r#"
        CREATE TABLE comments (
            id TEXT PRIMARY KEY,
            tutorial_id TEXT,
            post_id TEXT,
            author TEXT NOT NULL,
            rate_limit_key TEXT NOT NULL DEFAULT '',
            content TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            votes INTEGER NOT NULL DEFAULT 0,
            is_admin BOOLEAN NOT NULL DEFAULT FALSE,
            CONSTRAINT fk_comments_tutorial FOREIGN KEY (tutorial_id) REFERENCES tutorials(id) ON DELETE CASCADE
        )
        "# ,
    )
    .execute(&mut **tx)
    .await?;

    // 3. Migrate data from the old schema to the new one
    sqlx::query(
        r#"
        INSERT INTO comments (id, tutorial_id, post_id, author, rate_limit_key, content, created_at, votes, is_admin)
        SELECT id, tutorial_id, post_id, author,
               COALESCE(NULLIF(rate_limit_key, ''), author), content,
               created_at, votes, is_admin
        FROM comments_old
        "#,
    )
    .execute(&mut **tx)
    .await?;

    // 4. Cleanup old temporary table
    sqlx::query("DROP TABLE comments_old")
        .execute(&mut **tx)
        .await?;

    // 5. Recreate performance indices on the new table
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_comments_tutorial ON comments(tutorial_id)")
        .execute(&mut **tx)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_comments_post ON comments(post_id)")
        .execute(&mut **tx)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_comments_rate_limit ON comments(rate_limit_key)")
        .execute(&mut **tx)
        .await?;

    // 6. Persist migration state to prevent re-execution
    sqlx::query("INSERT INTO app_metadata (key, value) VALUES ('comment_schema_fixed_v1', 'true')")
        .execute(&mut **tx)
        .await?;

    Ok(())
}
