//! Backend Middleware Collection
//!
//! This module aggregates all middleware layers used by the Axum server,
//! including security headers, authentication, and CORS configuration.

pub mod auth;     // Identity and session verification
pub mod cors;     // Cross-origin resource sharing
pub mod security; // Defense-in-depth security policies
