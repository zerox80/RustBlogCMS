use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Represents a user-submitted comment on content.
/// 
/// Comments can be attached to either a `Tutorial` or a `SitePost`.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Comment {
    /// Unique UUID (v4) for the comment.
    pub id: String,
    /// ID of the tutorial this comment belongs to (if any).
    pub tutorial_id: Option<String>,
    /// ID of the blog post this comment belongs to (if any).
    pub post_id: Option<String>,
    /// Display name of the author.
    pub author: String,
    /// The comment body, supports Markdown syntax.
    pub content: String,
    /// ISO 8601 timestamp of creation.
    pub created_at: String,
    /// Net karma score (upvotes minus downvotes).
    pub votes: i64,
    /// Whether the comment author is an administrator.
    pub is_admin: bool,
}
