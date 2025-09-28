//! Cache manager implementation
//!
//! This module provides the main CacheManager struct
//! for Redis operations and connection management.

use crate::errors::CacheError;
use config::CacheConfig;
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Redis-based cache manager
#[derive(Clone)]
pub struct CacheManager {
    client: Arc<Client>,
    config: Arc<CacheConfig>,
    connection_pool: Arc<RwLock<Option<redis::aio::MultiplexedConnection>>>,
}

impl Debug for CacheManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let connection_status = {
            match self.connection_pool.try_read() {
                Ok(pool) => {
                    if pool.is_some() {
                        "connected"
                    } else {
                        "no_connection"
                    }
                }
                Err(_) => "lock_error",
            }
        };

        f.debug_struct("CacheManager")
            .field("config", &self.config)
            .field("connected", &connection_status)
            .finish()
    }
}

impl CacheManager {
    /// Create a new cache manager
    pub fn new(config: CacheConfig) -> Result<Self, CacheError> {
        let client = Client::open(config.redis_url.as_str())?;

        Ok(Self {
            client: Arc::new(client),
            config: Arc::new(config),
            connection_pool: Arc::new(RwLock::new(None)),
        })
    }

    /// Get or create Redis connection
    async fn get_connection(&self) -> Result<redis::aio::MultiplexedConnection, CacheError> {
        let mut pool = self.connection_pool.write().await;

        if pool.is_none() {
            let connection = self.client.get_multiplexed_async_connection().await?;
            *pool = Some(connection);
        }

        // Safe extraction: we just ensured pool contains a connection above
        Ok(pool
            .as_ref()
            .ok_or_else(|| CacheError::Connection("Failed to get connection from pool".into()))?
            .clone())
    }

    /// Generate cache key for record by ID
    fn build_record_key(&self, prefix: &str, table_name: &str, id: &str) -> String {
        format!("{}:{}:record:{}", prefix, table_name, id)
    }

    /// Generate cache key for query results
    fn build_query_key(&self, prefix: &str, table_name: &str, query_hash: &str) -> String {
        format!("{}:{}:query:{}", prefix, table_name, query_hash)
    }

    /// Generate hash for query parameters
    pub fn hash_query<T: Hash>(&self, query: &T) -> String {
        let mut hasher = DefaultHasher::new();
        query.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Get single record from cache by ID
    pub async fn get_record<T>(
        &self,
        prefix: &str,
        table_name: &str,
        id: &str,
    ) -> Result<Option<T>, CacheError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let cache_key = self.build_record_key(prefix, table_name, id);
        let mut conn = self.get_connection().await?;

        let cached_data: Option<String> = conn.get(&cache_key).await?;

        match cached_data {
            Some(json_str) => {
                let value: T = serde_json::from_str(&json_str)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Set single record in cache by ID
    pub async fn set_record<T>(
        &self,
        prefix: &str,
        table_name: &str,
        id: &str,
        value: &T,
        ttl: u64,
    ) -> Result<(), CacheError>
    where
        T: Serialize,
    {
        self.set_record_with_ttl(prefix, table_name, id, value, ttl)
            .await
    }

    /// Set single record in cache with custom TTL
    pub async fn set_record_with_ttl<T>(
        &self,
        prefix: &str,
        table_name: &str,
        id: &str,
        value: &T,
        ttl: u64,
    ) -> Result<(), CacheError>
    where
        T: Serialize,
    {
        let cache_key = self.build_record_key(prefix, table_name, id);
        let json_str = serde_json::to_string(value)?;
        let mut conn = self.get_connection().await?;

        let _: () = conn.set_ex(&cache_key, &json_str, ttl).await?;
        Ok(())
    }

    /// Get query results from cache
    pub async fn get_query<T>(
        &self,
        prefix: &str,
        table_name: &str,
        query_hash: &str,
    ) -> Result<Option<Vec<T>>, CacheError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let cache_key = self.build_query_key(prefix, table_name, query_hash);
        let mut conn = self.get_connection().await?;

        let cached_data: Option<String> = conn.get(&cache_key).await?;

        match cached_data {
            Some(json_str) => {
                let values: Vec<T> = serde_json::from_str(&json_str)?;
                Ok(Some(values))
            }
            None => Ok(None),
        }
    }

    /// Set query results in cache
    pub async fn set_query<T>(
        &self,
        prefix: &str,
        table_name: &str,
        query_hash: &str,
        results: &Vec<T>,
        ttl: u64,
    ) -> Result<(), CacheError>
    where
        T: Serialize,
    {
        self.set_query_with_ttl(prefix, table_name, query_hash, results, ttl)
            .await
    }

    /// Set query results in cache with custom TTL
    pub async fn set_query_with_ttl<T>(
        &self,
        prefix: &str,
        table_name: &str,
        query_hash: &str,
        results: &Vec<T>,
        ttl: u64,
    ) -> Result<(), CacheError>
    where
        T: Serialize,
    {
        let cache_key = self.build_query_key(prefix, table_name, query_hash);
        let json_str = serde_json::to_string(results)?;
        let mut conn = self.get_connection().await?;

        let _: () = conn.set_ex(&cache_key, &json_str, ttl).await?;
        Ok(())
    }

    /// Delete specific record from cache
    pub async fn delete_record(
        &self,
        prefix: &str,
        table_name: &str,
        id: &str,
    ) -> Result<bool, CacheError> {
        let cache_key = self.build_record_key(prefix, table_name, id);
        let mut conn = self.get_connection().await?;

        let deleted: i32 = conn.del(&cache_key).await?;
        Ok(deleted > 0)
    }

    /// Delete multiple records from cache
    pub async fn delete_records(
        &self,
        prefix: &str,
        table_name: &str,
        ids: Vec<String>,
    ) -> Result<i32, CacheError> {
        if ids.is_empty() {
            return Ok(0);
        }

        let cache_keys: Vec<String> = ids
            .into_iter()
            .map(|id| self.build_record_key(prefix, table_name, &id))
            .collect();

        let mut conn = self.get_connection().await?;
        let deleted: i32 = conn.del(cache_keys).await?;
        Ok(deleted)
    }

    /// Invalidate all query cache for a table (when data changes)
    pub async fn invalidate_queries(
        &self,
        prefix: &str,
        table_name: &str,
    ) -> Result<i32, CacheError> {
        let pattern = format!("{}:{}:query:*", prefix, table_name);
        let mut conn = self.get_connection().await?;

        // Get all matching query keys
        let keys: Vec<String> = conn.keys(&pattern).await?;

        if keys.is_empty() {
            return Ok(0);
        }

        // Delete all query cache keys
        let deleted: i32 = conn.del(keys).await?;
        Ok(deleted)
    }

    /// Full invalidation for table (records + queries) - use sparingly
    pub async fn invalidate_table(
        &self,
        prefix: &str,
        table_name: &str,
    ) -> Result<i32, CacheError> {
        let record_pattern = format!("{}:{}:record:*", prefix, table_name);
        let query_pattern = format!("{}:{}:query:*", prefix, table_name);
        let mut conn = self.get_connection().await?;

        // Get all matching keys
        let mut all_keys: Vec<String> = Vec::new();
        all_keys.extend(conn.keys::<_, Vec<String>>(&record_pattern).await?);
        all_keys.extend(conn.keys::<_, Vec<String>>(&query_pattern).await?);

        if all_keys.is_empty() {
            return Ok(0);
        }

        // Delete all keys
        let deleted: i32 = conn.del(all_keys).await?;
        Ok(deleted)
    }

    /// Check if record exists in cache
    pub async fn record_exists(
        &self,
        prefix: &str,
        table_name: &str,
        id: &str,
    ) -> Result<bool, CacheError> {
        let cache_key = self.build_record_key(prefix, table_name, id);
        let mut conn = self.get_connection().await?;

        let exists: bool = conn.exists(&cache_key).await?;
        Ok(exists)
    }

    /// Get TTL for a record
    pub async fn record_ttl(
        &self,
        prefix: &str,
        table_name: &str,
        id: &str,
    ) -> Result<i64, CacheError> {
        let cache_key = self.build_record_key(prefix, table_name, id);
        let mut conn = self.get_connection().await?;

        let ttl: i64 = conn.ttl(&cache_key).await?;
        Ok(ttl)
    }

    /// Ping Redis to check connectivity
    pub async fn ping(&self) -> Result<String, CacheError> {
        let mut conn = self.get_connection().await?;

        // Use redis::cmd! macro for ping
        let pong: String = redis::cmd("PING").query_async(&mut conn).await?;
        Ok(pong)
    }

    /// Get current configuration
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }
}
