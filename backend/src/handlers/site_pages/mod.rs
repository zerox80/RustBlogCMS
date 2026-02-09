//! Site Pages HTTP Handlers
//!
//! This module provides a comprehensive API for managing site pages and their
//! dynamic layouts (hero, layout JSON). It handles both administrative
//! CRUD operations and public-facing content retrieval.

use crate::{
    db,
    models::{
        CreateSitePageRequest, ErrorResponse, NavigationItemResponse, NavigationResponse,
        SitePageListResponse, SitePageResponse, SitePageWithPostsResponse, SitePostDetailResponse,
        SitePostResponse, UpdateSitePageRequest,
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
use sqlx;

/// Maximum length for a page title (200 characters)
const MAX_TITLE_LEN: usize = 200;
/// Maximum length for a page SEO description (1000 characters)
const MAX_DESCRIPTION_LEN: usize = 1000;
/// Maximum length for a navigation label (100 characters)
const MAX_NAV_LABEL_LEN: usize = 100;
/// Maximum allowed size for hero/layout JSON payloads (200KB)
const MAX_JSON_BYTES: usize = 200_000;

/// Ensures the current user has administrative privileges.
fn ensure_admin(claims: &auth::Claims) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if claims.role != "admin" {
        // Return 403 Forbidden if not admin
        Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Insufficient permissions".to_string(),
            }),
        ))
    } else {
        Ok(())
    }
}

/// Maps database errors to user-facing HTTP responses.
fn map_sqlx_error(err: sqlx::Error, context: &str) -> (StatusCode, Json<ErrorResponse>) {
    match err {
        // Handle 404
        sqlx::Error::RowNotFound => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("{context} not found"),
            }),
        ),
        // Handle malformed requests
        sqlx::Error::Protocol(e) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        ),
        // Handle unique constraint violations (e.g. slug conflict)
        sqlx::Error::Database(db_err) => {
            if db_err.is_unique_violation() {
                (
                    StatusCode::CONFLICT,
                    Json(ErrorResponse {
                        error: db_err
                            .constraint()
                            .map(|c| format!("Duplicate value violates unique constraint '{c}'"))
                            .unwrap_or_else(|| {
                                "Duplicate value violates unique constraint".to_string()
                            }),
                    }),
                )
            } else {
                // General DB error
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Database error".to_string(),
                    }),
                )
            }
        }
        // Fallback for unexpected errors
        other => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Unexpected database error: {other}"),
            }),
        ),
    }
}

/// Validates that a JSON value, when serialized, doesn't exceed the byte limit.
fn validate_json_size(value: &Value, field: &str) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    match serde_json::to_string(value) {
        // Within bounds
        Ok(serialized) if serialized.len() <= MAX_JSON_BYTES => Ok(()),
        // Over limit
        Ok(_) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("{field} JSON exceeds maximum size of {MAX_JSON_BYTES} bytes"),
            }),
        )),
        // Invalid JSON content
        Err(err) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Invalid {field} JSON: {err}"),
            }),
        )),
    }
}

/// Normalizes and validates a payload for creating a new site page.
fn sanitize_create_payload(
    mut payload: CreateSitePageRequest,
) -> Result<CreateSitePageRequest, (StatusCode, Json<ErrorResponse>)> {
    // Slug normalization: trim and lowercase
    payload.slug = payload.slug.trim().to_lowercase();
    if payload.slug.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Slug cannot be empty".to_string(),
            }),
        ));
    }

    // Title normalization and length check
    payload.title = payload.title.trim().to_string();
    if payload.title.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Title cannot be empty".to_string(),
            }),
        ));
    }
    if payload.title.len() > MAX_TITLE_LEN {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Title too long (max {MAX_TITLE_LEN} characters)"),
            }),
        ));
    }

    // Description length check
    payload.description = payload.description.map(|desc| desc.trim().to_string());
    if let Some(desc) = payload.description.as_ref() {
        if desc.len() > MAX_DESCRIPTION_LEN {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Description too long (max {MAX_DESCRIPTION_LEN} characters)"),
                }),
            ));
        }
    }

    // Navigation label normalization
    payload.nav_label = payload.nav_label.and_then(|label| {
        let trimmed = label.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });
    if let Some(label) = payload.nav_label.as_ref() {
        if label.len() > MAX_NAV_LABEL_LEN {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!(
                        "Navigation label too long (max {MAX_NAV_LABEL_LEN} characters)"
                    ),
                }),
            ));
        }
    }

    // Large JSON field size validation
    validate_json_size(&payload.hero, "hero")?;
    validate_json_size(&payload.layout, "layout")?;

    Ok(payload)
}

/// Normalizes and validates a payload for updating an existing site page.
fn sanitize_update_payload(
    mut payload: UpdateSitePageRequest,
) -> Result<UpdateSitePageRequest, (StatusCode, Json<ErrorResponse>)> {
    // Partial slug update
    if let Some(ref mut slug) = payload.slug {
        *slug = slug.trim().to_lowercase();
        if slug.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Slug cannot be empty".to_string(),
                }),
            ));
        }
    }

    // Partial title update
    if let Some(ref mut title) = payload.title {
        *title = title.trim().to_string();
        if title.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Title cannot be empty".to_string(),
                }),
            ));
        }
        if title.len() > MAX_TITLE_LEN {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Title too long (max {MAX_TITLE_LEN} characters)"),
                }),
            ));
        }
    }

    // Partial description update
    if let Some(ref mut description) = payload.description {
        *description = description.trim().to_string();
        if description.len() > MAX_DESCRIPTION_LEN {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Description too long (max {MAX_DESCRIPTION_LEN} characters)"),
                }),
            ));
        }
    }

    // Partial navigation label update
    if let Some(mut nav_label_option) = payload.nav_label.take() {
        nav_label_option = match nav_label_option {
            Some(label) => {
                let trimmed = label.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    if trimmed.len() > MAX_NAV_LABEL_LEN {
                        return Err((
                            StatusCode::BAD_REQUEST,
                            Json(ErrorResponse {
                                error: format!(
                                    "Navigation label too long (max {MAX_NAV_LABEL_LEN} characters)"
                                ),
                            }),
                        ));
                    }
                    Some(trimmed)
                }
            }
            None => None,
        };

        payload.nav_label = Some(nav_label_option);
    }

    // Partial JSON field update
    if let Some(ref hero) = payload.hero {
        validate_json_size(hero, "hero")?;
    }
    if let Some(ref layout) = payload.layout {
        validate_json_size(layout, "layout")?;
    }

    Ok(payload)
}

/// Maps a database SitePage record to a rich response model, including JSON parsing.
fn map_page(
    page: crate::models::SitePage,
) -> Result<SitePageResponse, (StatusCode, Json<ErrorResponse>)> {
    let crate::models::SitePage {
        id,
        slug,
        title,
        description,
        nav_label,
        show_in_nav,
        order_index,
        is_published,
        hero_json,
        layout_json,
        created_at,
        updated_at,
    } = page;

    // Parse hero JSON string from database into a serde_json::Value
    let hero = serde_json::from_str::<Value>(&hero_json).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to parse stored hero JSON: {err}"),
            }),
        )
    })?;

    // Parse layout JSON string from database into a serde_json::Value
    let layout = serde_json::from_str::<Value>(&layout_json).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to parse stored layout JSON: {err}"),
            }),
        )
    })?;

    // Normalize slug for output
    let sanitized_slug = slug.trim().to_lowercase();

    // Default title to slug if the title field is empty
    let sanitized_title = match title.trim() {
        "" => sanitized_slug.clone(),
        value => value.to_string(),
    };

    // Trim description
    let sanitized_description = description.trim().to_string();

    // Clean up navigation label
    let sanitized_nav_label = nav_label.and_then(|label| {
        let trimmed = label.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });

    // Assemble response
    Ok(SitePageResponse {
        id,
        slug: sanitized_slug,
        title: sanitized_title,
        description: sanitized_description,
        nav_label: sanitized_nav_label,
        show_in_nav,
        order_index,
        is_published,
        hero,
        layout,
        created_at,
        updated_at,
    })
}

/// Maps a database SitePost record to a public response model.
fn map_post(post: crate::models::SitePost) -> SitePostResponse {
    SitePostResponse {
        id: post.id,
        page_id: post.page_id,
        title: post.title,
        slug: post.slug,
        excerpt: post.excerpt,
        content_markdown: post.content_markdown,
        is_published: post.is_published,
        published_at: post.published_at,
        order_index: post.order_index,
        created_at: post.created_at,
        updated_at: post.updated_at,
        allow_comments: post.allow_comments,
    }
}

/// Handler for listing all site pages.
/// Admin-only. Used for managing the page tree in the CMS.
pub async fn list_site_pages(
    claims: auth::Claims,
    State(pool): State<db::DbPool>,
) -> Result<Json<SitePageListResponse>, (StatusCode, Json<ErrorResponse>)> {
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
) -> Result<Json<SitePageResponse>, (StatusCode, Json<ErrorResponse>)> {
    // RBAC: Verify admin role
    ensure_admin(&claims)?;

    // Retrieve from repository
    let record = repositories::pages::get_site_page_by_id(&pool, &id)
        .await
        .map_err(|err| map_sqlx_error(err, "Site page"))?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Site page not found".to_string(),
                }),
            )
        })?;

    // Return mapped JSON
    Ok(Json(map_page(record)?))
}

/// Handler to create a new site page.
/// Admin-only. Performs input sanitation and size checks before insertion.
pub async fn create_site_page(
    claims: auth::Claims,
    State(pool): State<db::DbPool>,
    Json(payload): Json<CreateSitePageRequest>,
) -> Result<Json<SitePageResponse>, (StatusCode, Json<ErrorResponse>)> {
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
) -> Result<Json<SitePageResponse>, (StatusCode, Json<ErrorResponse>)> {
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
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
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
) -> Result<Json<SitePageWithPostsResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Normalize lookup slug
    let lookup_slug = slug.trim().to_lowercase();
    if lookup_slug.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Slug cannot be empty".to_string(),
            }),
        ));
    }

    // Fetch page metadata from repository
    let page = repositories::pages::get_site_page_by_slug(&pool, &lookup_slug)
        .await
        .map_err(|err| map_sqlx_error(err, "Site page"))?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Page not found".to_string(),
                }),
            )
        })?;

    // SECURITY: Ensure the page is actually marked as published
    if !page.is_published {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Page not published".to_string(),
            }),
        ));
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
) -> Result<Json<NavigationResponse>, (StatusCode, Json<ErrorResponse>)> {
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
) -> Result<Json<SitePostDetailResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Basic validation of slug components
    let lookup_page_slug = page_slug.trim().to_lowercase();
    let lookup_post_slug = post_slug.trim().to_lowercase();

    if lookup_page_slug.is_empty() || lookup_post_slug.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Slug cannot be empty".to_string(),
            }),
        ));
    }

    // Step 1: Find the parent page and verify visibility
    let page = repositories::pages::get_site_page_by_slug(&pool, &lookup_page_slug)
        .await
        .map_err(|err| map_sqlx_error(err, "Site page"))?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Page not found".to_string(),
                }),
            )
        })?;

    if !page.is_published {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Page not published".to_string(),
            }),
        ));
    }

    // Step 2: Find the specific post belonging to this page
    let post = repositories::posts::get_published_post_by_slug(&pool, &page.id, &lookup_post_slug)
        .await
        .map_err(|err| map_sqlx_error(err, "Post"))?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Post not found".to_string(),
                }),
            )
        })?;

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
) -> Result<Json<Vec<String>>, (StatusCode, Json<ErrorResponse>)> {
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
