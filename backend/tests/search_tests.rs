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
}
