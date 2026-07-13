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
//! - Content length validation (1-1000 characters)
//! - Foreign key cascade deletion (comments deleted with tutorial)
//!
//! # Security
//! - Comments require authentication and CSRF protection
//! - Author name extracted from JWT token (prevents impersonation)
//! - Content length limits prevent abuse
//! - Tutorial ID validation prevents injection

use crate::{
    db::DbPool, handlers::tutorials::validate_tutorial_id,
    middleware::security as security_middleware, models::*, repositories, security::auth,
};
use axum::{
    extract::{ConnectInfo, Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

mod comment_models;
use comment_models::sanitize_comment_content;
use comment_models::{CommentListQuery, CommentResponse, CreateCommentRequest};

/// Handler for listing comments on a tutorial
///
/// Returns a paginated list of comments for the specified tutorial.
pub async fn list_comments(
    State(pool): State<DbPool>,
    Path(tutorial_id): Path<String>,
    Query(params): Query<CommentListQuery>,
) -> Result<Json<Vec<CommentResponse>>, ApiError> {
    validate_tutorial_id(&tutorial_id).map_err(bad_request)?;

    let exists = repositories::tutorials::check_tutorial_exists(&pool, &tutorial_id)
        .await
        .map_err(internal_error("Failed to fetch comments"))?;

    if !exists {
        return Err(not_found("Tutorial not found"));
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
    .map_err(internal_error("Failed to fetch comments"))?;

    let response_comments: Vec<CommentResponse> =
        comments.into_iter().map(CommentResponse::from).collect();

    Ok(Json(response_comments))
}

/// Handler for creating a comment on a tutorial
///
/// Validates the tutorial existence and delegates to internal creation logic.
pub async fn create_comment(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(tutorial_id): Path<String>,
    claims: auth::Claims,
    _csrf: crate::security::csrf::CsrfGuard,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<Json<CommentResponse>, ApiError> {
    validate_tutorial_id(&tutorial_id).map_err(bad_request)?;

    // Verify tutorial exists
    let exists = repositories::tutorials::check_tutorial_exists(&pool, &tutorial_id)
        .await
        .map_err(internal_error("Failed to create comment"))?;

    if !exists {
        return Err(not_found("Tutorial not found"));
    }

    let client_ip = security_middleware::extract_client_ip(&headers, addr.ip());

    create_comment_internal(
        pool,
        Some(tutorial_id),
        None,
        payload,
        Some(claims),
        client_ip.to_string(),
    )
    .await
}

/// Handler for listing comments on a blog post
///
/// Returns a paginated list of comments for the specified post.
pub async fn list_post_comments(
    State(pool): State<DbPool>,
    Path(post_id): Path<String>,
    Query(params): Query<CommentListQuery>,
) -> Result<Json<Vec<CommentResponse>>, ApiError> {
    // Verify post exists
    let exists = repositories::posts::check_post_exists(&pool, &post_id)
        .await
        .map_err(internal_error("Failed to fetch comments"))?;

    if !exists {
        return Err(not_found("Post not found"));
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
    .map_err(internal_error("Failed to fetch comments"))?;

    let response_comments: Vec<CommentResponse> =
        comments.into_iter().map(CommentResponse::from).collect();

    Ok(Json(response_comments))
}

/// Handler for creating a comment on a blog post
///
/// Supports both authenticated users and guest comments.
pub async fn create_post_comment(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(post_id): Path<String>,
    auth::OptionalClaims(claims): auth::OptionalClaims,
    _csrf: crate::security::csrf::CsrfGuard,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<Json<CommentResponse>, ApiError> {
    // Verify post exists
    let exists = repositories::posts::check_post_exists(&pool, &post_id)
        .await
        .map_err(internal_error("Failed to create comment"))?;

    if !exists {
        return Err(not_found("Post not found"));
    }

    let client_ip = security_middleware::extract_client_ip(&headers, addr.ip());

    create_comment_internal(
        pool,
        None,
        Some(post_id),
        payload,
        claims,
        client_ip.to_string(),
    )
    .await
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
) -> Result<Json<CommentResponse>, ApiError> {
    let comment_content = sanitize_comment_content(&payload.content)?;

    let (author, rate_limit_key, author_username, is_guest) = if let Some(ref c) = claims {
        let display_name = if c.role == "admin" {
            "Administrator".to_string()
        } else {
            c.sub.clone()
        };
        // Admin posts are stored with the display author, so rate limiting must use the same key.
        // author_username/is_guest record the real identity for later ownership
        // checks in delete_comment. Populated for admins too (harmless -- admin
        // deletion is governed by the separate is_admin/role check, not these
        // fields).
        (
            display_name.clone(),
            display_name,
            Some(c.sub.clone()),
            Some(false),
        )
    } else {
        // Guest comment
        match payload.author {
            Some(name) => {
                let trimmed = name.trim();
                if trimmed.len() < 2 || trimmed.len() > 50 {
                    return Err(bad_request("Name must be between 2 and 50 characters"));
                }

                // Enforce strict name validation (alphanumeric and spaces)
                static NAME_REGEX: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
                let name_regex =
                    NAME_REGEX.get_or_init(|| regex::Regex::new(r"^[a-zA-Z0-9 ]+$").unwrap());

                if !name_regex.is_match(trimmed) {
                    return Err(bad_request(
                        "Name can only contain letters, numbers, and spaces",
                    ));
                }

                // Prevent using "Administrator" or "Admin" as guest name
                if trimmed.eq_ignore_ascii_case("admin")
                    || trimmed.eq_ignore_ascii_case("administrator")
                    || trimmed.eq_ignore_ascii_case("root")
                {
                    return Err(bad_request("This name is reserved"));
                }

                // Use the IP address as the guest rate-limit key to prevent name-change bypasses.
                // A guest never has a real identity to record.
                (trimmed.to_string(), ip_address, None, Some(true))
            }
            None => return Err(bad_request("Name is required for guest comments")),
        }
    };

    // Rate limiting
    let last_comment_time = repositories::comments::get_last_comment_time(&pool, &rate_limit_key)
        .await
        .map_err(internal_error("Failed to create comment"))?;

    if let Some(created_at_str) = last_comment_time {
        if let Ok(created_at) = chrono::DateTime::parse_from_rfc3339(&created_at_str) {
            let now = chrono::Utc::now();
            let diff = now.signed_duration_since(created_at);
            if diff.num_seconds() < 60 {
                return Err(api_error(
                    StatusCode::TOO_MANY_REQUESTS,
                    format!(
                        "Please wait {} seconds before posting another comment",
                        60 - diff.num_seconds()
                    ),
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
        &rate_limit_key,
        &comment_content,
        &now,
        is_admin,
        author_username,
        is_guest,
    )
    .await
    .map_err(internal_error("Failed to create comment"))?;

    Ok(Json(CommentResponse::from(comment)))
}

/// Handler for deleting a comment
///
/// Requires the user to be either an administrator or the original author.
pub async fn delete_comment(
    claims: auth::Claims,
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    _csrf: crate::security::csrf::CsrfGuard,
) -> Result<StatusCode, ApiError> {
    // Fetch the comment first to check ownership
    let comment = repositories::comments::get_comment(&pool, &id)
        .await
        .map_err(internal_error("Failed to fetch comment"))?
        .ok_or_else(|| not_found("Comment not found"))?;

    // Check permissions: Admin or Author.
    //
    // Ownership is determined by the real authenticated identity
    // (author_username), NOT the spoofable free-text `author` display name
    // (guests can type any name, including another real user's username).
    //
    // Four states, disambiguated by (author_username, is_guest). Every arm
    // is written out explicitly (no wildcard `_`) so that a row shape no
    // current insert path produces can never silently fall through to the
    // spoofable legacy comparison -- it must be reachable only for rows that
    // are *provably* pre-migration (both columns NULL).
    //   - author_username = Some(u): post-migration authenticated comment.
    //     Compare real identity directly.
    //   - author_username = None, is_guest = Some(true): post-migration guest
    //     comment. A guest never has a real identity to match -- never allow
    //     the display-name fallback, or the impersonation hole stays open
    //     forever for new guest comments.
    //   - author_username = None, is_guest = None: pre-migration legacy row
    //     of unknown origin (could be guest or authenticated). Fall back to
    //     the legacy display-name match to avoid regressing self-service
    //     deletion for real users' historical comments. This is an accepted,
    //     time-bounded residual risk limited to rows that already existed
    //     when this fix shipped; it cannot apply to anything created after.
    //   - author_username = None, is_guest = Some(false): inconsistent state
    //     that no current code path produces (an authenticated comment
    //     should always carry author_username). Reject rather than fall
    //     back to the spoofable comparison, so a future bug or manual data
    //     edit that produces this shape fails closed instead of silently
    //     reopening the impersonation hole this migration closes.
    // Admin-authored comments are excluded from all of the above; they're
    // already covered by the is_admin role check.
    let is_admin = claims.role == "admin";
    let is_author = match (&comment.author_username, comment.is_guest) {
        (Some(username), _) => !comment.is_admin && *username == claims.sub,
        (None, Some(true)) => false,
        (None, None) => !comment.is_admin && comment.author == claims.sub,
        (None, Some(false)) => false,
    };

    if !is_admin && !is_author {
        return Err(forbidden("Insufficient permissions"));
    }

    let deleted = repositories::comments::delete_comment(&pool, &id)
        .await
        .map_err(internal_error("Failed to delete comment"))?;

    if !deleted {
        // Should not happen since we just fetched it, but good for safety
        return Err(not_found("Comment not found"));
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
) -> Result<Json<CommentResponse>, ApiError> {
    // Check if comment exists
    let exists = repositories::comments::check_comment_exists(&pool, &id)
        .await
        .map_err(internal_error("Failed to vote on comment"))?;

    if !exists {
        return Err(not_found("Comment not found"));
    }

    // Determine voter ID
    let voter_id = claims.sub;

    // Check if already voted
    let has_voted = repositories::comments::check_vote_exists(&pool, &id, &voter_id)
        .await
        .map_err(internal_error("Failed to check votes"))?;

    if has_voted {
        return Err(api_error(
            StatusCode::CONFLICT,
            "You have already voted on this comment",
        ));
    }

    // Record vote and increment votes
    repositories::comments::add_vote(&pool, &id, &voter_id)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return api_error(
                        StatusCode::CONFLICT,
                        "You have already voted on this comment",
                    );
                }
            }
            internal_error("Failed to record vote")(e)
        })?;

    // Return updated comment
    let comment = repositories::comments::get_comment(&pool, &id)
        .await
        .map_err(internal_error("Failed to fetch updated comment"))?
        .ok_or_else(|| internal_error_plain("Comment disappeared after voting"))?;

    Ok(Json(CommentResponse::from(comment)))
}

#[cfg(test)]
mod tests;
