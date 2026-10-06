use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

/// Hashes a password with Argon2id (default params) into a PHC string.
pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| anyhow::anyhow!("password hashing failed: {e}"))
}

/// Returns false for a wrong password or a malformed stored hash.
pub fn verify_password(password: &str, stored_hash: &str) -> bool {
    PasswordHash::new(stored_hash)
        .map(|parsed| {
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok()
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_verifies_with_correct_password() {
        let hash = hash_password("S3cret!").unwrap();
        assert!(hash.starts_with("$argon2id$"));
        assert!(verify_password("S3cret!", &hash));
    }

    #[test]
    fn wrong_password_does_not_verify() {
        let hash = hash_password("S3cret!").unwrap();
        assert!(!verify_password("wrong", &hash));
    }

    #[test]
    fn malformed_hash_does_not_verify() {
        assert!(!verify_password("anything", "not-a-phc-string"));
    }
}
