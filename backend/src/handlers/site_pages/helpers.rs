use super::*;

/// Maximum length for a page title (200 characters)
pub(super) const MAX_TITLE_LEN: usize = 200;
/// Maximum length for a page SEO description (1000 characters)
pub(super) const MAX_DESCRIPTION_LEN: usize = 1000;
/// Maximum length for a navigation label (100 characters)
pub(super) const MAX_NAV_LABEL_LEN: usize = 100;
/// Maximum allowed size for hero/layout JSON payloads (200KB)
pub(super) const MAX_JSON_BYTES: usize = 200_000;

/// Validates that a JSON value, when serialized, doesn't exceed the byte limit.
pub(super) fn validate_json_size(value: &Value, field: &str) -> Result<(), ApiError> {
    match serde_json::to_string(value) {
        // Within bounds
        Ok(serialized) if serialized.len() <= MAX_JSON_BYTES => Ok(()),
        // Over limit
        Ok(_) => Err(bad_request(format!(
            "{field} JSON exceeds maximum size of {MAX_JSON_BYTES} bytes"
        ))),
        // Invalid JSON content
        Err(err) => Err(bad_request(format!("Invalid {field} JSON: {err}"))),
    }
}

/// Normalizes and validates a payload for creating a new site page.
pub(super) fn sanitize_create_payload(
    mut payload: CreateSitePageRequest,
) -> Result<CreateSitePageRequest, ApiError> {
    // Slug normalization: trim and lowercase
    payload.slug = payload.slug.trim().to_lowercase();
    if payload.slug.is_empty() {
        return Err(bad_request("Slug cannot be empty"));
    }

    // Title normalization and length check
    payload.title = payload.title.trim().to_string();
    if payload.title.is_empty() {
        return Err(bad_request("Title cannot be empty"));
    }
    if payload.title.len() > MAX_TITLE_LEN {
        return Err(bad_request(format!(
            "Title too long (max {MAX_TITLE_LEN} characters)"
        )));
    }

    // Description length check
    payload.description = payload.description.map(|desc| desc.trim().to_string());
    if let Some(desc) = payload.description.as_ref() {
        if desc.len() > MAX_DESCRIPTION_LEN {
            return Err(bad_request(format!(
                "Description too long (max {MAX_DESCRIPTION_LEN} characters)"
            )));
        }
    }

    // Navigation label normalization
    payload.nav_label = payload.nav_label.and_then(|label| {
        let trimmed = label.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });
    if let Some(label) = payload.nav_label.as_ref() {
        if label.len() > MAX_NAV_LABEL_LEN {
            return Err(bad_request(format!(
                "Navigation label too long (max {MAX_NAV_LABEL_LEN} characters)"
            )));
        }
    }

    // Large JSON field size validation
    validate_json_size(&payload.hero, "hero")?;
    validate_json_size(&payload.layout, "layout")?;

    Ok(payload)
}

/// Normalizes and validates a payload for updating an existing site page.
pub(super) fn sanitize_update_payload(
    mut payload: UpdateSitePageRequest,
) -> Result<UpdateSitePageRequest, ApiError> {
    // Partial slug update
    if let Some(ref mut slug) = payload.slug {
        *slug = slug.trim().to_lowercase();
        if slug.is_empty() {
            return Err(bad_request("Slug cannot be empty"));
        }
    }

    // Partial title update
    if let Some(ref mut title) = payload.title {
        *title = title.trim().to_string();
        if title.is_empty() {
            return Err(bad_request("Title cannot be empty"));
        }
        if title.len() > MAX_TITLE_LEN {
            return Err(bad_request(format!(
                "Title too long (max {MAX_TITLE_LEN} characters)"
            )));
        }
    }

    // Partial description update
    if let Some(ref mut description) = payload.description {
        *description = description.trim().to_string();
        if description.len() > MAX_DESCRIPTION_LEN {
            return Err(bad_request(format!(
                "Description too long (max {MAX_DESCRIPTION_LEN} characters)"
            )));
        }
    }

    // Partial navigation label update
    if let Some(mut nav_label_option) = payload.nav_label.take() {
        nav_label_option = match nav_label_option {
            Some(label) => {
                let trimmed = label.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    if trimmed.len() > MAX_NAV_LABEL_LEN {
                        return Err(bad_request(format!(
                            "Navigation label too long (max {MAX_NAV_LABEL_LEN} characters)"
                        )));
                    }
                    Some(trimmed)
                }
            }
            None => None,
        };

        payload.nav_label = Some(nav_label_option);
    }

    // Partial JSON field update
    if let Some(ref hero) = payload.hero {
        validate_json_size(hero, "hero")?;
    }
    if let Some(ref layout) = payload.layout {
        validate_json_size(layout, "layout")?;
    }

    Ok(payload)
}

/// Maps a database SitePage record to a rich response model, including JSON parsing.
pub(super) fn map_page(page: crate::models::SitePage) -> Result<SitePageResponse, ApiError> {
    let crate::models::SitePage {
        id,
        slug,
        title,
        description,
        nav_label,
        show_in_nav,
        order_index,
        is_published,
        hero_json,
        layout_json,
        created_at,
        updated_at,
    } = page;

    // Parse hero JSON string from database into a serde_json::Value
    let hero = serde_json::from_str::<Value>(&hero_json)
        .map_err(internal_error("Failed to parse stored hero JSON"))?;

    // Parse layout JSON string from database into a serde_json::Value
    let layout = serde_json::from_str::<Value>(&layout_json)
        .map_err(internal_error("Failed to parse stored layout JSON"))?;

    // Normalize slug for output
    let sanitized_slug = slug.trim().to_lowercase();

    // Default title to slug if the title field is empty
    let sanitized_title = match title.trim() {
        "" => sanitized_slug.clone(),
        value => value.to_string(),
    };

    // Trim description
    let sanitized_description = description.trim().to_string();

    // Clean up navigation label
    let sanitized_nav_label = nav_label.and_then(|label| {
        let trimmed = label.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });

    // Assemble response
    Ok(SitePageResponse {
        id,
        slug: sanitized_slug,
        title: sanitized_title,
        description: sanitized_description,
        nav_label: sanitized_nav_label,
        show_in_nav,
        order_index,
        is_published,
        hero,
        layout,
        created_at,
        updated_at,
    })
}

/// Maps a database SitePost record to a public response model.
pub(super) fn map_post(post: crate::models::SitePost) -> SitePostResponse {
    SitePostResponse {
        id: post.id,
        page_id: post.page_id,
        title: post.title,
        slug: post.slug,
        excerpt: post.excerpt,
        content_markdown: post.content_markdown,
        is_published: post.is_published,
        published_at: post.published_at,
        order_index: post.order_index,
        created_at: post.created_at,
        updated_at: post.updated_at,
        allow_comments: post.allow_comments,
    }
}
