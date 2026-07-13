use super::*;

#[test]
fn test_csrf_secret_initialization() {
    // Ensure secret is initialized for tests
    if CSRF_SECRET.get().is_none() {
        env::set_var(
            "CSRF_SECRET",
            "this_is_a_very_long_secret_key_for_testing_purposes_only_at_least_32_bytes",
        );
        let _ = init_csrf_secret();
    }

    assert!(CSRF_SECRET.get().is_some());
}

#[test]
fn test_issue_csrf_token() {
    if CSRF_SECRET.get().is_none() {
        env::set_var(
            "CSRF_SECRET",
            "this_is_a_very_long_secret_key_for_testing_purposes_only_at_least_32_bytes",
        );
        let _ = init_csrf_secret();
    }

    let username = "testuser";
    let token = issue_csrf_token(username).expect("Failed to issue token");

    let parts: Vec<&str> = token.split('|').collect();
    // v1|base64(username)|expiry|nonce|signature
    assert_eq!(parts.len(), 5);
    assert_eq!(parts[0], "v1");

    // Verify username part
    let decoded_username = Base64UrlUnpadded::decode_vec(parts[1]).unwrap();
    assert_eq!(String::from_utf8(decoded_username).unwrap(), username);
}

#[test]
fn test_validate_csrf_token_valid() {
    if CSRF_SECRET.get().is_none() {
        env::set_var(
            "CSRF_SECRET",
            "this_is_a_very_long_secret_key_for_testing_purposes_only_at_least_32_bytes",
        );
        let _ = init_csrf_secret();
    }

    let username = "valid_user";
    let token = issue_csrf_token(username).unwrap();

    assert!(validate_csrf_token(&token, username).is_ok());
}

#[test]
fn test_validate_csrf_token_wrong_user() {
    if CSRF_SECRET.get().is_none() {
        env::set_var(
            "CSRF_SECRET",
            "this_is_a_very_long_secret_key_for_testing_purposes_only_at_least_32_bytes",
        );
        let _ = init_csrf_secret();
    }

    let token = issue_csrf_token("user_a").unwrap();
    let result = validate_csrf_token(&token, "user_b");

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "CSRF token not issued for this account"
    );
}

#[test]
fn test_validate_csrf_token_tampered() {
    if CSRF_SECRET.get().is_none() {
        env::set_var(
            "CSRF_SECRET",
            "this_is_a_very_long_secret_key_for_testing_purposes_only_at_least_32_bytes",
        );
        let _ = init_csrf_secret();
    }

    let token = issue_csrf_token("test_tamper").unwrap();
    let mut parts: Vec<&str> = token.split('|').collect();

    // Tamper with the nonce (part 3)
    let mut tampered_token = String::new();
    parts[3] = "tampered_nonce_value";

    // Reconstruct manually
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            tampered_token.push('|');
        }
        tampered_token.push_str(part);
    }

    let result = validate_csrf_token(&tampered_token, "test_tamper");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "CSRF signature mismatch");
}

#[test]
fn test_validate_csrf_token_expired() {
    if CSRF_SECRET.get().is_none() {
        env::set_var(
            "CSRF_SECRET",
            "this_is_a_very_long_secret_key_for_testing_purposes_only_at_least_32_bytes",
        );
        let _ = init_csrf_secret();
    }

    // Manually construct an expired token
    let username = "expired_user";
    let username_b64 = Base64UrlUnpadded::encode_string(username.as_bytes());
    // 1 second in the past
    let expiry = Utc::now().timestamp() - 1;
    let nonce = Uuid::new_v4().to_string();

    let payload = format!("{username_b64}|{expiry}|{nonce}");
    let versioned_payload = format!("{CSRF_VERSION}|{payload}");

    let mut mac = HmacSha256::new_from_slice(get_secret()).unwrap();
    mac.update(versioned_payload.as_bytes());
    let signature = Base64UrlUnpadded::encode_string(&mac.finalize().into_bytes());

    let token = format!("{versioned_payload}|{signature}");

    let result = validate_csrf_token(&token, username);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "CSRF token expired");
}

#[test]
fn browser_origin_check_allows_requests_without_origin_or_referer() {
    // Non-browser clients (curl, tests) send neither header and carry no
    // ambient browser credentials -- must not be blocked.
    let headers = HeaderMap::new();
    assert!(validate_browser_origin(&headers).is_ok());
}

#[test]
fn browser_origin_check_allows_same_host_origin() {
    let mut headers = HeaderMap::new();
    headers.insert("host", HeaderValue::from_static("blog.example.com"));
    headers.insert(
        "origin",
        HeaderValue::from_static("https://blog.example.com"),
    );
    assert!(validate_browser_origin(&headers).is_ok());
}

#[test]
fn browser_origin_check_allows_same_host_with_differing_port() {
    // Hostname comparison only: an attacker cannot serve content under
    // the victim's hostname, so the port is not part of the trust
    // boundary here.
    let mut headers = HeaderMap::new();
    headers.insert("host", HeaderValue::from_static("blog.example.com:8489"));
    headers.insert(
        "origin",
        HeaderValue::from_static("https://blog.example.com"),
    );
    assert!(validate_browser_origin(&headers).is_ok());
}

#[test]
fn browser_origin_check_rejects_cross_site_origin() {
    let mut headers = HeaderMap::new();
    headers.insert("host", HeaderValue::from_static("blog.example.com"));
    headers.insert(
        "origin",
        HeaderValue::from_static("https://evil.example.net"),
    );
    assert!(validate_browser_origin(&headers).is_err());
}

#[test]
fn browser_origin_check_rejects_null_origin() {
    // Sandboxed iframes and some redirect chains send the literal
    // "null" origin; it is unverifiable and must be rejected.
    let mut headers = HeaderMap::new();
    headers.insert("host", HeaderValue::from_static("blog.example.com"));
    headers.insert("origin", HeaderValue::from_static("null"));
    assert!(validate_browser_origin(&headers).is_err());
}

#[test]
fn browser_origin_check_falls_back_to_referer() {
    let mut headers = HeaderMap::new();
    headers.insert("host", HeaderValue::from_static("blog.example.com"));
    headers.insert(
        "referer",
        HeaderValue::from_static("https://evil.example.net/attack.html"),
    );
    assert!(validate_browser_origin(&headers).is_err());
}

#[test]
fn test_build_csrf_cookie() {
    let token = "test_token_value";
    let cookie = build_csrf_cookie(token);

    assert_eq!(cookie.name(), CSRF_COOKIE_NAME);
    assert_eq!(cookie.value(), token);
    assert_eq!(cookie.path(), Some("/"));
    assert_eq!(cookie.same_site(), Some(SameSite::Strict));
    assert_eq!(cookie.http_only(), Some(false));
    assert!(cookie.max_age().is_some());
}
