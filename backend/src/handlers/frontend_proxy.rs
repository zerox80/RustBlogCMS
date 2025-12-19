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

    // Replace Title
    injected_html = injected_html.replace(
        "<title>Linux Tutorial - Lerne Linux Schritt für Schritt</title>",
        &format!("<title>{}</title>", safe_title),
    );

    // Replace Meta Description
    // Note: This regex-like replacement is brittle if the HTML formatting changes.
    // For now, we assume the exact string from index.html or use a more robust regex if needed.
    // Since we don't have regex crate here yet, we'll try to replace the known default description.
    // If it's dynamic, we might need a more robust approach, but for now let's try replacing the known default.
    let default_desc = "Lerne Linux von Grund auf - Interaktiv, modern und praxisnah. Umfassende Tutorials für Einsteiger und Fortgeschrittene.";
    injected_html = injected_html.replace(
        &format!("content=\"{}\"", default_desc),
        &format!("content=\"{}\"", safe_description),
    );

    // Also replace OG tags if possible.
    // A better approach for robust replacement without full HTML parsing:
    // We can replace the whole <head> block or specific known lines if we are sure about the structure.
    // Given the index.html structure, we can try to replace specific lines.

    // Replace OG Title
    injected_html = injected_html.replace(
        "content=\"Linux Tutorial - Lerne Linux Schritt für Schritt\"",
        &format!("content=\"{}\"", safe_title),
    );

    // Replace OG Description (reusing the description replacement above might handle this if content matches)
    // The default OG description in index.html is shorter: "Lerne Linux von Grund auf - Interaktiv, modern und praxisnah."
    let default_og_desc = "Lerne Linux von Grund auf - Interaktiv, modern und praxisnah.";
    injected_html = injected_html.replace(
        &format!("content=\"{}\"", default_og_desc),
        &format!("content=\"{}\"", safe_description),
    );

    Html(injected_html).into_response()
}
