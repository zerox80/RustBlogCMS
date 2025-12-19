use crate::db::DbPool;
use crate::models::Comment;
use sqlx;

/// Fetches a paginated list of comments for a specific tutorial, with optional sorting.
pub async fn list_comments(
    pool: &DbPool,
    tutorial_id: &str,
    limit: i64,
    offset: i64,
    sort: Option<&str>,
) -> Result<Vec<Comment>, sqlx::Error> {
    // Dynamic query building for different sort orders
    let mut query_builder = sqlx::QueryBuilder::new(
        "SELECT id, tutorial_id, post_id, author, content, created_at, votes, is_admin FROM comments WHERE tutorial_id = "
    );
    query_builder.push_bind(tutorial_id);

    match sort {
        Some("top") => {
            query_builder.push(" ORDER BY votes DESC, created_at DESC");
        }
        _ => {
            query_builder.push(" ORDER BY created_at DESC");
        }
    }

    query_builder.push(" LIMIT ");
    query_builder.push_bind(limit);
    query_builder.push(" OFFSET ");
    query_builder.push_bind(offset);

    query_builder
        .build_query_as::<Comment>()
        .fetch_all(pool)
        .await
}

pub async fn list_post_comments(
    pool: &DbPool,
    post_id: &str,
    limit: i64,
    offset: i64,
    sort: Option<&str>,
) -> Result<Vec<Comment>, sqlx::Error> {
    let mut query_builder = sqlx::QueryBuilder::new(
        "SELECT id, tutorial_id, post_id, author, content, created_at, votes, is_admin FROM comments WHERE post_id = "
    );
    query_builder.push_bind(post_id);

    match sort {
        Some("top") => {
            query_builder.push(" ORDER BY votes DESC, created_at DESC");
        }
        _ => {
            query_builder.push(" ORDER BY created_at DESC");
        }
    }

    query_builder.push(" LIMIT ");
    query_builder.push_bind(limit);
    query_builder.push(" OFFSET ");
    query_builder.push_bind(offset);

    query_builder
        .build_query_as::<Comment>()
        .fetch_all(pool)
        .await
}

pub async fn create_comment(
    pool: &DbPool,
    id: &str,
    tutorial_id: Option<String>,
    post_id: Option<String>,
    author: &str,
    content: &str,
    created_at: &str,
    is_admin: bool,
) -> Result<Comment, sqlx::Error> {
    sqlx::query(
        "INSERT INTO comments (id, tutorial_id, post_id, author, content, created_at, votes, is_admin) VALUES (?, ?, ?, ?, ?, ?, 0, ?)"
    )
    .bind(id)
    .bind(&tutorial_id)
    .bind(&post_id)
    .bind(author)
    .bind(content)
    .bind(created_at)
    .bind(is_admin)
    .execute(pool)
    .await?;

    Ok(Comment {
        id: id.to_string(),
        tutorial_id,
        post_id,
        author: author.to_string(),
        content: content.to_string(),
        created_at: created_at.to_string(),
        votes: 0,
        is_admin,
    })
}

pub async fn get_comment(pool: &DbPool, id: &str) -> Result<Option<Comment>, sqlx::Error> {
    sqlx::query_as::<_, Comment>("SELECT * FROM comments WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn delete_comment(pool: &DbPool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM comments WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn check_comment_exists(pool: &DbPool, id: &str) -> Result<bool, sqlx::Error> {
    let exists: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM comments WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(exists.is_some())
}

pub async fn check_vote_exists(
    pool: &DbPool,
    comment_id: &str,
    voter_id: &str,
) -> Result<bool, sqlx::Error> {
    let exists: Option<(i64,)> =
        sqlx::query_as("SELECT 1 FROM comment_votes WHERE comment_id = ? AND voter_id = ?")
            .bind(comment_id)
            .bind(voter_id)
            .fetch_optional(pool)
            .await?;
    Ok(exists.is_some())
}

/// Records a vote for a comment and increments the total vote count in a transaction.
pub async fn add_vote(pool: &DbPool, comment_id: &str, voter_id: &str) -> Result<(), sqlx::Error> {
    // Audit vote within a transaction to ensure consistency between vote count and records
    let mut tx = pool.begin().await?;

    // Step 1: Record unique voter ID to prevent multiple votes
    sqlx::query("INSERT INTO comment_votes (comment_id, voter_id) VALUES (?, ?)")
        .bind(comment_id)
        .bind(voter_id)
        .execute(&mut *tx)
        .await?;

    // Step 2: Increment cumulative counter on the comment record
    sqlx::query("UPDATE comments SET votes = votes + 1 WHERE id = ?")
        .bind(comment_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn get_last_comment_time(
    pool: &DbPool,
    author: &str,
) -> Result<Option<String>, sqlx::Error> {
    let last_comment: Option<(String,)> = sqlx::query_as(
        "SELECT created_at FROM comments WHERE author = ? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(author)
    .fetch_optional(pool)
    .await?;

    Ok(last_comment.map(|(t,)| t))
}
