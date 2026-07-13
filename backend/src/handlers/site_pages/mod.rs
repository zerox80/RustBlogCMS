//! Site Pages HTTP Handlers
//!
//! This module provides a comprehensive API for managing site pages and their
//! dynamic layouts (hero, layout JSON). It handles both administrative
//! CRUD operations and public-facing content retrieval.

use crate::{
    db,
    handlers::common::{ensure_admin, map_sqlx_error},
    models::{
        bad_request, internal_error, not_found, ApiError, CreateSitePageRequest,
        NavigationItemResponse, NavigationResponse, SitePageListResponse, SitePageResponse,
        SitePageWithPostsResponse, SitePostDetailResponse, SitePostResponse, UpdateSitePageRequest,
    },
    repositories,
    security::auth,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::Value;

mod helpers;
use helpers::*;

/// Handler for listing all site pages.
/// Admin-only. Used for managing the page tree in the CMS.
pub async fn list_site_pages(
    claims: auth::Claims,
    State(pool): State<db::DbPool>,
) -> Result<Json<SitePageListResponse>, ApiError> {
    // RBAC: Verify admin role
    ensure_admin(&claims)?;

    // Fetch records
    let records = repositories::pages::list_site_pages(&pool)
        .await
        .map_err(|err| map_sqlx_error(err, "Site page"))?;

    // Map each database record to a JSON response
    let mut items = Vec::with_capacity(records.len());
    for record in records {
        items.push(map_page(record)?);
    }

    Ok(Json(SitePageListResponse { items }))
}

/// Handler to retrieve full details of a single site page by its ID.
/// Admin-only.
pub async fn get_site_page(
    claims: auth::Claims,
    State(pool): State<db::DbPool>,
    Path(id): Path<String>,
) -> Result<Json<SitePageResponse>, ApiError> {
    // RBAC: Verify admin role
    ensure_admin(&claims)?;

    // Retrieve from repository
    let record = repositories::pages::get_site_page_by_id(&pool, &id)
        .await
        .map_err(|err| map_sqlx_error(err, "Site page"))?
        .ok_or_else(|| not_found("Site page not found"))?;

    // Return mapped JSON
    Ok(Json(map_page(record)?))
}

/// Handler to create a new site page.
/// Admin-only. Performs input sanitation and size checks before insertion.
pub async fn create_site_page(
    claims: auth::Claims,
    State(pool): State<db::DbPool>,
    Json(payload): Json<CreateSitePageRequest>,
) -> Result<Json<SitePageResponse>, ApiError> {
    // RBAC: Ensure admin privileges
    ensure_admin(&claims)?;

    // Validate and clean the request payload
    let payload = sanitize_create_payload(payload)?;

    // Perform the database operation
    let record = repositories::pages::create_site_page(&pool, payload)
        .await
        .map_err(|err| map_sqlx_error(err, "Site page"))?;

    // Audit log
    tracing::info!(
        action = "create_page",
        user = %claims.sub,
        page_id = %record.id,
        page_slug = %record.slug,
        "Admin created new page"
    );

    // Return the newly created state
    Ok(Json(map_page(record)?))
}

/// Handler to update an existing site page.
/// Admin-only. Supports partial updates via UpdateSitePageRequest.
pub async fn update_site_page(
    claims: auth::Claims,
    State(pool): State<db::DbPool>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateSitePageRequest>,
) -> Result<Json<SitePageResponse>, ApiError> {
    // RBAC: Ensure admin privileges
    ensure_admin(&claims)?;

    // Clean and validate the partial update
    let payload = sanitize_update_payload(payload)?;

    // Perform database UPDATE
    let record = repositories::pages::update_site_page(&pool, &id, payload)
        .await
        .map_err(|err| map_sqlx_error(err, "Site page"))?;

    // Audit log
    tracing::info!(
        action = "update_page",
        user = %claims.sub,
        page_id = %id,
        "Admin updated page"
    );

    // Return updated record
    Ok(Json(map_page(record)?))
}

/// Handler to permanently delete a site page and its references.
/// Admin-only.
pub async fn delete_site_page(
    claims: auth::Claims,
    State(pool): State<db::DbPool>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    // RBAC: Verify admin role
    ensure_admin(&claims)?;

    // Execute deletion logic
    repositories::pages::delete_site_page(&pool, &id)
        .await
        .map_err(|err| map_sqlx_error(err, "Site page"))?;

    // Audit log
    tracing::info!(
        action = "delete_page",
        user = %claims.sub,
        page_id = %id,
        "Admin deleted page"
    );

    Ok(StatusCode::NO_CONTENT)
}

/// Handler to retrieve a published page (and its associated posts) by its URL slug.
/// Publicly accessible.
pub async fn get_published_page_by_slug(
    State(pool): State<db::DbPool>,
    Path(slug): Path<String>,
) -> Result<Json<SitePageWithPostsResponse>, ApiError> {
    // Normalize lookup slug
    let lookup_slug = slug.trim().to_lowercase();
    if lookup_slug.is_empty() {
        return Err(bad_request("Slug cannot be empty"));
    }

    // Fetch page metadata from repository
    let page = repositories::pages::get_site_page_by_slug(&pool, &lookup_slug)
        .await
        .map_err(|err| map_sqlx_error(err, "Site page"))?
        .ok_or_else(|| not_found("Page not found"))?;

    // SECURITY: Ensure the page is actually marked as published
    if !page.is_published {
        return Err(not_found("Page not published"));
    }

    // Load child posts for the page (only published ones)
    let posts = repositories::posts::list_published_posts_for_page(&pool, &page.id)
        .await
        .map_err(|err| map_sqlx_error(err, "Posts"))?;

    // Map posts to public DTOs
    let mut post_responses = Vec::with_capacity(posts.len());
    for post in posts {
        post_responses.push(map_post(post));
    }

    // Return the bundle
    Ok(Json(SitePageWithPostsResponse {
        page: map_page(page)?,
        posts: post_responses,
    }))
}

/// Handler to retrieve the dynamic navigation menu.
/// Publicly accessible. Generates a list of navigation items ordered by index.
pub async fn get_navigation(
    State(pool): State<db::DbPool>,
) -> Result<Json<NavigationResponse>, ApiError> {
    // Fetch all records marked for navigation display
    let pages = repositories::pages::list_nav_pages(&pool)
        .await
        .map_err(|err| map_sqlx_error(err, "Navigation"))?;

    // Transform database records into clean navigation items
    let mut items = Vec::with_capacity(pages.len());
    for page in pages {
        let normalized_slug = page.slug.trim().to_lowercase();
        if normalized_slug.is_empty() {
            continue;
        }
        items.push(NavigationItemResponse {
            id: page.id,
            slug: normalized_slug.clone(),
            // Use navigation label if provided, otherwise fallback to page title
            label: page
                .nav_label
                .clone()
                .filter(|label| !label.trim().is_empty())
                .unwrap_or_else(|| page.title.trim().to_string()),
            order_index: page.order_index,
        });
    }

    Ok(Json(NavigationResponse { items }))
}

/// Handler to retrieve a specific published post by both page and post slugs.
/// Publicly accessible. Used for the dynamic routing of blog posts.
pub async fn get_published_post_by_slug(
    State(pool): State<db::DbPool>,
    Path((page_slug, post_slug)): Path<(String, String)>,
) -> Result<Json<SitePostDetailResponse>, ApiError> {
    // Basic validation of slug components
    let lookup_page_slug = page_slug.trim().to_lowercase();
    let lookup_post_slug = post_slug.trim().to_lowercase();

    if lookup_page_slug.is_empty() || lookup_post_slug.is_empty() {
        return Err(bad_request("Slug cannot be empty"));
    }

    // Step 1: Find the parent page and verify visibility
    let page = repositories::pages::get_site_page_by_slug(&pool, &lookup_page_slug)
        .await
        .map_err(|err| map_sqlx_error(err, "Site page"))?
        .ok_or_else(|| not_found("Page not found"))?;

    if !page.is_published {
        return Err(not_found("Page not published"));
    }

    // Step 2: Find the specific post belonging to this page
    let post = repositories::posts::get_published_post_by_slug(&pool, &page.id, &lookup_post_slug)
        .await
        .map_err(|err| map_sqlx_error(err, "Post"))?
        .ok_or_else(|| not_found("Post not found"))?;

    // Assemble the full detail response
    Ok(Json(SitePostDetailResponse {
        page: map_page(page)?,
        post: map_post(post),
    }))
}

/// Handler to list all published page slugs.
/// Publicly accessible. Useful for generating sitemaps or static path pre-generation.
pub async fn list_published_page_slugs(
    State(pool): State<db::DbPool>,
) -> Result<Json<Vec<String>>, ApiError> {
    // Load all published pages
    let pages = repositories::pages::list_published_pages(&pool)
        .await
        .map_err(|err| map_sqlx_error(err, "Navigation"))?;

    // Extract and normalize slugs
    let slugs = pages
        .into_iter()
        .filter_map(|page| {
            let normalized = page.slug.trim().to_lowercase();
            if normalized.is_empty() {
                None
            } else {
                Some(normalized)
            }
        })
        .collect();

    Ok(Json(slugs))
}
