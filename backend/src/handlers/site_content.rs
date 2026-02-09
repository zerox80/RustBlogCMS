//! Site Content HTTP Handlers
//!
//! This module provides an API to manage global, semi-static content sections
//! of the website, such as header, footer, hero, and site metadata.
//! Content is stored as JSON in the database, allowing for a flexible,
//! schema-less landing page design.

use crate::{
    security::auth, db,
    models::{
        ErrorResponse, SiteContentListResponse, SiteContentResponse, UpdateSiteContentRequest,
    },
    repositories,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::Value;
use std::collections::HashSet;

/// Maximum size allowed for a single content section's JSON payload (5MB)
const MAX_CONTENT_BYTES: usize = 5_000_000;

/// A globally initialized set of section names that the API is allowed to manage.
/// Prevents accidental or malicious creation of arbitrary content sections.
fn allowed_sections() -> &'static HashSet<&'static str> {
    use std::sync::OnceLock;

    static ALLOWED: OnceLock<HashSet<&'static str>> = OnceLock::new();
    ALLOWED.get_or_init(|| {
        [
            "hero",              // Landing page hero
            "tutorial_section",  // Tutorial overview header
            "header",            // Main navigation
            "footer",            // Footer links/info
            "site_meta",         // SEO titles/description
            "stats",             // Numbers/stats display
            "cta_section",       // Call to action
            "settings",          // System-wide toggles
            "login",             // Custom login page text
        ]
        .into_iter()
        .collect()
    })
}

/// Validates if a section name is within the whitelist of allowed sections.
fn validate_section(section: &str) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if allowed_sections().contains(section) {
        Ok(())
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Unknown content section '{section}'"),
            }),
        ))
    }
}

/// Dispatches validation to section-specific structure checkers.
/// Ensures the incoming JSON follows the expected format for that section.
fn validate_content_structure(
    section: &str,
    content: &Value,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let result = match section {
        "hero" => validate_hero_structure(content),
        "tutorial_section" => validate_tutorial_section_structure(content),
        "header" => validate_header_structure(content),
        "footer" => validate_footer_structure(content),
        "settings" => validate_settings_structure(content),
        "site_meta" => validate_site_meta_structure(content),
        "game_config" => Ok(()), // Legacy/Future use
        "stats" => Ok(()),
        "cta_section" => Ok(()),
        "login" => validate_login_structure(content),
        _ => Ok(()),
    };

    result.map_err(|err| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Invalid structure for section '{section}': {err}"),
            }),
        )
    })
}

/// Validates the site metadata (SEO) structure.
fn validate_site_meta_structure(content: &Value) -> Result<(), &'static str> {
    let obj = content.as_object().ok_or("Expected JSON object")?;
    // Ensure basic SEO fields are present
    if !obj.contains_key("title") {
        return Err("Missing required field 'title'");
    }
    if !obj.contains_key("description") {
        return Err("Missing required field 'description'");
    }
    // Perform type checking on optional fields
    if let Some(kw) = obj.get("keywords") {
        if !kw.is_string() {
             return Err("Field 'keywords' must be a string");
        }
    }
    Ok(())
}

/// Validates the hero section structure.
fn validate_hero_structure(content: &Value) -> Result<(), &'static str> {
    let obj = content.as_object().ok_or("Expected JSON object")?;
    // Hero needs a title and a list of feature highlights
    if !obj.contains_key("title") || !obj.contains_key("features") {
        return Err("Missing required fields 'title' or 'features'");
    }
    if !obj.get("features").map(|v| v.is_array()).unwrap_or(false) {
        return Err("Field 'features' must be an array");
    }
    Ok(())
}

/// Validates the tutorial overview section structure.
fn validate_tutorial_section_structure(content: &Value) -> Result<(), &'static str> {
    let obj = content.as_object().ok_or("Expected JSON object")?;
    if !obj.contains_key("title") || !obj.contains_key("description") {
        return Err("Missing required fields 'title' or 'description'");
    }
    Ok(())
}

/// Validates the global header (navigation) structure.
fn validate_header_structure(content: &Value) -> Result<(), &'static str> {
    let obj = content.as_object().ok_or("Expected JSON object")?;
    // Must have brand info (logo/name) and a list of nav items
    if !obj.contains_key("brand") || !obj.contains_key("navItems") {
        return Err("Missing required fields 'brand' or 'navItems'");
    }
    
    // Check if navItems is an array.
    let nav_items = obj.get("navItems").and_then(|v| v.as_array()).ok_or("Field 'navItems' must be an array")?;

    // Fix Bug 4: Header Validation Incomplete
    // Validate that each item in the array has at least an 'id' and 'label', and a valid target ('path', 'slug', 'url', etc.)
    for (_i, item) in nav_items.iter().enumerate() {
        let item_obj = item.as_object().ok_or("Nav item must be an object")?;
        
        if !item_obj.contains_key("id") || !item_obj.contains_key("label") {
             return Err("Nav item missing required fields 'id' or 'label'");
        }

        // Check for at least one target field if it's not a section header (type="section" might not need a path)
        let has_target = item_obj.contains_key("path") 
            || item_obj.contains_key("slug") 
            || item_obj.contains_key("url")
            || item_obj.contains_key("value")
            || item_obj.get("type").map(|v| v == "section").unwrap_or(false);

        if !has_target {
             return Err("Nav item must have a target (path, slug, url, or value) or be type='section'");
        }

        // Validate slug is not empty if present
        if let Some(slug) = item_obj.get("slug").and_then(|v| v.as_str()) {
            if slug.trim().is_empty() {
                return Err("Nav item slug cannot be empty");
            }
        }
    }
    Ok(())
}

/// Validates the global footer structure.
fn validate_footer_structure(content: &Value) -> Result<(), &'static str> {
    let obj = content.as_object().ok_or("Expected JSON object")?;
    if !obj.contains_key("brand") || !obj.contains_key("quickLinks") {
        return Err("Missing required fields 'brand' or 'quickLinks'");
    }
    Ok(())
}

/// Validates global site settings structure.
fn validate_settings_structure(content: &Value) -> Result<(), &'static str> {
    let obj = content.as_object().ok_or("Expected JSON object")?;
    // Example: check for a boolean PDF toggle
    if let Some(val) = obj.get("pdfEnabled") {
        if !val.is_boolean() {
            return Err("Field 'pdfEnabled' must be a boolean");
        }
    }
    Ok(())
}

/// Validates customizations for the login page.
fn validate_login_structure(content: &Value) -> Result<(), &'static str> {
    let obj = content.as_object().ok_or("Expected JSON object")?;
    // Ensure the login page at least defines a welcome title
    if !obj.contains_key("title") {
        return Err("Missing required field 'title'");
    }
    Ok(())
}

/// Ensures the size of the serialized JSON doesn't exceed the safe threshold.
fn validate_content_size(content: &Value) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    match serde_json::to_string(content) {
        // If length is within boundaries, accept it
        Ok(serialized) if serialized.len() <= MAX_CONTENT_BYTES => Ok(()),
        // Otherwise, reject due to payload size
        Ok(_) => Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(ErrorResponse {
                error: format!("Content too large (max {MAX_CONTENT_BYTES} bytes)"),
            }),
        )),
        // Handle serialization errors
        Err(err) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Invalid JSON content: {err}"),
            }),
        )),
    }
}

/// Maps a database content record to a public response structure.
/// Involves decoding the stored JSON string back into a JSON object.
fn map_record(
    record: crate::models::SiteContent,
) -> Result<SiteContentResponse, (StatusCode, Json<ErrorResponse>)> {
    // Attempt to parse the stored string from the 'content_json' table column
    let content: Value = serde_json::from_str(&record.content_json).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to parse stored content JSON: {err}"),
            }),
        )
    })?;

    // Construct the response
    Ok(SiteContentResponse {
        section: record.section,
        content,
        updated_at: record.updated_at,
    })
}

/// Handler to fetch all managed site content sections in bulk.
pub async fn list_site_content(
    State(pool): State<db::DbPool>,
) -> Result<Json<SiteContentListResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Fetch all records from the site_content table
    let records = repositories::content::fetch_all_site_content(&pool)
        .await
        .map_err(|err| {
            tracing::error!("Failed to load site content: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to load site content".to_string(),
                }),
            )
        })?;

    // Convert each record from string-based JSON to object-based JSON
    let mut items = Vec::with_capacity(records.len());
    for record in records {
        items.push(map_record(record)?);
    }

    Ok(Json(SiteContentListResponse { items }))
}

/// Handler to fetch a single content section by its name.
pub async fn get_site_content(
    State(pool): State<db::DbPool>,
    Path(section): Path<String>,
) -> Result<Json<SiteContentResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Security check: only allow pre-defined sections
    validate_section(&section)?;

    // Retrieve from database
    let record = repositories::content::fetch_site_content_by_section(&pool, &section)
        .await
        .map_err(|err| {
            tracing::error!("Failed to load site content '{}': {}", section, err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to load site content".to_string(),
                }),
            )
        })?
        .ok_or_else(|| {
            // Section name is valid but no content exists yet
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Content section '{section}' not found"),
                }),
            )
        })?;

    // Map and return
    Ok(Json(map_record(record)?))
}

/// Handler to update or create a content section.
/// Admin-only endpoint with strict validation on structure and size.
pub async fn update_site_content(
    claims: auth::Claims,
    State(pool): State<db::DbPool>,
    Path(section): Path<String>,
    Json(payload): Json<UpdateSiteContentRequest>,
) -> Result<Json<SiteContentResponse>, (StatusCode, Json<ErrorResponse>)> {
    // RBAC: Verify user has 'admin' role
    if claims.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Insufficient permissions".to_string(),
            }),
        ));
    }

    // Comprehensive validation
    validate_section(&section)?;                // Whitelist check
    validate_content_size(&payload.content)?;    // Size sanity check
    validate_content_structure(&section, &payload.content)?; // Format correctness check

    // Upsert (Insert or Update) in database
    let record = repositories::content::upsert_site_content(&pool, &section, &payload.content)
        .await
        .map_err(|err| {
            tracing::error!("Failed to update site content '{}': {}", section, err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to update site content".to_string(),
                }),
            )
        })?;

    // Return the updated state
    Ok(Json(map_record(record)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_header_structure_relaxed() {
        // Case 1: Standard link with path
        let content_standard = json!({
            "brand": { "name": "Test" },
            "navItems": [
                { "id": "1", "label": "Blog", "path": "/blog" }
            ]
        });
        assert!(validate_header_structure(&content_standard).is_ok());

        // Case 2: Section link with type="section" (no explicit target field)
        let content_section = json!({
            "brand": { "name": "Test" },
            "navItems": [
                { "id": "home", "label": "Home", "type": "section" }
            ]
        });
        assert!(validate_header_structure(&content_section).is_ok(), "Should accept type='section' without other target fields");

        // Case 3: Link with 'value' field (e.g. from some frontend logic)
        let content_value = json!({
            "brand": { "name": "Test" },
            "navItems": [
                { "id": "2", "label": "About", "value": "about-us" }
            ]
        });
        assert!(validate_header_structure(&content_value).is_ok(), "Should accept 'value' field as target");

        // Case 4: Invalid item (missing target)
        let content_invalid = json!({
            "brand": { "name": "Test" },
            "navItems": [
                { "id": "3", "label": "Invalid" }
            ]
        });
        assert!(validate_header_structure(&content_invalid).is_err());
    }

    #[test]
    fn test_validate_login_structure() {
        // Case 1: Valid login content
        let content_valid = json!({
            "title": "Login",
            "subtitle": "Welcome back"
        });
        assert!(validate_login_structure(&content_valid).is_ok());

        // Case 2: Missing title
        let content_invalid = json!({
            "subtitle": "Welcome back"
        });
        assert!(validate_login_structure(&content_invalid).is_err());
    }

    #[test]
    fn test_validate_header_structure_rejects_empty_target() {
        // Case: Empty slug should be rejected
        let content_empty_slug = json!({
            "brand": { "name": "Test" },
            "navItems": [
                { "id": "1", "label": "Empty Slug", "slug": "" }
            ]
        });
        assert!(validate_header_structure(&content_empty_slug).is_err(), "Should reject empty slug");

        // Case: Whitespace-only slug should be rejected
        let content_whitespace_slug = json!({
            "brand": { "name": "Test" },
            "navItems": [
                { "id": "2", "label": "Whitespace Slug", "slug": "   " }
            ]
        });
        assert!(validate_header_structure(&content_whitespace_slug).is_err(), "Should reject whitespace-only slug");
    }
}
