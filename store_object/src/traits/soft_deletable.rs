//! Trait definitions
//!
//! This module defines core traits for database operations.

use super::core::StoreObject;
use crate::StorehausError;
use async_trait::async_trait;

/// Trait for objects that support soft deletion (field marked with #[soft_delete])
#[async_trait]
pub trait SoftDeletable: StoreObject {
    /// List only active objects
    async fn list_active(&self) -> Result<Vec<Self::Model>, StorehausError>;

    /// Set active status for an object
    async fn set_active(&self, id: &Self::Id, is_active: bool) -> Result<bool, StorehausError>;

    /// Count active objects
    async fn count_active(&self) -> Result<i64, StorehausError>;
}
