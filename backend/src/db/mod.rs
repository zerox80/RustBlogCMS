//! Database Layer
//!
//! This module coordinates database initialization, schema migrations,
//! and initial data seeding. It provides a shared connection pool
//! used by all repository instances.

pub mod migrations; // SQL schema versioning
pub mod pool; // Connection lifecycle management
pub mod seed; // Initial data (Default User, etc.)

pub use pool::{create_pool, DbPool};
