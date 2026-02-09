/**
 * HTTP Handlers Module
 *
 * This module contains all HTTP request handlers for the Rust Blog CMS API.
 * Each handler module is organized by domain and follows RESTful conventions
 * for HTTP endpoints. All handlers include comprehensive error handling,
 * input validation, and security controls.
 *
 * Handler Organization:
 *
 * ## Core Functionality
 *
 * ### [`auth`](mod@auth)
 * **Authentication and Authorization Endpoints**
 * - `POST /api/auth/login` - User authentication with CSRF protection
 * - `POST /api/auth/logout` - Session termination with cookie cleanup
 * - `GET /api/auth/me` - Current user profile retrieval
 *
 * ### [`search`](mod@search)
 * **Full-Text Search Functionality**
 * - `GET /api/search/tutorials` - Tutorial search with FTS5
 * - `GET /api/search/topics` - Topic discovery and filtering
 *
 * ## Content Management
 *
 * ### [`tutorials`](mod@tutorials)
 * **Tutorial CRUD Operations**
 * - `GET /api/tutorials` - List all tutorials
 * - `GET /api/tutorials/{id}` - Get specific tutorial
 * - `POST /api/tutorials` - Create new tutorial (admin)
 * - `PUT /api/tutorials/{id}` - Update tutorial (admin)
 * - `DELETE /api/tutorials/{id}` - Delete tutorial (admin)
 *
 * ### [`comments`](mod@comments)
 * **Comment System**
 * - `GET /api/tutorials/{id}/comments` - List tutorial comments
 * - `POST /api/tutorials/{id}/comments` - Create comment (admin)
 * - `DELETE /api/comments/{id}` - Delete comment (admin)
 *
 * ## Site Content Management
 *
 * ### [`site_content`](mod@site_content)
 * **Dynamic Site Content**
 * - `GET /api/content` - List all content sections
 * - `GET /api/content/{section}` - Get specific section content
 * - `PUT /api/content/{section}` - Update section content (admin)
 *
 * ### [`site_pages`](mod@site_pages)
 * **Static Page Management**
 * - `GET /api/pages` - List all pages (admin)
 * - `GET /api/pages/{id}` - Get specific page (admin)
 * - `POST /api/pages` - Create new page (admin)
 * - `PUT /api/pages/{id}` - Update page (admin)
 * - `DELETE /api/pages/{id}` - Delete page (admin)
 *
 * ### [`site_posts`](mod@site_posts)
 * **Blog Post Management**
 * - `GET /api/pages/{page_id}/posts` - List posts for page (admin)
 * - `GET /api/posts/{id}` - Get specific post (admin)
 * - `POST /api/pages/{page_id}/posts` - Create post (admin)
 * - `PUT /api/posts/{id}` - Update post (admin)
 * - `DELETE /api/posts/{id}` - Delete post (admin)
 *
 * ## Public Endpoints
 *
 * These are automatically accessible without authentication:
 * - `GET /api/public/pages/{slug}` - Get published page by slug
 * - `GET /api/public/pages/{slug}/posts/{post_slug}` - Get published post
 * - `GET /api/public/navigation` - Get site navigation structure
 * - `GET /api/public/published-pages` - List published page slugs
 *
 * # Security Features
 *
 * ## Authentication & Authorization
 * - JWT token validation on protected endpoints
 * - Role-based access control (admin-only for mutations)
 * - Session management with HttpOnly cookies
 * - Automatic token refresh and expiration
 *
 * ## CSRF Protection
 * - Double-submit cookie pattern for state-changing requests
 * - HMAC-SHA256 signed tokens with user binding
 * - Short TTL (6 hours) to reduce attack window
 * - Constant-time signature verification
 *
 * ## Input Validation
 * - Comprehensive request body validation
 * - SQL injection prevention with parameterized queries
 * - File upload size limits and type validation
 * - Rate limiting on sensitive endpoints
 *
 * ## Error Handling
 * - Standardized error response format
 * - Secure error messages without information leakage
 * - Proper HTTP status codes
 * - Comprehensive logging for debugging
 *
 * # Response Formats
 *
 * ## Success Responses
 * ```json
 * {
 *   "data": { ... },  // Response data
 *   "status": "success"
 * }
 * ```
 *
 * ## Error Responses
 * ```json
 * {
 *   "error": "Human-readable error message",
 *   "details": "Additional error context (optional)"
 * }
 * ```
 *
 * ## List Responses
 * ```json
 * {
 *   "items": [ ... ],     // Array of items
 *   "total": 42,          // Total count (when paginated)
 *   "hasMore": true       // Pagination indicator
 * }
 * ```
 *
 * # Rate Limiting
 *
 * Different endpoints have different rate limits:
 * - Authentication endpoints: 5 requests per second
 * - Content creation endpoints: 3 requests per second
 * - Read-only endpoints: No rate limiting
 * - Public search endpoints: 10 requests per second
 *
 * # Performance Optimizations
 *
 * - Database connection pooling
 * - Efficient query patterns with proper indexing
 * - Response compression for large payloads
 * - Caching headers for static content
 * - Async/await for high concurrency
 */
// HTTP Handler Modules - Organized by Domain

// Core System Handlers
pub mod auth; // Authentication and authorization
pub mod search; // Full-text search functionality

// Content Management Handlers
pub mod tutorials;
pub mod upload;
// Tutorial CRUD operations
pub mod comments; // Comment system management

// Site Content Handlers
pub mod frontend_proxy;
pub mod site_content; // Dynamic site content sections
pub mod site_pages; // Static page management
pub mod site_posts; // Blog post management // Frontend proxy for server-side injection
