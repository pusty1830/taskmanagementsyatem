//! One-time verification codes for email 2FA.
//!
//! Codes are stored as HMAC-SHA256(secret, "challenge_id:code"). A 6-digit code has only 10^6
//! values, so a slow hash would not stop offline brute force; the server-side secret does.
//! Binding the challenge id means a stored hash cannot be replayed against another challenge.

use hmac::{Hmac, Mac};
use rand::{rngs::OsRng, Rng};
use sha2::Sha256;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

pub const CODE_LENGTH: usize = 6;

/// Uniformly random, zero-padded 6-digit code from the OS CSPRNG.
pub fn generate_code() -> String {
    format!("{:06}", OsRng.gen_range(0..1_000_000u32))
}

fn mac(secret: &str, challenge_id: Uuid, code: &str) -> HmacSha256 {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts keys of any length");
    mac.update(challenge_id.as_bytes());
    mac.update(b":");
    mac.update(code.as_bytes());
    mac
}

pub fn hash_code(secret: &str, challenge_id: Uuid, code: &str) -> String {
    hex::encode(mac(secret, challenge_id, code).finalize().into_bytes())
}

/// Constant-time comparison against the stored hex hash.
pub fn verify_code(secret: &str, challenge_id: Uuid, code: &str, stored_hex: &str) -> bool {
    let Ok(expected) = hex::decode(stored_hex) else {
        return false;
    };
    mac(secret, challenge_id, code)
        .verify_slice(&expected)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "unit-test-secret";

    #[test]
    fn generated_codes_are_six_digits() {
        for _ in 0..200 {
            let code = generate_code();
            assert_eq!(code.len(), CODE_LENGTH);
            assert!(code.chars().all(|c| c.is_ascii_digit()));
        }
    }

    #[test]
    fn hash_verifies_only_with_same_challenge_and_code() {
        let id = Uuid::new_v4();
        let stored = hash_code(SECRET, id, "123456");

        assert!(verify_code(SECRET, id, "123456", &stored));
        assert!(!verify_code(SECRET, id, "123457", &stored));
        assert!(!verify_code(SECRET, Uuid::new_v4(), "123456", &stored));
        assert!(!verify_code("other-secret", id, "123456", &stored));
    }

    #[test]
    fn malformed_stored_hash_never_verifies() {
        assert!(!verify_code(SECRET, Uuid::new_v4(), "123456", "zz-not-hex"));
    }
}
