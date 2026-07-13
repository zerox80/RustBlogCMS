use super::*;

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
pub(super) async fn apply_comment_author_identity_migration(
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

/// Adds the `last_attempt_at` column to `login_attempts`.
///
/// Without a timestamp, rows with fewer than 3 failures (blocked_until NULL)
/// could never be aged out, so the table grew unbounded under attack traffic
/// from rotating IPs (each IP+username combination is its own row). The
/// column lets `repositories::users::cleanup_stale_login_attempts` purge rows
/// that have been inactive long enough that their lockout state is moot.
/// Pre-existing rows are backfilled with the migration time so they enter the
/// same aging window instead of staying unpurgeable forever.
pub(super) async fn apply_login_attempt_migrations(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<(), sqlx::Error> {
    let has_last_attempt_at: bool = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('login_attempts') WHERE name='last_attempt_at'",
    )
    .fetch_one(&mut **tx)
    .await
    .map(|count: i64| count > 0)?;

    if !has_last_attempt_at {
        tracing::info!("Adding last_attempt_at column to login_attempts table");
        add_column_if_missing_race_safe(
            tx,
            "ALTER TABLE login_attempts ADD COLUMN last_attempt_at TEXT DEFAULT NULL",
        )
        .await?;

        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE login_attempts SET last_attempt_at = ? WHERE last_attempt_at IS NULL")
            .bind(now)
            .execute(&mut **tx)
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
pub(super) async fn add_column_if_missing_race_safe(
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
pub(super) fn is_duplicate_column_error(err: &sqlx::Error) -> bool {
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
pub(super) async fn apply_token_blacklist_hash_migration(
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
pub(super) fn is_sha256_hex(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

pub(super) async fn apply_site_post_migrations(
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
