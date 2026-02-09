use rust_blog_backend::security::auth::init_jwt_secret;
use std::env;

#[test]
fn test_jwt_secret_validation_rules() {
    // Too short
    env::set_var("JWT_SECRET", "too_short_123!");
    let result = init_jwt_secret();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("high-entropy"));

    // Known blacklist
    env::set_var("JWT_SECRET", "CHANGE_ME_OR_APP_WILL_FAIL");
    let result = init_jwt_secret();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("placeholder"));
}
