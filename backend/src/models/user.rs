use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Represents a registered system user.
///
/// This struct maps directly to the `users` database table.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    /// Unique database identifier.
    pub id: i64,
    /// Unique username string.
    pub username: String,
    /// The user's password hash.
    ///
    /// Marked with `#[serde(skip_serializing)]` to prevent accidental exposure in API responses.
    #[serde(skip_serializing)]
    pub password_hash: String,
    /// User's role (e.g., "admin").
    pub role: String,
    /// ISO 8601 timestamp of account creation.
    pub created_at: String,
}

/// Data payload for user login requests.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// The username of the account.
    pub username: String,
    /// The password for authentication.
    pub password: String,
}

/// Response payload for a successful login.
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    /// JWT token for authenticated session access.
    pub token: String,
    /// Public details of the logged-in user.
    pub user: UserResponse,
}

/// A public view of the User model, stripping sensitive data.
#[derive(Debug, Serialize)]
pub struct UserResponse {
    /// The username.
    pub username: String,
    /// The user's role.
    pub role: String,
}
