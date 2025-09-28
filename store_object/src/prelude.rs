//! Convenience re-exports for common store-object usage

// Core traits
pub use crate::traits::{Filterable, SoftDeletable, StoreFilter, StoreObject, TableMetadata};

// Database executor from table_metadata module
pub use crate::traits::table_metadata::DatabaseExecutor;

// Error types
pub use crate::errors::StorehausError;

// Core store functionality
pub use crate::generic_store::GenericStore;

// ID type - use what's actually available
pub use crate::id_type::{HasUniversalId, UniversalId};

// Validation
pub use crate::validation::{ValidatedFieldName, ValidatedTableName, ValidationError};

// Tagged data functionality
pub use crate::tagged_data::TaggedData;

// Query building
pub use crate::query_builder::{QueryBuilder, QueryFilter, SortOrder};

// Cache params (re-exported from cache_system)
pub use crate::CacheParams;

// Common external dependencies that are frequently used
pub use async_trait::async_trait;
pub use serde::{Deserialize, Serialize};
pub use sqlx::{FromRow, PgPool, Row};
pub use uuid::Uuid;
