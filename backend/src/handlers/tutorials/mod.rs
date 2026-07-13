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

use crate::{db::DbPool, handlers::common::ensure_admin, models::*, repositories, security::auth};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::collections::HashSet;
use std::convert::TryInto;
use uuid::Uuid;

mod validation;
use validation::*;
pub(crate) use validation::{validate_color, validate_icon, validate_tutorial_id};

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
