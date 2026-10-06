pub mod auth;
pub mod user;

use serde::{Deserialize, Deserializer};

/// Trims and lowercases an email during deserialization, so validation sees the normalized value.
pub(crate) fn normalized_email<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    Ok(String::deserialize(d)?.trim().to_lowercase())
}
