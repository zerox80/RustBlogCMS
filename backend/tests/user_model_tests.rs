use rust_blog_backend::models::user::{User, LoginRequest, LoginResponse, UserResponse};
use serde_json::json;

#[test]
fn test_user_serialization_skips_password() {
    let user = User {
        id: 1,
        username: "testuser".to_string(),
        password_hash: "secret_hash".to_string(),
        role: "admin".to_string(),
        created_at: "2023-01-01T00:00:00Z".to_string(),
    };

    let serialized = serde_json::to_string(&user).unwrap();
    assert!(!serialized.contains("password_hash"));
    assert!(!serialized.contains("secret_hash"));
    assert!(serialized.contains("\"username\":\"testuser\""));
}

#[test]
fn test_login_request_deserialization() {
    let data = json!({
        "username": "testuser",
        "password": "password123"
    });

    let request: LoginRequest = serde_json::from_value(data).unwrap();
    assert_eq!(request.username, "testuser");
    assert_eq!(request.password, "password123");
}

#[test]
fn test_user_response_serialization() {
    let response = UserResponse {
        username: "testuser".to_string(),
        role: "admin".to_string(),
    };

    let serialized = serde_json::to_string(&response).unwrap();
    assert_eq!(serialized, "{\"username\":\"testuser\",\"role\":\"admin\"}");
}

#[test]
fn test_login_response_serialization() {
    let response = LoginResponse {
        token: "fake_token".to_string(),
        user: UserResponse {
            username: "testuser".to_string(),
            role: "admin".to_string(),
        },
    };

    let serialized = serde_json::to_string(&response).unwrap();
    assert!(serialized.contains("\"token\":\"fake_token\""));
    assert!(serialized.contains("\"user\":{\"username\":\"testuser\",\"role\":\"admin\"}"));
}
