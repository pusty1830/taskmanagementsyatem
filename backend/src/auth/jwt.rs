use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{Role, User};

pub const ISSUER: &str = "task-api";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub role: Role,
    pub iat: i64,
    pub exp: i64,
    pub iss: String,
}

/// Issues an HS256 access token; returns the token and its lifetime in seconds.
pub fn issue_token(user: &User, secret: &str, ttl_minutes: i64) -> anyhow::Result<(String, i64)> {
    let now = Utc::now();
    let ttl = Duration::minutes(ttl_minutes);
    let claims = Claims {
        sub: user.id,
        email: user.email.clone(),
        role: user.role,
        iat: now.timestamp(),
        exp: (now + ttl).timestamp(),
        iss: ISSUER.to_string(),
    };
    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok((token, ttl.num_seconds()))
}

pub fn decode_token(token: &str, secret: &str) -> jsonwebtoken::errors::Result<Claims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 0;
    validation.set_issuer(&[ISSUER]);
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "unit-test-jwt-secret";

    fn user() -> User {
        User {
            id: Uuid::new_v4(),
            full_name: "James Bond".into(),
            email: "jamesbond@example.com".into(),
            hashed_password: String::new(),
            role: Role::Staff,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn issued_token_round_trips() {
        let u = user();
        let (token, expires_in) = issue_token(&u, SECRET, 60).unwrap();

        let claims = decode_token(&token, SECRET).unwrap();

        assert_eq!(expires_in, 3600);
        assert_eq!(claims.sub, u.id);
        assert_eq!(claims.email, u.email);
        assert_eq!(claims.role, Role::Staff);
    }

    #[test]
    fn expired_token_is_rejected() {
        let (token, _) = issue_token(&user(), SECRET, -1).unwrap();
        assert!(decode_token(&token, SECRET).is_err());
    }

    #[test]
    fn tampered_token_is_rejected() {
        let (token, _) = issue_token(&user(), SECRET, 60).unwrap();
        let mut parts: Vec<&str> = token.split('.').collect();
        let forged_payload = parts[1].chars().rev().collect::<String>();
        parts[1] = &forged_payload;

        assert!(decode_token(&parts.join("."), SECRET).is_err());
        assert!(decode_token(&token, "wrong-secret").is_err());
    }
}
