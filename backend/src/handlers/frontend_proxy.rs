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
use reqwest::Client;
use std::env;

/// Internal URL for the frontend service in the container network
const DEFAULT_FRONTEND_URL: &str = "http://frontend";

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
    let client = Client::new();
    let html_content = match client.get(&index_url).send().await {
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

    // SECURITY: Thoroughly escape database-sourced text to prevent XSS via meta tags
    let safe_title = html_escape::encode_text(&title);
    let safe_description = html_escape::encode_text(&description);

    // Replace Title using Regex
    let title_regex = regex::Regex::new(r"<title>.*?</title>")
        .unwrap_or_else(|_| regex::Regex::new("").unwrap()); // Fallback (should not happen with valid regex)
    
    injected_html = title_regex
        .replace(&injected_html, format!("<title>{}</title>", safe_title))
        .to_string();

    // Replace Meta Description using Regex
    // Matches <meta name="description" content="..."> allowing for whitespace variations
    let desc_regex = regex::Regex::new(r#"(?i)<meta\s+name=["']description["']\s+content=["'].*?["']\s*/?>"#)
        .unwrap_or_else(|_| regex::Regex::new("").unwrap());

    injected_html = desc_regex
        .replace(
            &injected_html, 
            format!(r#"<meta name="description" content="{}">"#, safe_description)
        )
        .to_string();

    // Replace OG Title
    // Matches <meta property="og:title" content="...">
    let og_title_regex = regex::Regex::new(r#"(?i)<meta\s+property=["']og:title["']\s+content=["'].*?["']\s*/?>"#)
        .unwrap_or_else(|_| regex::Regex::new("").unwrap());

    injected_html = og_title_regex
        .replace(
            &injected_html,
            format!(r#"<meta property="og:title" content="{}">"#, safe_title)
        )
        .to_string();

    // Replace OG Description
    // Matches <meta property="og:description" content="...">
    let og_desc_regex = regex::Regex::new(r#"(?i)<meta\s+property=["']og:description["']\s+content=["'].*?["']\s*/?>"#)
        .unwrap_or_else(|_| regex::Regex::new("").unwrap());

    injected_html = og_desc_regex
        .replace(
            &injected_html,
            format!(r#"<meta property="og:description" content="{}">"#, safe_description)
        )
        .to_string();

    Html(injected_html).into_response()
}
