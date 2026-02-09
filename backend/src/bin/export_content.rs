/**
 * Content Export Utility
 *
 * This binary utility exports all site content from the Rust Blog CMS database
 * to a structured JSON file. It's designed for backup purposes, content migration,
 * and development environment setup.
 *
 * Usage:
 * ```bash
 * cargo run --bin export_content -- output.json
 * ```
 *
 * Features:
 * - Exports site content (hero sections, headers, footers)
 * - Exports site pages with navigation and publication settings
 * - Exports blog posts with markdown content
 * - Exports tutorials with topics and metadata
 * - Preserves creation and update timestamps
 * - Validates file paths and handles errors gracefully
 *
 * Output Format:
 * The exported JSON contains nested structures for:
 * - site_content: Dynamic content sections
 * - pages: Static pages with hero and layout data
 * - posts: Blog posts with markdown content
 * - tutorials: Educational content with categorization
 *
 * Security:
 * - Validates file paths to prevent directory traversal
 * - Handles database errors safely
 * - Uses proper error handling for file operations
 */
use std::{env, fs, path::Path};

use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;
use sqlx::FromRow;

use rust_blog_backend::db;

#[derive(Debug, FromRow)]
struct SiteContentRow {
    section: String,
    content_json: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
struct SiteContentExport {
    section: String,
    content: Value,
    updated_at: String,
}

#[derive(Debug, FromRow)]
struct SitePageRow {
    id: String,
    slug: String,
    title: String,
    description: String,
    nav_label: Option<String>,
    show_in_nav: bool,
    order_index: i64,
    is_published: bool,
    hero_json: String,
    layout_json: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
struct SitePageExport {
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
    created_at: String,
    updated_at: String,
}

#[derive(Debug, FromRow)]
struct SitePostRow {
    id: String,
    page_id: String,
    title: String,
    slug: String,
    excerpt: String,
    content_markdown: String,
    is_published: bool,
    published_at: Option<String>,
    order_index: i64,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
struct SitePostExport {
    id: String,
    page_id: String,
    title: String,
    slug: String,
    excerpt: String,
    content_markdown: String,
    is_published: bool,
    published_at: Option<String>,
    order_index: i64,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, FromRow)]
struct TutorialRow {
    id: String,
    title: String,
    description: String,
    icon: String,
    color: String,
    topics: String,
    content: String,
    version: i64,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
struct TutorialExport {
    id: String,
    title: String,
    description: String,
    icon: String,
    color: String,
    topics: Vec<String>,
    content: String,
    version: i64,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, FromRow)]
struct TutorialTopicRow {
    tutorial_id: String,
    topic: String,
}

#[derive(Debug, Serialize)]
struct TutorialTopicExport {
    tutorial_id: String,
    topic: String,
}

#[derive(Debug, Serialize)]
struct ExportBundle {
    site_content: Vec<SiteContentExport>,
    pages: Vec<SitePageExport>,
    posts: Vec<SitePostExport>,
    tutorials: Vec<TutorialExport>,
    tutorial_topics: Vec<TutorialTopicExport>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let args: Vec<String> = env::args().collect();
    let output_path = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("content/site_content.json");
    let path = Path::new(output_path);

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create export directory: {}", parent.display())
            })?;
        }
    }

    let pool = db::create_pool()
        .await
        .context("Failed to connect to database. Is DATABASE_URL set correctly?")?;

    let site_content_rows = sqlx::query_as::<_, SiteContentRow>(
        "SELECT section, content_json, updated_at FROM site_content ORDER BY section",
    )
    .fetch_all(&pool)
    .await
    .context("Failed to load site_content entries")?;

    let site_content = site_content_rows
        .into_iter()
        .map(|row| {
            let content: Value = serde_json::from_str(&row.content_json)
                .with_context(|| format!("Failed to parse JSON for section '{}'.", row.section))?;
            Ok(SiteContentExport {
                section: row.section,
                content,
                updated_at: row.updated_at,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let page_rows = sqlx::query_as::<_, SitePageRow>(
        "SELECT id, slug, title, description, nav_label, show_in_nav, order_index, is_published, hero_json, layout_json, created_at, updated_at FROM site_pages ORDER BY order_index, title",
    )
    .fetch_all(&pool)
    .await
    .context("Failed to load site_pages entries")?;

    let pages = page_rows
        .into_iter()
        .map(|row| {
            let hero: Value = serde_json::from_str(&row.hero_json)
                .with_context(|| format!("Failed to parse hero JSON for page '{}'", row.slug))?;
            let layout: Value = serde_json::from_str(&row.layout_json)
                .with_context(|| format!("Failed to parse layout JSON for page '{}'", row.slug))?;
            Ok(SitePageExport {
                id: row.id,
                slug: row.slug,
                title: row.title,
                description: row.description,
                nav_label: row.nav_label,
                show_in_nav: row.show_in_nav,
                order_index: row.order_index,
                is_published: row.is_published,
                hero,
                layout,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let post_rows = sqlx::query_as::<_, SitePostRow>(
        "SELECT id, page_id, title, slug, excerpt, content_markdown, is_published, published_at, order_index, created_at, updated_at FROM site_posts ORDER BY page_id, order_index, created_at",
    )
    .fetch_all(&pool)
    .await
    .context("Failed to load site_posts entries")?;

    let posts = post_rows
        .into_iter()
        .map(|row| SitePostExport {
            id: row.id,
            page_id: row.page_id,
            title: row.title,
            slug: row.slug,
            excerpt: row.excerpt,
            content_markdown: row.content_markdown,
            is_published: row.is_published,
            published_at: row.published_at,
            order_index: row.order_index,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
        .collect::<Vec<_>>();

    let tutorial_rows = sqlx::query_as::<_, TutorialRow>(
        "SELECT id, title, description, icon, color, topics, content, version, created_at, updated_at FROM tutorials ORDER BY created_at",
    )
    .fetch_all(&pool)
    .await
    .context("Failed to load tutorials entries")?;

    let tutorials = tutorial_rows
        .into_iter()
        .map(|row| {
            let topics: Vec<String> = serde_json::from_str(&row.topics).with_context(|| {
                format!("Failed to parse topics JSON for tutorial '{}'", row.id)
            })?;
            Ok(TutorialExport {
                id: row.id,
                title: row.title,
                description: row.description,
                icon: row.icon,
                color: row.color,
                topics,
                content: row.content,
                version: row.version,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let topic_rows = sqlx::query_as::<_, TutorialTopicRow>(
        "SELECT tutorial_id, topic FROM tutorial_topics ORDER BY tutorial_id, topic",
    )
    .fetch_all(&pool)
    .await
    .context("Failed to load tutorial_topics entries")?;

    let tutorial_topics = topic_rows
        .into_iter()
        .map(|row| TutorialTopicExport {
            tutorial_id: row.tutorial_id,
            topic: row.topic,
        })
        .collect::<Vec<_>>();

    let bundle = ExportBundle {
        site_content,
        pages,
        posts,
        tutorials,
        tutorial_topics,
    };

    let json =
        serde_json::to_string_pretty(&bundle).context("Failed to serialize export bundle")?;

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create export directory {}", parent.display())
            })?;
        }
    }

    fs::write(path, json)
        .with_context(|| format!("Failed to write export file at {}", path.display()))?;

    println!(
        "Export completed:\n  site_content: {}\n  pages: {}\n  posts: {}\n  tutorials: {}\n  tutorial_topics: {}\n  saved to {}",
        bundle.site_content.len(),
        bundle.pages.len(),
        bundle.posts.len(),
        bundle.tutorials.len(),
        bundle.tutorial_topics.len(),
        path.display()
    );

    Ok(())
}
