//! Security Layer
//!
//! This module implements core security primitives including identity
//! management (JWT) and request integrity (CSRF).

use sha2::{Digest, Sha256};

pub mod auth; // JWT token lifecycle and verification
pub mod csrf; // Double-submit cookie CSRF protection

/// Returns the lowercase hex-encoded SHA-256 digest of `data`.
///
/// Shared primitive for callers that need a one-way digest (e.g. hashing a
/// login identifier before rate-limit lookups, or hashing a JWT before it is
/// persisted in the token blacklist) — keeps the hashing recipe in one place
/// instead of every caller re-implementing `Sha256::new/update/finalize`.
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
