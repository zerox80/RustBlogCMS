use rust_blog_backend::models::site::{SitePage, SitePost, CreateSitePostRequest};
use serde_json::json;

#[test]
fn test_site_post_defaults() {
    let data = json!({
        "title": "Test Post",
        "slug": "test-post",
        "content_markdown": "Hello world"
    });

    let request: CreateSitePostRequest = serde_json::from_value(data).unwrap();
    assert_eq!(request.allow_comments, true); // Default should be true
    assert_eq!(request.is_published, false); // Default should be false
}

#[test]
fn test_site_post_serialization() {
    let post = SitePost {
        id: "abc-123".to_string(),
        page_id: "page-1".to_string(),
        title: "Title".to_string(),
        slug: "slug".to_string(),
        excerpt: "Excerpt".to_string(),
        content_markdown: "Markdown".to_string(),
        is_published: true,
        allow_comments: true,
        published_at: Some("2023-01-01".to_string()),
        order_index: 0,
        created_at: "created".to_string(),
        updated_at: "updated".to_string(),
    };

    let serialized = serde_json::to_string(&post).unwrap();
    assert!(serialized.contains("\"id\":\"abc-123\""));
}

#[test]
fn test_site_page_serialization_with_json_fields() {
    let page = SitePage {
        id: "p1".to_string(),
        slug: "s1".to_string(),
        title: "T1".to_string(),
        description: "D1".to_string(),
        nav_label: None,
        show_in_nav: true,
        order_index: 1,
        is_published: true,
        hero_json: "{\"title\":\"Hero\"}".to_string(),
        layout_json: "[]".to_string(),
        created_at: "c".to_string(),
        updated_at: "u".to_string(),
    };

    let serialized = serde_json::to_string(&page).unwrap();
    assert!(serialized.contains("\"hero_json\":\"{\\\"title\\\":\\\"Hero\\\"}\""));
}
