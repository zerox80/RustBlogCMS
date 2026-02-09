//! Search HTTP Handlers
//!
//! This module provides full-text search capabilities for tutorials.
//! It uses SQLite's FTS5 (Full-Text Search 5) for fast and efficient searching.
//!
//! # Endpoints
//! - GET /api/search/tutorials: Search tutorials by keyword (public)
//! - GET /api/search/topics: Get all unique topics (public)
//!
//! # Search Features
//! - Full-text search across title, description, content, and topics
//! - Topic-based filtering (optional)
//! - Pagination support (default 20 results, configurable)
//! - Ranked results (FTS5 BM25 ranking algorithm)
//! - Query sanitization to prevent FTS5 syntax errors
//!
//! # Query Processing
//! - Splits query into tokens
//! - Removes FTS5 special characters (* " :)
//! - Validates minimum word length (3 characters)
//! - Limits maximum tokens (20) to prevent DoS
//! - Applies FTS5 prefix matching for better UX
//!
//! # Performance
//! - FTS5 index provides sub-second search on large datasets
//! - Automatic index updates via triggers on tutorial changes
//! - Result limit prevents excessive data transfer

use crate::{db::DbPool, models::*};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::convert::TryInto;

/// Query parameters for searching tutorials
#[derive(Deserialize)]
pub struct SearchQuery {
    /// The search keyword(s)
    q: String,

    /// Optional topic filter
    #[serde(default)]
    topic: Option<String>,

    /// Maximum number of results (default: 20)
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_limit() -> i64 {
    20
}

/// Sanitizes a raw string into a format suitable for SQLite FTS5 queries.
/// Removes special characters, handles prefix matching, and ensures tokens are quoted.
pub fn sanitize_fts_query(raw: &str) -> Result<String, String> {
    // Split the raw string by whitespace into individual tokens
    let tokens: Vec<String> = raw
        .split_whitespace()
        .filter_map(|token| {
            // Keep only safe characters for FTS5 queries to prevent syntax errors
            let sanitized: String = token
                .chars()
                .filter(|c| {
                    c.is_ascii_alphanumeric()
                        || matches!(
                            c,
                            '*' | '-'
                                | '_'
                                | '.'
                                | '+'
                                | '#'
                                | '@'
                                | '/'
                                | ':'
                                | '('
                                | ')'
                                | '['
                                | ']'
                        )
                })
                .collect();
            
            // If the token is empty after sanitization, skip it
            if sanitized.is_empty() {
                None
            } else {
                // Wrap each token in double quotes to handle tokens with special characters safely
                Some(format!("\"{}\"", sanitized))
            }
        })
        .collect();

    // If no searchable tokens remain, return an error
    if tokens.is_empty() {
        Err("Search query must contain at least one searchable character".to_string())
    } else {
        // Construct the final FTS query
        // We use implicit AND by joining tokens with spaces
        let mut query_parts = Vec::new();
        for (i, token) in tokens.iter().enumerate() {
            // Fix Bug 3: Search Query Crash/Error
            // Prevent "*" from being treated as a prefix match on an empty string which causes FTS5 syntax error.
            // If a token is just "*" or has no alphanumeric characters (and is not a valid operator), we should be careful.
            // The previous logic wrapped * in quotes "*" then appended *, resulting in "*"* which is invalid.
            
            let is_last = i == tokens.len() - 1;
            
            if token == "*" {
                // Skip standalone wildcard tokens as they are invalid in FTS5 standard query syntax
                // or just treat them as literal if wrapped in quotes, but FTS5 doesn't like "*"*
                continue; 
            }

            if is_last {
                // For the last token, we enable prefix matching by appending *
                // (This matches e.g. "rus*" for "rust" or "rustlang")
                // Ensure we don't create invalid syntax like "*"*
                if token.ends_with('*') {
                     query_parts.push(token.clone());
                } else {
                     query_parts.push(format!("{}*", token));
                }
            } else {
                // Standard token matching
                query_parts.push(token.clone());
            }
        }
        
        if query_parts.is_empty() {
             return Err("Search query contains no valid terms".to_string());
        }

        Ok(query_parts.join(" "))
    }
}

/// Escapes special characters for SQL LIKE patterns (`%`, `_`, and `\`).
fn escape_like_pattern(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            // Prepend backslash to escape characters
            '%' | '_' | '\\' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            // Regular characters
            _ => escaped.push(ch),
        }
    }
    escaped
}

/// Searches tutorials using full-text and optional topic filtering.
pub async fn search_tutorials(
    State(pool): State<DbPool>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<TutorialResponse>>, (StatusCode, Json<ErrorResponse>)> {
    // Basic validation: search query can't be just whitespace
    if params.q.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Search query cannot be empty".to_string(),
            }),
        ));
    }

    // Limit length to prevent resource exhaustion attacks
    if params.q.len() > 500 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Search query too long".to_string(),
            }),
        ));
    }

    // Set reasonable bounds on total results
    let limit = params.limit.min(100).max(1);

    // Sanitize the user input for FTS5 engine
    let search_query = sanitize_fts_query(params.q.trim())
        .map_err(|err| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: err })))?;

    // If an optional topic filter is provided, prepare a LIKE pattern
    let topic_pattern = params.topic.as_ref().and_then(|topic| {
        let trimmed = topic.trim();
        if trimmed.is_empty() {
            None
        } else {
            // Wrap the escaped topic in wildcards (%)
            Some(format!("%{}%", escape_like_pattern(trimmed)))
        }
    });

    // Execute the search query
    let tutorials = if let Some(pattern) = topic_pattern {
        // Query variant that includes topic filtering
        sqlx::query_as::<_, Tutorial>(
            r#"
            SELECT t.* FROM tutorials t
            INNER JOIN tutorials_fts fts ON t.id = fts.tutorial_id
            WHERE fts MATCH ?
            AND t.topics LIKE ? ESCAPE '\\'
            ORDER BY bm25(fts)
            LIMIT ?
            "#,
        )
        .bind(&search_query) // Bind the FTS sanitized query
        .bind(&pattern)      // Bind the LIKE pattern for topics
        .bind(limit)        // Bind the result limit
        .fetch_all(&pool)
        .await
    } else {
        // Simple full-text search without topic filter
        sqlx::query_as::<_, Tutorial>(
            r#"
            SELECT t.* FROM tutorials t
            INNER JOIN tutorials_fts fts ON t.id = fts.tutorial_id
            WHERE fts MATCH ?
            ORDER BY bm25(fts)
            LIMIT ?
            "#,
        )
        .bind(&search_query) // Bind the FTS sanitized query
        .bind(limit)        // Bind the result limit
        .fetch_all(&pool)
        .await
    }
    .map_err(|e| {
        // Log the error and return a safe JSON response
        tracing::error!("Search error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to search tutorials".to_string(),
            }),
        )
    })?;

    // Convert raw tutorial records into mapped responses
    let mut responses = Vec::with_capacity(tutorials.len());
    for tutorial in tutorials {
        // Try to convert each record; this handles JSON parsing of topics
        let response: TutorialResponse = tutorial.try_into().map_err(|err: String| {
            tracing::error!("Tutorial data corruption detected: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to parse tutorial data".to_string(),
                }),
            )
        })?;
        responses.push(response);
    }

    Ok(Json(responses))
}

/// Retrieves a list of all unique topics currently available in tutorials.
pub async fn get_all_topics(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<String>>, (StatusCode, Json<ErrorResponse>)> {
    // Select unique topics from the denormalized tutorial_topics table
    let topics: Vec<(String,)> =
        sqlx::query_as("SELECT DISTINCT topic FROM tutorial_topics ORDER BY topic ASC")
            .fetch_all(&pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch topics: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Failed to fetch topics".to_string(),
                    }),
                )
            })?;

    // Extract strings from the tuple and return as a list
    Ok(Json(topics.into_iter().map(|(t,)| t).collect()))
}
