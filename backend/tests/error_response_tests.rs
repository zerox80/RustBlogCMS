use rust_blog_backend::models::tutorial::ErrorResponse;
use serde_json::json;

#[test]
fn test_error_response_serialization() {
    let err = ErrorResponse {
        error: "Not Found".to_string(),
    };

    let serialized = serde_json::to_string(&err).unwrap();
    assert_eq!(serialized, "{\"error\":\"Not Found\"}");
}

#[test]
fn test_error_response_deserialization() {
    let data = json!({
        "error": "Access Denied"
    });

    let err: ErrorResponse = serde_json::from_value(data).unwrap();
    assert_eq!(err.error, "Access Denied");
}
