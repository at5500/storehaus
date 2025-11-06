//! Store Object - Core database abstraction layer for Storehaus
//!
//! This crate provides the foundational types and traits for database operations,
//! including generic stores, query builders, and validation utilities.

pub mod errors;
pub mod generic_store;
pub mod id_type;
pub mod prelude;
pub mod query_builder;
pub mod tagged_data;
pub mod traits;
pub mod validation;

pub use cache_system::CacheParams;
pub use errors::StorehausError;
pub use generic_store::GenericStore;
pub use id_type::{HasUniversalId, NoId, UniversalId};
pub use query_builder::{QueryBuilder, QueryFilter, QueryOperator, SortOrder};
pub use tagged_data::TaggedData;
pub use traits::table_metadata::DatabaseExecutor;
pub use traits::*;
pub use validation::{ValidatedFieldName, ValidatedTableName, ValidationError};

use sqlx::PgPool;

pub type DbPool = PgPool;
