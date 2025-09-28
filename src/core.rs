//! Core StoreHaus functionality
//!
//! This module contains the main StoreHaus struct and its implementation,
//! providing centralized coordination for database operations, caching, and signals.

use sqlx::PgPool;
use std::collections::HashMap;
use std::time::Duration;
use store_object::traits::StoreObject;

use crate::errors::StoreHausError;
use config::DatabaseConfig;

/// Main StoreHaus coordinator that manages database connection and store objects
pub struct StoreHaus {
    pool: PgPool,
    stores: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl StoreHaus {
    /// Create new StoreHaus with database connection
    pub async fn new(config: DatabaseConfig) -> Result<Self, StoreHausError> {
        let connection_string = config.connection_string();

        let mut pool_options = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .idle_timeout(Duration::from_secs(config.idle_timeout_seconds));

        // Set max lifetime if specified
        if config.max_lifetime_seconds > 0 {
            pool_options =
                pool_options.max_lifetime(Duration::from_secs(config.max_lifetime_seconds));
        }

        let pool = pool_options.connect(&connection_string).await?;

        Ok(Self {
            pool,
            stores: HashMap::new(),
        })
    }

    /// Get database pool reference
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Register a store object with a given name
    pub fn register_store<T>(&mut self, name: String, store: T) -> Result<(), StoreHausError>
    where
        T: StoreObject + Send + Sync + 'static,
    {
        if self.stores.contains_key(&name) {
            return Err(StoreHausError::StoreAlreadyRegistered(name));
        }

        self.stores.insert(name, Box::new(store));
        Ok(())
    }

    /// Get a registered store object by name
    pub fn get_store<T>(&self, name: &str) -> Result<&T, StoreHausError>
    where
        T: StoreObject + Send + Sync + 'static,
    {
        self.stores
            .get(name)
            .and_then(|store| store.downcast_ref::<T>())
            .ok_or_else(|| StoreHausError::StoreNotFound(name.to_string()))
    }

    /// Get a mutable reference to a registered store object by name
    pub fn get_store_mut<T>(&mut self, name: &str) -> Result<&mut T, StoreHausError>
    where
        T: StoreObject + Send + Sync + 'static,
    {
        self.stores
            .get_mut(name)
            .and_then(|store| store.downcast_mut::<T>())
            .ok_or_else(|| StoreHausError::StoreNotFound(name.to_string()))
    }

    /// List all registered store names
    pub fn list_stores(&self) -> Vec<&String> {
        self.stores.keys().collect()
    }

    /// Remove a store object by name
    pub fn unregister_store(&mut self, name: &str) -> Result<(), StoreHausError> {
        self.stores
            .remove(name)
            .map(|_| ())
            .ok_or_else(|| StoreHausError::StoreNotFound(name.to_string()))
    }

    /// Check database connection health
    pub async fn health_check(&self) -> Result<(), StoreHausError> {
        sqlx::query("SELECT 1").fetch_one(&self.pool).await?;
        Ok(())
    }
}
