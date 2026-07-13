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
    sqlx::query(concat!(
        "INSERT INTO comments ",
        "(id, tutorial_id, post_id, author, content, created_at, votes, is_admin, ",
        "author_username, is_guest) ",
        "VALUES (?, 'tutorial-1', NULL, ?, 'content', datetime('now'), 0, ?, ?, ?)"
    ))
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
