//! Site Posts HTTP Handlers
//!
//! This module provides an API for managing blog posts associated with site pages.
//! It includes full CRUD operations, validation, and administrative controls.

use crate::{
    db,
    handlers::common::{ensure_admin, map_sqlx_error},
    models::{
        bad_request, not_found, ApiError, CreateSitePostRequest, SitePostListResponse,
        SitePostResponse, UpdateSitePostRequest,
    },
    repositories,
    security::auth,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

/// Maximum length for a post title (200 characters)
const MAX_TITLE_LEN: usize = 200;
/// Maximum length for a URL-friendly slug (100 characters)
const MAX_SLUG_LEN: usize = 100;
/// Maximum length for a post excerpt (500 characters)
const MAX_EXCERPT_LEN: usize = 500;
/// Maximum length for the markdown content of a post (100KB)
const MAX_CONTENT_LEN: usize = 100_000;

/// Maps a database SitePost record to a public response structure.
fn map_post(record: crate::models::SitePost) -> SitePostResponse {
    SitePostResponse {
        id: record.id,
        page_id: record.page_id,
        title: record.title,
        slug: record.slug,
        excerpt: record.excerpt,
        content_markdown: record.content_markdown,
        is_published: record.is_published,
        published_at: record.published_at,
        order_index: record.order_index,
        created_at: record.created_at,
        updated_at: record.updated_at,
        allow_comments: record.allow_comments,
    }
}

/// Normalizes a slug (trims and converts to lowercase).
fn sanitize_slug(slug: &str) -> String {
    slug.trim().to_lowercase()
}

fn validate_post_fields(
    title: &str,
    slug: &str,
    excerpt: Option<&str>,
    content: &str,
) -> Result<(), ApiError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(bad_request("Title cannot be empty"));
    }
    if title.len() > MAX_TITLE_LEN {
        return Err(bad_request(format!(
            "Title too long (max {MAX_TITLE_LEN} characters)"
        )));
    }

    let slug = slug.trim().to_lowercase();
    if slug.is_empty() {
        return Err(bad_request("Slug cannot be empty"));
    }
    if slug.len() > MAX_SLUG_LEN {
        return Err(bad_request(format!(
            "Slug too long (max {MAX_SLUG_LEN} characters)"
        )));
    }

    if let Some(excerpt) = excerpt {
        if excerpt.len() > MAX_EXCERPT_LEN {
            return Err(bad_request(format!(
                "Excerpt too long (max {MAX_EXCERPT_LEN} characters)"
            )));
        }
    }

    if content.len() > MAX_CONTENT_LEN {
        return Err(bad_request(format!(
            "Content too long (max {MAX_CONTENT_LEN} characters)"
        )));
    }

    Ok(())
}

/// Handler for listing all posts belonging to a specific site page.
/// Admin-only.
pub async fn list_posts_for_page(
    claims: auth::Claims,
    State(pool): State<db::DbPool>,
    Path(page_id): Path<String>,
) -> Result<Json<SitePostListResponse>, ApiError> {
    ensure_admin(&claims)?;

    repositories::pages::get_site_page_by_id(&pool, &page_id)
        .await
        .map_err(|err| map_sqlx_error(err, "Site page"))?
        .ok_or_else(|| not_found("Site page not found"))?;

    let posts = repositories::posts::list_site_posts_for_page(&pool, &page_id)
        .await
        .map_err(|err| map_sqlx_error(err, "Site post"))?;

    let mut items = Vec::with_capacity(posts.len());
    for post in posts {
        items.push(map_post(post));
    }

    Ok(Json(SitePostListResponse { items }))
}

/// Handler to retrieve a single site post by its ID.
/// Admin-only.
pub async fn get_post(
    claims: auth::Claims,
    State(pool): State<db::DbPool>,
    Path(id): Path<String>,
) -> Result<Json<SitePostResponse>, ApiError> {
    ensure_admin(&claims)?;

    let post = repositories::posts::get_site_post_by_id(&pool, &id)
        .await
        .map_err(|err| map_sqlx_error(err, "Site post"))?
        .ok_or_else(|| not_found("Site post not found"))?;

    Ok(Json(map_post(post)))
}

/// Handler to create a new site post for a specific page.
/// Admin-only, protected by CSRF.
pub async fn create_post(
    claims: auth::Claims,
    _csrf: crate::security::csrf::CsrfGuard,
    State(pool): State<db::DbPool>,
    Path(page_id): Path<String>,
    Json(payload): Json<CreateSitePostRequest>,
) -> Result<Json<SitePostResponse>, ApiError> {
    ensure_admin(&claims)?;

    let trimmed_title = payload.title.trim().to_string();
    let sanitized_slug = sanitize_slug(&payload.slug);
    let excerpt = payload.excerpt.as_ref().map(|e| e.trim());
    validate_post_fields(
        &trimmed_title,
        &sanitized_slug,
        excerpt,
        &payload.content_markdown,
    )?;

    repositories::pages::get_site_page_by_id(&pool, &page_id)
        .await
        .map_err(|err| map_sqlx_error(err, "Site page"))?
        .ok_or_else(|| not_found("Site page not found"))?;

    let record = repositories::posts::create_site_post(
        &pool,
        &page_id,
        CreateSitePostRequest {
            title: trimmed_title,
            slug: sanitized_slug,
            excerpt: payload.excerpt.map(|e| e.trim().to_string()),
            content_markdown: payload.content_markdown,
            is_published: payload.is_published,
            published_at: payload.published_at,
            order_index: payload.order_index,
            allow_comments: payload.allow_comments,
        },
    )
    .await
    .map_err(|err| map_sqlx_error(err, "Site post"))?;

    tracing::info!(
        action = "create_post",
        user = %claims.sub,
        post_id = %record.id,
        post_title = %record.title,
        page_id = %record.page_id,
        "Admin created new post"
    );

    Ok(Json(map_post(record)))
}

/// Handler to update an existing site post.
/// Admin-only, protected by CSRF. Supports partial updates via UpdateSitePostRequest.
pub async fn update_post(
    claims: auth::Claims,
    _csrf: crate::security::csrf::CsrfGuard,
    State(pool): State<db::DbPool>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateSitePostRequest>,
) -> Result<Json<SitePostResponse>, ApiError> {
    ensure_admin(&claims)?;

    if let Some(ref slug) = payload.slug {
        let sanitized = sanitize_slug(slug);
        if sanitized.is_empty() {
            return Err(bad_request("Slug cannot be empty"));
        }
        if sanitized.len() > MAX_SLUG_LEN {
            return Err(bad_request(format!(
                "Slug too long (max {MAX_SLUG_LEN} characters)"
            )));
        }
    }

    if let Some(ref excerpt) = payload.excerpt {
        if excerpt.len() > MAX_EXCERPT_LEN {
            return Err(bad_request(format!(
                "Excerpt too long (max {MAX_EXCERPT_LEN} characters)"
            )));
        }
    }

    if let Some(ref content) = payload.content_markdown {
        if content.len() > MAX_CONTENT_LEN {
            return Err(bad_request(format!(
                "Content too long (max {MAX_CONTENT_LEN} characters)"
            )));
        }
    }

    if let Some(ref title) = payload.title {
        let trimmed = title.trim();
        if trimmed.is_empty() {
            return Err(bad_request("Title cannot be empty"));
        }
        if trimmed.len() > MAX_TITLE_LEN {
            return Err(bad_request(format!(
                "Title must be 1..={MAX_TITLE_LEN} characters"
            )));
        }
    }

    let mut payload = payload;
    if let Some(title) = payload.title.as_mut() {
        *title = title.trim().to_string();
    }
    if let Some(slug) = payload.slug.as_mut() {
        *slug = sanitize_slug(slug);
    }

    let record = repositories::posts::update_site_post(&pool, &id, payload)
        .await
        .map_err(|err| map_sqlx_error(err, "Site post"))?;

    tracing::info!(
        action = "update_post",
        user = %claims.sub,
        post_id = %id,
        "Admin updated post"
    );

    Ok(Json(map_post(record)))
}

/// Handler to permanently delete a site post.
/// Admin-only, protected by CSRF.
pub async fn delete_post(
    claims: auth::Claims,
    _csrf: crate::security::csrf::CsrfGuard,
    State(pool): State<db::DbPool>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    ensure_admin(&claims)?;

    repositories::posts::delete_site_post(&pool, &id)
        .await
        .map_err(|err| map_sqlx_error(err, "Site post"))?;

    tracing::info!(
        action = "delete_post",
        user = %claims.sub,
        post_id = %id,
        "Admin deleted post"
    );

    Ok(StatusCode::NO_CONTENT)
}
