use super::*;

/// Request payload for creating a comment
#[derive(Deserialize)]
pub struct CreateCommentRequest {
    /// The actual comment text
    pub(super) content: String,
    /// The author's name (optional for guests)
    pub(super) author: Option<String>,
}

/// Query parameters for listing comments with pagination and sorting
#[derive(Deserialize)]
pub struct CommentListQuery {
    /// Maximum number of comments to return (default: 50)
    #[serde(default = "default_comment_limit")]
    pub(super) limit: i64,

    /// Number of comments to skip for pagination
    #[serde(default)]
    pub(super) offset: i64,

    /// Sorting criteria (e.g., "created_at:desc")
    #[serde(default)]
    pub(super) sort: Option<String>,
}

pub(super) fn default_comment_limit() -> i64 {
    50
}

/// Local DTO for comment responses, mapping from the database model
#[derive(Serialize, sqlx::FromRow)]
pub struct CommentResponse {
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
impl From<crate::models::Comment> for CommentResponse {
    fn from(c: crate::models::Comment) -> Self {
        CommentResponse {
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
pub(super) fn sanitize_comment_content(raw: &str) -> Result<String, ApiError> {
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
