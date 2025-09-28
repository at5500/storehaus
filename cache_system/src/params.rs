//! Cache parameter configuration
//!
//! This module defines the CacheParams struct
//! for configuring cache behavior and TTL settings.

use crate::CacheManager;
use std::sync::Arc;

/// Cache parameters for configuring cache behavior per store/entity
#[derive(Debug, Clone)]
pub struct CacheParams {
    /// The cache manager instance
    pub manager: Arc<CacheManager>,
    /// TTL for this store in seconds
    pub ttl: u64,
    /// Prefix for cache keys
    pub prefix: String,
}

impl CacheParams {
    pub fn new(manager: Arc<CacheManager>, ttl: u64, prefix: &str) -> Self {
        Self {
            ttl,
            prefix: prefix.to_string(),
            manager,
        }
    }
}
