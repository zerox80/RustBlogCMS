use super::*;

#[test]
fn test_secret_entropy_validation() {
    // Too short (fails length check)
    assert!(!secret_has_min_entropy("Short1!"));

    // Only one character class (fails class count check)
    let low_entropy = concat!(
        "this_is_a_very_long_secret_but_only_contains_lowercase_and_underscores_",
        "which_is_not_enough_classes"
    );
    assert!(!secret_has_min_entropy(low_entropy));

    // Only two character classes (fails class count check)
    assert!(!secret_has_min_entropy(
        "ThisIsAVeryLongSecretWithUppercaseAndLowercaseButNoNumbersOrSpecialChars"
    ));

    // Too few unique characters (fails uniqueness check)
    assert!(!secret_has_min_entropy(
        "A1!A1!A1!A1!A1!A1!A1!A1!A1!A1!A1!A1!A1!A1!A1!A1!A1!A1!A1!A1!A1!A1!"
    ));

    // Valid high entropy secret (meets all requirements)
    assert!(secret_has_min_entropy(
        "p@ssW0rd_Extremely_Long_And_Secure_With_Many_Chars_123!"
    ));
}

#[test]
fn test_jwt_initialization_flow() {
    // Since JWT_SECRET is a global OnceLock, it might be initialized by other tests.
    // We just verify that if we attempt to initialize it, we either succeed or
    // get an "already initialized" error.
    env::set_var(
        "JWT_SECRET",
        "this_is_a_test_secret_with_adequate_entropy_123_ABC_!!!",
    );
    let result = init_jwt_secret();

    match result {
        Ok(_) => assert!(JWT_SECRET.get().is_some()),
        Err(e) => assert!(
            e.contains("already initialized") || e.contains("JWT_SECRET already initialized")
        ),
    }
}

#[test]
fn test_jwt_create_and_verify() {
    // Ensure secret is set
    if JWT_SECRET.get().is_none() {
        env::set_var(
            "JWT_SECRET",
            "this_is_another_test_secret_with_adequate_entropy_123_XYZ_!!!",
        );
        let _ = init_jwt_secret();
    }

    let username = "auth_test_user".to_string();
    let role = "admin".to_string();

    let token = create_jwt(username.clone(), role.clone()).expect("Failed to create JWT");
    let decoded = verify_jwt(&token).expect("Failed to verify JWT");

    assert_eq!(decoded.sub, username);
    assert_eq!(decoded.role, role);
}

#[test]
fn test_parse_bearer_token() {
    assert_eq!(
        parse_bearer_token("Bearer my_token"),
        Some("my_token".to_string())
    );
    assert_eq!(
        parse_bearer_token("bearer  my_token "),
        Some("my_token".to_string())
    );
    assert_eq!(parse_bearer_token("token_without_bearer"), None);
    assert_eq!(parse_bearer_token("Bearer "), None);
}

#[test]
fn test_build_auth_cookie() {
    let token = "test_jwt_cookie_token";
    let cookie = build_auth_cookie(token);

    assert_eq!(cookie.name(), AUTH_COOKIE_NAME);
    assert_eq!(cookie.value(), token);
    assert_eq!(cookie.path(), Some("/"));
    assert_eq!(cookie.http_only(), Some(true));
    assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    assert!(cookie.max_age().is_some());
}
