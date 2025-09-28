//! Trait definitions
//!
//! This module defines core traits for database operations.

use crate::id_type::HasUniversalId;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Metadata about database table structure and operations
/// This trait should be derived using the `#[model]` attribute macro, which
/// automatically includes all necessary derives.
///
/// Recommended usage:
/// ```
/// use table_derive::model;
///
/// #[model]
/// #[table(name = "customers")]
/// pub struct Customer {
///     #[primary_key]
///     pub id: Uuid, // or i32, i64, String, etc.
///
///     #[field(create, update)]
///     pub first_name: String,
///
///     #[field(readonly)]
///     pub __created_at__: DateTime<Utc>,
///
///     #[soft_delete]
///     pub is_enabled: bool,
/// }
/// ```
///
/// Manual usage (not recommended):
/// ```
/// use table_derive::TableMetadata;
///
/// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow, sqlx::Type, TableMetadata)]
/// #[table(name = "customers")]
/// pub struct Customer {
///     // fields...
/// }
/// ```
pub trait TableMetadata:
    Clone + Send + Sync + Debug + Serialize + for<'de> Deserialize<'de>
{
    /// The type used for the primary key
    type Id: Clone
        + Send
        + Sync
        + Debug
        + Unpin
        + HasUniversalId
        + for<'q> sqlx::Encode<'q, sqlx::Postgres>
        + for<'r> sqlx::Decode<'r, sqlx::Postgres>
        + sqlx::Type<sqlx::Postgres>;

    /// The table name in the database
    fn table_name() -> &'static str;

    /// SQL for CREATE operation (with placeholders)
    fn create_sql() -> &'static str;

    /// SQL for UPDATE operation (with placeholders)
    fn update_sql() -> &'static str;

    /// SQL for SELECT all operation (optimized static query)
    fn list_all_sql() -> &'static str;

    /// SQL for DELETE by ID operation (optimized static query)
    fn delete_by_id_sql() -> &'static str;

    /// SQL for SELECT by ID operation (optimized static query)
    fn get_by_id_sql() -> &'static str;

    /// SQL for COUNT all operation (optimized static query)
    fn count_all_sql() -> &'static str;

    /// SQL for SELECT base operation (optimized static query)
    fn select_base_sql() -> &'static str;

    /// SQL for COUNT base operation (optimized static query)
    fn count_base_sql() -> &'static str;

    /// Whether this entity supports soft deletion
    fn supports_soft_delete() -> bool {
        false
    }

    /// Get the name of the soft delete field if it exists
    fn soft_delete_field() -> Option<&'static str> {
        None
    }

    /// Extract ID from model instance
    fn extract_id(&self) -> Self::Id;

    /// Get field names for CREATE operation
    fn create_fields() -> Vec<&'static str>;

    /// Get field names for UPDATE operation
    fn update_fields() -> Vec<&'static str>;

    /// Get the primary key field name
    fn primary_key_field() -> &'static str;

    /// Generate CREATE TABLE SQL statement
    fn create_table_sql() -> String;

    /// Generate DROP TABLE SQL statement
    fn drop_table_sql() -> String {
        format!("DROP TABLE IF EXISTS {}", Self::table_name())
    }

    /// Generate CREATE INDEX SQL statements
    fn create_indexes_sql() -> Vec<String> {
        vec![]
    }

    /// Get field names with their PostgreSQL types for table creation
    fn get_table_fields() -> Vec<(&'static str, &'static str)> {
        vec![]
    }

    /// Generate UPDATE WHERE SQL statement (for bulk updates)
    fn update_where_sql() -> &'static str {
        "UPDATE table_placeholder SET field_placeholder WHERE condition_placeholder"
    }

    /// Generate DELETE WHERE SQL statement (for bulk deletes)
    fn delete_where_sql() -> &'static str {
        "DELETE FROM table_placeholder WHERE condition_placeholder"
    }

    /// Bind parameters for UPDATE operations to a query
    /// Returns a new query with bound parameters
    fn bind_update_params_owned<'a>(
        &'a self,
        sql: &'a str,
    ) -> sqlx::query::QueryAs<'a, sqlx::Postgres, Self, sqlx::postgres::PgArguments>
    where
        Self: Sized;

    /// Bind parameters for UPDATE operations to a raw query
    /// Returns a new query with bound parameters
    fn bind_update_params_raw_owned<'a>(
        &'a self,
        sql: &'a str,
    ) -> sqlx::query::Query<'a, sqlx::Postgres, sqlx::postgres::PgArguments>;
}

/// Async trait for database operations that properly handles async/await
/// This separates database operations from metadata, providing better error handling and abstraction
#[async_trait]
pub trait DatabaseExecutor: TableMetadata {
    /// Execute CREATE query with bound parameters
    /// Returns the created entity with proper error handling
    async fn execute_create(
        &self,
        pool: &sqlx::PgPool,
    ) -> Result<Self, crate::errors::StorehausError>
    where
        Self: Sized + Send + Sync,
        Self: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>;

    /// Execute UPDATE query with bound parameters
    /// Returns the updated entity with proper error handling
    async fn execute_update(
        &self,
        pool: &sqlx::PgPool,
    ) -> Result<Self, crate::errors::StorehausError>
    where
        Self: Sized + Send + Sync,
        Self: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>;

    /// Execute CREATE query with bound parameters using a transaction
    /// Returns the created entity with proper error handling
    async fn execute_create_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Self, crate::errors::StorehausError>
    where
        Self: Sized + Send + Sync,
        Self: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>;

    /// Execute UPDATE query with bound parameters using a transaction
    /// Returns the updated entity with proper error handling
    async fn execute_update_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Self, crate::errors::StorehausError>
    where
        Self: Sized + Send + Sync,
        Self: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>;
}
