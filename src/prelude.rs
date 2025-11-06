//! Convenience re-exports for common StoreHaus usage
//!
//! This prelude module re-exports the most commonly used items from the StoreHaus ecosystem,
//! making it easier to import everything you need with a single use statement.
//!
//! # Example
//!
//! ```rust
//! use storehaus::prelude::*;
//!
//! // Now you have access to all the common StoreHaus types and traits
//! ```

// Core StoreHaus components
pub use crate::core::StoreHaus;
pub use crate::errors::StoreHausError;
pub use crate::migration;

// Re-export centralized config
pub use config::{AppConfig, CacheConfig, DatabaseConfig, SignalConfig};

// Re-export commonly used store-object types for convenience
pub use store_object::prelude::*;

// Re-export store_object module for macro-generated code
pub use store_object;

// Re-export signal system for event handling
pub use signal_system::prelude::*;

// Re-export cache system
pub use cache_system::prelude::*;

// Re-export table derive for model creation
pub use table_derive::{TableMetadata, model};

// Common external dependencies
pub use anyhow;
pub use async_trait;
pub use sqlx;
pub use tokio;

// Commonly used sqlx types
pub use sqlx::{FromRow, PgPool, Row, Transaction, Encode, Decode, Type, Postgres};
