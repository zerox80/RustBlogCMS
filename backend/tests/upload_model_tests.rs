use rust_blog_backend::models::tutorial::UploadResponse;
use serde_json::json;

#[test]
fn test_upload_response_serialization() {
    let res = UploadResponse {
        url: "http://example.com/file.png".to_string(),
    };

    let serialized = serde_json::to_string(&res).unwrap();
    assert!(serialized.contains("\"url\":\"http://example.com/file.png\""));
}

#[test]
fn test_upload_response_deserialization() {
    let data = json!({
        "url": "/uploads/test.jpg"
    });

    let res: UploadResponse = serde_json::from_value(data).unwrap();
    assert_eq!(res.url, "/uploads/test.jpg");
}
