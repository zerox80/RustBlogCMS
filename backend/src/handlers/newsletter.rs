use crate::{
    db::DbPool,
    models::{bad_request, internal_error, ApiError},
    repositories,
};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct NewsletterSubscriptionRequest {
    email: String,
}

#[derive(Debug, Serialize)]
pub struct NewsletterSubscriptionResponse {
    subscribed: bool,
}

/// Registers an email address. Repeated requests are intentionally idempotent.
pub async fn subscribe_to_newsletter(
    State(pool): State<DbPool>,
    Json(payload): Json<NewsletterSubscriptionRequest>,
) -> Result<Json<NewsletterSubscriptionResponse>, ApiError> {
    let email = validate_and_normalize_email(&payload.email).map_err(bad_request)?;

    repositories::newsletter::subscribe(&pool, &email)
        .await
        .map_err(internal_error("Newsletter-Anmeldung fehlgeschlagen"))?;

    Ok(Json(NewsletterSubscriptionResponse { subscribed: true }))
}

fn validate_and_normalize_email(value: &str) -> Result<String, &'static str> {
    const INVALID_EMAIL: &str = "Ungültige E-Mail-Adresse";
    let email = value.trim();
    if email.is_empty() || email.len() > 254 || !email.is_ascii() {
        return Err(INVALID_EMAIL);
    }

    let Some((local, domain)) = email.split_once('@') else {
        return Err(INVALID_EMAIL);
    };
    if local.is_empty()
        || local.len() > 64
        || local.starts_with('.')
        || local.ends_with('.')
        || local.contains("..")
        || domain.is_empty()
        || domain.contains('@')
        || !domain.contains('.')
    {
        return Err(INVALID_EMAIL);
    }

    let valid_local = local
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || b".!#$%&'*+-/=?^_`{|}~".contains(&byte));
    let valid_domain = domain.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    });

    if !valid_local || !valid_domain {
        return Err(INVALID_EMAIL);
    }

    Ok(email.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::validate_and_normalize_email;

    #[test]
    fn normalizes_valid_email_addresses() {
        assert_eq!(
            validate_and_normalize_email("  Reader+Blog@Example.COM "),
            Ok("reader+blog@example.com".to_string())
        );
    }

    #[test]
    fn rejects_malformed_email_addresses() {
        for email in [
            "",
            "missing-at.example.com",
            "two@@example.com",
            ".reader@example.com",
            "reader@example",
            "reader@-example.com",
        ] {
            assert!(validate_and_normalize_email(email).is_err(), "{email}");
        }
    }
}
