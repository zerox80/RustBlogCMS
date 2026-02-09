use rust_blog_backend::models::site::{SiteContent, SiteContentResponse, UpdateSiteContentRequest};
use serde_json::json;

#[test]
fn test_site_content_serialization() {
    let content = SiteContent {
        section: "hero".to_string(),
        content_json: "{\"title\":\"Hello\"}".to_string(),
        updated_at: "now".to_string(),
    };

    let serialized = serde_json::to_string(&content).unwrap();
    assert!(serialized.contains("\"section\":\"hero\""));
}

#[test]
fn test_site_content_response() {
    let res = SiteContentResponse {
        section: "footer".to_string(),
        content: json!({"link": "home"}),
        updated_at: "today".to_string(),
    };

    let serialized = serde_json::to_string(&res).unwrap();
    assert!(serialized.contains("\"section\":\"footer\""));
    assert!(serialized.contains("\"content\":{\"link\":\"home\"}"));
}

#[test]
fn test_update_site_content_request() {
    let data = json!({
        "content": {"new": "data"}
    });

    let req: UpdateSiteContentRequest = serde_json::from_value(data).unwrap();
    assert_eq!(req.content, json!({"new": "data"}));
}
