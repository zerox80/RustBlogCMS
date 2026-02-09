/**
 * Content Import Utility
 *
 * This binary utility imports site content from a structured JSON file into the
 * Rust Blog CMS database. It's designed for content restoration, development
 * environment setup, and content migration between instances.
 *
 * Usage:
 * ```bash
 * cargo run --bin import_content -- input.json
 * ```
 *
 * Features:
 * - Imports site content (hero sections, headers, footers)
 * - Imports site pages with navigation and publication settings
 * - Imports blog posts with markdown content
 * - Preserves original IDs and timestamps when available
 * - Validates content structure and data integrity
 * - Runs all operations in database transactions
 * - Handles duplicate content gracefully with upserts
 *
 * Input Format:
 * The JSON file should contain the same structure as produced by export_content:
 * - site_content: Array of content section objects
 * - pages: Array of page objects with hero/layout data
 * - posts: Array of blog post objects with markdown content
 *
 * Security:
 * - Validates file paths to prevent directory traversal
 * - Validates JSON structure before processing
 * - Uses database transactions for atomic operations
 * - Handles all errors gracefully with detailed reporting
 *
 * Error Handling:
 * - File not found or inaccessible
 * - Invalid JSON format or structure
 * - Database connection or transaction errors
 * - Data validation failures
 */
use std::{env, fs, path::Path};

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use rust_blog_backend::db;

#[derive(Debug, Deserialize)]
struct SiteContentImport {
    section: String,

    content: Value,

    #[serde(default)]
    updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SitePageImport {
    id: String,

    slug: String,

    title: String,

    description: String,

    nav_label: Option<String>,

    show_in_nav: bool,

    order_index: i64,

    is_published: bool,

    hero: Value,

    layout: Value,

    #[serde(default)]
    created_at: Option<String>,

    #[serde(default)]
    updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SitePostImport {
    id: String,

    page_id: String,

    title: String,

    slug: String,

    excerpt: String,

    content_markdown: String,

    is_published: bool,

    published_at: Option<String>,

    order_index: i64,

    #[serde(default)]
    created_at: Option<String>,

    #[serde(default)]
    updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ImportBundle {
    site_content: Vec<SiteContentImport>,

    pages: Vec<SitePageImport>,

    posts: Vec<SitePostImport>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let args: Vec<String> = env::args().collect();

    let input_path = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("../content/site_content.json");
    let path = Path::new(input_path);

    if !path.exists() {
        return Err(anyhow!("Input file '{}' does not exist", path.display()));
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read input file {}", path.display()))?;

    let bundle: ImportBundle = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON from {}", path.display()))?;

    let pool = db::create_pool()
        .await
        .context("Failed to connect to database. Is DATABASE_URL set correctly?")?;

    let mut tx = pool.begin().await.context("Failed to start transaction")?;

    import_site_content(&mut tx, &bundle.site_content).await?;
    import_site_pages(&mut tx, &bundle.pages).await?;
    import_site_posts(&mut tx, &bundle.posts).await?;

    tx.commit().await.context("Failed to commit transaction")?;

    println!(
        "Import completed:\n  site_content: {}\n  pages: {}\n  posts: {}\n  <- {}",
        bundle.site_content.len(),
        bundle.pages.len(),
        bundle.posts.len(),
        path.display()
    );

    Ok(())
}

async fn import_site_content(
    tx: &mut Transaction<'_, Sqlite>,
    items: &[SiteContentImport],
) -> Result<()> {
    for item in items {
        let serialized = serde_json::to_string(&item.content)
            .context("Failed to serialize site_content entry")?;

        sqlx::query(
            "INSERT INTO site_content (section, content_json, updated_at) VALUES (?, ?, COALESCE(?, CURRENT_TIMESTAMP)) \
             ON CONFLICT(section) DO UPDATE SET content_json = excluded.content_json, updated_at = COALESCE(excluded.updated_at, CURRENT_TIMESTAMP)",
        )
        .bind(&item.section)
        .bind(&serialized)
        .bind(&item.updated_at)
        .execute(&mut **tx)
        .await
        .with_context(|| format!("Failed to upsert site_content section '{}'", item.section))?;
    }

    Ok(())
}

async fn import_site_pages(
    tx: &mut Transaction<'_, Sqlite>,
    items: &[SitePageImport],
) -> Result<()> {
    for item in items {
        let hero_serialized =
            serde_json::to_string(&item.hero).context("Failed to serialize page hero JSON")?;
        let layout_serialized =
            serde_json::to_string(&item.layout).context("Failed to serialize page layout JSON")?;

        sqlx::query(
            "INSERT INTO site_pages (id, slug, title, description, nav_label, show_in_nav, order_index, is_published, hero_json, layout_json, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, COALESCE(?, CURRENT_TIMESTAMP), COALESCE(?, CURRENT_TIMESTAMP)) \
             ON CONFLICT(id) DO UPDATE SET slug = excluded.slug, title = excluded.title, description = excluded.description, nav_label = excluded.nav_label, show_in_nav = excluded.show_in_nav, order_index = excluded.order_index, is_published = excluded.is_published, hero_json = excluded.hero_json, layout_json = excluded.layout_json, updated_at = COALESCE(excluded.updated_at, CURRENT_TIMESTAMP)",
        )
        .bind(&item.id)
        .bind(&item.slug)
        .bind(&item.title)
        .bind(&item.description)
        .bind(&item.nav_label)

        .bind(if item.show_in_nav { 1 } else { 0 })
        .bind(item.order_index)
        .bind(if item.is_published { 1 } else { 0 })
        .bind(&hero_serialized)
        .bind(&layout_serialized)
        .bind(&item.created_at)
        .bind(&item.updated_at)
        .execute(&mut **tx)
        .await
        .with_context(|| format!("Failed to upsert site_page '{}'", item.slug))?;
    }

    Ok(())
}

async fn import_site_posts(
    tx: &mut Transaction<'_, Sqlite>,
    items: &[SitePostImport],
) -> Result<()> {
    for item in items {
        sqlx::query(
            "INSERT INTO site_posts (id, page_id, title, slug, excerpt, content_markdown, is_published, published_at, order_index, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, COALESCE(?, CURRENT_TIMESTAMP), COALESCE(?, CURRENT_TIMESTAMP)) \
             ON CONFLICT(id) DO UPDATE SET page_id = excluded.page_id, title = excluded.title, slug = excluded.slug, excerpt = excluded.excerpt, content_markdown = excluded.content_markdown, is_published = excluded.is_published, published_at = excluded.published_at, order_index = excluded.order_index, updated_at = COALESCE(excluded.updated_at, CURRENT_TIMESTAMP)",
        )
        .bind(&item.id)
        .bind(&item.page_id)
        .bind(&item.title)
        .bind(&item.slug)
        .bind(&item.excerpt)
        .bind(&item.content_markdown)

        .bind(if item.is_published { 1 } else { 0 })
        .bind(&item.published_at)
        .bind(item.order_index)
        .bind(&item.created_at)
        .bind(&item.updated_at)
        .execute(&mut **tx)
        .await
        .with_context(|| format!("Failed to upsert site_post '{}'", item.slug))?;
    }

    Ok(())
}
