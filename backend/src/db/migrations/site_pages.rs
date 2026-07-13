use super::*;

pub(super) async fn ensure_site_page_schema(pool: &DbPool) -> Result<(), sqlx::Error> {
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
