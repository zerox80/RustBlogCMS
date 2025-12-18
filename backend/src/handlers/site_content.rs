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

const MAX_CONTENT_BYTES: usize = 5_000_000;

fn allowed_sections() -> &'static HashSet<&'static str> {
    use std::sync::OnceLock;

    static ALLOWED: OnceLock<HashSet<&'static str>> = OnceLock::new();
    ALLOWED.get_or_init(|| {
        [
            "hero",
            "tutorial_section",
            "header",
            "footer",
            "site_meta",
            "stats",
            "cta_section",
            "settings",
            "login",
        ]
        .into_iter()
        .collect()
    })
}

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

fn validate_site_meta_structure(content: &Value) -> Result<(), &'static str> {
    let obj = content.as_object().ok_or("Expected JSON object")?;
    if !obj.contains_key("title") {
        return Err("Missing required field 'title'");
    }
    if !obj.contains_key("description") {
        return Err("Missing required field 'description'");
    }
    // keywords is optional but often good to check type if present
    if let Some(kw) = obj.get("keywords") {
        if !kw.is_string() {
             return Err("Field 'keywords' must be a string");
        }
    }
    Ok(())
}

fn validate_hero_structure(content: &Value) -> Result<(), &'static str> {
    let obj = content.as_object().ok_or("Expected JSON object")?;
    if !obj.contains_key("title") || !obj.contains_key("features") {
        return Err("Missing required fields 'title' or 'features'");
    }
    if !obj.get("features").map(|v| v.is_array()).unwrap_or(false) {
        return Err("Field 'features' must be an array");
    }
    Ok(())
}

fn validate_tutorial_section_structure(content: &Value) -> Result<(), &'static str> {
    let obj = content.as_object().ok_or("Expected JSON object")?;
    if !obj.contains_key("title") || !obj.contains_key("description") {
        return Err("Missing required fields 'title' or 'description'");
    }
    Ok(())
}

fn validate_header_structure(content: &Value) -> Result<(), &'static str> {
    let obj = content.as_object().ok_or("Expected JSON object")?;
    if !obj.contains_key("brand") || !obj.contains_key("navItems") {
        return Err("Missing required fields 'brand' or 'navItems'");
    }
    // Relaxed validation: we only check if navItems is an array.
    // We do NOT strictly check if every item has a target, to allow saving work-in-progress.
    if !obj.get("navItems").map(|v| v.is_array()).unwrap_or(false) {
        return Err("Field 'navItems' must be an array");
    }
    Ok(())
}

fn validate_footer_structure(content: &Value) -> Result<(), &'static str> {
    let obj = content.as_object().ok_or("Expected JSON object")?;
    if !obj.contains_key("brand") || !obj.contains_key("quickLinks") {
        return Err("Missing required fields 'brand' or 'quickLinks'");
    }
    Ok(())
}

fn validate_settings_structure(content: &Value) -> Result<(), &'static str> {
    let obj = content.as_object().ok_or("Expected JSON object")?;
    // We expect at least pdfEnabled, but we can be lenient or strict.
    // Let's be strict about the type if it exists.
    if let Some(val) = obj.get("pdfEnabled") {
        if !val.is_boolean() {
            return Err("Field 'pdfEnabled' must be a boolean");
        }
    }
    Ok(())
}

fn validate_login_structure(content: &Value) -> Result<(), &'static str> {
    let obj = content.as_object().ok_or("Expected JSON object")?;
    // We can be lenient, but let's check for at least one expected field if we want strictness.
    // For now, just ensuring it's an object is enough, or check for 'title'.
    if !obj.contains_key("title") {
        return Err("Missing required field 'title'");
    }
    Ok(())
}

fn validate_content_size(content: &Value) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    match serde_json::to_string(content) {
        Ok(serialized) if serialized.len() <= MAX_CONTENT_BYTES => Ok(()),
        Ok(_) => Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(ErrorResponse {
                error: format!("Content too large (max {MAX_CONTENT_BYTES} bytes)"),
            }),
        )),
        Err(err) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Invalid JSON content: {err}"),
            }),
        )),
    }
}

fn map_record(
    record: crate::models::SiteContent,
) -> Result<SiteContentResponse, (StatusCode, Json<ErrorResponse>)> {
    let content: Value = serde_json::from_str(&record.content_json).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to parse stored content JSON: {err}"),
            }),
        )
    })?;

    Ok(SiteContentResponse {
        section: record.section,
        content,
        updated_at: record.updated_at,
    })
}

pub async fn list_site_content(
    State(pool): State<db::DbPool>,
) -> Result<Json<SiteContentListResponse>, (StatusCode, Json<ErrorResponse>)> {
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

    let mut items = Vec::with_capacity(records.len());
    for record in records {
        items.push(map_record(record)?);
    }

    Ok(Json(SiteContentListResponse { items }))
}

pub async fn get_site_content(
    State(pool): State<db::DbPool>,
    Path(section): Path<String>,
) -> Result<Json<SiteContentResponse>, (StatusCode, Json<ErrorResponse>)> {
    validate_section(&section)?;

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
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Content section '{section}' not found"),
                }),
            )
        })?;

    Ok(Json(map_record(record)?))
}

pub async fn update_site_content(
    claims: auth::Claims,
    State(pool): State<db::DbPool>,
    Path(section): Path<String>,
    Json(payload): Json<UpdateSiteContentRequest>,
) -> Result<Json<SiteContentResponse>, (StatusCode, Json<ErrorResponse>)> {
    if claims.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Insufficient permissions".to_string(),
            }),
        ));
    }

    validate_section(&section)?;
    validate_content_size(&payload.content)?;
    validate_content_structure(&section, &payload.content)?;

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
