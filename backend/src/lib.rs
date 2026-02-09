/**
 * Linux Tutorial CMS - Library Root Module
 *
 * This module serves as the main entry point for the Linux Tutorial CMS backend library.
 * It organizes all application modules into a coherent structure for both binary and library usage.
 *
 * Architecture Overview:
 * The backend is organized into distinct modules, each with specific responsibilities:
 *
 * # Modules
 *
 * ## [`auth`](mod@auth)
 * Handles JWT-based authentication and authorization including:
 * - JWT token generation and validation
 * - User login/logout functionality
 * - Authentication middleware for protected routes
 * - Password security and rate limiting
 *
 * ## [`csrf`](mod@csrf)
 * Provides Cross-Site Request Forgery protection including:
 * - HMAC-SHA256 signed CSRF tokens
 * - Per-user token binding and expiration
 * - Cookie and header-based token validation
 * - AXUM middleware for automatic protection
 *
 * ## [`db`](mod@db)
 * Manages SQLite database operations and migrations including:
 * - Database connection pooling with SQLx
 * - Schema migrations and versioning
 * - Full-text search with FTS5
 * - Data access patterns and validation
 *
 * ## [`handlers`](mod@handlers)
 * Contains HTTP request handlers organized by feature:
 * - [`auth`](mod@handlers::auth): Authentication endpoints
 * - [`tutorials`](mod@handlers::tutorials): Tutorial CRUD operations
 * - [`comments`](mod@handlers::comments): Comment management
 * - [`search`](mod@handlers::search): Full-text search functionality
 * - [`site_content`](mod@handlers::site_content): Site content management
 * - [`site_pages`](mod@handlers::site_pages): Static page management
 * - [`site_posts`](mod@handlers::site_posts): Blog post management
 *
 * ## [`models`](mod@models)
 * Defines data structures and API models including:
 * - Database entity models with serialization
 * - API request/response structures
 * - Data validation and transformation logic
 * - Security-conscious field exposure
 *
 * # Security Features
 *
 * ## Authentication & Authorization
 * - JWT tokens with expiration and role-based access
 * - Secure password hashing with bcrypt
 * - Rate limiting on authentication endpoints
 * - Session management with HttpOnly cookies
 *
 * ## CSRF Protection
 * - HMAC-SHA256 signed tokens
 * - Per-user token binding
 * - Short token TTL (6 hours)
 * - Double-submit cookie pattern
 *
 * ## Data Security
 * - SQL injection prevention with parameterized queries
 * - Input validation and sanitization
 * - Secure error handling without information leakage
 * - Encrypted sensitive data storage
 *
 * # Performance Features
 *
 * - Connection pooling for database operations
 * - Full-text search with FTS5 indexing
 * - Proper query optimization and indexing
 * - Efficient JSON serialization/deserialization
 * - Async/await patterns for high concurrency
 *
 * # Configuration
 *
 * The backend requires several environment variables for proper operation:
 * - `DATABASE_URL`: SQLite database connection string
 * - `JWT_SECRET`: Secret key for JWT token signing
 * - `CSRF_SECRET`: Secret key for CSRF token signing
 * - `ADMIN_USERNAME`: Initial admin user (optional)
 * - `ADMIN_PASSWORD`: Initial admin password (optional)
 *
 * # Usage
 *
 * This library can be used as:
 * 1. A binary application (via main.rs)
 * 2. A library for testing or integration
 * 3. A component in larger applications
 *
 * Example as a library:
 * ```rust
 * use rust_blog_backend::db;
 * use rust_blog_backend::security::auth;
 *
 * #[tokio::main]
 * async fn main() -> Result<(), Box<dyn std::error::Error>> {
 *     let pool = db::create_pool().await?;
 *     // Your custom application logic here
 *     Ok(())
 * }
 * ```
 */
// Core application modules
pub mod security; // Authentication, authorization, and CSRF protection
pub mod db; // Database operations and migrations
pub mod handlers; // HTTP request handlers
pub mod middleware; // HTTP middleware
pub mod models; // Data structures and API models
pub mod repositories; // Database repositories
pub mod routes; // Route definitions
