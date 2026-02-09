use rust_blog_backend::security::auth::{
    build_auth_cookie, build_cookie_removal, AUTH_COOKIE_NAME,
};
use rust_blog_backend::security::csrf::csrf_cookie_name;

#[test]
fn test_auth_cookie_properties() {
    let token = "test_token";
    let cookie = build_auth_cookie(token);

    assert_eq!(cookie.name(), AUTH_COOKIE_NAME);
    assert_eq!(cookie.value(), token);
    assert_eq!(cookie.path(), Some("/"));
    assert_eq!(cookie.http_only(), Some(true));
}

#[test]
fn test_auth_cookie_removal_properties() {
    let cookie = build_cookie_removal();

    assert_eq!(cookie.name(), AUTH_COOKIE_NAME);
    assert_eq!(cookie.value(), "");
    assert!(cookie.expires().is_some());
}

#[test]
fn test_csrf_cookie_name_const() {
    assert_eq!(csrf_cookie_name(), "ltcms_csrf");
}
