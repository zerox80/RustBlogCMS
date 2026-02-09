use rust_blog_backend::models::site::{NavigationItemResponse, NavigationResponse};
use serde_json::json;

#[test]
fn test_navigation_response_serialization() {
    let res = NavigationResponse {
        items: vec![
            NavigationItemResponse {
                id: "1".to_string(),
                slug: "home".to_string(),
                label: "Home".to_string(),
                order_index: 0,
            },
            NavigationItemResponse {
                id: "2".to_string(),
                slug: "blog".to_string(),
                label: "Blog".to_string(),
                order_index: 1,
            },
        ],
    };

    let serialized = serde_json::to_string(&res).unwrap();
    assert!(serialized.contains("\"slug\":\"home\""));
    assert!(serialized.contains("\"label\":\"Blog\""));
}

#[test]
fn test_navigation_item_deserialization() {
    let data = json!({
        "id": "3",
        "slug": "contact",
        "label": "Contact Us",
        "order_index": 2
    });

    let item: NavigationItemResponse = serde_json::from_value(data).unwrap();
    assert_eq!(item.slug, "contact");
    assert_eq!(item.order_index, 2);
}
