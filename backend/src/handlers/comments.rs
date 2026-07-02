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
    /// The comment content as plain text
    pub content: String,
    /// RFC3339 formatted creation timestamp
    pub created_at: String,
    /// Total number of votes/likes
    pub votes: i64,
    /// Whether the comment was posted by an administrator
    pub is_admin: bool,
    /// Real authenticated username of the commenter, used for server-side
    /// ownership checks only. NEVER sent to clients: for admin comments,
    /// `author` is deliberately the anonymized literal "Administrator"
    /// string, and leaking this field would de-anonymize which real admin
    /// account posted it.
    #[serde(skip_serializing)]
    pub author_username: Option<String>,
    /// Guest/authenticated marker, used for server-side ownership checks
    /// only. Never sent to clients, for the same reason as `author_username`.
    #[serde(skip_serializing)]
    pub is_guest: Option<bool>,
}

/// Converts the repository's `Comment` model into this handler's response
/// DTO. Kept as an explicit `From` impl (rather than returning the model
/// type directly) because the two types intentionally diverge on
/// serialization: this DTO marks `author_username`/`is_guest` as
/// `#[serde(skip_serializing)]` so they never reach the client, while the
/// model type serializes them (it's also used for internal deserialization).
impl From<crate::models::Comment> for Comment {
    fn from(c: crate::models::Comment) -> Self {
        Comment {
            id: c.id,
            tutorial_id: c.tutorial_id,
            post_id: c.post_id,
            author: c.author,
            content: c.content,
            created_at: c.created_at,
            votes: c.votes,
            is_admin: c.is_admin,
            author_username: c.author_username,
            is_guest: c.is_guest,
        }
    }
}

/// Validates and sanitizes comment content
///
/// Trims whitespace and checks length constraints.
fn sanitize_comment_content(raw: &str) -> Result<String, ApiError> {
    let trimmed = raw.trim();

    if trimmed.is_empty() {
        return Err(bad_request("Comment content cannot be empty"));
    }

    if trimmed.len() > 1_000 {
        return Err(bad_request("Comment too long (max 1000 characters)"));
    }

    // Content is stored as raw text; escaping happens at render time in the
    // frontend (React). Escaping here as well would double-encode and hurt
    // searchability and future flexibility (e.g. markdown support).
    Ok(trimmed.to_string())
}

/// Handler for listing comments on a tutorial
///
/// Returns a paginated list of comments for the specified tutorial.
pub async fn list_comments(
    State(pool): State<DbPool>,
    Path(tutorial_id): Path<String>,
    Query(params): Query<CommentListQuery>,
) -> Result<Json<Vec<Comment>>, ApiError> {
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

    let response_comments: Vec<Comment> = comments.into_iter().map(Comment::from).collect();

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
) -> Result<Json<Comment>, ApiError> {
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
) -> Result<Json<Vec<Comment>>, ApiError> {
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

    let response_comments: Vec<Comment> = comments.into_iter().map(Comment::from).collect();

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
) -> Result<Json<Comment>, ApiError> {
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
) -> Result<Json<Comment>, ApiError> {
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

    Ok(Json(Comment::from(comment)))
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
) -> Result<Json<Comment>, ApiError> {
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

    Ok(Json(Comment::from(comment)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    async fn setup_comments_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("create in-memory sqlite pool");

        sqlx::query(
            r#"
            CREATE TABLE comments (
                id TEXT PRIMARY KEY,
                tutorial_id TEXT,
                post_id TEXT,
                author TEXT NOT NULL,
                rate_limit_key TEXT NOT NULL DEFAULT '',
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                votes INTEGER NOT NULL DEFAULT 0,
                is_admin BOOLEAN NOT NULL DEFAULT FALSE,
                author_username TEXT DEFAULT NULL,
                is_guest BOOLEAN DEFAULT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create comments table");

        pool
    }

    /// Inserts a comment row directly with full control over every column,
    /// for exercising `delete_comment`'s ownership logic against specific
    /// (author, author_username, is_guest, is_admin) combinations.
    #[allow(clippy::too_many_arguments)]
    async fn insert_comment_row(
        pool: &SqlitePool,
        id: &str,
        author: &str,
        author_username: Option<&str>,
        is_guest: Option<bool>,
        is_admin: bool,
    ) {
        sqlx::query(
            "INSERT INTO comments (id, tutorial_id, post_id, author, content, created_at, votes, is_admin, author_username, is_guest) \
             VALUES (?, 'tutorial-1', NULL, ?, 'content', datetime('now'), 0, ?, ?, ?)",
        )
        .bind(id)
        .bind(author)
        .bind(is_admin)
        .bind(author_username)
        .bind(is_guest)
        .execute(pool)
        .await
        .expect("insert comment row");
    }

    #[tokio::test]
    async fn admin_tutorial_comment_uses_claims_without_author_payload() {
        let pool = setup_comments_pool().await;
        let claims = auth::Claims {
            sub: "admin".to_string(),
            role: "admin".to_string(),
            exp: usize::MAX,
        };

        let result = create_comment_internal(
            pool,
            Some("tutorial-1".to_string()),
            None,
            CreateCommentRequest {
                content: "Admin note".to_string(),
                author: None,
            },
            Some(claims),
            "127.0.0.1".to_string(),
        )
        .await;
        let Json(comment) = match result {
            Ok(comment) => comment,
            Err((status, _)) => panic!("admin comment failed with status {status}"),
        };

        assert_eq!(comment.author, "Administrator");
        assert!(comment.is_admin);
        // Admins get their real identity recorded too (harmless -- admin
        // deletion is governed by the is_admin/role check, not this field).
        assert_eq!(comment.author_username, Some("admin".to_string()));
        assert_eq!(comment.is_guest, Some(false));
    }

    #[tokio::test]
    async fn guest_rate_limit_uses_ip_even_when_author_changes() {
        let pool = setup_comments_pool().await;

        let first_result = create_comment_internal(
            pool.clone(),
            None,
            Some("post-1".to_string()),
            CreateCommentRequest {
                content: "First comment".to_string(),
                author: Some("Alice".to_string()),
            },
            None,
            "203.0.113.5".to_string(),
        )
        .await;
        let Json(first_comment) = match first_result {
            Ok(comment) => comment,
            Err((status, _)) => panic!("first guest comment failed with status {status}"),
        };
        assert_eq!(first_comment.author_username, None);
        assert_eq!(first_comment.is_guest, Some(true));

        let result = create_comment_internal(
            pool,
            None,
            Some("post-1".to_string()),
            CreateCommentRequest {
                content: "Second comment".to_string(),
                author: Some("Bob".to_string()),
            },
            None,
            "203.0.113.5".to_string(),
        )
        .await;
        let err = match result {
            Ok(_) => panic!("same client IP should still be rate limited"),
            Err(err) => err,
        };

        assert_eq!(err.0, StatusCode::TOO_MANY_REQUESTS);
    }

    fn claims_for(sub: &str, role: &str) -> auth::Claims {
        auth::Claims {
            sub: sub.to_string(),
            role: role.to_string(),
            exp: usize::MAX,
        }
    }

    async fn call_delete_comment(
        pool: SqlitePool,
        id: &str,
        claims: auth::Claims,
    ) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
        delete_comment(
            claims,
            State(pool),
            Path(id.to_string()),
            crate::security::csrf::CsrfGuard,
        )
        .await
    }

    #[tokio::test]
    async fn authenticated_user_can_delete_own_post_migration_comment() {
        let pool = setup_comments_pool().await;
        insert_comment_row(&pool, "c1", "bob", Some("bob"), Some(false), false).await;

        let result = call_delete_comment(pool, "c1", claims_for("bob", "user")).await;

        assert_eq!(result.unwrap(), StatusCode::NO_CONTENT);
    }

    /// Critical regression test: a guest can type any display name,
    /// including a real, registered user's username. Before the
    /// author_username/is_guest fix, `comment.author == claims.sub` would
    /// let that real user delete the guest's comment. This must now be
    /// rejected because the guest comment has no real identity attached.
    #[tokio::test]
    async fn authenticated_user_cannot_delete_guest_comment_with_spoofed_name() {
        let pool = setup_comments_pool().await;
        insert_comment_row(&pool, "c2", "bob", None, Some(true), false).await;

        let result = call_delete_comment(pool, "c2", claims_for("bob", "user")).await;

        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn admin_can_delete_any_comment_regardless_of_authorship() {
        let pool = setup_comments_pool().await;
        insert_comment_row(
            &pool,
            "c3",
            "Administrator",
            Some("realadminuser"),
            Some(false),
            true,
        )
        .await;

        let result = call_delete_comment(pool, "c3", claims_for("different-admin", "admin")).await;

        assert_eq!(result.unwrap(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn non_admin_cannot_delete_admin_authored_comment_even_with_same_username() {
        let pool = setup_comments_pool().await;
        insert_comment_row(
            &pool,
            "c7",
            "Administrator",
            Some("alice"),
            Some(false),
            true,
        )
        .await;

        let result = call_delete_comment(pool, "c7", claims_for("alice", "user")).await;

        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn legacy_row_real_owner_can_still_self_delete() {
        let pool = setup_comments_pool().await;
        // Simulates a genuinely pre-migration row: both new columns are NULL.
        insert_comment_row(&pool, "c4", "carol", None, None, false).await;

        let result = call_delete_comment(pool, "c4", claims_for("carol", "user")).await;

        assert_eq!(result.unwrap(), StatusCode::NO_CONTENT);
    }

    /// Documents the accepted, time-bounded residual risk: a pre-migration
    /// row (author_username and is_guest both NULL) that was actually
    /// posted by a guest who happened to type "carol" as their display name
    /// can still be deleted by the real user "carol" via the legacy
    /// fallback. This is intentional -- the alternative (blocking legacy
    /// self-service deletion entirely) was rejected as a worse regression --
    /// and applies only to rows that existed before this fix shipped.
    #[tokio::test]
    async fn legacy_row_ambiguous_origin_is_a_known_accepted_gap() {
        let pool = setup_comments_pool().await;
        insert_comment_row(&pool, "c5", "carol", None, None, false).await;

        let result = call_delete_comment(pool, "c5", claims_for("carol", "user")).await;

        assert_eq!(result.unwrap(), StatusCode::NO_CONTENT);
    }

    /// Guards the fail-closed arm added for the (author_username=None,
    /// is_guest=Some(false)) state: no current insert path produces it (an
    /// authenticated comment always sets author_username), but nothing in
    /// the schema forbids it either. If a future bug or manual data edit
    /// ever produces this shape, ownership must be rejected rather than
    /// silently falling back to the spoofable `author == claims.sub` match.
    #[tokio::test]
    async fn inconsistent_row_with_no_username_but_marked_authenticated_is_rejected() {
        let pool = setup_comments_pool().await;
        insert_comment_row(&pool, "c6", "carol", None, Some(false), false).await;

        let result = call_delete_comment(pool, "c6", claims_for("carol", "user")).await;

        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::FORBIDDEN);
    }
}
