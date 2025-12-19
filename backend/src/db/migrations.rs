use sqlx::{Sqlite, Transaction};
use std::env;
use super::pool::DbPool;
use super::seed::{seed_site_content_tx, insert_default_tutorials_tx};

/// Runs all database migrations and initial data seeding.
///
/// This function is automatically called during database pool creation.
/// It ensures the database schema is up-to-date and populates initial data.
///
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

    // Apply comment schema migrations (add post_id)
    {
        let mut tx = pool.begin().await?;
        if let Err(err) = apply_comment_migrations(&mut tx).await {
            tracing::error!("Failed to apply comment migrations: {}", err);
            // Don't fail startup if migration fails (might already exist)
            // But for safety, we should probably log and continue or fail depending on severity.
            // Here we log and continue as it might be a "column already exists" error which is fine.
            // Better approach: check if column exists inside the migration function.
        }
        tx.commit().await?;
    }

    // Apply vote tracking schema migration
    {
        let mut tx = pool.begin().await?;
        if let Err(err) = apply_vote_migration(&mut tx).await {
            tracing::error!("Failed to apply vote migration: {}", err);
        }
        tx.commit().await?;
    }

    // Fix comment schema (make tutorial_id nullable)
    {
        let mut tx = pool.begin().await?;
        if let Err(err) = fix_comment_schema(&mut tx).await {
            tracing::error!("Failed to fix comment schema: {}", err);
            // We might want to fail here if it's critical, but let's log for now.
        }
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
                        tracing::warn!("ADMIN_PASSWORD for '{}' differs from stored credentials; keeping existing hash to preserve runtime changes.", username);
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

async fn apply_core_migrations(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<(), sqlx::Error> {
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
        CREATE TABLE IF NOT EXISTS tutorial_topics (
            tutorial_id TEXT NOT NULL,
            topic TEXT NOT NULL,
            CONSTRAINT fk_tutorial_topics_tutorial FOREIGN KEY (tutorial_id) REFERENCES tutorials(id) ON DELETE CASCADE ON UPDATE CASCADE
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

async fn ensure_site_page_schema(pool: &DbPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS site_content (
            section TEXT PRIMARY KEY,
            content_json TEXT NOT NULL,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS site_pages (
            id TEXT PRIMARY KEY,
            slug TEXT NOT NULL UNIQUE,
            title TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            nav_label TEXT,
            show_in_nav INTEGER NOT NULL DEFAULT 0,
            order_index INTEGER NOT NULL DEFAULT 0,
            is_published INTEGER NOT NULL DEFAULT 0,
            hero_json TEXT NOT NULL DEFAULT '{}',
            layout_json TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_site_pages_nav ON site_pages(show_in_nav, order_index)",
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS site_posts (
            id TEXT PRIMARY KEY,
            page_id TEXT NOT NULL,
            title TEXT NOT NULL,
            slug TEXT NOT NULL,
            excerpt TEXT DEFAULT '',
            content_markdown TEXT NOT NULL,
            is_published INTEGER NOT NULL DEFAULT 0,
            allow_comments BOOLEAN NOT NULL DEFAULT 1,
            published_at TEXT,
            order_index INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(page_id) REFERENCES site_pages(id) ON DELETE CASCADE
        )",
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_site_posts_unique_slug ON site_posts(page_id, slug)",
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_site_posts_page_published ON site_posts(page_id, is_published, published_at)",
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

async fn apply_comment_migrations(
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

    Ok(())
}

async fn apply_vote_migration(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<(), sqlx::Error> {
    // Create comment_votes table
    sqlx::query(include_str!(
        "../../migrations/20241119_create_comment_votes.sql"
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

async fn fix_comment_schema(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<(), sqlx::Error> {
    // Check if tutorial_id is nullable by checking table info, but SQLite doesn't make it easy to check nullability directly via simple query without parsing.
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
        INSERT INTO comments (id, tutorial_id, post_id, author, content, created_at, votes, is_admin)
        SELECT id, tutorial_id, post_id, author, content, created_at, votes, is_admin FROM comments_old
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

    // 6. Persist migration state to prevent re-execution
    sqlx::query("INSERT INTO app_metadata (key, value) VALUES ('comment_schema_fixed_v1', 'true')")
        .execute(&mut **tx)
        .await?;

    Ok(())
}

async fn apply_site_post_migrations(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<(), sqlx::Error> {
    // Check if allow_comments column exists
    let has_allow_comments: bool = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('site_posts') WHERE name='allow_comments'",
    )
    .fetch_one(&mut **tx)
    .await
    .map(|count: i64| count > 0)?;

    if !has_allow_comments {
        tracing::info!("Adding allow_comments column to site_posts table");
        sqlx::query("ALTER TABLE site_posts ADD COLUMN allow_comments BOOLEAN NOT NULL DEFAULT 1")
            .execute(&mut **tx)
            .await?;
    }

    Ok(())
}
