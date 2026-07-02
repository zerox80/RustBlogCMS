//! Frontend Proxy and SEO Meta Injection
//!
//! This module acts as a bridge between the Rust backend and the static frontend.
//! Instead of serving files directly, it proxies requests to the frontend service
//! and dynamically injects SEO metadata (title, description) from the database
//! into the HTML response. This ensures search engines and social media crawlers
//! see relevant page information even for a Single Page Application (SPA).

use crate::db;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use regex::Regex;
use reqwest::Client;
use std::env;
use std::sync::LazyLock;
use std::time::Duration;

/// Internal URL for the frontend service in the container network
const DEFAULT_FRONTEND_URL: &str = "http://frontend";

/// Upper bound on how long we wait for the frontend service to respond.
/// Without this, a hung frontend upstream ties up backend request handlers
/// indefinitely on every page load (reqwest has no timeout by default).
const FRONTEND_FETCH_TIMEOUT: Duration = Duration::from_secs(5);

/// Shared HTTP client for proxying requests to the frontend service.
/// Reused across requests to benefit from connection pooling.
static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(FRONTEND_FETCH_TIMEOUT)
        .build()
        .expect("failed to build frontend proxy HTTP client")
});

/// Matches the <title> tag for replacement with database-driven values.
static TITLE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<title>.*?</title>").expect("valid title regex"));

/// Matches <meta name="description" content="..."> allowing whitespace variations.
static DESC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)<meta\s+name=["']description["']\s+content=["'].*?["']\s*/?>"#)
        .expect("valid description regex")
});

/// Matches <meta property="og:title" content="...">.
static OG_TITLE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)<meta\s+property=["']og:title["']\s+content=["'].*?["']\s*/?>"#)
        .expect("valid og:title regex")
});

/// Matches <meta property="og:description" content="...">.
static OG_DESC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)<meta\s+property=["']og:description["']\s+content=["'].*?["']\s*/?>"#)
        .expect("valid og:description regex")
});

/// Core handler to serve the application entry point (index.html).
///
/// This function:
/// 1. Proxies the raw index.html from the frontend service.
/// 2. Fetches global site metadata (site_meta section) from the database.
/// 3. Performs string-based injection of <title> and <meta> tags.
/// 4. Provides fallback defaults if database records are missing.
pub async fn serve_index(State(pool): State<db::DbPool>) -> impl IntoResponse {
    let frontend_url =
        env::var("FRONTEND_URL").unwrap_or_else(|_| DEFAULT_FRONTEND_URL.to_string());
    let index_url = format!("{}/index.html", frontend_url);

    // Proxied Fetch: Retrieve the template from the frontend service
    let html_content = match HTTP_CLIENT.get(&index_url).send().await {
        Ok(resp) => match resp.text().await {
            Ok(text) => text,
            Err(e) => {
                tracing::error!("Failed to read index.html body: {}", e);
                return Html(
                    "<h1>Internal Server Error</h1><p>Failed to load application.</p>".to_string(),
                )
                .into_response();
            }
        },
        Err(e) => {
            tracing::error!("Failed to fetch index.html from {}: {}", index_url, e);
            return Html(
                "<h1>Internal Server Error</h1><p>Failed to connect to frontend service.</p>"
                    .to_string(),
            )
            .into_response();
        }
    };

    // Metadata Retrieval: Fetch SEO config from database section 'site_meta'
    let site_meta =
        match crate::repositories::content::fetch_site_content_by_section(&pool, "site_meta").await
        {
            Ok(Some(record)) => {
                match serde_json::from_str::<serde_json::Value>(&record.content_json) {
                    Ok(json) => json,
                    Err(_) => serde_json::json!({}),
                }
            }
            _ => serde_json::json!({}),
        };

    // Extract title from JSON, providing a sensible fallback
    let title = site_meta
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Linux Tutorial - Lerne Linux Schritt für Schritt");

    // Extract description from JSON, providing a sensible fallback
    let description = site_meta
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("Lerne Linux von Grund auf - Interaktiv, modern und praxisnah.");

    // Injection Phase:
    // We use simple string replacement to swap hardcoded defaults in the build
    // for dynamic database-driven values.
    let mut injected_html = html_content;

    // SECURITY: Escape database-sourced text for the context it lands in.
    // `encode_text` escapes `<`, `>`, and `&` but NOT `"`, so it is only
    // safe inside element text content (e.g. <title>...</title>). Values
    // interpolated inside a `content="..."` attribute must additionally
    // escape double quotes, or a stored value containing `"` could break
    // out of the attribute and inject new attributes (e.g. event handlers).
    let safe_title_text = html_escape::encode_text(&title);
    let safe_title_attr = html_escape::encode_double_quoted_attribute(&title);
    let safe_description_attr = html_escape::encode_double_quoted_attribute(&description);

    // Replace Title (text content)
    injected_html = TITLE_REGEX
        .replace(
            &injected_html,
            format!("<title>{}</title>", safe_title_text),
        )
        .to_string();

    // Replace Meta Description (attribute content)
    injected_html = DESC_REGEX
        .replace(
            &injected_html,
            format!(
                r#"<meta name="description" content="{}">"#,
                safe_description_attr
            ),
        )
        .to_string();

    // Replace OG Title (attribute content)
    injected_html = OG_TITLE_REGEX
        .replace(
            &injected_html,
            format!(
                r#"<meta property="og:title" content="{}">"#,
                safe_title_attr
            ),
        )
        .to_string();

    // Replace OG Description (attribute content)
    injected_html = OG_DESC_REGEX
        .replace(
            &injected_html,
            format!(
                r#"<meta property="og:description" content="{}">"#,
                safe_description_attr
            ),
        )
        .to_string();

    Html(injected_html).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression test for the 502 panic fixed by enabling the `unicode-perl`
    /// regex feature: these statics use `\s`, which requires that feature to
    /// compile. Without it, `LazyLock` initialization panics (and poisons the
    /// lock) on first access instead of at `cargo build` time, so this test
    /// exercises them directly rather than relying on `serve_index`.
    #[test]
    fn seo_meta_regexes_compile_and_match() {
        let html = r#"<html><head><title>Old Title</title>
<meta name="description" content="Old description">
<meta property="og:title" content="Old OG Title">
<meta property="og:description" content="Old OG description">
</head></html>"#;

        let replaced = TITLE_REGEX
            .replace(html, "<title>New Title</title>")
            .to_string();
        assert!(replaced.contains("<title>New Title</title>"));

        let replaced = DESC_REGEX
            .replace(
                &replaced,
                r#"<meta name="description" content="New description">"#,
            )
            .to_string();
        assert!(replaced.contains(r#"content="New description""#));

        let replaced = OG_TITLE_REGEX
            .replace(
                &replaced,
                r#"<meta property="og:title" content="New OG Title">"#,
            )
            .to_string();
        assert!(replaced.contains(r#"content="New OG Title""#));

        let replaced = OG_DESC_REGEX
            .replace(
                &replaced,
                r#"<meta property="og:description" content="New OG description">"#,
            )
            .to_string();
        assert!(replaced.contains(r#"content="New OG description""#));
    }

    /// Regression test: `encode_text` (used for the old single escaping
    /// pass) strips `<`/`>`/`&` but leaves `"` untouched. A site_meta value
    /// containing a double quote would previously break out of the
    /// `content="..."` attribute and inject arbitrary attributes/markup.
    /// Attribute-context values must escape `"` as well.
    #[test]
    fn attribute_context_escaping_neutralizes_double_quotes() {
        let malicious = r#"Title" onmouseover="alert(1)"#;

        let text_escaped = html_escape::encode_text(malicious);
        // encode_text alone is NOT sufficient for attribute context.
        assert!(text_escaped.contains('"'));

        let attr_escaped = html_escape::encode_double_quoted_attribute(malicious);
        assert!(
            !attr_escaped.contains('"'),
            "attribute-context escaping must neutralize double quotes: {attr_escaped}"
        );

        let injected = format!(r#"<meta property="og:title" content="{}">"#, attr_escaped);
        // The payload must not be able to close the content attribute early.
        assert!(!injected.contains(r#"content="Title" onmouseover"#));
    }
}
