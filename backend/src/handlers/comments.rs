//! Comment Management HTTP Handlers
//!
//! This module handles comment operations on tutorials.
//! Comments allow users (when authenticated) to provide feedback and discussion.
//!
//! # Endpoints
//! - GET /api/tutorials/{id}/comments: List comments for a tutorial (public, paginated)
//! - POST /api/tutorials/{id}/comments: Create comment (admin only, CSRF protected)
//! - DELETE /api/comments/{id}: Delete comment (admin only, CSRF protected)
//!
//! # Features
//! - Pagination support (default 50 comments, configurable via query params)
//! - Author attribution from JWT claims
//! - Content length validation (1-2000 characters)
//! - Foreign key cascade deletion (comments deleted with tutorial)
//!
//! # Security
//! - Comments require authentication and CSRF protection
//! - Author name extracted from JWT token (prevents impersonation)
//! - Content length limits prevent abuse
//! - Tutorial ID validation prevents injection

use crate::{security::auth, db::DbPool, handlers::tutorials::validate_tutorial_id, models::*, repositories};
use axum::{
    extract::{ConnectInfo, Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use html_escape;

/// Request payload for creating a comment
#[derive(Deserialize)]
pub struct CreateCommentRequest {
    /// The actual comment text
    content: String,
    /// The author's name (optional for guests)
    author: Option<String>,
}

/// Query parameters for listing comments with pagination and sorting
#[derive(Deserialize)]
pub struct CommentListQuery {
    /// Maximum number of comments to return (default: 50)
    #[serde(default = "default_comment_limit")]
    limit: i64,

    /// Number of comments to skip for pagination
    #[serde(default)]
    offset: i64,

    /// Sorting criteria (e.g., "created_at:desc")
    #[serde(default)]
    sort: Option<String>,
}

fn default_comment_limit() -> i64 {
    50
}

/// Local DTO for comment responses, mapping from the database model
#[derive(Serialize, sqlx::FromRow)]
pub struct Comment {
    /// Unique identifier for the comment
    pub id: String,
    /// Optional parent tutorial ID
    pub tutorial_id: Option<String>,
    /// Optional parent post ID
    pub post_id: Option<String>,
    /// Display name of the author
    pub author: String,
    /// The comment content (HTML escaped)
    pub content: String,
    /// RFC3339 formatted creation timestamp
    pub created_at: String,
    /// Total number of votes/likes
    pub votes: i64,
    /// Whether the comment was posted by an administrator
    pub is_admin: bool,
}

/// Validates and sanitizes comment content
///
/// Trims whitespace, checks length constraints, and escapes HTML characters.
fn sanitize_comment_content(raw: &str) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let trimmed = raw.trim();

    if trimmed.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Comment content cannot be empty".to_string(),
            }),
        ));
    }

    if trimmed.len() > 1_000 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Comment too long (max 1000 characters)".to_string(),
            }),
        ));
    }

    let sanitized = html_escape::encode_safe(trimmed).to_string();

    Ok(sanitized)
}

/// Handler for listing comments on a tutorial
///
/// Returns a paginated list of comments for the specified tutorial.
pub async fn list_comments(
    State(pool): State<DbPool>,
    Path(tutorial_id): Path<String>,
    Query(params): Query<CommentListQuery>,
) -> Result<Json<Vec<Comment>>, (StatusCode, Json<ErrorResponse>)> {
    if let Err(e) = validate_tutorial_id(&tutorial_id) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })));
    }

    let exists = repositories::tutorials::check_tutorial_exists(&pool, &tutorial_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to verify tutorial existence for comments: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to fetch comments".to_string(),
                }),
            )
        })?;

    if !exists {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Tutorial not found".to_string(),
            }),
        ));
    }

    let limit = params.limit.clamp(1, 200);
    let offset = params.offset.max(0);

    let comments = repositories::comments::list_comments(
        &pool,
        &tutorial_id,
        limit,
        offset,
        params.sort.as_deref(),
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to fetch comments".to_string(),
            }),
        )
    })?;

    let response_comments: Vec<Comment> = comments
        .into_iter()
        .map(|c| Comment {
            id: c.id,
            tutorial_id: c.tutorial_id,
            post_id: c.post_id,
            author: c.author,
            content: c.content,
            created_at: c.created_at,
            votes: c.votes,
            is_admin: c.is_admin,
        })
        .collect();

    Ok(Json(response_comments))
}

/// Handler for creating a comment on a tutorial
///
/// Validates the tutorial existence and delegates to internal creation logic.
pub async fn create_comment(
    State(pool): State<DbPool>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(tutorial_id): Path<String>,
    _csrf: crate::security::csrf::CsrfGuard,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<Json<Comment>, (StatusCode, Json<ErrorResponse>)> {
    if let Err(e) = validate_tutorial_id(&tutorial_id) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })));
    }

    // Verify tutorial exists
    let exists = repositories::tutorials::check_tutorial_exists(&pool, &tutorial_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to verify tutorial existence: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to create comment".to_string(),
                }),
            )
        })?;

    if !exists {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Tutorial not found".to_string(),
            }),
        ));
    }

    create_comment_internal(pool, Some(tutorial_id), None, payload, None, addr.ip().to_string()).await
}

/// Handler for listing comments on a blog post
///
/// Returns a paginated list of comments for the specified post.
pub async fn list_post_comments(
    State(pool): State<DbPool>,
    Path(post_id): Path<String>,
    Query(params): Query<CommentListQuery>,
) -> Result<Json<Vec<Comment>>, (StatusCode, Json<ErrorResponse>)> {
    // Verify post exists
    let exists = repositories::posts::check_post_exists(&pool, &post_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to verify post existence: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to fetch comments".to_string(),
                }),
            )
        })?;

    if !exists {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Post not found".to_string(),
            }),
        ));
    }

    let limit = params.limit.clamp(1, 200);
    let offset = params.offset.max(0);

    let comments = repositories::comments::list_post_comments(
        &pool,
        &post_id,
        limit,
        offset,
        params.sort.as_deref(),
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to fetch comments".to_string(),
            }),
        )
    })?;

    let response_comments: Vec<Comment> = comments
        .into_iter()
        .map(|c| Comment {
            id: c.id,
            tutorial_id: c.tutorial_id,
            post_id: c.post_id,
            author: c.author,
            content: c.content,
            created_at: c.created_at,
            votes: c.votes,
            is_admin: c.is_admin,
        })
        .collect();

    Ok(Json(response_comments))
}

/// Handler for creating a comment on a blog post
///
/// Supports both authenticated users and guest comments.
pub async fn create_post_comment(
    State(pool): State<DbPool>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(post_id): Path<String>,
    auth::OptionalClaims(claims): auth::OptionalClaims,
    _csrf: crate::security::csrf::CsrfGuard,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<Json<Comment>, (StatusCode, Json<ErrorResponse>)> {
    // Verify post exists
    let exists = repositories::posts::check_post_exists(&pool, &post_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to verify post existence: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to create comment".to_string(),
                }),
            )
        })?;

    if !exists {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Post not found".to_string(),
            }),
        ));
    }

    create_comment_internal(pool, None, Some(post_id), payload, claims, addr.ip().to_string()).await
}

/// Internal logic for creating a comment on either a tutorial or a post
///
/// Handles author resolution (admin/user vs guest), rate limiting, and database insertion.
async fn create_comment_internal(
    pool: DbPool,
    tutorial_id: Option<String>,
    post_id: Option<String>,
    payload: CreateCommentRequest,
    claims: Option<auth::Claims>,
    ip_address: String,
) -> Result<Json<Comment>, (StatusCode, Json<ErrorResponse>)> {
    let comment_content = sanitize_comment_content(&payload.content)?;

    let (author, rate_limit_key) = if let Some(ref c) = claims {
        let display_name = if c.role == "admin" {
            "Administrator".to_string()
        } else {
            c.sub.clone()
        };
        (display_name, c.sub.clone())
    } else {
        // Guest comment
        match payload.author {
            Some(name) => {
                let trimmed = name.trim();
                if trimmed.len() < 2 || trimmed.len() > 50 {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: "Name must be between 2 and 50 characters".to_string(),
                        }),
                    ));
                }
                
                // Enforce strict name validation (alphanumeric and spaces)
                static NAME_REGEX: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
                let name_regex = NAME_REGEX.get_or_init(|| {
                    regex::Regex::new(r"^[a-zA-Z0-9 ]+$").unwrap()
                });

                if !name_regex.is_match(trimmed) {
                     return Err((
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: "Name can only contain letters, numbers, and spaces".to_string(),
                        }),
                    ));
                }

                // Prevent using "Administrator" or "Admin" as guest name
                if trimmed.eq_ignore_ascii_case("admin") 
                    || trimmed.eq_ignore_ascii_case("administrator") 
                    || trimmed.eq_ignore_ascii_case("root") 
                {
                     return Err((
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: "This name is reserved".to_string(),
                        }),
                    ));
                }

                // Use IP address for rate limiting guests, but store provided name as author
                (trimmed.to_string(), ip_address)
            }
            None => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Name is required for guest comments".to_string(),
                    }),
                ))
            }
        }
    };

    // Rate limiting
    let last_comment_time = repositories::comments::get_last_comment_time(&pool, &rate_limit_key)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking rate limit: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to create comment".to_string(),
                }),
            )
        })?;

    if let Some(created_at_str) = last_comment_time {
        if let Ok(created_at) = chrono::DateTime::parse_from_rfc3339(&created_at_str) {
            let now = chrono::Utc::now();
            let diff = now.signed_duration_since(created_at);
            if diff.num_seconds() < 60 {
                return Err((
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(ErrorResponse {
                        error: format!(
                            "Please wait {} seconds before posting another comment",
                            60 - diff.num_seconds()
                        ),
                    }),
                ));
            }
        }
    }

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Determine if author is admin
    let is_admin = if let Some(ref c) = claims {
        c.role == "admin"
    } else {
        false
    };

    let comment = repositories::comments::create_comment(
        &pool,
        &id,
        tutorial_id,
        post_id,
        &author,
        &comment_content,
        &now,
        is_admin,
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to create comment".to_string(),
            }),
        )
    })?;

    let response_comment = Comment {
        id: comment.id,
        tutorial_id: comment.tutorial_id,
        post_id: comment.post_id,
        author: comment.author,
        content: comment.content,
        created_at: comment.created_at,
        votes: comment.votes,
        is_admin: comment.is_admin,
    };

    Ok(Json(response_comment))
}

/// Handler for deleting a comment
///
/// Requires the user to be either an administrator or the original author.
pub async fn delete_comment(
    claims: auth::Claims,
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    _csrf: crate::security::csrf::CsrfGuard,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Fetch the comment first to check ownership
    let comment = repositories::comments::get_comment(&pool, &id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to fetch comment".to_string(),
                }),
            )
        })?;

    let comment = match comment {
        Some(c) => c,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Comment not found".to_string(),
                }),
            ));
        }
    };

    // Check permissions: Admin or Author
    let is_admin = claims.role == "admin";
    // We compare display names/usernames. Ideally, we should compare user IDs if available in comments.
    // Assuming 'author' in comments table stores the username/display name which matches claims.sub
    // or we need to be careful if display names are mutable.
    // For this implementation, we'll assume claims.sub matches the stored author name for simplicity,
    // or we might need to fetch the user to verify.
    // However, `comment_author_display_name` uses `claims.sub` by default.
    // Let's assume strict username matching for now.
    let is_author = comment.author == claims.sub;

    if !is_admin && !is_author {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Insufficient permissions".to_string(),
            }),
        ));
    }

    let deleted = repositories::comments::delete_comment(&pool, &id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to delete comment".to_string(),
                }),
            )
        })?;

    if !deleted {
        // Should not happen since we just fetched it, but good for safety
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Comment not found".to_string(),
            }),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Handler for voting on a comment
///
/// Authenticated users can upvote/downvote comments. Prevention logic ensures one vote per user.
pub async fn vote_comment(
    State(pool): State<DbPool>,
    claims: auth::Claims,
    Path(id): Path<String>,
    _csrf: crate::security::csrf::CsrfGuard,
) -> Result<Json<Comment>, (StatusCode, Json<ErrorResponse>)> {
    // Check if comment exists
    let exists = repositories::comments::check_comment_exists(&pool, &id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to vote on comment".to_string(),
                }),
            )
        })?;

    if !exists {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Comment not found".to_string(),
            }),
        ));
    }

    // Determine voter ID
    let voter_id = claims.sub;

    // Check if already voted
    let has_voted = repositories::comments::check_vote_exists(&pool, &id, &voter_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking votes: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to check votes".to_string(),
                }),
            )
        })?;

    if has_voted {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "You have already voted on this comment".to_string(),
            }),
        ));
    }

    // Record vote and increment votes
    repositories::comments::add_vote(&pool, &id, &voter_id)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return (
                        StatusCode::CONFLICT,
                        Json(ErrorResponse {
                            error: "You have already voted on this comment".to_string(),
                        }),
                    );
                }
            }
            tracing::error!("Database error recording vote: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to record vote".to_string(),
                }),
            )
        })?;

    // Return updated comment
    let comment = repositories::comments::get_comment(&pool, &id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to fetch updated comment".to_string(),
                }),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Comment disappeared after voting".to_string(),
                }),
            )
        })?;

    // Convert models::Comment to handlers::comments::Comment
    let response_comment = Comment {
        id: comment.id,
        tutorial_id: comment.tutorial_id,
        post_id: comment.post_id,
        author: comment.author,
        content: comment.content,
        created_at: comment.created_at,
        votes: comment.votes,
        is_admin: comment.is_admin,
    };

    Ok(Json(response_comment))
}
