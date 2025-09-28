//! Generic store implementations
//!
//! This module provides generic database store functionality.

use super::core::GenericStore;
use crate::errors::StorehausError;
use crate::table_metadata::TableMetadata;
use crate::traits::SoftDeletable;
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl<T> SoftDeletable for GenericStore<T>
where
    T: TableMetadata
        + crate::traits::table_metadata::DatabaseExecutor
        + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
        + sqlx::Type<sqlx::Postgres>
        + serde::Serialize
        + Unpin,
{
    async fn list_active(&self) -> Result<Vec<Self::Model>, StorehausError> {
        if !T::supports_soft_delete() {
            return Err(StorehausError::validation(
                T::table_name(),
                "soft_delete",
                "Entity does not support soft deletion",
            ));
        }

        let soft_delete_field = T::soft_delete_field().ok_or_else(|| {
            StorehausError::validation(
                T::table_name(),
                "soft_delete",
                "Soft delete field not found",
            )
        })?;

        let sql = format!(
            "SELECT * FROM {} WHERE {} = true ORDER BY __created_at__ DESC",
            T::table_name(),
            soft_delete_field
        );
        let results = sqlx::query_as::<_, T>(&sql)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| StorehausError::database_operation(T::table_name(), "list_active", e))?;
        Ok(results)
    }

    async fn set_active(&self, id: &Self::Id, is_active: bool) -> Result<bool, StorehausError> {
        if !T::supports_soft_delete() {
            return Err(StorehausError::validation(
                T::table_name(),
                "soft_delete",
                "Entity does not support soft deletion",
            ));
        }

        let soft_delete_field = T::soft_delete_field().ok_or_else(|| {
            StorehausError::validation(
                T::table_name(),
                "soft_delete",
                "Soft delete field not found",
            )
        })?;

        let sql = format!(
            "UPDATE {} SET {} = $1, __updated_at__ = NOW() WHERE id = $2",
            T::table_name(),
            soft_delete_field
        );
        let result = sqlx::query(&sql)
            .bind(is_active)
            .bind(id)
            .execute(&self.db_pool)
            .await
            .map_err(|e| StorehausError::database_operation(T::table_name(), "set_active", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn count_active(&self) -> Result<i64, StorehausError> {
        if !T::supports_soft_delete() {
            return Err(StorehausError::validation(
                T::table_name(),
                "soft_delete",
                "Entity does not support soft deletion",
            ));
        }

        let soft_delete_field = T::soft_delete_field().ok_or_else(|| {
            StorehausError::validation(
                T::table_name(),
                "soft_delete",
                "Soft delete field not found",
            )
        })?;

        let sql = format!(
            "SELECT COUNT(*) as active FROM {} WHERE {} = true",
            T::table_name(),
            soft_delete_field
        );
        let result = sqlx::query(&sql)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|e| StorehausError::database_operation(T::table_name(), "count_active", e))?;

        let active: i64 = result.get("active");
        Ok(active)
    }
}
