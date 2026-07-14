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
            VALUES (
                'tutorial-1', 'Legacy Tutorial', 'Legacy description', 'book',
                '#000000', 'legacy', 'Legacy content'
            )
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

    let has_newsletter_table: bool = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master \
         WHERE type = 'table' AND name = 'newsletter_subscriptions'",
    )
    .fetch_one(&pool)
    .await
    .map(|count: i64| count == 1)
    .expect("check newsletter table");

    assert!(has_newsletter_table);

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

/// Updating the application must never rewrite or remove existing blog posts.
/// The one-page blog is a presentation change; legacy page associations remain
/// the durable storage model so old links and post metadata keep working.
#[tokio::test]
async fn rerunning_migrations_preserves_existing_blog_posts() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("create sqlite pool");

    run_migrations(&pool).await.expect("create current schema");

    sqlx::query(
        r#"
        INSERT INTO site_pages (
            id, slug, title, description, show_in_nav, order_index,
            is_published, hero_json, layout_json
        ) VALUES ('legacy-page', 'gedanken', 'Gedanken', 'Bestehende Seite', 1, 7, 1, '{}', '{}')
        "#,
    )
    .execute(&pool)
    .await
    .expect("insert existing page");

    sqlx::query(
        r#"
        INSERT INTO site_posts (
            id, page_id, title, slug, excerpt, content_markdown,
            is_published, allow_comments, published_at, order_index
        ) VALUES (
            'legacy-post', 'legacy-page', 'Mein bestehender Beitrag', 'bestehender-beitrag',
            'Unverwechselbarer Auszug', '# Unverwechselbarer Inhalt', 1, 0,
            '2025-04-03T12:30:00Z', 11
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("insert existing post");

    run_migrations(&pool)
        .await
        .expect("rerun migrations after update");

    let preserved: (
        String,
        String,
        String,
        String,
        String,
        i64,
        i64,
        String,
        i64,
    ) = sqlx::query_as(
        r#"
            SELECT page_id, title, slug, excerpt, content_markdown,
                   is_published, allow_comments, published_at, order_index
            FROM site_posts
            WHERE id = 'legacy-post'
            "#,
    )
    .fetch_one(&pool)
    .await
    .expect("existing post must still be present");

    assert_eq!(
        preserved,
        (
            "legacy-page".into(),
            "Mein bestehender Beitrag".into(),
            "bestehender-beitrag".into(),
            "Unverwechselbarer Auszug".into(),
            "# Unverwechselbarer Inhalt".into(),
            1,
            0,
            "2025-04-03T12:30:00Z".into(),
            11,
        )
    );
}

#[tokio::test]
async fn run_migrations_updates_persisted_site_branding() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("create sqlite pool");

    run_migrations(&pool).await.expect("create current schema");

    let stale_brand = ["Zero", "Point"].join(" ");
    let stale_content = serde_json::json!({
        "brand": { "name": stale_brand },
        "nested": [format!("{} archive", ["Zero", "Point"].join(" "))]
    });
    sqlx::query("UPDATE site_content SET content_json = ? WHERE section = 'header'")
        .bind(stale_content.to_string())
        .execute(&pool)
        .await
        .expect("insert stale site branding");

    run_migrations(&pool)
        .await
        .expect("migrate persisted site branding");

    let (content_json,): (String,) =
        sqlx::query_as("SELECT content_json FROM site_content WHERE section = 'header'")
            .fetch_one(&pool)
            .await
            .expect("read migrated site branding");
    let content: serde_json::Value = serde_json::from_str(&content_json).expect("valid JSON");

    assert_eq!(content["brand"]["name"], "minos");
    assert_eq!(content["nested"][0], "minos archive");
}
