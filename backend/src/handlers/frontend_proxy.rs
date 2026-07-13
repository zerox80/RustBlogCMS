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
    http::StatusCode,
    response::{Html, IntoResponse},
};
use regex::{NoExpand, Regex};
use reqwest::Client;
use std::borrow::Cow;
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

/// Replaces the first regex match with `replacement`, treating the
/// replacement as a literal string.
///
/// Two deliberate properties:
/// - `NoExpand` is required for correctness, not style: the replacement
///   contains database-sourced text, and `Regex::replace` would otherwise
///   interpret `$0`/`$name` inside it as capture-group references — a title
///   containing `$0` would expand to the matched original tag.
/// - String replacement over HTML is inherently fragile against template
///   changes, so a non-match (e.g. the frontend build renamed or dropped the
///   target tag) is logged loudly instead of failing silently and shipping
///   stale SEO metadata.
fn inject_or_warn(regex: &Regex, html: String, replacement: &str, target: &str) -> String {
    match regex.replace(&html, NoExpand(replacement)) {
        Cow::Owned(replaced) => replaced,
        Cow::Borrowed(_) => {
            tracing::warn!(
                target_tag = target,
                "SEO injection target not found in index.html; serving original markup"
            );
            html
        }
    }
}

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

    // Proxied Fetch: Retrieve the template from the frontend service.
    // Upstream failures must surface as 502, not 200: crawlers, caches, and
    // health checks would otherwise treat the error page as a valid document.
    let html_content = match HTTP_CLIENT.get(&index_url).send().await {
        Ok(resp) => match resp.text().await {
            Ok(text) => text,
            Err(e) => {
                tracing::error!("Failed to read index.html body: {}", e);
                return (
                    StatusCode::BAD_GATEWAY,
                    Html("<h1>Bad Gateway</h1><p>Failed to load application.</p>".to_string()),
                )
                    .into_response();
            }
        },
        Err(e) => {
            tracing::error!("Failed to fetch index.html from {}: {}", index_url, e);
            return (
                StatusCode::BAD_GATEWAY,
                Html(
                    "<h1>Bad Gateway</h1><p>Failed to connect to frontend service.</p>".to_string(),
                ),
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
    let stored_title = site_meta
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Zero Point – Persönlicher Blog");
    let is_starter_content =
        stored_title.starts_with("Linux Tutorial") || stored_title.starts_with("IT Wissensportal");
    let title = if is_starter_content {
        "Zero Point – Persönlicher Blog"
    } else {
        stored_title
    };

    // Extract description from JSON, providing a sensible fallback
    let stored_description = site_meta
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("Persönliche Notizen über Technik, Projekte, Ideen und alles dazwischen.");
    let description = if is_starter_content {
        "Persönliche Notizen über Technik, Projekte, Ideen und alles dazwischen."
    } else {
        stored_description
    };

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
    injected_html = inject_or_warn(
        &TITLE_REGEX,
        injected_html,
        &format!("<title>{}</title>", safe_title_text),
        "<title>",
    );

    // Replace Meta Description (attribute content)
    injected_html = inject_or_warn(
        &DESC_REGEX,
        injected_html,
        &format!(
            r#"<meta name="description" content="{}">"#,
            safe_description_attr
        ),
        "meta description",
    );

    // Replace OG Title (attribute content)
    injected_html = inject_or_warn(
        &OG_TITLE_REGEX,
        injected_html,
        &format!(
            r#"<meta property="og:title" content="{}">"#,
            safe_title_attr
        ),
        "og:title",
    );

    // Replace OG Description (attribute content)
    injected_html = inject_or_warn(
        &OG_DESC_REGEX,
        injected_html,
        &format!(
            r#"<meta property="og:description" content="{}">"#,
            safe_description_attr
        ),
        "og:description",
    );

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

    /// Regression test: replacement strings are database-sourced, so `$`
    /// must be treated literally. Without `NoExpand`, a title like
    /// "Save 50% - only $0.99" would expand `$0` to the entire matched
    /// original tag, corrupting the output HTML.
    #[test]
    fn injection_treats_dollar_signs_in_replacement_literally() {
        let html = "<html><head><title>Old Title</title></head></html>".to_string();
        let title_with_dollar = "Deal: $0.99";

        let replaced = inject_or_warn(
            &TITLE_REGEX,
            html,
            &format!("<title>{}</title>", title_with_dollar),
            "<title>",
        );

        assert!(
            replaced.contains("<title>Deal: $0.99</title>"),
            "dollar sign must survive literally, got: {replaced}"
        );
        assert!(!replaced.contains("Old Title"));
    }

    /// A template without the target tag must be returned unchanged (and
    /// warn) rather than silently corrupted or panicking.
    #[test]
    fn injection_leaves_html_unchanged_when_target_missing() {
        let html = "<html><head></head></html>".to_string();

        let replaced = inject_or_warn(&TITLE_REGEX, html.clone(), "<title>New</title>", "<title>");

        assert_eq!(replaced, html);
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
