//! Security Layer
//! 
//! This module implements core security primitives including identity 
//! management (JWT) and request integrity (CSRF). 

pub mod auth; // JWT token lifecycle and verification
pub mod csrf; // Double-submit cookie CSRF protection
