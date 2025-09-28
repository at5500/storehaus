//! Error types for cache operations
//!
//! This module defines all error types that can occur
//! during cache operations and Redis interactions.

use thiserror::Error;

/// Cache system errors
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Redis connection error: {0}")]
    ConnectionError(#[from] redis::RedisError),

    #[error("Connection pool error: {0}")]
    Connection(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Cache operation timeout")]
    Timeout,

    #[error("Invalid TTL value: {0}")]
    InvalidTtl(u64),

    #[error("Cache is disabled")]
    Disabled,

    #[error("General cache error: {0}")]
    General(String),
}
