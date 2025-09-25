use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Redis cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Redis connection string (redis://localhost:6379)
    pub redis_url: String,

    /// Default TTL for cache entries (in seconds)
    pub default_ttl: u64,

    /// Key prefix for all cache entries
    pub key_prefix: String,

    /// Maximum number of connections in the pool
    pub max_connections: Option<u32>,

    /// Connection timeout in milliseconds
    pub connection_timeout: Option<u64>,
}

impl CacheConfig {
    pub fn new(redis_url: String, default_ttl: u64, key_prefix: String) -> Self {
        Self {
            redis_url,
            default_ttl,
            key_prefix,
            max_connections: Some(10),
            connection_timeout: Some(5000),
        }
    }

    pub fn with_max_connections(mut self, max_connections: u32) -> Self {
        self.max_connections = Some(max_connections);
        self
    }

    pub fn with_connection_timeout(mut self, timeout_ms: u64) -> Self {
        self.connection_timeout = Some(timeout_ms);
        self
    }

    /// Get TTL as Duration
    pub fn ttl_duration(&self) -> Duration {
        Duration::from_secs(self.default_ttl)
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://localhost:6379".to_string(),
            default_ttl: 3600, // 1 hour
            key_prefix: "storehaus".to_string(),
            max_connections: Some(10),
            connection_timeout: Some(5000),
        }
    }
}