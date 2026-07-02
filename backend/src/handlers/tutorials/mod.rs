//! Tutorial Management HTTP Handlers
//!
//! This module provides a robust API for managing educational tutorials.
//! It includes strict input validation, role-based access control, and
//! seamless integration with full-text search (FTS5).
//!
//! Tutorials are structured with:
//! - Metadata: Title, Description, Topics, Icon (Lucide), Color (Tailwind)
//! - Content: Markdown-based learning material
//! - Versioning: Optimistic concurrency control via version numbers
//! - Identifiers: Custom slugs or auto-generated UUIDs

use crate::{
    db::DbPool, handlers::common::ensure_admin, models::*, repositories, security::auth,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::collections::HashSet;
use std::convert::TryInto;
use uuid::Uuid;

/// Validates a tutorial ID for length and character safety.
/// Used to prevent path injection and ensure URL compatibility.
pub(crate) fn validate_tutorial_id(id: &str) -> Result<(), String> {
    // Check length bounds to prevent buffer overflow or DoS attacks
    if id.is_empty() || id.len() > 100 {
        return Err("Invalid tutorial ID (must be 1-100 characters)".to_string());
    }

    // Ensure only safe characters for database and URL usage
    if !id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(
            "Tutorial ID contains invalid characters (allowed: alphanumeric, -, _, .)".to_string(),
        );
    }
    Ok(())
}

/// Validates the core text content of a tutorial.
fn validate_tutorial_data(title: &str, description: &str, content: &str) -> Result<(), String> {
    // Title validation
    let title_trimmed = title.trim();
    if title_trimmed.is_empty() {
        return Err("Title cannot be empty".to_string());
    }
    if title_trimmed.len() > 200 {
        return Err("Title too long (max 200 characters)".to_string());
    }

    // Description validation
    let description_trimmed = description.trim();
    if description_trimmed.is_empty() {
        return Err("Description cannot be empty".to_string());
    }
    if description_trimmed.len() > 1000 {
        return Err("Description too long (max 1000 characters)".to_string());
    }

    // Markdown content validation
    let content_trimmed = content.trim();
    if content_trimmed.is_empty() {
        return Err("Content cannot be empty".to_string());
    }
    if content_trimmed.len() > 100_000 {
        return Err("Content too long (max 100,000 characters)".to_string());
    }
    Ok(())
}

/// Validates that the provided icon name is within the allowed Lucide whitelist.
pub(crate) fn validate_icon(icon: &str) -> Result<(), String> {
    /// Whitelist of Lucide icon identifiers used in the frontend
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

/// Validates a Tailwind CSS gradient string.
/// Ensures the format 'from-COLOR [via-COLOR] to-COLOR' is followed.
pub(crate) fn validate_color(color: &str) -> Result<(), String> {
    const MAX_SEGMENT_LEN: usize = 32;

    /// Checks if a single tailwind class segment is valid (e.g. 'from-blue-500')
    fn validate_segment(segment: &str, prefix: &str) -> bool {
        // Handle responsive modifiers (e.g., dark:from-..., md:hover:to-...)
        // We look at the last part after ':' or the whole string if no ':'
        let base_class = segment.split(':').next_back().unwrap_or(segment);

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

    // Gradients must have 2 (from/to) or 3 (from/via/to) segments
    if !(segments.len() == 2 || segments.len() == 3) {
        return Err(
            "Invalid color gradient. Expected Tailwind style 'from-… [via-…] to-…' format."
                .to_string(),
        );
    }

    // Note: The logic below assumes the order is always (modifiers:)?from -> (modifiers:)?via -> (modifiers:)?to
    // This might be too strict if user writes "to-red-500 from-blue-500", but Tailwind usually encourages ordered.
    // The original code enforced order segments[0]=from, segments[1]=via/to. We keep this but allow modifiers.

    // Validate 'from-' segment
    if !validate_segment(segments[0], "from-") {
        return Err(
            "Invalid color gradient: 'from-*' segment malformed, too long, or missing.".to_string(),
        );
    }

    if segments.len() == 3 {
        // Validation for middle segment - check if it is 'via-' or 'to-'?
        // Original code expected: 0=from, 1=via, 2=to.
        // Validate internal 'via-' segment
        if !validate_segment(segments[1], "via-") {
            return Err(
                "Invalid color gradient: Middle segment must be 'via-*' in a 3-part gradient."
                    .to_string(),
            );
        }
        // Validate 'to-' segment
        if !validate_segment(segments[2], "to-") {
            return Err("Invalid color gradient: Last segment must be 'to-*'.".to_string());
        }
    } else if !validate_segment(segments[1], "to-") {
        // Validate 'to-' segment for 2-part gradient
        return Err("Invalid color gradient: Last segment must be 'to-*'.".to_string());
    }

    Ok(())
}

/// Sanitizes a list of topics.
/// Normalizes to lowercase, removes duplicates, and trims long strings.
fn sanitize_topics(topics: &[String]) -> Result<Vec<String>, String> {
    // SECURITY: Limit number of topics to prevent indexing DoS
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

        // ENFORCEMENT: Truncate excessively long topic names
        let limited: String = if trimmed.len() > 100 {
            trimmed.chars().take(100).collect()
        } else {
            trimmed.to_string()
        };

        // Normalize to lowercase for duplicate detection
        let canonical = limited
            .chars()
            .map(|c| c.to_ascii_lowercase())
            .collect::<String>();

        if !seen.insert(canonical) {
            return Err("Duplicate topics are not allowed".to_string());
        }

        sanitized.push(limited);
    }

    // Requirements
    if sanitized.is_empty() {
        return Err("At least one topic is required".to_string());
    }

    Ok(sanitized)
}

/// Query parameters for paginated tutorial listing.
#[derive(Deserialize)]
pub struct TutorialListQuery {
    /// Number of items to return (default: 50, max: 100)
    #[serde(default = "default_tutorial_limit")]
    limit: i64,

    /// Number of items to skip for pagination
    #[serde(default)]
    offset: i64,
}

/// Default limit for tutorial lists
fn default_tutorial_limit() -> i64 {
    50
}

/// Handler for listing tutorials with pagination.
/// Publicly accessible. Excludes full tutorial content to minimize payload size.
pub async fn list_tutorials(
    State(pool): State<DbPool>,
    Query(params): Query<TutorialListQuery>,
) -> Result<Json<Vec<TutorialSummaryResponse>>, ApiError> {
    // Clamp pagination parameters
    let limit = params.limit.clamp(1, 100);
    let offset = params.offset.max(0);

    // Optimized repository call: Fetches summary data without markdown content
    let tutorials = repositories::tutorials::list_tutorials(&pool, limit, offset)
        .await
        .map_err(internal_error("Failed to fetch tutorials"))?;

    // Transform database records into summary response models
    let mut responses = Vec::with_capacity(tutorials.len());
    for tutorial in tutorials {
        // TryInto implementation handles JSON parsing of the 'topics' field
        let response: TutorialSummaryResponse = tutorial
            .try_into()
            .map_err(internal_error("Failed to parse stored tutorial data"))?;
        responses.push(response);
    }

    Ok(Json(responses))
}

/// Handler to retrieve full details of a specific tutorial by its string ID.
/// Publicly accessible. Includes full markdown content.
pub async fn get_tutorial(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
) -> Result<Json<TutorialResponse>, ApiError> {
    // Validate ID format before touching the database
    validate_tutorial_id(&id).map_err(bad_request)?;

    // Attempt to retrieve record from database
    let tutorial = repositories::tutorials::get_tutorial(&pool, &id)
        .await
        .map_err(internal_error("Failed to fetch tutorial"))?
        // Handle 404
        .ok_or_else(|| not_found("Tutorial not found"))?;

    // Transform database record (Tutorial) into full response model (TutorialResponse)
    // This step parses the 'topics' JSON string into a Vec<String>.
    let response: TutorialResponse = tutorial
        .try_into()
        .map_err(internal_error("Failed to parse stored tutorial data"))?;

    Ok(Json(response))
}

/// Handler to create a new tutorial.
/// Admin-only. Protected by RBAC (claims check).
/// Performs comprehensive validation of ID, titles, content, icons, colors, and topics.
pub async fn create_tutorial(
    claims: auth::Claims,
    State(pool): State<DbPool>,
    Json(payload): Json<CreateTutorialRequest>,
) -> Result<Json<TutorialResponse>, ApiError> {
    // RBAC: Verify admin privileges
    ensure_admin(&claims)?;

    // Sanitize basic text fields
    let title = payload.title.trim().to_string();
    let description = payload.description.trim().to_string();
    let content = payload.content.trim().to_string();

    // Perform deep validation of tutorial metadata
    validate_tutorial_data(&title, &description, &content).map_err(bad_request)?;
    validate_icon(&payload.icon).map_err(bad_request)?;
    validate_color(&payload.color).map_err(bad_request)?;

    // Determine ID: either custom (validated/checked for collisions) or auto-generated UUID
    let id = if let Some(custom_id) = &payload.id {
        let trimmed = custom_id.trim();
        validate_tutorial_id(trimmed).map_err(bad_request)?;
        // Collision detection for custom IDs
        let exists = repositories::tutorials::check_tutorial_exists(&pool, trimmed)
            .await
            .map_err(internal_error("Failed to create tutorial"))?;

        if exists {
            return Err(api_error(
                StatusCode::CONFLICT,
                "Tutorial ID already exists",
            ));
        }
        trimmed.to_string()
    } else {
        // Fallback to random identifier
        Uuid::new_v4().to_string()
    };

    // Sanitize and serialize topics
    let sanitized_topics = sanitize_topics(&payload.topics).map_err(bad_request)?;
    let topics_json = serde_json::to_string(&sanitized_topics)
        .map_err(internal_error("Failed to create tutorial"))?;

    // Persist to database
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
    .map_err(internal_error("Failed to create tutorial"))?;

    // Final mapping to response model
    let response: TutorialResponse = tutorial
        .try_into()
        .map_err(internal_error("Failed to parse stored tutorial data"))?;

    Ok(Json(response))
}

/// Handler to update an existing tutorial.
/// Admin-only. Implements optimistic concurrency control using a version number.
pub async fn update_tutorial(
    claims: auth::Claims,
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTutorialRequest>,
) -> Result<Json<TutorialResponse>, ApiError> {
    tracing::info!("Updating tutorial with id: {}", id);

    // RBAC: Verify admin role
    if claims.role != "admin" {
        tracing::warn!(
            "Unauthorized update attempt for tutorial {} by user {}",
            id,
            claims.sub
        );
        return Err(forbidden("Insufficient permissions"));
    }

    // Validate ID before database interaction
    validate_tutorial_id(&id).map_err(|e| {
        tracing::warn!("Invalid tutorial ID during update: {}", id);
        bad_request(e)
    })?;

    // Step 1: Pre-fetch current state to check existence and current version
    let tutorial = repositories::tutorials::get_tutorial(&pool, &id)
        .await
        .map_err(internal_error("Failed to fetch tutorial"))?
        .ok_or_else(|| not_found("Tutorial not found"))?;

    // Step 2: Merge partial updates with existing data
    // Title update
    let title = match payload.title {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(bad_request("Title cannot be empty"));
            }
            trimmed.to_string()
        }
        None => tutorial.title.trim().to_string(),
    };

    // Description update
    let description = match payload.description {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(bad_request("Description cannot be empty"));
            }
            trimmed.to_string()
        }
        None => tutorial.description.trim().to_string(),
    };

    let icon = payload.icon.unwrap_or(tutorial.icon);
    let color = payload.color.unwrap_or(tutorial.color);

    // Content update
    let content = match payload.content {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(bad_request("Content cannot be empty"));
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

    // Step 3: Deep validation of merged tutorial state
    validate_tutorial_data(&title, &description, &content).map_err(|e| {
        tracing::warn!("Validation failed for tutorial {}: {}", id, e);
        bad_request(e)
    })?;

    validate_icon(&icon).map_err(bad_request)?;
    validate_color(&color).map_err(bad_request)?;

    // Step 5: Handle topics serialization
    let (topics_json, topics_vec) = if let Some(t) = payload.topics {
        // Sanitize new topics if provided
        let sanitized = sanitize_topics(&t).map_err(bad_request)?;

        let serialized = serde_json::to_string(&sanitized)
            .map_err(internal_error("Failed to update tutorial"))?;

        (serialized, sanitized)
    } else {
        // Carry over existing topics
        let existing_topics = serde_json::from_str::<Vec<String>>(&tutorial.topics)
            .map_err(internal_error("Failed to read stored tutorial topics"))?;
        (tutorial.topics.clone(), existing_topics)
    };

    // Step 6: Atomic Update operation in repository
    // The repository handles the version increment and the WHERE version = current_version check
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
        tutorial.version as i32, // The repository checks WHERE version = current_version
    )
    .await
    .map_err(internal_error("Failed to update tutorial"))?
    .ok_or_else(|| {
        // If query returns None, it likely means the version ID mismatch (concurrency conflict)
        api_error(
            StatusCode::CONFLICT,
            "Tutorial was modified by another request. Please refresh and try again.",
        )
    })?;

    // Success mapping
    tracing::info!("Successfully updated tutorial {}", id);
    let response: TutorialResponse = updated_tutorial
        .try_into()
        .map_err(internal_error("Failed to parse stored tutorial data"))?;

    Ok(Json(response))
}

/// Handler to permanently delete a tutorial.
/// Admin-only.
pub async fn delete_tutorial(
    claims: auth::Claims,
    State(pool): State<DbPool>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    // RBAC: Verify admin role
    ensure_admin(&claims)?;

    // Validate ID before database interaction
    validate_tutorial_id(&id).map_err(bad_request)?;

    // Attempt deletion in repository
    let deleted = repositories::tutorials::delete_tutorial(&pool, &id)
        .await
        .map_err(internal_error("Failed to delete tutorial"))?;

    // Handle 404
    if !deleted {
        return Err(not_found("Tutorial not found"));
    }

    Ok(StatusCode::NO_CONTENT)
}
