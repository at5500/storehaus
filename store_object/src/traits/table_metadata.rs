use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Metadata about database table structure and operations
/// This trait should be derived using #[derive(TableMetadata)]
///
/// Attribute macros for field-level configuration
/// Example usage:
/// ```
/// #[derive(TableMetadata)]
/// #[table(name = "customers")]
/// pub struct Customer {
///     #[primary_key]
///     pub id: Uuid, // or i32, i64, String, etc.
///
///     #[field(create, update)]
///     pub first_name: String,
///
///     #[field(readonly)]
///     pub created_at: DateTime<Utc>,
///
///     #[soft_delete]
///     pub is_active: bool,
/// }
/// ```
pub trait TableMetadata: Clone + Send + Sync + Debug + Serialize + for<'de> Deserialize<'de> {
    /// The type used for the primary key
    type Id: Clone + Send + Sync + Debug + for<'q> sqlx::Encode<'q, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>;

    /// The table name in the database
    fn table_name() -> &'static str;

    /// SQL for CREATE operation (with placeholders)
    fn create_sql() -> &'static str;

    /// SQL for UPDATE operation (with placeholders)
    fn update_sql() -> &'static str;

    /// Whether this entity supports soft deletion (has is_active field)
    fn supports_soft_delete() -> bool {
        false
    }

    /// Extract ID from model instance
    fn extract_id(&self) -> Self::Id;

    /// Get field names for CREATE operation
    fn create_fields() -> Vec<&'static str>;

    /// Get field names for UPDATE operation
    fn update_fields() -> Vec<&'static str>;

    /// Get the primary key field name
    fn primary_key_field() -> &'static str;

    /// Execute CREATE query with bound parameters
    fn execute_create(&self, pool: &sqlx::PgPool) -> impl std::future::Future<Output = Result<Self, sqlx::Error>> + Send
    where
        Self: Sized;

    /// Execute UPDATE query with bound parameters
    fn execute_update(&self, pool: &sqlx::PgPool) -> impl std::future::Future<Output = Result<Self, sqlx::Error>> + Send
    where
        Self: Sized;

    /// Execute CREATE query with bound parameters using a transaction
    fn execute_create_tx(&self, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> impl std::future::Future<Output = Result<Self, sqlx::Error>> + Send
    where
        Self: Sized;

    /// Execute UPDATE query with bound parameters using a transaction
    fn execute_update_tx(&self, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> impl std::future::Future<Output = Result<Self, sqlx::Error>> + Send
    where
        Self: Sized;

    /// Generate CREATE TABLE SQL statement
    fn create_table_sql() -> String {
        "-- Auto-generated table creation not implemented".to_string()
    }

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
}