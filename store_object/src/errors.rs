//! Error types for store object operations
//!
//! This module defines all error types that can occur during
//! store object operations and database interactions.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorehausError {
    #[error("Database operation failed on table '{table}': {operation}")]
    DatabaseOperation {
        table: String,
        operation: String,
        #[source]
        source: sqlx::Error,
    },

    #[error("Query execution failed on table '{table}'")]
    QueryExecution {
        table: String,
        query: String,
        #[source]
        source: sqlx::Error,
    },

    #[error("Transaction failed: {operation}")]
    TransactionError {
        operation: String,
        #[source]
        source: sqlx::Error,
    },

    #[error("Cache operation failed: {operation}")]
    CacheError {
        operation: String,
        key: Option<String>,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Serialization failed for type '{type_name}'")]
    SerializationError {
        type_name: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("Record not found: {resource} with {identifier}")]
    NotFound {
        resource: String,
        identifier: String,
    },

    #[error("Validation failed for field '{field}' in table '{table}': {reason}")]
    ValidationError {
        table: String,
        field: String,
        reason: String,
    },

    #[error("Signal processing failed: {operation}")]
    SignalError {
        operation: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Configuration error: {parameter}")]
    ConfigurationError { parameter: String, reason: String },

    #[error("Invalid configuration: {message}")]
    InvalidConfiguration { message: String },

    #[error("Connection pool error")]
    ConnectionPoolError {
        #[source]
        source: sqlx::Error,
    },

    // Legacy errors for backward compatibility
    #[error("Database error: {0}")]
    #[deprecated(note = "Use DatabaseOperation instead")]
    DatabaseError(String),

    #[error("Internal server error: {0}")]
    #[deprecated(note = "Use specific error types instead")]
    InternalServerError(String),
}

impl StorehausError {
    /// Create a database operation error with context
    pub fn database_operation(table: &str, operation: &str, source: sqlx::Error) -> Self {
        Self::DatabaseOperation {
            table: table.to_string(),
            operation: operation.to_string(),
            source,
        }
    }

    /// Create a query execution error with context
    pub fn query_execution(table: &str, query: &str, source: sqlx::Error) -> Self {
        Self::QueryExecution {
            table: table.to_string(),
            query: query.to_string(),
            source,
        }
    }

    /// Create a validation error with context
    pub fn validation(table: &str, field: &str, reason: &str) -> Self {
        Self::ValidationError {
            table: table.to_string(),
            field: field.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Create a not found error with context
    pub fn not_found(resource: &str, identifier: &str) -> Self {
        Self::NotFound {
            resource: resource.to_string(),
            identifier: identifier.to_string(),
        }
    }

    /// Create a cache error with context
    pub fn cache_operation(
        operation: &str,
        key: Option<&str>,
        source: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        Self::CacheError {
            operation: operation.to_string(),
            key: key.map(|k| k.to_string()),
            source,
        }
    }

    /// Create a serialization error with context
    pub fn serialization(type_name: &str, source: serde_json::Error) -> Self {
        Self::SerializationError {
            type_name: type_name.to_string(),
            source,
        }
    }

    /// Create a transaction error with context
    pub fn transaction(operation: &str, source: sqlx::Error) -> Self {
        Self::TransactionError {
            operation: operation.to_string(),
            source,
        }
    }

    /// Check if this is a transient error that can be retried
    pub fn is_transient(&self) -> bool {
        match self {
            Self::DatabaseOperation { source, .. }
            | Self::QueryExecution { source, .. }
            | Self::TransactionError { source, .. }
            | Self::ConnectionPoolError { source } => {
                matches!(
                    source,
                    sqlx::Error::PoolTimedOut | sqlx::Error::PoolClosed | sqlx::Error::Io(_)
                )
            }
            Self::CacheError { .. } => true, // Cache errors are usually transient
            _ => false,
        }
    }

    /// Get the error context for logging
    pub fn context(&self) -> std::collections::HashMap<String, String> {
        let mut context = std::collections::HashMap::new();

        match self {
            Self::DatabaseOperation {
                table, operation, ..
            } => {
                context.insert("table".to_string(), table.clone());
                context.insert("operation".to_string(), operation.clone());
            }
            Self::QueryExecution { table, query, .. } => {
                context.insert("table".to_string(), table.clone());
                context.insert("query".to_string(), query.clone());
            }
            Self::ValidationError {
                table,
                field,
                reason,
            } => {
                context.insert("table".to_string(), table.clone());
                context.insert("field".to_string(), field.clone());
                context.insert("reason".to_string(), reason.clone());
            }
            Self::NotFound {
                resource,
                identifier,
            } => {
                context.insert("resource".to_string(), resource.clone());
                context.insert("identifier".to_string(), identifier.clone());
            }
            Self::CacheError { operation, key, .. } => {
                context.insert("operation".to_string(), operation.clone());
                if let Some(key) = key {
                    context.insert("key".to_string(), key.clone());
                }
            }
            _ => {}
        }

        context
    }
}
