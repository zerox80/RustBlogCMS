use super::pool::DbPool;
use super::seed::{insert_default_tutorials_tx, seed_site_content_tx};
use sqlx::{Sqlite, Transaction};
use std::env;

/// Runs all database migrations and initial data seeding.
/// # Migration Steps
/// 1. **Core Schema**: Create core tables (users, tutorials, comments, login_attempts)
/// 2. **Site Schema**: Create site-related tables (pages, posts, content)
/// 3. **FTS Index**: Create and populate full-text search index
/// 4. **Default Content**: Seed default site content (hero, footer, etc.)
/// 5. **Admin User**: Create admin account from environment variables
/// 6. **Default Tutorials**: Optionally seed sample tutorials
///
/// # Admin User Creation
/// If `ADMIN_USERNAME` and `ADMIN_PASSWORD` are set:
/// - Password must be ≥ 12 characters (NIST recommendation)
/// - User created with role "admin"
/// - Existing users are not overwritten (preserves runtime changes)
/// - Password hash created with bcrypt
///
/// # Default Tutorials
/// If `ENABLE_DEFAULT_TUTORIALS` is not "false":
/// - Inserts 8 sample tutorials on first run
/// - Skipped if tutorials already exist
/// - Marked as seeded in app_metadata
///
/// # Arguments
/// * `pool` - The database connection pool
///
/// # Returns
/// - `Ok(())` if all migrations succeed
/// - `Err(sqlx::Error)` if any migration fails
///
/// # Errors
/// - Schema creation failure
/// - Admin password too weak (< 12 characters)
/// - bcrypt hashing failure
/// - Transaction rollback on any error
///
/// # Environment Variables
/// - `ADMIN_USERNAME`: Admin account username (optional)
/// - `ADMIN_PASSWORD`: Admin account password (optional, min 12 chars)
/// - `ENABLE_DEFAULT_TUTORIALS`: "false" to disable tutorial seeding (default: true)
pub async fn run_migrations(pool: &DbPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Apply core schema migrations (users, tutorials, comments, etc.)
    if let Err(err) = apply_core_migrations(&mut tx).await {
        tx.rollback().await?;
        return Err(err);
    }

    tx.commit().await?;

    // Apply login attempt schema migrations (add last_attempt_at for cleanup)
    {
        let mut tx = pool.begin().await?;
        apply_login_attempt_migrations(&mut tx).await?;
        tx.commit().await?;
    }

    // Apply comment schema migrations (add post_id and rate_limit_key)
    {
        let mut tx = pool.begin().await?;
        apply_comment_migrations(&mut tx).await?;
        tx.commit().await?;
    }

    // Apply vote tracking schema migration
    {
        let mut tx = pool.begin().await?;
        apply_vote_migration(&mut tx).await?;
        tx.commit().await?;
    }

    // Fix comment schema (make tutorial_id nullable)
    {
        let mut tx = pool.begin().await?;
        fix_comment_schema(&mut tx).await?;
        tx.commit().await?;
    }

    // Apply comment author identity migration (author_username / is_guest for ownership checks)
    {
        let mut tx = pool.begin().await?;
        apply_comment_author_identity_migration(&mut tx).await?;
        tx.commit().await?;
    }

    // Rehash legacy plaintext rows in token_blacklist (raw JWTs -> SHA-256)
    {
        let mut tx = pool.begin().await?;
        apply_token_blacklist_hash_migration(&mut tx).await?;
        tx.commit().await?;
    }

    // Create site-related schema (pages, posts, content)
    ensure_site_page_schema(pool).await?;

    // Apply site post schema migrations (add allow_comments)
    {
        let mut tx = pool.begin().await?;
        if let Err(err) = apply_site_post_migrations(&mut tx).await {
            tracing::error!("Failed to apply site post migrations: {}", err);
        }
        tx.commit().await?;
    }

    // Seed default site content (hero, footer, etc.)
    {
        let mut tx = pool.begin().await?;
        seed_site_content_tx(&mut tx).await?;
        tx.commit().await?;
    }

    // Create admin user from environment variables
    let admin_username = env::var("ADMIN_USERNAME").ok();
    let admin_password = env::var("ADMIN_PASSWORD").ok();

    match (admin_username, admin_password) {
        (Some(username), Some(password)) if !username.is_empty() && !password.is_empty() => {
            if password.len() < 12 {
                tracing::error!(
                    "ADMIN_PASSWORD must be at least 12 characters long (NIST recommendation)!"
                );
                return Err(sqlx::Error::Protocol("Admin password too weak".into()));
            }

            // bcrypt only uses the first 72 bytes of the input; anything beyond
            // that has no effect on the resulting hash. Not treated as an error
            // since ADMIN_PASSWORD is operator-controlled via a trusted
            // environment variable, but worth surfacing so a longer passphrase
            // isn't assumed to add entropy it doesn't.
            if password.len() > 72 {
                tracing::warn!(concat!(
                    "ADMIN_PASSWORD exceeds 72 bytes; bcrypt only uses the first 72 bytes for hashing. ",
                    "Characters beyond this limit have no effect on security."
                ));
            }

            let existing_user: Option<(i64, String)> =
                sqlx::query_as("SELECT id, password_hash FROM users WHERE username = ?")
                    .bind(&username)
                    .fetch_optional(pool)
                    .await?;

            match existing_user {
                Some((_, current_hash)) => match bcrypt::verify(&password, &current_hash) {
                    Ok(true) => {
                        tracing::info!(
                            "Admin user '{}' already exists with correct password",
                            username
                        );
                    }
                    Ok(false) => {
                        tracing::warn!(
                            "ADMIN_PASSWORD for '{}' differs from stored credentials; \
                             keeping existing hash to preserve runtime changes.",
                            username
                        );
                    }
                    Err(e) => {
                        tracing::error!("Password verification failed: {}", e);
                        return Err(sqlx::Error::Protocol("Password verification error".into()));
                    }
                },
                None => {
                    let password_hash =
                        bcrypt::hash(&password, bcrypt::DEFAULT_COST).map_err(|e| {
                            tracing::error!("Failed to hash admin password: {}", e);
                            sqlx::Error::Protocol("Failed to hash admin password".into())
                        })?;
                    sqlx::query(
                        "INSERT INTO users (username, password_hash, role) VALUES (?, ?, ?)",
                    )
                    .bind(&username)
                    .bind(password_hash)
                    .bind("admin")
                    .execute(pool)
                    .await?;

                    tracing::info!("Created admin user '{}'", username);
                }
            }
        }
        _ => {
            tracing::warn!(
                "ADMIN_USERNAME and ADMIN_PASSWORD not set or empty. No admin user created."
            );
            tracing::warn!("Set these environment variables to create an admin user on startup.");
        }
    }

    let seed_enabled = env::var("ENABLE_DEFAULT_TUTORIALS")
        .map(|v| !v.trim().eq_ignore_ascii_case("false"))
        .unwrap_or(true);

    let mut tx = pool.begin().await?;

    if seed_enabled {
        let already_seeded: Option<(String,)> =
            sqlx::query_as("SELECT value FROM app_metadata WHERE key = 'default_tutorials_seeded'")
                .fetch_optional(&mut *tx)
                .await?;

        let tutorial_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tutorials")
            .fetch_one(&mut *tx)
            .await?;

        if already_seeded.is_none() && tutorial_count.0 == 0 {
            insert_default_tutorials_tx(&mut tx).await?;
            let timestamp = chrono::Utc::now().to_rfc3339();
            sqlx::query(
                "INSERT INTO app_metadata (key, value) VALUES ('default_tutorials_seeded', ?) \
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            )
            .bind(timestamp)
            .execute(&mut *tx)
            .await?;
            tracing::info!("Inserted default tutorials");
        }
    } else {
        tracing::info!(
            "ENABLE_DEFAULT_TUTORIALS disabled or not set – skipping default tutorial seeding"
        );
    }

    tx.commit().await?;

    Ok(())
}

async fn apply_core_migrations(tx: &mut Transaction<'_, Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'user',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            CONSTRAINT users_username_unique UNIQUE (username)
        )
        "#,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS login_attempts (
            username TEXT PRIMARY KEY,
            fail_count INTEGER NOT NULL DEFAULT 0,
            blocked_until TEXT
        )
        "#,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS token_blacklist (
            token TEXT PRIMARY KEY,
            expires_at TEXT NOT NULL
        )
        "#,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tutorials (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            icon TEXT NOT NULL,
            color TEXT NOT NULL,
            topics TEXT NOT NULL,
            content TEXT NOT NULL DEFAULT '',
            version INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_tutorials_created_at ON tutorials(created_at)")
        .execute(&mut **tx)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_tutorials_updated_at ON tutorials(updated_at)")
        .execute(&mut **tx)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS app_metadata (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )
        "#,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS newsletter_subscriptions (
            id TEXT PRIMARY KEY,
            email TEXT NOT NULL COLLATE NOCASE UNIQUE,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tutorial_topics (
            tutorial_id TEXT NOT NULL,
            topic TEXT NOT NULL,
            CONSTRAINT fk_tutorial_topics_tutorial
                FOREIGN KEY (tutorial_id) REFERENCES tutorials(id)
                ON DELETE CASCADE ON UPDATE CASCADE
        )
        "#,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_tutorial_topics_tutorial ON tutorial_topics(tutorial_id)",
    )
    .execute(&mut **tx)
    .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_tutorial_topics_topic ON tutorial_topics(topic)")
        .execute(&mut **tx)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS comments (
            id TEXT PRIMARY KEY,
            tutorial_id TEXT NOT NULL,
            author TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            CONSTRAINT fk_comments_tutorial FOREIGN KEY (tutorial_id) REFERENCES tutorials(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_comments_tutorial ON comments(tutorial_id)")
        .execute(&mut **tx)
        .await?;

    sqlx::query("DROP TRIGGER IF EXISTS tutorials_ai")
        .execute(&mut **tx)
        .await?;
    sqlx::query("DROP TRIGGER IF EXISTS tutorials_ad")
        .execute(&mut **tx)
        .await?;
    sqlx::query("DROP TRIGGER IF EXISTS tutorials_au")
        .execute(&mut **tx)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS tutorials_fts")
        .execute(&mut **tx)
        .await?;

    sqlx::query(
        r#"
        CREATE VIRTUAL TABLE tutorials_fts USING fts5(
            tutorial_id UNINDEXED,
            title,
            description,
            content,
            topics
        )
        "#,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        CREATE TRIGGER tutorials_ai AFTER INSERT ON tutorials BEGIN
            INSERT INTO tutorials_fts(tutorial_id, title, description, content, topics)
            VALUES (new.id, new.title, new.description, new.content, new.topics);
        END
        "#,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        CREATE TRIGGER tutorials_ad AFTER DELETE ON tutorials BEGIN
            DELETE FROM tutorials_fts WHERE tutorial_id = old.id;
        END
        "#,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        CREATE TRIGGER tutorials_au AFTER UPDATE ON tutorials BEGIN
            DELETE FROM tutorials_fts WHERE tutorial_id = old.id;
            INSERT INTO tutorials_fts(tutorial_id, title, description, content, topics)
            VALUES (new.id, new.title, new.description, new.content, new.topics);
        END
        "#,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO tutorials_fts(tutorial_id, title, description, content, topics)
        SELECT id, title, description, content, topics FROM tutorials
        "#,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

mod site_pages;
use site_pages::*;

mod comments;
use comments::*;

mod maintenance;
use maintenance::*;

#[cfg(test)]
mod tests;
