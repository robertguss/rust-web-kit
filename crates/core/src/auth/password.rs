//! Argon2id password hashing.

use argon2::Argon2;
use password_hash::phc::PasswordHash;
use password_hash::{PasswordHasher, PasswordVerifier};

use crate::AppError;

/// Precomputed Argon2id hash of a dummy password. Used on login when no user
/// (or no `password_hash`) exists so verification cost stays uniform.
pub const DUMMY_PASSWORD_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$pCIA7HAmYrRvJPoQkAkKiQ$izj1mxBexcky9dyqB3uW2a5qaEOO6B6YfdQPj4leor0";

/// PHC-encoded Argon2id hash.
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let hash = Argon2::default()
        .hash_password(password.as_bytes())
        .map_err(|error| AppError::Internal(anyhow::anyhow!(error)))?;
    Ok(hash.to_string())
}

/// Constant-time verify. `false` on parse errors.
pub fn verify_password(password: &str, password_hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(password_hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}
