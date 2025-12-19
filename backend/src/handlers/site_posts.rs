use crate::{
    security::auth, db,
    models::{
        CreateSitePostRequest, ErrorResponse, SitePostListResponse, SitePostResponse,
        UpdateSitePostRequest,
    },
    repositories,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx;

const MAX_TITLE_LEN: usize = 200;
const MAX_SLUG_LEN: usize = 100;
const MAX_EXCERPT_LEN: usize = 500;
const MAX_CONTENT_LEN: usize = 100_000;

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

fn sanitize_slug(slug: &str) -> String {
    slug.trim().to_lowercase()
}

fn validate_post_fields(
    title: &str,
    slug: &str,
    excerpt: Option<&str>,
    content: &str,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let title = title.trim();
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

    let slug = slug.trim().to_lowercase();
    if slug.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Slug cannot be empty".to_string(),
            }),
        ));
    }
    if slug.len() > MAX_SLUG_LEN {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Slug too long (max {MAX_SLUG_LEN} characters)"),
            }),
        ));
    }

    if let Some(excerpt) = excerpt {
        if excerpt.len() > MAX_EXCERPT_LEN {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Excerpt too long (max {MAX_EXCERPT_LEN} characters)"),
                }),
            ));
        }
    }

    if content.len() > MAX_CONTENT_LEN {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Content too long (max {MAX_CONTENT_LEN} characters)"),
            }),
        ));
    }

    Ok(())
}

pub async fn list_posts_for_page(
    claims: auth::Claims,
    State(pool): State<db::DbPool>,
    Path(page_id): Path<String>,
) -> Result<Json<SitePostListResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_admin(&claims)?;

    repositories::pages::get_site_page_by_id(&pool, &page_id)
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

    let posts = repositories::posts::list_site_posts_for_page(&pool, &page_id)
        .await
        .map_err(|err| map_sqlx_error(err, "Site post"))?;

    let mut items = Vec::with_capacity(posts.len());
    for post in posts {
        items.push(map_post(post));
    }

    Ok(Json(SitePostListResponse { items }))
}

pub async fn get_post(
    claims: auth::Claims,
    State(pool): State<db::DbPool>,
    Path(id): Path<String>,
) -> Result<Json<SitePostResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_admin(&claims)?;

    let post = repositories::posts::get_site_post_by_id(&pool, &id)
        .await
        .map_err(|err| map_sqlx_error(err, "Site post"))?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Site post not found".to_string(),
                }),
            )
        })?;

    Ok(Json(map_post(post)))
}

pub async fn create_post(
    claims: auth::Claims,
    _csrf: crate::security::csrf::CsrfGuard,
    State(pool): State<db::DbPool>,
    Path(page_id): Path<String>,
    Json(payload): Json<CreateSitePostRequest>,
) -> Result<Json<SitePostResponse>, (StatusCode, Json<ErrorResponse>)> {
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
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Site page not found".to_string(),
                }),
            )
        })?;

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

pub async fn update_post(
    claims: auth::Claims,
    _csrf: crate::security::csrf::CsrfGuard,
    State(pool): State<db::DbPool>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateSitePostRequest>,
) -> Result<Json<SitePostResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_admin(&claims)?;

    if let Some(ref slug) = payload.slug {
        let sanitized = sanitize_slug(slug);
        if sanitized.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Slug cannot be empty".to_string(),
                }),
            ));
        }
        if sanitized.len() > MAX_SLUG_LEN {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Slug too long (max {MAX_SLUG_LEN} characters)"),
                }),
            ));
        }
    }

    if let Some(ref excerpt) = payload.excerpt {
        if excerpt.len() > MAX_EXCERPT_LEN {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Excerpt too long (max {MAX_EXCERPT_LEN} characters)"),
                }),
            ));
        }
    }

    if let Some(ref content) = payload.content_markdown {
        if content.len() > MAX_CONTENT_LEN {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Content too long (max {MAX_CONTENT_LEN} characters)"),
                }),
            ));
        }
    }

    if let Some(ref title) = payload.title {
        if title.trim().is_empty() || title.trim().len() > MAX_TITLE_LEN {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Title must be 1..={MAX_TITLE_LEN} characters"),
                }),
            ));
        }
    }

    let mut payload = payload;
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

pub async fn delete_post(
    claims: auth::Claims,
    _csrf: crate::security::csrf::CsrfGuard,
    State(pool): State<db::DbPool>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
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
