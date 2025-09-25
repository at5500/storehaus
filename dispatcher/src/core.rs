use sqlx::PgPool;
use std::collections::HashMap;
use store_object::traits::StoreObject;

use crate::errors::DispatcherError;
use crate::config::DatabaseConfig;

/// Main dispatcher that manages database connection and store objects
pub struct Dispatcher {
    pool: PgPool,
    stores: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl Dispatcher {
    /// Create new dispatcher with database connection
    pub async fn new(config: DatabaseConfig) -> Result<Self, DispatcherError> {
        let connection_string = config.connection_string();

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&connection_string)
            .await?;

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
    pub fn register_store<T>(&mut self, name: String, store: T) -> Result<(), DispatcherError>
    where
        T: StoreObject + Send + Sync + 'static,
    {
        if self.stores.contains_key(&name) {
            return Err(DispatcherError::StoreAlreadyRegistered(name));
        }

        self.stores.insert(name, Box::new(store));
        Ok(())
    }

    /// Get a registered store object by name
    pub fn get_store<T>(&self, name: &str) -> Result<&T, DispatcherError>
    where
        T: StoreObject + Send + Sync + 'static,
    {
        self.stores
            .get(name)
            .and_then(|store| store.downcast_ref::<T>())
            .ok_or_else(|| DispatcherError::StoreNotFound(name.to_string()))
    }

    /// Get a mutable reference to a registered store object by name
    pub fn get_store_mut<T>(&mut self, name: &str) -> Result<&mut T, DispatcherError>
    where
        T: StoreObject + Send + Sync + 'static,
    {
        self.stores
            .get_mut(name)
            .and_then(|store| store.downcast_mut::<T>())
            .ok_or_else(|| DispatcherError::StoreNotFound(name.to_string()))
    }

    /// List all registered store names
    pub fn list_stores(&self) -> Vec<&String> {
        self.stores.keys().collect()
    }

    /// Remove a store object by name
    pub fn unregister_store(&mut self, name: &str) -> Result<(), DispatcherError> {
        self.stores
            .remove(name)
            .map(|_| ())
            .ok_or_else(|| DispatcherError::StoreNotFound(name.to_string()))
    }

    /// Check database connection health
    pub async fn health_check(&self) -> Result<(), DispatcherError> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }

}