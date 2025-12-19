use crate::{
    security::auth, db,
    models::{
        CreateSitePageRequest, ErrorResponse, NavigationItemResponse, NavigationResponse,
        SitePageListResponse, SitePageResponse, SitePageWithPostsResponse, SitePostDetailResponse,
        SitePostResponse, UpdateSitePageRequest,
    },
    repositories,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::Value;
use sqlx;

const MAX_TITLE_LEN: usize = 200;
const MAX_DESCRIPTION_LEN: usize = 1000;
const MAX_NAV_LABEL_LEN: usize = 100;
const MAX_JSON_BYTES: usize = 200_000;

fn ensure_admin(claims: &auth::Claims) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if claims.role != "admin" {
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

fn map_sqlx_error(err: sqlx::Error, context: &str) -> (StatusCode, Json<ErrorResponse>) {
    match err {
        sqlx::Error::RowNotFound => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("{context} not found"),
            }),
        ),
        sqlx::Error::Protocol(e) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        ),
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
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Database error".to_string(),
                    }),
                )
            }
        }
        other => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Unexpected database error: {other}"),
            }),
        ),
    }
}

fn validate_json_size(value: &Value, field: &str) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    match serde_json::to_string(value) {
        Ok(serialized) if serialized.len() <= MAX_JSON_BYTES => Ok(()),
        Ok(_) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("{field} JSON exceeds maximum size of {MAX_JSON_BYTES} bytes"),
            }),
        )),
        Err(err) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Invalid {field} JSON: {err}"),
            }),
        )),
    }
}

fn sanitize_create_payload(
    mut payload: CreateSitePageRequest,
) -> Result<CreateSitePageRequest, (StatusCode, Json<ErrorResponse>)> {
    payload.slug = payload.slug.trim().to_lowercase();
    if payload.slug.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Slug cannot be empty".to_string(),
            }),
        ));
    }

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

    validate_json_size(&payload.hero, "hero")?;
    validate_json_size(&payload.layout, "layout")?;

    Ok(payload)
}

fn sanitize_update_payload(
    mut payload: UpdateSitePageRequest,
) -> Result<UpdateSitePageRequest, (StatusCode, Json<ErrorResponse>)> {
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

    if let Some(ref hero) = payload.hero {
        validate_json_size(hero, "hero")?;
    }
    if let Some(ref layout) = payload.layout {
        validate_json_size(layout, "layout")?;
    }

    Ok(payload)
}

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

    let hero = serde_json::from_str::<Value>(&hero_json).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to parse stored hero JSON: {err}"),
            }),
        )
    })?;

    let layout = serde_json::from_str::<Value>(&layout_json).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to parse stored layout JSON: {err}"),
            }),
        )
    })?;

    let sanitized_slug = slug.trim().to_lowercase();
    let sanitized_title = match title.trim() {
        "" => sanitized_slug.clone(),
        value => value.to_string(),
    };

    let sanitized_description = description.trim().to_string();

    let sanitized_nav_label = nav_label.and_then(|label| {
        let trimmed = label.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });

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

pub async fn list_site_pages(
    claims: auth::Claims,
    State(pool): State<db::DbPool>,
) -> Result<Json<SitePageListResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_admin(&claims)?;

    let records = repositories::pages::list_site_pages(&pool)
        .await
        .map_err(|err| map_sqlx_error(err, "Site page"))?;

    let mut items = Vec::with_capacity(records.len());
    for record in records {
        items.push(map_page(record)?);
    }

    Ok(Json(SitePageListResponse { items }))
}

pub async fn get_site_page(
    claims: auth::Claims,
    State(pool): State<db::DbPool>,
    Path(id): Path<String>,
) -> Result<Json<SitePageResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_admin(&claims)?;

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

    Ok(Json(map_page(record)?))
}

pub async fn create_site_page(
    claims: auth::Claims,
    State(pool): State<db::DbPool>,
    Json(payload): Json<CreateSitePageRequest>,
) -> Result<Json<SitePageResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_admin(&claims)?;

    let payload = sanitize_create_payload(payload)?;

    let record = repositories::pages::create_site_page(&pool, payload)
        .await
        .map_err(|err| map_sqlx_error(err, "Site page"))?;

    tracing::info!(
        action = "create_page",
        user = %claims.sub,
        page_id = %record.id,
        page_slug = %record.slug,
        "Admin created new page"
    );

    Ok(Json(map_page(record)?))
}

pub async fn update_site_page(
    claims: auth::Claims,
    State(pool): State<db::DbPool>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateSitePageRequest>,
) -> Result<Json<SitePageResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_admin(&claims)?;

    let payload = sanitize_update_payload(payload)?;

    let record = repositories::pages::update_site_page(&pool, &id, payload)
        .await
        .map_err(|err| map_sqlx_error(err, "Site page"))?;

    tracing::info!(
        action = "update_page",
        user = %claims.sub,
        page_id = %id,
        "Admin updated page"
    );

    Ok(Json(map_page(record)?))
}

pub async fn delete_site_page(
    claims: auth::Claims,
    State(pool): State<db::DbPool>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    ensure_admin(&claims)?;

    repositories::pages::delete_site_page(&pool, &id)
        .await
        .map_err(|err| map_sqlx_error(err, "Site page"))?;

    tracing::info!(
        action = "delete_page",
        user = %claims.sub,
        page_id = %id,
        "Admin deleted page"
    );

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_published_page_by_slug(
    State(pool): State<db::DbPool>,
    Path(slug): Path<String>,
) -> Result<Json<SitePageWithPostsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let lookup_slug = slug.trim().to_lowercase();
    if lookup_slug.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Slug cannot be empty".to_string(),
            }),
        ));
    }

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

    if !page.is_published {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Page not published".to_string(),
            }),
        ));
    }

    let posts = repositories::posts::list_published_posts_for_page(&pool, &page.id)
        .await
        .map_err(|err| map_sqlx_error(err, "Posts"))?;

    let mut post_responses = Vec::with_capacity(posts.len());
    for post in posts {
        post_responses.push(map_post(post));
    }

    Ok(Json(SitePageWithPostsResponse {
        page: map_page(page)?,
        posts: post_responses,
    }))
}

pub async fn get_navigation(
    State(pool): State<db::DbPool>,
) -> Result<Json<NavigationResponse>, (StatusCode, Json<ErrorResponse>)> {
    let pages = repositories::pages::list_nav_pages(&pool)
        .await
        .map_err(|err| map_sqlx_error(err, "Navigation"))?;

    let mut items = Vec::with_capacity(pages.len());
    for page in pages {
        let normalized_slug = page.slug.trim().to_lowercase();
        if normalized_slug.is_empty() {
            continue;
        }
        items.push(NavigationItemResponse {
            id: page.id,
            slug: normalized_slug.clone(),
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

pub async fn get_published_post_by_slug(
    State(pool): State<db::DbPool>,
    Path((page_slug, post_slug)): Path<(String, String)>,
) -> Result<Json<SitePostDetailResponse>, (StatusCode, Json<ErrorResponse>)> {
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

    Ok(Json(SitePostDetailResponse {
        page: map_page(page)?,
        post: map_post(post),
    }))
}

pub async fn list_published_page_slugs(
    State(pool): State<db::DbPool>,
) -> Result<Json<Vec<String>>, (StatusCode, Json<ErrorResponse>)> {
    let pages = repositories::pages::list_published_pages(&pool)
        .await
        .map_err(|err| map_sqlx_error(err, "Navigation"))?;

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
