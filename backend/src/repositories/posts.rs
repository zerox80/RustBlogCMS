use crate::db::DbPool;
use crate::models::{CreateSitePostRequest, SitePost, UpdateSitePostRequest};
use crate::repositories::common::validate_slug;
use sqlx;

/// Lists all posts belonging to a specific page (admin view).
pub async fn list_site_posts_for_page(
    pool: &DbPool,
    page_id: &str,
) -> Result<Vec<SitePost>, sqlx::Error> {
    sqlx::query_as::<_, SitePost>(
        "SELECT id, page_id, title, slug, excerpt, content_markdown, is_published, allow_comments, published_at, order_index, created_at, updated_at
         FROM site_posts
         WHERE page_id = ?
         ORDER BY order_index, created_at",
    )
    .bind(page_id)
    .fetch_all(pool)
    .await
}

/// Lists all published posts for a specific page, sorted by order index and publication date.
pub async fn list_published_posts_for_page(
    pool: &DbPool,
    page_id: &str,
) -> Result<Vec<SitePost>, sqlx::Error> {
    sqlx::query_as::<_, SitePost>(
        "SELECT id, page_id, title, slug, excerpt, content_markdown, is_published, allow_comments, published_at, order_index, created_at, updated_at
         FROM site_posts
         WHERE page_id = ? AND is_published = 1
         ORDER BY order_index, COALESCE(published_at, created_at)",
    )
    .bind(page_id)
    .fetch_all(pool)
    .await
}

pub async fn get_published_post_by_slug(
    pool: &DbPool,
    page_id: &str,
    post_slug: &str,
) -> Result<Option<SitePost>, sqlx::Error> {
    sqlx::query_as::<_, SitePost>(
        "SELECT id, page_id, title, slug, excerpt, content_markdown, is_published, allow_comments, published_at, order_index, created_at, updated_at
         FROM site_posts
         WHERE page_id = ? AND slug = ? AND is_published = 1",
    )
    .bind(page_id)
    .bind(post_slug)
    .fetch_optional(pool)
    .await
}

pub async fn get_site_post_by_id(pool: &DbPool, id: &str) -> Result<Option<SitePost>, sqlx::Error> {
    sqlx::query_as::<_, SitePost>(
        "SELECT id, page_id, title, slug, excerpt, content_markdown, is_published, allow_comments, published_at, order_index, created_at, updated_at
         FROM site_posts WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Creates a new blog post for a parent page.
pub async fn create_site_post(
    pool: &DbPool,
    page_id: &str,
    payload: CreateSitePostRequest,
) -> Result<SitePost, sqlx::Error> {
    // Validate slug hygiene
    validate_slug(&payload.slug)?;

    let id = uuid::Uuid::new_v4().to_string();
    let excerpt = payload.excerpt.unwrap_or_default();
    let order_index = payload.order_index.unwrap_or(0);

    // Insert record
    sqlx::query(
        "INSERT INTO site_posts (id, page_id, title, slug, excerpt, content_markdown, is_published, allow_comments, published_at, order_index)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(page_id)
    .bind(&payload.title)
    .bind(&payload.slug)
    .bind(excerpt)
    .bind(&payload.content_markdown)
    .bind(if payload.is_published { 1 } else { 0 })
    .bind(if payload.allow_comments { 1 } else { 0 })
    .bind(payload.published_at)
    .bind(order_index)
    .execute(pool)
    .await?;

    // Return created state
    get_site_post_by_id(pool, &id)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)
}

/// Updates an existing blog post using field merging.
pub async fn update_site_post(
    pool: &DbPool,
    id: &str,
    payload: UpdateSitePostRequest,
) -> Result<SitePost, sqlx::Error> {
    if let Some(slug) = payload.slug.as_deref() {
        validate_slug(slug)?;
    }

    // Load existing
    let mut existing = get_site_post_by_id(pool, id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

    // Merge changes
    if let Some(title) = payload.title {
        existing.title = title;
    }
    if let Some(slug) = payload.slug {
        existing.slug = slug;
    }
    if let Some(excerpt) = payload.excerpt {
        existing.excerpt = excerpt;
    }
    if let Some(content) = payload.content_markdown {
        existing.content_markdown = content;
    }
    if let Some(is_published) = payload.is_published {
        existing.is_published = is_published;
    }
    if let Some(allow_comments) = payload.allow_comments {
        existing.allow_comments = allow_comments;
    }
    if let Some(published_at) = payload.published_at {
        existing.published_at = published_at;
    }
    if let Some(order_index) = payload.order_index {
        existing.order_index = order_index;
    }

    // Save back to DB
    sqlx::query(
        "UPDATE site_posts
         SET title = ?, slug = ?, excerpt = ?, content_markdown = ?, is_published = ?, allow_comments = ?, published_at = ?, order_index = ?, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?",
    )
    .bind(&existing.title)
    .bind(&existing.slug)
    .bind(&existing.excerpt)
    .bind(&existing.content_markdown)
    .bind(if existing.is_published { 1 } else { 0 })
    .bind(if existing.allow_comments { 1 } else { 0 })
    .bind(&existing.published_at)
    .bind(existing.order_index)
    .bind(id)
    .execute(pool)
    .await?;

    get_site_post_by_id(pool, id)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)
}

pub async fn delete_site_post(pool: &DbPool, id: &str) -> Result<(), sqlx::Error> {
    let result = sqlx::query("DELETE FROM site_posts WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        Err(sqlx::Error::RowNotFound)
    } else {
        Ok(())
    }
}

pub async fn check_post_exists(pool: &DbPool, id: &str) -> Result<bool, sqlx::Error> {
    let exists: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM site_posts WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(exists.is_some())
}
