use std::{env, str::FromStr};

use anyhow::{bail, Context};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheBackend {
    Redis,
    /// Per-process cache: not shared between instances and lost on restart.
    Memory,
}

impl FromStr for CacheBackend {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "redis" => Ok(Self::Redis),
            "memory" => Ok(Self::Memory),
            other => bail!("CACHE_BACKEND must be 'redis' or 'memory', got '{other}'"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub app_env: String,
    pub bind_addr: String,
    pub database_url: String,
    pub redis_url: String,
    pub cache_backend: CacheBackend,
    pub cache_ttl_seconds: u64,
    pub jwt_secret: String,
    pub jwt_ttl_minutes: i64,
    pub otp_secret: String,
    pub otp_ttl_seconds: i64,
    pub otp_max_attempts: i32,
    pub cors_origin: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let config = Self {
            app_env: var_or("APP_ENV", "development"),
            bind_addr: var_or("BIND_ADDR", "127.0.0.1:8080"),
            database_url: required("DATABASE_URL")?,
            redis_url: var_or("REDIS_URL", "redis://localhost:6379"),
            cache_backend: parsed_or("CACHE_BACKEND", CacheBackend::Redis)?,
            cache_ttl_seconds: parsed_or("CACHE_TTL_SECONDS", 300)?,
            jwt_secret: required("JWT_SECRET")?,
            jwt_ttl_minutes: parsed_or("JWT_TTL_MINUTES", 60)?,
            otp_secret: required("OTP_SECRET")?,
            otp_ttl_seconds: parsed_or("OTP_TTL_SECONDS", 300)?,
            otp_max_attempts: parsed_or("OTP_MAX_ATTEMPTS", 5)?,
            cors_origin: var_or("CORS_ORIGIN", "http://localhost:5173"),
        };

        if config.jwt_secret.len() < 32 || config.otp_secret.len() < 32 {
            bail!("JWT_SECRET and OTP_SECRET must each be at least 32 characters");
        }
        Ok(config)
    }

    /// Seed and dev routes are only mounted in development.
    pub fn is_development(&self) -> bool {
        self.app_env == "development"
    }
}

fn var_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn required(key: &str) -> anyhow::Result<String> {
    env::var(key).with_context(|| format!("missing required env var {key}"))
}

fn parsed_or<T>(key: &str, default: T) -> anyhow::Result<T>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    match env::var(key) {
        Ok(raw) => raw
            .parse()
            .map_err(|e| anyhow::anyhow!("invalid value for {key}: {e}")),
        Err(_) => Ok(default),
    }
}
