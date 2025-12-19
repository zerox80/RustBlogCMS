use crate::db::DbPool;
use crate::models::{CreateSitePageRequest, SitePage, UpdateSitePageRequest};
use crate::repositories::common::{serialize_json_value, validate_slug};
use sqlx;

/// Fetches all site pages, ordered by their custom navigation index and title.
pub async fn list_site_pages(pool: &DbPool) -> Result<Vec<SitePage>, sqlx::Error> {
    sqlx::query_as::<_, SitePage>(
        "SELECT id, slug, title, description, nav_label, show_in_nav, order_index, is_published, hero_json, layout_json, created_at, updated_at FROM site_pages ORDER BY order_index, title",
    )
    .fetch_all(pool)
    .await
}

/// Fetches pages that are specifically marked to appear in the navigation menu.
pub async fn list_nav_pages(pool: &DbPool) -> Result<Vec<SitePage>, sqlx::Error> {
    sqlx::query_as::<_, SitePage>(
        "SELECT id, slug, title, description, nav_label, show_in_nav, order_index, is_published, hero_json, layout_json, created_at, updated_at
         FROM site_pages
         WHERE show_in_nav = 1 AND is_published = 1
         ORDER BY order_index, title",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_published_pages(pool: &DbPool) -> Result<Vec<SitePage>, sqlx::Error> {
    sqlx::query_as::<_, SitePage>(
        "SELECT id, slug, title, description, nav_label, show_in_nav, order_index, is_published, hero_json, layout_json, created_at, updated_at
         FROM site_pages
         WHERE is_published = 1
         ORDER BY order_index, title",
    )
    .fetch_all(pool)
    .await
}

pub async fn get_site_page_by_id(pool: &DbPool, id: &str) -> Result<Option<SitePage>, sqlx::Error> {
    sqlx::query_as::<_, SitePage>(
        "SELECT id, slug, title, description, nav_label, show_in_nav, order_index, is_published, hero_json, layout_json, created_at, updated_at FROM site_pages WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Fetches a single site page by its URL slug.
pub async fn get_site_page_by_slug(
    pool: &DbPool,
    slug: &str,
) -> Result<Option<SitePage>, sqlx::Error> {
    sqlx::query_as::<_, SitePage>(
        "SELECT id, slug, title, description, nav_label, show_in_nav, order_index, is_published, hero_json, layout_json, created_at, updated_at FROM site_pages WHERE slug = ?",
    )
    .bind(slug)
    .fetch_optional(pool)
    .await
}

/// Creates a new site page with default UUID and serialized JSON content.
pub async fn create_site_page(
    pool: &DbPool,
    page: CreateSitePageRequest,
) -> Result<SitePage, sqlx::Error> {
    // Validate slug hygiene
    validate_slug(&page.slug)?;

    let id = uuid::Uuid::new_v4().to_string();
    let hero_json = serialize_json_value(&page.hero)?;
    let layout_json = serialize_json_value(&page.layout)?;
    let description = page.description.unwrap_or_default();
    let order_index = page.order_index.unwrap_or(0);

    // Insert record
    sqlx::query(
        "INSERT INTO site_pages (id, slug, title, description, nav_label, show_in_nav, order_index, is_published, hero_json, layout_json)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&page.slug)
    .bind(&page.title)
    .bind(description)
    .bind(page.nav_label)
    .bind(if page.show_in_nav { 1 } else { 0 })
    .bind(order_index)
    .bind(if page.is_published { 1 } else { 0 })
    .bind(hero_json)
    .bind(layout_json)
    .execute(pool)
    .await?;

    // Return the inserted state
    get_site_page_by_id(pool, &id)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)
}

/// Updates an existing site page using selective field merging.
pub async fn update_site_page(
    pool: &DbPool,
    id: &str,
    payload: UpdateSitePageRequest,
) -> Result<SitePage, sqlx::Error> {
    if let Some(slug) = payload.slug.as_deref() {
        validate_slug(slug)?;
    }

    // Load existing to allow partial updates
    let mut existing = get_site_page_by_id(pool, id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

    // Apply updates
    if let Some(slug) = payload.slug { existing.slug = slug; }
    if let Some(title) = payload.title { existing.title = title; }
    if let Some(description) = payload.description { existing.description = description; }
    if let Some(nav_label_opt) = payload.nav_label { existing.nav_label = nav_label_opt; }
    if let Some(show_in_nav) = payload.show_in_nav { existing.show_in_nav = show_in_nav; }
    if let Some(order_index) = payload.order_index { existing.order_index = order_index; }
    if let Some(is_published) = payload.is_published { existing.is_published = is_published; }
    if let Some(hero) = payload.hero { existing.hero_json = serialize_json_value(&hero)?; }
    if let Some(layout) = payload.layout { existing.layout_json = serialize_json_value(&layout)?; }

    // Execute UPDATE
    sqlx::query(
        "UPDATE site_pages
         SET slug = ?, title = ?, description = ?, nav_label = ?, show_in_nav = ?, order_index = ?, is_published = ?, hero_json = ?, layout_json = ?, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?",
    )
    .bind(&existing.slug)
    .bind(&existing.title)
    .bind(&existing.description)
    .bind(&existing.nav_label)
    .bind(if existing.show_in_nav { 1 } else { 0 })
    .bind(existing.order_index)
    .bind(if existing.is_published { 1 } else { 0 })
    .bind(&existing.hero_json)
    .bind(&existing.layout_json)
    .bind(id)
    .execute(pool)
    .await?;

    get_site_page_by_id(pool, id)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)
}

pub async fn delete_site_page(pool: &DbPool, id: &str) -> Result<(), sqlx::Error> {
    let result = sqlx::query("DELETE FROM site_pages WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        Err(sqlx::Error::RowNotFound)
    } else {
        Ok(())
    }
}
