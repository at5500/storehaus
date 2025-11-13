//! Trait definitions
//!
//! This module defines core traits for database operations.

use crate::StorehausError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Trait that defines common database operations for all entities
#[async_trait]
pub trait StoreObject: Clone + Send + Sync + Debug {
    /// The model type that this object represents
    type Model: Clone + Send + Sync + Debug + Serialize + for<'de> Deserialize<'de>;

    /// The ID type used for this object (UUID, i32, String, etc.)
    type Id: Clone + Send + Sync + Debug;

    /// Create a new instance of this object
    async fn create(
        &self,
        data: Self::Model,
        tags: Option<Vec<String>>,
    ) -> Result<Self::Model, StorehausError>;

    /// Get an object by its ID
    async fn get_by_id(&self, id: &Self::Id) -> Result<Option<Self::Model>, StorehausError>;

    /// List all objects of this type
    async fn list_all(&self) -> Result<Vec<Self::Model>, StorehausError>;

    /// Update an object by its ID
    async fn update(
        &self,
        id: &Self::Id,
        data: Self::Model,
        tags: Option<Vec<String>>,
    ) -> Result<Self::Model, StorehausError>;

    /// Update multiple objects by their IDs
    async fn update_many(
        &self,
        updates: Vec<(Self::Id, Self::Model)>,
    ) -> Result<Vec<Self::Model>, StorehausError>;

    /// Delete an object by its ID
    async fn delete(&self, id: &Self::Id) -> Result<bool, StorehausError>;

    /// Delete multiple objects by their IDs
    async fn delete_many(&self, ids: Vec<Self::Id>) -> Result<Vec<Self::Id>, StorehausError>;

    /// Count total objects of this type
    async fn count(&self) -> Result<i64, StorehausError>;

    /// Find records matching query conditions
    async fn find(&self, query: crate::QueryBuilder) -> Result<Vec<Self::Model>, StorehausError>;

    /// Find first record matching query conditions
    async fn find_one(
        &self,
        query: crate::QueryBuilder,
    ) -> Result<Option<Self::Model>, StorehausError>;

    /// Update records matching query conditions
    ///
    /// - If query contains UpdateSet operations, data can be None
    /// - If using legacy mode (no UpdateSet), data must be Some(model)
    async fn update_where(
        &self,
        query: crate::QueryBuilder,
        data: Option<Self::Model>,
    ) -> Result<Vec<Self::Model>, StorehausError>;

    /// Delete records matching query conditions
    async fn delete_where(
        &self,
        query: crate::QueryBuilder,
    ) -> Result<Vec<Self::Id>, StorehausError>;

    /// Count records matching query conditions
    async fn count_where(&self, query: crate::QueryBuilder) -> Result<i64, StorehausError>;
}
