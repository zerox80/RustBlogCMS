//! Data Access Layer (Repositories)
//!
//! This module provides the interface for interacting with the database.
//! Repositories are organized by domain and abstract away the underlying
//! SQL structure using `sqlx`. They handle connections, transactions,
//! and map database rows to application models.

pub mod app_metadata; // Generic key-value storage
pub mod comments; // Comment and voting persistence
pub mod common; // Shared validation and serialization utilities
pub mod content; // Dynamic landing page sections
pub mod pages; // Site page structure
pub mod posts; // Detailed blog post content
pub mod token_blacklist; // Authentication revocation state
pub mod tutorials; // Course material and topic indexing
pub mod users; // User identity and brute-force tracking
