use super::pool::DbPool;
use super::seed::{insert_default_tutorials_tx, seed_site_content_tx};
use sqlx::{Sqlite, Transaction};
use std::env;

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
                tracing::warn!(
                    "ADMIN_PASSWORD exceeds 72 bytes; bcrypt only uses the first 72 bytes for hashing. Characters beyond this limit have no effect on security."
                );
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

async fn apply_comment_migrations(tx: &mut Transaction<'_, Sqlite>) -> Result<(), sqlx::Error> {
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

async fn apply_vote_migration(tx: &mut Transaction<'_, Sqlite>) -> Result<(), sqlx::Error> {
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

async fn fix_comment_schema(tx: &mut Transaction<'_, Sqlite>) -> Result<(), sqlx::Error> {
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
        SELECT id, tutorial_id, post_id, author, COALESCE(NULLIF(rate_limit_key, ''), author), content, created_at, votes, is_admin FROM comments_old
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

/// Adds `author_username` and `is_guest` columns to `comments` for authorization.
///
/// `author` is a free-text display name that guests can set almost arbitrarily,
/// so it must never be used to authorize comment deletion (a registered user
/// could otherwise delete a guest comment that happens to share their
/// username). These two columns record the real, unspoofable identity of the
/// commenter going forward:
/// - `author_username = Some(username)`: authenticated commenter (their real
///   username, never the "Administrator" display name used for admins).
/// - `author_username = NULL, is_guest = Some(true)`: guest commenter, who
///   never has a real identity to record.
/// - `author_username = NULL, is_guest = NULL`: pre-migration row of unknown
///   origin. `is_guest` is required in addition to `author_username` because
///   guest comments are *permanently* NULL for `author_username` -- without a
///   separate marker, a NULL-based fallback to the old (spoofable) `author`
///   string match would stay exploitable for every future guest comment, not
///   just historical ones. See `delete_comment` for how this three-state
///   model is used.
async fn apply_comment_author_identity_migration(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<(), sqlx::Error> {
    let has_author_username: bool = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('comments') WHERE name='author_username'",
    )
    .fetch_one(&mut **tx)
    .await
    .map(|count: i64| count > 0)?;

    if !has_author_username {
        tracing::info!("Adding author_username column to comments table");
        add_column_if_missing_race_safe(
            tx,
            "ALTER TABLE comments ADD COLUMN author_username TEXT DEFAULT NULL",
        )
        .await?;
    }

    let has_is_guest: bool = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('comments') WHERE name='is_guest'",
    )
    .fetch_one(&mut **tx)
    .await
    .map(|count: i64| count > 0)?;

    if !has_is_guest {
        tracing::info!("Adding is_guest column to comments table");
        add_column_if_missing_race_safe(
            tx,
            "ALTER TABLE comments ADD COLUMN is_guest BOOLEAN DEFAULT NULL",
        )
        .await?;
    }

    Ok(())
}

/// Runs an `ALTER TABLE ... ADD COLUMN` statement, tolerating a "duplicate
/// column name" failure.
///
/// The existence check above this call and the `ALTER TABLE` itself are not
/// atomic: if two instances of the app start concurrently against the same
/// SQLite file (e.g. a rolling deploy with overlapping replicas), both can
/// observe the column as missing before either commits its `ALTER TABLE`,
/// and the loser would otherwise fail its whole migration with "duplicate
/// column name" and abort startup. Since the only possible cause of that
/// specific error here is a concurrent run of this same idempotent
/// migration, it is safe to treat it as success rather than propagate it.
async fn add_column_if_missing_race_safe(
    tx: &mut Transaction<'_, Sqlite>,
    alter_statement: &str,
) -> Result<(), sqlx::Error> {
    match sqlx::query(alter_statement).execute(&mut **tx).await {
        Ok(_) => Ok(()),
        Err(e) if is_duplicate_column_error(&e) => {
            tracing::warn!(
                "Column already added by a concurrent migration run, continuing: {}",
                e
            );
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Detects SQLite's "duplicate column name" error, which `ALTER TABLE ...
/// ADD COLUMN` raises when the column already exists.
fn is_duplicate_column_error(err: &sqlx::Error) -> bool {
    err.as_database_error()
        .map(|db_err| db_err.message().contains("duplicate column name"))
        .unwrap_or(false)
}

/// Rehashes legacy plaintext rows in `token_blacklist` to SHA-256.
///
/// The repository layer used to store raw JWTs in the blacklist and now
/// stores and looks up only their SHA-256 hashes (see
/// `repositories::token_blacklist::hash_token`). On a database that predates
/// that change, existing rows still hold raw tokens, which the hashed lookup
/// can never match -- without this backfill, every token revoked before the
/// upgrade (e.g. via logout) would silently become valid again for its
/// remaining lifetime, undoing the revocation the user already performed.
///
/// Gated by an `app_metadata` flag like the other one-time migrations. The
/// per-row hex check is a second, defensive layer: a raw JWT always contains
/// `.` separators and can never look like a 64-char hex digest, so
/// already-hashed rows are never double-hashed even if the flag is somehow
/// lost (manual metadata edit, partial restore from backup).
async fn apply_token_blacklist_hash_migration(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<(), sqlx::Error> {
    let migrated: Option<(String,)> =
        sqlx::query_as("SELECT value FROM app_metadata WHERE key = 'token_blacklist_hashed_v1'")
            .fetch_optional(&mut **tx)
            .await?;

    if migrated.is_some() {
        return Ok(());
    }

    let rows: Vec<(String,)> = sqlx::query_as("SELECT token FROM token_blacklist")
        .fetch_all(&mut **tx)
        .await?;

    let mut rehashed = 0u64;
    for (token,) in rows {
        if is_sha256_hex(&token) {
            continue;
        }
        // UPDATE OR REPLACE: if the hashed form somehow already exists as its
        // own row, replace it instead of failing the whole startup on a
        // primary-key conflict. Either way the raw token is gone afterwards.
        sqlx::query("UPDATE OR REPLACE token_blacklist SET token = ? WHERE token = ?")
            .bind(crate::security::sha256_hex(token.as_bytes()))
            .bind(&token)
            .execute(&mut **tx)
            .await?;
        rehashed += 1;
    }

    if rehashed > 0 {
        tracing::info!(
            "Rehashed {} legacy plaintext token_blacklist row(s) to SHA-256",
            rehashed
        );
    }

    // OR REPLACE keeps a concurrent second instance (rolling deploy) from
    // aborting startup on a duplicate-key conflict for the flag itself.
    sqlx::query(
        "INSERT OR REPLACE INTO app_metadata (key, value) VALUES ('token_blacklist_hashed_v1', 'true')",
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

/// True if `s` is exactly a lowercase-or-uppercase 64-character hex string,
/// i.e. shaped like a SHA-256 digest. Raw JWTs always contain `.` separators,
/// so they can never satisfy this.
fn is_sha256_hex(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

async fn apply_site_post_migrations(tx: &mut Transaction<'_, Sqlite>) -> Result<(), sqlx::Error> {
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

#[cfg(test)]
mod tests {
    use super::run_migrations;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn run_migrations_backfills_rate_limit_key_for_legacy_comments() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("create sqlite pool");

        sqlx::query(
            r#"
            CREATE TABLE tutorials (
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
        .execute(&pool)
        .await
        .expect("create legacy tutorials table");

        sqlx::query(
            r#"
            INSERT INTO tutorials (id, title, description, icon, color, topics, content)
            VALUES ('tutorial-1', 'Legacy Tutorial', 'Legacy description', 'book', '#000000', 'legacy', 'Legacy content')
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert legacy tutorial");

        sqlx::query(
            r#"
            CREATE TABLE comments (
                id TEXT PRIMARY KEY,
                tutorial_id TEXT NOT NULL,
                author TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create legacy comments table");

        sqlx::query(
            r#"
            INSERT INTO comments (id, tutorial_id, author, content, created_at)
            VALUES ('legacy-comment', 'tutorial-1', 'Legacy Author', 'Old comment', '2024-01-01T00:00:00Z')
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert legacy comment");

        run_migrations(&pool)
            .await
            .expect("migrate legacy comments table");

        let has_rate_limit_key: bool = sqlx::query_scalar(
            "SELECT COUNT(*) FROM pragma_table_info('comments') WHERE name='rate_limit_key'",
        )
        .fetch_one(&pool)
        .await
        .map(|count: i64| count > 0)
        .expect("check rate_limit_key column");

        assert!(has_rate_limit_key);

        let (rate_limit_key,): (String,) =
            sqlx::query_as("SELECT rate_limit_key FROM comments WHERE id = 'legacy-comment'")
                .fetch_one(&pool)
                .await
                .expect("read migrated comment");

        assert_eq!(rate_limit_key, "Legacy Author");
    }

    /// Regression test for the blacklist-hashing upgrade path: a database
    /// written by a pre-hashing version of the app holds raw JWTs in
    /// token_blacklist. After migrations run, those rows must be rehashed so
    /// the (now hash-based) revocation lookup still recognizes them --
    /// otherwise every token revoked before the upgrade would silently
    /// become valid again for its remaining lifetime.
    #[tokio::test]
    async fn run_migrations_rehashes_legacy_plaintext_blacklist_tokens() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("create sqlite pool");

        // Simulate the pre-hashing schema and a raw bearer token revoked under it.
        sqlx::query(
            r#"
            CREATE TABLE token_blacklist (
                token TEXT PRIMARY KEY,
                expires_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create legacy token_blacklist table");

        let raw_token = ["legacy", "plaintext", "revocation-token", "before-hashing"].join("-");
        sqlx::query("INSERT INTO token_blacklist (token, expires_at) VALUES (?, ?)")
            .bind(&raw_token)
            .bind("2999-01-01T00:00:00+00:00")
            .execute(&pool)
            .await
            .expect("insert legacy plaintext token");

        run_migrations(&pool).await.expect("run migrations");

        // The raw token must be gone from storage...
        let stored: (String,) = sqlx::query_as("SELECT token FROM token_blacklist")
            .fetch_one(&pool)
            .await
            .expect("read migrated blacklist row");
        assert_ne!(stored.0, raw_token, "raw token must not survive migration");
        assert_eq!(stored.0, crate::security::sha256_hex(raw_token.as_bytes()));

        // ...while the hash-based revocation check still recognizes it.
        assert!(
            crate::repositories::token_blacklist::is_token_blacklisted(&pool, &raw_token)
                .await
                .expect("blacklist lookup"),
            "token revoked before the upgrade must still be treated as revoked"
        );

        // Idempotence: a second run (flag set, row already hashed) must not
        // double-hash the stored value.
        run_migrations(&pool).await.expect("re-run migrations");
        let stored_again: (String,) = sqlx::query_as("SELECT token FROM token_blacklist")
            .fetch_one(&pool)
            .await
            .expect("read blacklist row after re-run");
        assert_eq!(stored_again.0, stored.0);
    }
}
