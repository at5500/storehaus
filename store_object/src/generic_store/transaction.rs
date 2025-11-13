//! Transaction support for GenericStore
//!
//! This module provides database transaction functionality for GenericStore,
//! allowing multiple operations to be executed atomically.

use super::GenericStore;
use crate::table_metadata::TableMetadata;
use crate::errors::StorehausError;
use sqlx::{Postgres, Transaction};
use std::marker::PhantomData;

/// A transactional context for GenericStore operations
///
/// This struct wraps a sqlx transaction and provides commit/rollback functionality.
/// The underlying transaction can be accessed via `as_mut()` for executing queries.
///
/// # Example
/// ```ignore
/// let mut tx = store.begin_transaction().await?;
///
/// // Use repository methods that accept &mut Transaction
/// wallet_repo.update_balance_tx(tx.as_mut(), wallet_id_1, -100).await?;
/// wallet_repo.update_balance_tx(tx.as_mut(), wallet_id_2, 100).await?;
///
/// // Commit the transaction
/// tx.commit().await?;
/// ```
pub struct GenericStoreTransaction<'a, T: TableMetadata> {
    tx: Transaction<'a, Postgres>,
    _phantom: PhantomData<T>,
}

impl<T: TableMetadata> GenericStore<T> {
    /// Begin a new database transaction
    pub async fn begin_transaction(&self) -> Result<GenericStoreTransaction<'_, T>, StorehausError> {
        let tx = self.db_pool.begin().await
            .map_err(|e| StorehausError::DatabaseError(format!("Failed to begin transaction: {}", e)))?;
        Ok(GenericStoreTransaction {
            tx,
            _phantom: PhantomData,
        })
    }

    /// Get a reference to the underlying pool
    /// This is needed for repositories to create transactions
    pub fn pool(&self) -> &sqlx::PgPool {
        &self.db_pool
    }
}

impl<'a, T: TableMetadata> GenericStoreTransaction<'a, T> {
    /// Commit the transaction
    pub async fn commit(self) -> Result<(), StorehausError> {
        self.tx.commit().await
            .map_err(|e| StorehausError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;
        Ok(())
    }

    /// Rollback the transaction
    pub async fn rollback(self) -> Result<(), StorehausError> {
        self.tx.rollback().await
            .map_err(|e| StorehausError::DatabaseError(format!("Failed to rollback transaction: {}", e)))?;
        Ok(())
    }

    /// Get a mutable reference to the underlying transaction
    /// Use this to execute queries within the transaction
    pub fn as_mut(&mut self) -> &mut Transaction<'a, Postgres> {
        &mut self.tx
    }
}