use rust_blog_backend::security::auth::{create_jwt, init_jwt_secret, verify_jwt, Claims};
use std::env;

#[test]
fn test_jwt_lifecycle() {
    // Setup secret
    env::set_var(
        "JWT_SECRET",
        "this_is_a_very_long_test_secret_for_jwt_checks_12345_ABC_abc_!!!",
    );
    let _ = init_jwt_secret();

    let username = "testuser".to_string();
    let role = "admin".to_string();

    let token = create_jwt(username.clone(), role.clone()).expect("Failed to create token");
    let decoded = verify_jwt(&token).expect("Failed to verify token");

    assert_eq!(decoded.sub, username);
    assert_eq!(decoded.role, role);
}

#[test]
fn test_claims_new() {
    let username = "test".to_string();
    let role = "user".to_string();
    let claims = Claims::new(username.clone(), role.clone());

    assert_eq!(claims.sub, username);
    assert_eq!(claims.role, role);
    // Expiration should be roughly 24 hours from now
}
