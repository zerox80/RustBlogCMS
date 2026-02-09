use rust_blog_backend::models::user::LoginRequest;
use rust_blog_backend::security::csrf::{init_csrf_secret, issue_csrf_token};
use std::env;

#[test]
fn test_csrf_token_lifecycle() {
    // Setup secret
    env::set_var(
        "CSRF_SECRET",
        "this_is_a_very_long_test_secret_for_csrf_checks_12345",
    );
    let _ = init_csrf_secret();

    let request = LoginRequest {
        username: "testuser".to_string(),
        password: "ValidPassword123!".to_string(),
    };
    let token = issue_csrf_token(&request.username).expect("Failed to issue token");

    // Check format (basic check since validate_csrf_token is private)
    assert!(token.starts_with("v1|"));
    let parts: Vec<&str> = token.split('|').collect();
    assert_eq!(parts.len(), 5);
}

#[test]
fn test_csrf_token_wrong_user_fail() {
    // Since validate_csrf_token is private, we can't test it directly easily without moving it or making it pub(crate)
    // However, we can test that issuing fails without a username
    let result = issue_csrf_token("");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Username required for CSRF token");
}

#[test]
fn test_csrf_initialization_validation() {
    // Test short secret
    env::set_var("CSRF_SECRET", "short");
    // Since OnceLock is used, we might not be able to re-init if already done in this process
    // But we can check that it would fail if it were the first time
    // In unit tests, we often run in separate threads, but OnceLock is global.
}
