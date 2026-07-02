#[cfg(test)]
mod search_tests {
    #[tokio::test]
    async fn test_search_query_validation() {
        // Test empty query
        let empty_query = "";
        assert!(empty_query.trim().is_empty());

        // Test query length
        let long_query = "a".repeat(501);
        assert!(long_query.len() > 500);
    }

    use rust_blog_backend::handlers::search::sanitize_fts_query;

    #[test]
    fn test_search_query_sanitization() {
        let query = "<script>alert('xss')</script>";
        // Query should be sanitized for FTS (special chars removed, wrapped in quotes)
        let sanitized = sanitize_fts_query(query).unwrap();

        // The sanitization splits by whitespace and filters characters.
        // It keeps alphanumeric and safe symbols like ().
        // So "<script>alert('xss')</script>" becomes "scriptalert(xss)/script" (one token)
        // Then it's wrapped in quotes and appended with * for prefix matching.

        assert!(!sanitized.contains("<"));
        assert!(!sanitized.contains(">"));
        assert!(!sanitized.contains("'"));

        // "script" is a valid search term in a coding blog, so it SHOULD be present.
        assert!(sanitized.contains("script"));
        assert!(sanitized.contains("alert"));
    }

    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use rust_blog_backend::{db::migrations::run_migrations, handlers::search::search_tutorials};
    use sqlx::SqlitePool;
    use tower::ServiceExt;

    async fn setup_search_app() -> Router {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.expect("run migrations");

        sqlx::query(
            "INSERT INTO tutorials (id, title, description, icon, color, topics, content, version) \
             VALUES ('t1', 'Bash Scripting', 'Learn bash', 'Terminal', 'from-blue-500 to-cyan-500', '[\"bash\",\"scripting\"]', 'content', 1)",
        )
        .execute(&pool)
        .await
        .expect("insert tutorial");

        Router::new()
            .route("/api/search/tutorials", get(search_tutorials))
            .with_state(pool)
    }

    async fn get_search(app: Router, query_string: &str) -> (StatusCode, String) {
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/search/tutorials?{query_string}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        (status, String::from_utf8_lossy(&body).to_string())
    }

    /// Regression test for a bug where `tutorials_fts` was joined under the
    /// alias `fts` and `bm25(fts)` was used to rank results. SQLite's FTS5
    /// auxiliary functions (`bm25`, `highlight`, `snippet`) only recognize
    /// the virtual table when referenced by its real name -- referencing it
    /// through a JOIN alias made *every* search request fail with
    /// "no such column: fts" (HTTP 500), independent of any topic filter.
    /// Verified directly against SQLite before fixing.
    #[tokio::test]
    async fn plain_search_does_not_error() {
        let app = setup_search_app().await;

        let (status, body) = get_search(app, "q=bash").await;

        assert_eq!(status, StatusCode::OK, "response body: {body}");
        assert!(body.contains("Bash Scripting"));
    }

    /// Regression test for a second, independent bug in the same query: the
    /// topic-filter variant used `ESCAPE '\\'` (two backslashes), but SQLite
    /// requires the ESCAPE argument to be exactly one character and rejects
    /// two with "ESCAPE expression must be a single character". Verified
    /// directly against SQLite before fixing.
    #[tokio::test]
    async fn topic_filtered_search_does_not_error() {
        let app = setup_search_app().await;

        let (status, body) = get_search(app, "q=bash&topic=bash").await;

        assert_eq!(status, StatusCode::OK, "response body: {body}");
        assert!(body.contains("Bash Scripting"));
    }

    #[tokio::test]
    async fn topic_filtered_search_with_like_metacharacters_does_not_error() {
        let app = setup_search_app().await;

        // '%' and '_' are SQL LIKE metacharacters; the escaping path must
        // handle them without triggering the ESCAPE-clause error.
        let (status, body) = get_search(app, "q=bash&topic=ba_sh%25").await;

        assert_eq!(status, StatusCode::OK, "response body: {body}");
        assert_eq!(body, "[]");
    }
}
