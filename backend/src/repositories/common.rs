use regex::Regex;
use serde_json::Value;
use std::sync::OnceLock;

/// Returns the compiled slug validation regex pattern.
fn slug_regex() -> &'static Regex {
    static SLUG_RE: OnceLock<Regex> = OnceLock::new();
    SLUG_RE.get_or_init(|| Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").expect("valid slug regex"))
}

/// Validates a slug for use in URLs.
/// 
/// Enforces:
/// - Maximum length of 100 characters.
/// - Lowercase alphanumeric characters.
/// - Single hyphens as separators (no leading/trailing hyphens).
pub fn validate_slug(slug: &str) -> Result<(), sqlx::Error> {
    const MAX_SLUG_LENGTH: usize = 100;

    if slug.len() > MAX_SLUG_LENGTH {
        return Err(sqlx::Error::Protocol(
            format!("Invalid slug. Maximum length is {MAX_SLUG_LENGTH} characters: '{slug}'")
                .into(),
        ));
    }

    if slug_regex().is_match(slug) {
        Ok(())
    } else {
        Err(sqlx::Error::Protocol(
            format!("Invalid slug. Only lowercase letters, numbers and single hyphens allowed: '{slug}'")
                .into(),
        ))
    }
}

pub fn serialize_json_value(value: &Value) -> Result<String, sqlx::Error> {
    serde_json::to_string(value)
        .map_err(|e| sqlx::Error::Protocol(format!("Failed to serialize JSON: {e}").into()))
}

pub fn deserialize_json_value(value: &str) -> Result<Value, sqlx::Error> {
    serde_json::from_str(value)
        .map_err(|e| sqlx::Error::Protocol(format!("Failed to deserialize JSON: {e}").into()))
}
