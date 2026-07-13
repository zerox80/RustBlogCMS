use super::*;
use crate::db::migrations::run_migrations;
use sqlx::SqlitePool;

async fn setup_test_db() -> DbPool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    run_migrations(&pool)
        .await
        .expect("Failed to run migrations");
    pool
}

fn init_salts() {
    if LOGIN_ATTEMPT_SALT.get().is_none() {
        env::set_var(
            "LOGIN_ATTEMPT_SALT",
            "this_is_a_test_salt_for_login_attempts_at_least_32_chars",
        );
        let _ = init_login_attempt_salt();
    }
    if auth::JWT_SECRET.get().is_none() {
        env::set_var(
            "JWT_SECRET",
            "this_is_a_test_jwt_secret_with_adequate_entropy_123_ABC_!!!",
        );
        let _ = auth::init_jwt_secret();
    }
    // CSRF_SECRET is private, we just call init and ignore "already initialized" error
    env::set_var(
        "CSRF_SECRET",
        "this_is_a_very_long_secret_key_for_testing_purposes_only_at_least_32_bytes",
    );
    let _ = csrf::init_csrf_secret();
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    init_salts();
    let pool = setup_test_db().await;

    let payload = LoginRequest {
        username: "nonexistent".to_string(),
        password: "InvalidPassword123!".to_string(),
    };

    let addr = "127.0.0.1:1234".parse().unwrap();

    let result = login(
        State(pool),
        HeaderMap::new(),
        ConnectInfo(addr),
        Json(payload),
    )
    .await;

    assert!(result.is_err());
    let (status, Json(body)) = result.unwrap_err();
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body.error, "Invalid credentials");
}

/// Regression test for the password-spraying gap: rotating usernames
/// gives the attacker a fresh (IP+username) pair key on every attempt,
/// so only the IP-wide counter can stop them. After
/// IP_WIDE_SHORT_THRESHOLD failures from one address, the next attempt
/// must be rejected with 429 regardless of which username it targets.
#[tokio::test]
async fn test_ip_wide_lockout_blocks_username_rotation() {
    init_salts();
    let pool = setup_test_db().await;
    let addr: std::net::SocketAddr = "127.0.0.2:1234".parse().unwrap();

    for i in 0..IP_WIDE_SHORT_THRESHOLD {
        let result = login(
            State(pool.clone()),
            HeaderMap::new(),
            ConnectInfo(addr),
            Json(LoginRequest {
                username: format!("sprayed_user_{}", i),
                password: "WrongPassword123!".to_string(),
            }),
        )
        .await;

        let (status, _) = result.expect_err("login with bad password must fail");
        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "attempt {} should fail with 401, not yet be rate limited",
            i
        );
    }

    let result = login(
        State(pool),
        HeaderMap::new(),
        ConnectInfo(addr),
        Json(LoginRequest {
            username: "yet_another_user".to_string(),
            password: "WrongPassword123!".to_string(),
        }),
    )
    .await;

    let (status, _) = result.expect_err("attempt past the IP-wide threshold must fail");
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
}

#[test]
fn test_validate_username() {
    assert!(validate_username("admin").is_ok());
    assert!(validate_username("user.name").is_ok());
    assert!(validate_username("user_123").is_ok());
    assert!(validate_username("").is_err());
    assert!(validate_username("user!").is_err());
    assert!(validate_username("a".repeat(51).as_str()).is_err());
}

#[test]
fn test_validate_login_password() {
    assert!(validate_login_password("ValidPassword123!").is_ok());
    // Login must not enforce complexity rules: existing users whose
    // passwords predate the policy still need to authenticate.
    assert!(validate_login_password("short").is_ok());
    assert!(validate_login_password("NoSpecialChar123").is_ok());
    let ordinary_login_input = [
        'n', 'o', 'n', 'u', 'm', 'b', 'e', 'r', 's', 'p', 'e', 'c', '!',
    ]
    .into_iter()
    .collect::<String>();
    assert!(validate_login_password(&ordinary_login_input).is_ok());
    // Only emptiness and the bcrypt DoS length cap are rejected.
    assert!(validate_login_password("").is_err());
    assert!(validate_login_password(&"a".repeat(129)).is_err());
}
