//! Tutorial Management HTTP Handlers
//!
//! This module contains HTTP handlers for CRUD operations on tutorials.
//! Tutorials are the main content type of the application, representing
//! Linux learning modules with topics, icons, and markdown content.
//!
//! # Endpoints
//! - GET /api/tutorials: List all tutorials
//! - GET /api/tutorials/{id}: Get specific tutorial by ID
//! - POST /api/tutorials: Create new tutorial (admin only, CSRF protected)
//! - PUT /api/tutorials/{id}: Update tutorial (admin only, CSRF protected)
//! - DELETE /api/tutorials/{id}: Delete tutorial (admin only, CSRF protected)
//!
//! # Data Validation
//! - Tutorial IDs: Alphanumeric and hyphens only, max 100 characters
//! - Title: 1-200 characters
//! - Description: 1-1000 characters
//! - Content: Max 100,000 characters (markdown)
//! - Icons: Whitelist of allowed Lucide icon names
//! - Colors: Tailwind gradient classes only
//!
//! # Features
//! - Full-text search integration (automatic FTS5 indexing)
//! - Topic-based organization
//! - Version tracking for content updates
//! - Soft validation to preserve data integrity

use crate::{security::auth, db::DbPool, models::*, repositories};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::collections::HashSet;
use std::convert::TryInto;
use uuid::Uuid;

pub(crate) fn validate_tutorial_id(id: &str) -> Result<(), String> {
    // Check length bounds to prevent buffer overflow attacks
    if id.is_empty() || id.len() > 100 {
        return Err("Invalid tutorial ID".to_string());
    }

    // Ensure only safe characters for database and URL usage
    if !id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err("Tutorial ID contains invalid characters".to_string());
    }
    Ok(())
}

fn validate_tutorial_data(title: &str, description: &str, content: &str) -> Result<(), String> {
    let title_trimmed = title.trim();
    if title_trimmed.is_empty() {
        return Err("Title cannot be empty".to_string());
    }
    if title_trimmed.len() > 200 {
        return Err("Title too long (max 200 characters)".to_string());
    }
    let description_trimmed = description.trim();
    if description_trimmed.is_empty() {
        return Err("Description cannot be empty".to_string());
    }
    if description_trimmed.len() > 1000 {
        return Err("Description too long (max 1000 characters)".to_string());
    }
    let content_trimmed = content.trim();
    if content_trimmed.is_empty() {
        return Err("Content cannot be empty".to_string());
    }
    if content_trimmed.len() > 100_000 {
        return Err("Content too long (max 100,000 characters)".to_string());
    }
    Ok(())
}

pub(crate) fn validate_icon(icon: &str) -> Result<(), String> {
    const ALLOWED_ICONS: &[&str] = &[
        "Terminal",   // Command line and shell tutorials
        "FolderTree", // File system and directory tutorials
        "FileText",   // Text editing and file manipulation
        "Settings",   // System configuration and settings
        "Shield",     // Security and permissions
        "Network",    // Networking and connectivity
        "Database",   // Database and data management
        "Server",     // Server administration and services
    ];

    if ALLOWED_ICONS.contains(&icon) {
        Ok(())
    } else {
        Err(format!(
            "Invalid icon '{}'. Must be one of: {:?}",
            icon, ALLOWED_ICONS
        ))
    }
}

pub(crate) fn validate_color(color: &str) -> Result<(), String> {
    const MAX_SEGMENT_LEN: usize = 32;

    fn validate_segment(segment: &str, prefix: &str) -> bool {
        // Handle modifiers (e.g., dark:from-..., md:hover:to-...)
        // We look at the last part after ':' or the whole string if no ':'
        let base_class = segment.split(':').last().unwrap_or(segment);
        
        if !base_class.starts_with(prefix) {
            return false;
        }
        let suffix = &base_class[prefix.len()..];
        !suffix.is_empty()
            && suffix.len() <= MAX_SEGMENT_LEN
            && suffix
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-')
    }

    let segments: Vec<&str> = color.split_whitespace().collect();
    // Allow more complex gradients but ensure we have at least from and to
    // Typically 2 or 3 parts: from-... [via-...] to-...
    // But could be more with responsive? No, typically "from-X to-Y" is the base structure.
    // We stick to 2 or 3 segments for simplicity of storage/validation as per original design.
    if !(segments.len() == 2 || segments.len() == 3) {
        return Err(
            "Invalid color gradient. Expected Tailwind style 'from-… [via-…] to-…' format."
                .to_string(),
        );
    }

    // Note: The logic below assumes the order is always (modifiers:)?from -> (modifiers:)?via -> (modifiers:)?to
    // This might be too strict if user writes "to-red-500 from-blue-500", but Tailwind usually encourages ordered.
    // The original code enforced order segments[0]=from, segments[1]=via/to. We keep this but allow modifiers.

    if !validate_segment(segments[0], "from-") {
        return Err("Invalid color gradient: 'from-*' segment malformed, too long, or missing.".to_string());
    }

    if segments.len() == 3 {
        // Validation for middle segment - check if it is 'via-' or 'to-'?
        // Original code expected: 0=from, 1=via, 2=to.
        if !validate_segment(segments[1], "via-") {
             return Err(
                "Invalid color gradient: Middle segment must be 'via-*' in a 3-part gradient.".to_string(),
            );
        }
        if !validate_segment(segments[2], "to-") {
            return Err(
                "Invalid color gradient: Last segment must be 'to-*'.".to_string(),
            );
        }
    } else if !validate_segment(segments[1], "to-") {
        return Err("Invalid color gradient: Last segment must be 'to-*'.".to_string());
    }

    Ok(())
}

fn sanitize_topics(topics: &[String]) -> Result<Vec<String>, String> {
    if topics.len() > 20 {
        return Err("Too many topics (max 20)".to_string());
    }

    let mut sanitized = Vec::with_capacity(topics.len());
    let mut seen = HashSet::new();

    for topic in topics {
        let trimmed = topic.trim();
        if trimmed.is_empty() {
            continue;
        }

        let limited: String = if trimmed.len() > 100 {
            trimmed.chars().take(100).collect()
        } else {
            trimmed.to_string()
        };

        let canonical = limited
            .chars()
            .map(|c| c.to_ascii_lowercase())
            .collect::<String>();

        if !seen.insert(canonical) {
            return Err("Duplicate topics are not allowed".to_string());
        }

        sanitized.push(limited);
    }

    if sanitized.is_empty() {
        return Err("At least one topic is required".to_string());
    }

    Ok(sanitized)
}

#[derive(Deserialize)]
pub struct TutorialListQuery {
    #[serde(default = "default_tutorial_limit")]
    limit: i64,

    #[serde(default)]
    offset: i64,
}

fn default_tutorial_limit() -> i64 {
    50
}

pub async fn list_tutorials(
    State(pool): State<DbPool>,
    Query(params): Query<TutorialListQuery>,
) -> Result<Json<Vec<TutorialSummaryResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let limit = params.limit.clamp(1, 100);
    let offset = params.offset.max(0);

    // Optimized query: Exclude 'content' column to reduce payload size
    let tutorials = repositories::tutorials::list_tutorials(&pool, limit, offset)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to fetch tutorials".to_string(),
                }),
            )
        })?;

    let mut responses = Vec::with_capacity(tutorials.len());
    for tutorial in tutorials {
        let response: TutorialSummaryResponse = tutorial.try_into().map_err(|err: String| {
            tracing::error!("Tutorial data corruption detected: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to parse stored tutorial data".to_string(),
                }),
            )
        })?;
        responses.push(response);
    }

    Ok(Json(responses))
}

pub async fn get_tutorial(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
) -> Result<Json<TutorialResponse>, (StatusCode, Json<ErrorResponse>)> {
    if let Err(e) = validate_tutorial_id(&id) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })));
    }

    let tutorial = repositories::tutorials::get_tutorial(&pool, &id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to fetch tutorial".to_string(),
                }),
            )
        })?;

    let tutorial = tutorial.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Tutorial not found".to_string(),
            }),
        )
    })?;

    let response: TutorialResponse = tutorial.try_into().map_err(|err: String| {
        tracing::error!("Tutorial data corruption detected: {}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to parse stored tutorial data".to_string(),
            }),
        )
    })?;

    Ok(Json(response))
}

pub async fn create_tutorial(
    claims: auth::Claims,
    State(pool): State<DbPool>,
    Json(payload): Json<CreateTutorialRequest>,
) -> Result<Json<TutorialResponse>, (StatusCode, Json<ErrorResponse>)> {
    if claims.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Insufficient permissions".to_string(),
            }),
        ));
    }

    let title = payload.title.trim().to_string();
    let description = payload.description.trim().to_string();
    let content = payload.content.trim().to_string();

    if let Err(e) = validate_tutorial_data(&title, &description, &content) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })));
    }

    if let Err(e) = validate_icon(&payload.icon) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })));
    }
    if let Err(e) = validate_color(&payload.color) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })));
    }

    let id = if let Some(custom_id) = &payload.id {
        let trimmed = custom_id.trim();
        if let Err(e) = validate_tutorial_id(trimmed) {
            return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })));
        }
        // Check for collision
        let exists = repositories::tutorials::check_tutorial_exists(&pool, trimmed)
            .await
            .map_err(|e| {
                tracing::error!("Database error checking ID existence: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Failed to create tutorial".to_string(),
                    }),
                )
            })?;

        if exists {
            return Err((
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    error: "Tutorial ID already exists".to_string(),
                }),
            ));
        }
        trimmed.to_string()
    } else {
        Uuid::new_v4().to_string()
    };
    let sanitized_topics = sanitize_topics(&payload.topics)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })))?;
    let topics_json = serde_json::to_string(&sanitized_topics).map_err(|e| {
        tracing::error!("Failed to serialize topics: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to create tutorial".to_string(),
            }),
        )
    })?;
    let tutorial = repositories::tutorials::create_tutorial(
        &pool,
        &id,
        &title,
        &description,
        &content,
        &payload.icon,
        &payload.color,
        &topics_json,
        &sanitized_topics,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to create tutorial {}: {}", id, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to create tutorial".to_string(),
            }),
        )
    })?;

    let response: TutorialResponse = tutorial.try_into().map_err(|err: String| {
        tracing::error!(
            "Tutorial data corruption detected after create {}: {}",
            id,
            err
        );
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to create tutorial".to_string(),
            }),
        )
    })?;

    Ok(Json(response))
}

pub async fn update_tutorial(
    claims: auth::Claims,
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTutorialRequest>,
) -> Result<Json<TutorialResponse>, (StatusCode, Json<ErrorResponse>)> {
    tracing::info!("Updating tutorial with id: {}", id);

    if claims.role != "admin" {
        tracing::warn!(
            "Unauthorized update attempt for tutorial {} by user {}",
            id,
            claims.sub
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Insufficient permissions".to_string(),
            }),
        ));
    }

    if let Err(e) = validate_tutorial_id(&id) {
        tracing::warn!("Invalid tutorial ID during update: {}", id);
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })));
    }

    let tutorial = repositories::tutorials::get_tutorial(&pool, &id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to fetch tutorial".to_string(),
                }),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Tutorial not found".to_string(),
                }),
            )
        })?;

    let title = match payload.title {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Title cannot be empty".to_string(),
                    }),
                ));
            }
            trimmed.to_string()
        }
        None => tutorial.title.trim().to_string(),
    };

    let description = match payload.description {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Description cannot be empty".to_string(),
                    }),
                ));
            }
            trimmed.to_string()
        }
        None => tutorial.description.trim().to_string(),
    };

    let icon = payload.icon.unwrap_or(tutorial.icon);
    let color = payload.color.unwrap_or(tutorial.color);
    let content = match payload.content {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Content cannot be empty".to_string(),
                    }),
                ));
            }
            trimmed.to_string()
        }
        None => tutorial.content.trim().to_string(),
    };

    tracing::debug!(
        "Tutorial update data - title length: {}, description length: {}, content length: {}",
        title.len(),
        description.len(),
        content.len()
    );

    if let Err(e) = validate_tutorial_data(&title, &description, &content) {
        tracing::warn!("Validation failed for tutorial {}: {}", id, e);
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })));
    }

    if let Err(e) = validate_icon(&icon) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })));
    }
    if let Err(e) = validate_color(&color) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })));
    }

    let new_version = tutorial.version.checked_add(1).ok_or_else(|| {
        tracing::error!("Tutorial version overflow for id: {}", id);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Tutorial version overflow".to_string(),
            }),
        )
    })?;

    let (topics_json, topics_vec) = if let Some(t) = payload.topics {
        let sanitized = sanitize_topics(&t)
            .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })))?;

        let serialized = serde_json::to_string(&sanitized).map_err(|e| {
            tracing::error!("Failed to serialize topics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to update tutorial".to_string(),
                }),
            )
        })?;

        (serialized, sanitized)
    } else {
        match serde_json::from_str::<Vec<String>>(&tutorial.topics) {
            Ok(existing_topics) => (tutorial.topics.clone(), existing_topics),
            Err(e) => {
                tracing::error!(
                    "Failed to deserialize topics for tutorial {}: {}",
                    tutorial.id,
                    e
                );
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Failed to read stored tutorial topics".to_string(),
                    }),
                ));
            }
        }
    };

    let updated_tutorial = repositories::tutorials::update_tutorial(
        &pool,
        &id,
        &title,
        &description,
        &content,
        &icon,
        &color,
        &topics_json,
        &topics_vec,
        new_version.try_into().unwrap_or(1),
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to update tutorial {}: {}", id, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to update tutorial".to_string(),
            }),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "Tutorial was modified by another request. Please refresh and try again."
                    .to_string(),
            }),
        )
    })?;

    tracing::info!("Successfully updated tutorial {}", id);
    let response: TutorialResponse = updated_tutorial.try_into().map_err(|err: String| {
        tracing::error!(
            "Tutorial data corruption detected after update {}: {}",
            id,
            err
        );
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to update tutorial".to_string(),
            }),
        )
    })?;

    Ok(Json(response))
}

pub async fn delete_tutorial(
    claims: auth::Claims,
    State(pool): State<DbPool>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    if claims.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Insufficient permissions".to_string(),
            }),
        ));
    }

    if let Err(e) = validate_tutorial_id(&id) {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })));
    }

    let deleted = repositories::tutorials::delete_tutorial(&pool, &id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to delete tutorial".to_string(),
                }),
            )
        })?;

    if !deleted {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Tutorial not found".to_string(),
            }),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}
