use super::*;

/// Global salt for hashing login attempt identifiers.
/// Initialized once at startup via init_login_attempt_salt().
pub(super) static LOGIN_ATTEMPT_SALT: OnceLock<String> = OnceLock::new();

/// Per-(IP+username) lockout: short block (10s) after this many failures.
pub(super) const PAIR_SHORT_THRESHOLD: i64 = 3;
/// Per-(IP+username) lockout: long block (60s) after this many failures.
pub(super) const PAIR_LONG_THRESHOLD: i64 = 5;

/// IP-wide lockout thresholds. Looser than the pair key so shared addresses
/// (NAT, office networks) are not punished for one user's typos, but tight
/// enough that spraying many usernames from a single address stalls quickly.
pub(super) const IP_WIDE_SHORT_THRESHOLD: i64 = 10;
pub(super) const IP_WIDE_LONG_THRESHOLD: i64 = 20;
/// IP-wide lockout: short block duration in seconds.
pub(super) const IP_WIDE_SHORT_BLOCK_SECONDS: i64 = 60;
/// IP-wide lockout: long block duration in seconds.
pub(super) const IP_WIDE_LONG_BLOCK_SECONDS: i64 = 300;

/// Initializes the login attempt salt from environment.
///
/// This salt is used to hash usernames before storing them in the
/// login_attempts table, preventing username enumeration attacks.
///
/// # Returns
/// - `Ok(())` if initialization succeeds
/// - `Err(String)` with error message if validation fails
///
/// # Errors
/// - LOGIN_ATTEMPT_SALT environment variable not set
/// - Salt is too short (< 32 characters)
/// - Salt has insufficient entropy (< 10 unique characters)
/// - Salt was already initialized
pub fn init_login_attempt_salt() -> Result<(), String> {
    let raw = env::var("LOGIN_ATTEMPT_SALT")
        .map_err(|_| "LOGIN_ATTEMPT_SALT environment variable not set".to_string())?;
    let trimmed = raw.trim();

    if trimmed.len() < 32 {
        return Err("LOGIN_ATTEMPT_SALT must be at least 32 characters long".to_string());
    }

    let unique_chars = trimmed
        .chars()
        .collect::<std::collections::HashSet<_>>()
        .len();
    if unique_chars < 10 {
        return Err("LOGIN_ATTEMPT_SALT must contain at least 10 unique characters".to_string());
    }

    LOGIN_ATTEMPT_SALT
        .set(trimmed.to_string())
        .map_err(|_| "LOGIN_ATTEMPT_SALT already initialized".to_string())?;

    Ok(())
}

/// Retrieves the initialized login attempt salt.
///
/// # Panics
/// Panics if init_login_attempt_salt() has not been called yet.
pub(super) fn login_attempt_salt() -> &'static str {
    LOGIN_ATTEMPT_SALT
        .get()
        .expect("LOGIN_ATTEMPT_SALT not initialized. Call init_login_attempt_salt() first.")
        .as_str()
}

/// Hashes a username for login attempt tracking.
///
/// Creates a salted SHA-256 hash of the normalized username.
/// This prevents username enumeration by obscuring which accounts exist.
///
/// # Arguments
/// * `username` - The username to hash
///
/// # Returns
/// Hex-encoded SHA-256 hash
///
/// # Security
/// - Username is trimmed and lowercased for normalization
/// - Salt prevents rainbow table attacks
/// - Hash prevents direct username storage
pub(super) fn hash_login_identifier(username: &str) -> String {
    let mut data = login_attempt_salt().as_bytes().to_vec();
    data.extend_from_slice(username.trim().to_ascii_lowercase().as_bytes());
    crate::security::sha256_hex(&data)
}

/// Parses an optional RFC3339 timestamp string into a UTC DateTime.
///
/// # Arguments
/// * `value` - Optional RFC3339 timestamp string
///
/// # Returns
/// - `Some(DateTime<Utc>)` if parsing succeeds
/// - `None` if value is None or parsing fails
pub(super) fn parse_rfc3339_opt(value: &Option<String>) -> Option<DateTime<Utc>> {
    value
        .as_ref()
        .and_then(|timestamp| chrono::DateTime::parse_from_rfc3339(timestamp).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

/// Returns a precomputed dummy bcrypt hash for timing-attack resistance.
///
/// This hash is used during failed login attempts to ensure password
/// verification takes constant time regardless of whether the user exists.
///
/// # Returns
/// A static bcrypt hash string
///
/// # Security
/// Using a dummy hash when the user doesn't exist prevents timing attacks
/// that could enumerate valid usernames by measuring response times.
pub(super) fn dummy_bcrypt_hash() -> &'static str {
    static DUMMY_HASH: OnceLock<String> = OnceLock::new();

    DUMMY_HASH.get_or_init(|| match bcrypt::hash("dummy", bcrypt::DEFAULT_COST) {
        Ok(hash) => hash,
        Err(err) => {
            tracing::error!("Failed to generate dummy hash: {}", err);
            "$2b$12$eImiTXuWVxfM37uY4JANjQPzMzXZjQDzqzQpMv0xoGrTplPPNaE3W".to_string()
        }
    })
}

/// Validates a username meets security and format requirements.
///
/// # Arguments
/// * `username` - The username to validate
///
/// # Returns
/// - `Ok(())` if valid
/// - `Err(String)` with error message if invalid
///
/// # Validation Rules
/// - Not empty
/// - Length ≤ 50 characters
/// - Only alphanumeric, underscore, hyphen, and period allowed
pub(super) fn validate_username(username: &str) -> Result<(), String> {
    if username.is_empty() {
        return Err("Username cannot be empty".to_string());
    }
    if username.len() > 50 {
        return Err("Username too long".to_string());
    }

    if !username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err("Username contains invalid characters".to_string());
    }
    Ok(())
}

/// Validates a password submitted during login.
///
/// Deliberately minimal: complexity rules belong to password creation,
/// not login. Enforcing them here would lock out existing users whose
/// stored passwords predate the policy and would leak the policy to
/// attackers via distinguishable 400 responses.
///
/// # Validation Rules
/// - Not empty
/// - Length ≤ 128 characters (prevents DoS via expensive bcrypt hashing)
pub(super) fn validate_login_password(password: &str) -> Result<(), String> {
    if password.is_empty() {
        return Err("Password cannot be empty".to_string());
    }
    if password.len() > 128 {
        return Err("Password too long".to_string());
    }
    Ok(())
}
